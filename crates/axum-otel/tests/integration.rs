use std::{net::SocketAddr, sync::OnceLock};

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Method, Request, StatusCode},
    routing::get,
};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};
use http_body_util::BodyExt;
use opentelemetry::{
    global,
    trace::{SpanKind, Status, TracerProvider},
};
use opentelemetry_sdk::{
    Resource,
    trace::{InMemorySpanExporter, RandomIdGenerator, Sampler, SdkTracerProvider},
};
use tokio::sync::Mutex;
use tower::ServiceExt;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use tracing_subscriber::{Registry, layer::SubscriberExt};

fn test_lock() -> &'static Mutex<()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

#[instrument]
async fn hello() -> &'static str {
    "Hello, world!"
}

fn span_attr(span: &opentelemetry_sdk::trace::SpanData, key: &str) -> Option<String> {
    span.attributes
        .iter()
        .find(|kv| kv.key.as_str() == key)
        .map(|kv| {
            let s = kv.value.to_string();
            s.strip_prefix('"')
                .and_then(|unquoted| unquoted.strip_suffix('"'))
                .unwrap_or(&s)
                .to_string()
        })
}

fn app() -> Router<()> {
    Router::new()
        .route("/", get(hello))
        .route(
            "/error",
            get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
        )
        .route_layer(
            TraceLayer::new_for_http()
                .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                .on_failure(AxumOtelOnFailure::new()),
        )
}

#[test]
fn failure_status_does_not_depend_on_event_level() {
    use tower_http::{classify::ServerErrorsFailureClass, trace::OnFailure};

    for level in [Level::WARN, Level::INFO] {
        for (failure, is_error) in [
            (
                ServerErrorsFailureClass::Error("body read failed".into()),
                true,
            ),
            (
                ServerErrorsFailureClass::StatusCode(StatusCode::INTERNAL_SERVER_ERROR),
                true,
            ),
            (
                ServerErrorsFailureClass::StatusCode(StatusCode::BAD_REQUEST),
                false,
            ),
        ] {
            let exporter = InMemorySpanExporter::default();
            let provider = SdkTracerProvider::builder()
                .with_simple_exporter(exporter.clone())
                .build();
            let subscriber = Registry::default().with(
                tracing_opentelemetry::OpenTelemetryLayer::new(provider.tracer("failure-test")),
            );
            let description = failure.to_string();
            tracing::subscriber::with_default(subscriber, || {
                let span = tracing::info_span!(
                    "request",
                    otel.status_code = tracing::field::Empty,
                    otel.status_description = tracing::field::Empty,
                );
                let _entered = span.enter();
                AxumOtelOnFailure::new().level(level).on_failure(
                    failure,
                    std::time::Duration::ZERO,
                    &span,
                );
            });
            provider.force_flush().unwrap();
            let spans = exporter.get_finished_spans().unwrap();
            assert_eq!(spans.len(), 1);
            let expected = if is_error {
                Status::error(description)
            } else {
                Status::Unset
            };
            assert_eq!(spans[0].status, expected, "event level: {level}");
            provider.shutdown().unwrap();
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_axum_otel_middleware() {
    let _test_guard = test_lock().lock().await;

    // Set up in-memory exporter for testing
    let exporter = InMemorySpanExporter::default();
    let provider: SdkTracerProvider = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_simple_exporter(exporter.clone())
        .with_resource(Resource::builder().build())
        .build();

    global::set_tracer_provider(provider.clone());

    // Set up tracing subscriber with OpenTelemetry layer
    let tracer = provider.tracer("axum-otel-test".to_string());
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    let subscriber = Registry::default().with(otel_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = app();

    // Send request using oneshot
    let mut request = Request::builder()
        .uri("/?foo=bar")
        .header("host", "example.com")
        .header("user-agent", "integration-test")
        .header("x-forwarded-proto", "https")
        .method(Method::GET)
        .body(Body::empty())
        .expect("Failed to build request");
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([192, 0, 2, 10], 3000))));
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read body");

    assert_eq!(body.to_bytes(), "Hello, world!".as_bytes());

    // Force flush to ensure spans are exported
    let _ = provider.force_flush();

    // Verify spans were created
    let spans = exporter
        .get_finished_spans()
        .expect("Failed to get finished spans");

    assert!(
        !spans.is_empty(),
        "Expected at least one span to be created"
    );

    let request_span = spans
        .iter()
        .find(|s| s.name == "GET /")
        .expect("Request span not found");

    let hello_span = spans
        .iter()
        .find(|s| s.name == "hello")
        .expect("Handler span not found");

    assert_eq!(
        hello_span.parent_span_id,
        request_span.span_context.span_id(),
        "Handler span should be a child of the request span"
    );
    assert_eq!(request_span.span_kind, SpanKind::Server);
    assert_eq!(request_span.status, Status::Unset);
    assert_eq!(
        span_attr(request_span, "http.route"),
        Some("/".to_string()),
        "Expected http.route to be /"
    );
    assert_eq!(
        span_attr(request_span, "client.address"),
        Some("192.0.2.10".to_string()),
        "Expected client.address to be 192.0.2.10"
    );

    assert_eq!(
        span_attr(request_span, "http.request.method"),
        Some("GET".to_string()),
        "Expected http.request.method to be GET"
    );
    assert_eq!(
        span_attr(request_span, "http.response.status_code"),
        Some("200".to_string()),
        "Expected http.response.status_code to be 200"
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/error")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    response.into_body().collect().await.unwrap();
    provider.force_flush().unwrap();
    let spans = exporter.get_finished_spans().unwrap();
    let failed_span = spans.iter().find(|span| span.name == "GET /error").unwrap();
    assert!(matches!(failed_span.status, Status::Error { .. }));
    assert_eq!(
        span_attr(failed_span, "http.response.status_code"),
        Some("500".into())
    );

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider");
}

#[tokio::test(flavor = "current_thread")]
async fn test_axum_otel_omits_missing_peer() {
    let _test_guard = test_lock().lock().await;

    let exporter = InMemorySpanExporter::default();
    let provider: SdkTracerProvider = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_simple_exporter(exporter.clone())
        .with_resource(Resource::builder().build())
        .build();

    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("axum-otel-test-optional".to_string());
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    let subscriber = Registry::default().with(otel_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .method(Method::GET)
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let _body = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read body");

    let _ = provider.force_flush();

    let spans = exporter
        .get_finished_spans()
        .expect("Failed to get finished spans");

    let request_span = spans
        .iter()
        .find(|s| s.name == "GET /")
        .expect("Request span not found");

    assert_eq!(
        span_attr(request_span, "client.address"),
        None,
        "Expected client.address to be omitted when missing"
    );

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider");
}
