use std::sync::{Arc, Mutex};

use axum::{Router, http::StatusCode, routing::get};
use http::HeaderMap;
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt as _, TracerProvider as _},
};
use opentelemetry_http::HeaderExtractor;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{InMemorySpanExporter, SdkTracerProvider},
};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use tokio::net::TcpListener;
use tracing::{Instrument as _, instrument::WithSubscriber as _};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;
use tracing_subscriber::layer::SubscriberExt as _;

fn build_http_client(client: Client) -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(client)
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

#[tokio::test(flavor = "current_thread")]
async fn propagation_is_omitted_without_context_or_when_disabled() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let app = Router::new().route("/", get(|headers: HeaderMap| async move { headers }));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = build_http_client(Client::builder().no_proxy().build().unwrap());

    let response = client
        .get(format!("http://{address}/"))
        .send()
        .with_subscriber(tracing::subscriber::NoSubscriber::default())
        .await
        .unwrap();
    assert!(!response.headers().contains_key("traceparent"));
    assert!(!response.headers().contains_key("tracestate"));

    let provider = SdkTracerProvider::builder().build();
    let subscriber = tracing_subscriber::registry().with(
        tracing_opentelemetry::layer().with_tracer(provider.tracer("disabled-propagation-test")),
    );
    async {
        let parent = tracing::info_span!("parent");
        assert!(parent.context().span().span_context().is_valid());
        let response = client
            .get(format!("http://{address}/"))
            .with_extension(reqwest_tracing::DisableOtelPropagation)
            .send()
            .instrument(parent)
            .await
            .unwrap();
        assert!(!response.headers().contains_key("traceparent"));
        assert!(!response.headers().contains_key("tracestate"));
    }
    .with_subscriber(subscriber)
    .await;

    server.abort();
    provider.shutdown().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn client_context_is_propagated_across_retries() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("http-client-test")));

    let received = Arc::new(Mutex::new(Vec::<HeaderMap>::new()));
    let captured = received.clone();
    let app = Router::new().route(
        "/users/1",
        get(move |headers: HeaderMap| {
            let captured = captured.clone();
            async move {
                let mut received = captured.lock().unwrap();
                received.push(headers);
                if received.len() == 1 {
                    StatusCode::SERVICE_UNAVAILABLE
                } else {
                    StatusCode::OK
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    async {
        let parent_headers = HeaderMap::from_iter([
            (
                http::header::HeaderName::from_static("traceparent"),
                "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                    .parse()
                    .unwrap(),
            ),
            (
                http::header::HeaderName::from_static("tracestate"),
                "vendor=value".parse().unwrap(),
            ),
        ]);
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(&parent_headers))
        });
        let parent = tracing::info_span!("articles-request");
        parent.set_parent(parent_context).unwrap();
        let client = build_http_client(Client::builder().no_proxy().build().unwrap());
        let response = client
            .get(format!("http://{address}/users/1"))
            .send()
            .instrument(parent)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    .with_subscriber(subscriber)
    .await;

    server.abort();
    provider.force_flush().unwrap();
    let spans = exporter.get_finished_spans().unwrap();
    let parent = spans.iter().find(|s| s.name == "articles-request").unwrap();
    let clients: Vec<_> = spans
        .iter()
        .filter(|s| s.span_kind == SpanKind::Client)
        .collect();
    assert_eq!(clients.len(), 1);
    let client = clients[0];
    assert_eq!(client.parent_span_id, parent.span_context.span_id());
    assert_eq!(
        client.span_context.trace_id(),
        parent.span_context.trace_id()
    );
    assert_eq!(
        parent.span_context.trace_id().to_string(),
        "4bf92f3577b34da6a3ce929d0e0e4736"
    );
    assert_eq!(parent.parent_span_id.to_string(), "00f067aa0ba902b7");

    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    for headers in received.iter() {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(headers))
        });
        let span = context.span();
        let propagated = span.span_context();
        assert_eq!(propagated.trace_id(), client.span_context.trace_id());
        assert_eq!(propagated.span_id(), client.span_context.span_id());
        assert_eq!(propagated.trace_flags(), client.span_context.trace_flags());
        assert_eq!(headers["tracestate"], "vendor=value");
    }
    let status = client
        .attributes
        .iter()
        .find(|attribute| attribute.key.as_str() == "http.response.status_code")
        .map(|attribute| &attribute.value);
    assert_eq!(status, Some(&opentelemetry::Value::String("200".into())));
    provider.shutdown().unwrap();
}
