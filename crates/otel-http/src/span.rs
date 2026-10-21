//! HTTP request span creation utilities.
//!
//! [`make_request_span`] sets attributes that match [OpenTelemetry HTTP server spans](https://opentelemetry.io/docs/specs/semconv/http/http-spans/)
//! (for example `server.address`, `user_agent.original`, `url.path`, `url.scheme`, `network.protocol.*`).
//! See the [`axum_otel` crate](https://docs.rs/axum-otel) documentation for a migration table from older attribute names.

use crate::{context, dyn_span, fields};
use http::Request;
use tracing::{Level, Span, field::Empty};

/// Creates a new [`Span`] and customizes it before applying the remote parent.
///
/// The `customize_span` callback runs synchronously exactly once after common
/// HTTP fields are recorded and before the parent context is applied.
/// Adapter-owned `otel.name` and `otel.kind` values must be recorded in this
/// callback because they cannot be changed after the OpenTelemetry span is
/// materialized.
///
/// # Example
///
/// ```rust
/// use http::Request;
/// use tracing::Level;
/// use otel_http::span::make_request_span;
///
/// let request = Request::builder().uri("/items").body(()).unwrap();
/// let _span = make_request_span(Level::INFO, &request, |span| {
///     span.record("http.route", "/items");
/// });
/// ```
///
/// # Panics
///
/// Panics if the `customize_span` callback panics.
pub fn make_request_span<B>(
    level: Level,
    request: &Request<B>,
    customize_span: impl FnOnce(&Span),
) -> Span {
    let span = dyn_span!(
        level,
        "request",
        // HTTP fields
        client.address = Empty,
        http.request.method = %fields::extract_http_method(request),
        http.route = Empty,
        http.response.status_code = Empty,
        network.protocol.name = fields::extract_network_protocol_name(request),
        network.protocol.version = Empty,
        // OpenTelemetry fields
        otel.name = Empty,
        otel.kind = ?Empty,
        otel.status_code = Empty,
        otel.status_description = Empty,
        // Request tracking
        request_id = Empty,
        server.address = Empty,
        trace_id = Empty,
        url.path = fields::extract_url_path(request),
        url.query = Empty,
        url.scheme = Empty,
        user_agent.original = Empty
    );

    if let Some(host) = fields::extract_host(request) {
        span.record("server.address", host);
    }
    if let Some(user_agent) = fields::extract_user_agent(request) {
        span.record("user_agent.original", user_agent);
    }
    if let Some(version) = fields::extract_network_protocol_version(request) {
        span.record("network.protocol.version", version);
    }
    if let Some(request_id) = fields::extract_request_id(request) {
        span.record("request_id", request_id);
    }
    if let Some(query) = fields::extract_url_query(request) {
        span.record("url.query", query);
    }
    if let Some(scheme) = fields::extract_url_scheme(request) {
        span.record("url.scheme", scheme);
    }

    customize_span(&span);
    context::set_otel_parent(request.headers(), &span);
    span
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Method, Version};
    use opentelemetry::{
        global,
        trace::{SpanKind, TracerProvider as _},
    };
    use opentelemetry_sdk::{
        Resource,
        propagation::TraceContextPropagator,
        trace::{InMemorySpanExporter, Sampler, SdkTracerProvider},
    };
    use tracing::field;
    use tracing_subscriber::{Registry, layer::SubscriberExt as _};

    fn span_attr(span: &opentelemetry_sdk::trace::SpanData, key: &str) -> Option<String> {
        span.attributes
            .iter()
            .find(|kv| kv.key.as_str() == key)
            .map(|kv| {
                let value = kv.value.to_string();
                value
                    .strip_prefix('"')
                    .and_then(|unquoted| unquoted.strip_suffix('"'))
                    .unwrap_or(&value)
                    .to_string()
            })
    }

    fn export_request_span(request: Request<()>) -> opentelemetry_sdk::trace::SpanData {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(exporter.clone())
            .with_resource(Resource::builder().build())
            .build();

        let tracer = provider.tracer("http-span-test");
        let subscriber =
            Registry::default().with(tracing_opentelemetry::OpenTelemetryLayer::new(tracer));

        tracing::subscriber::with_default(subscriber, || {
            make_request_span(Level::INFO, &request, |_| {});
        });

        provider.force_flush().expect("spans should flush");
        let request_span = exporter
            .get_finished_spans()
            .expect("finished spans should be available")
            .into_iter()
            .find(|span| span.name == "request")
            .expect("request span should be exported");

        provider.shutdown().expect("provider should shut down");
        request_span
    }

    #[test]
    fn request_span_records_shared_http_fields() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("https://example.com/items?kind=test")
            .version(Version::HTTP_2)
            .header("host", "example.com")
            .header("user-agent", "span-test")
            .header("x-request-id", "request-123")
            .body(())
            .expect("request should be valid");
        let request_span = export_request_span(request);

        let expected = [
            ("http.request.method", "POST"),
            ("server.address", "example.com"),
            ("network.protocol.name", "http"),
            ("network.protocol.version", "2"),
            ("url.path", "/items"),
            ("url.query", "kind=test"),
            ("url.scheme", "https"),
            ("user_agent.original", "span-test"),
            ("request_id", "request-123"),
        ];
        for (key, value) in expected {
            assert_eq!(
                span_attr(&request_span, key),
                Some(value.to_string()),
                "{key}"
            );
        }

        for deprecated in [
            "http.method",
            "http.status_code",
            "http.target",
            "http.host",
            "http.user_agent",
        ] {
            assert_eq!(span_attr(&request_span, deprecated), None, "{deprecated}");
        }
    }

    #[test]
    fn request_span_omits_missing_optional_fields() {
        let request = Request::builder()
            .uri("/items")
            .body(())
            .expect("request should be valid");
        let request_span = export_request_span(request);

        for missing in [
            "client.address",
            "http.route",
            "server.address",
            "url.query",
            "url.scheme",
            "user_agent.original",
            "request_id",
        ] {
            assert_eq!(span_attr(&request_span, missing), None, "{missing}");
        }
        assert_eq!(
            span_attr(&request_span, "network.protocol.version"),
            Some("1.1".to_string())
        );
    }

    #[test]
    fn request_span_inherits_remote_parent() {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
        let parent_span_id = "00f067aa0ba902b7";
        let request = Request::builder()
            .uri("/items")
            .header("traceparent", format!("00-{trace_id}-{parent_span_id}-01"))
            .body(())
            .expect("request should be valid");
        let request_span = export_request_span(request);

        assert_eq!(request_span.span_context.trace_id().to_string(), trace_id);
        assert_eq!(request_span.parent_span_id.to_string(), parent_span_id);
        assert_eq!(
            span_attr(&request_span, "trace_id"),
            Some(trace_id.to_string())
        );
    }

    #[test]
    fn adapter_customization_runs_before_parent_materializes_span() {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
        let request = Request::builder()
            .uri("/items")
            .header("traceparent", format!("00-{trace_id}-00f067aa0ba902b7-01"))
            .body(())
            .expect("request should be valid");

        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(exporter.clone())
            .with_resource(Resource::builder().build())
            .build();
        let tracer = provider.tracer("http-span-customization-test");
        let subscriber =
            Registry::default().with(tracing_opentelemetry::OpenTelemetryLayer::new(tracer));

        tracing::subscriber::with_default(subscriber, || {
            make_request_span(Level::INFO, &request, |span| {
                span.record("http.route", "/items");
                span.record("client.address", field::display("192.0.2.1"));
                span.record("otel.name", "GET /items");
                span.record("otel.kind", field::debug(SpanKind::Server));
            });
        });

        provider.force_flush().expect("spans should flush");
        let request_span = exporter
            .get_finished_spans()
            .expect("finished spans should be available")
            .into_iter()
            .find(|span| span.name == "GET /items")
            .expect("customized request span should be exported");

        assert_eq!(request_span.span_kind, SpanKind::Server);
        assert_eq!(
            span_attr(&request_span, "http.route"),
            Some("/items".to_string())
        );
        assert_eq!(
            span_attr(&request_span, "client.address"),
            Some("192.0.2.1".to_string())
        );
        assert_eq!(request_span.span_context.trace_id().to_string(), trace_id);

        provider.shutdown().expect("provider should shut down");
    }
}
