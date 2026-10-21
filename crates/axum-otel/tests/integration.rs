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
    trace::{SpanKind, TracerProvider},
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
    Router::new().route("/", get(hello)).route_layer(
        TraceLayer::new_for_http()
            .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
            .on_response(AxumOtelOnResponse::new().level(Level::INFO))
            .on_failure(AxumOtelOnFailure::new()),
    )
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
    let response = app.oneshot(request).await.expect("Failed to send request");

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
