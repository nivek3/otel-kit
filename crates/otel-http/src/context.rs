//! OpenTelemetry trace context management.

use opentelemetry::{SpanId, TraceId};
use tracing::warn;

use crate::propagation::extract_context_from_headers;

/// The key for the trace id in the span attributes.
pub const TRACE_ID: &str = "trace_id";

/// Returns the `trace_id` of the current span according to the global tracing subscriber.
pub fn current_trace_id() -> TraceId {
    use opentelemetry::trace::TraceContextExt as _;
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
    tracing::Span::current()
        .context()
        .span()
        .span_context()
        .trace_id()
}

/// Returns the `span_id` of the current span according to the global tracing subscriber.
pub fn current_span_id() -> SpanId {
    use opentelemetry::trace::TraceContextExt as _;
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
    tracing::Span::current()
        .context()
        .span()
        .span_context()
        .span_id()
}

/// Set the parent span for the current span and record the trace id.
pub fn set_otel_parent(headers: &http::HeaderMap, span: &tracing::Span) {
    use opentelemetry::trace::TraceContextExt as _;
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
    let remote_context = extract_context_from_headers(headers);
    if let Err(e) = span.set_parent(remote_context) {
        warn!("Failed to set parent on span: {:?}", e);
    }

    let trace_id = span.context().span().span_context().trace_id();
    span.record(TRACE_ID, tracing::field::display(trace_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::TraceContextExt as _;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry::{KeyValue, global};
    use opentelemetry_sdk::Resource;
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use tracing::{Level, Span};
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    fn init_tracing() {
        global::set_text_map_propagator(TraceContextPropagator::new());

        let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("Failed to build the span exporter");
        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(otlp_exporter)
            .with_resource(
                Resource::builder()
                    .with_attribute(KeyValue::new("service.name", env!("CARGO_CRATE_NAME")))
                    .build(),
            )
            .build();
        let tracer = provider.tracer(env!("CARGO_CRATE_NAME"));
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        tracing_subscriber::registry()
            .with(telemetry)
            .try_init()
            .unwrap_or_default();

        let _root_span = tracing::info_span!("root_span").entered();
    }

    fn create_span() -> Span {
        tracing::span!(Level::INFO, "test_span")
    }

    #[tokio::test]
    async fn test_set_otel_parent_without_headers() {
        init_tracing();
        let headers = http::HeaderMap::new();
        let span = create_span();
        set_otel_parent(&headers, &span);

        let trace_id = span.context().span().span_context().trace_id().to_string();
        assert!(!trace_id.is_empty());
    }

    #[tokio::test]
    async fn test_set_otel_parent_with_valid_traceparent() {
        init_tracing();
        let mut headers = http::HeaderMap::new();
        let expected_trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
        let traceparent = format!("00-{expected_trace_id}-00f067aa0ba902b7-01");
        headers.insert("traceparent", traceparent.parse().unwrap());

        let span = create_span();
        set_otel_parent(&headers, &span);

        let trace_id = span.context().span().span_context().trace_id().to_string();
        assert_eq!(trace_id, expected_trace_id);
    }

    #[tokio::test]
    async fn test_current_trace_id() {
        init_tracing();
        let span = create_span();
        let _entered = span.enter();
        let outer_trace_id = span.context().span().span_context().trace_id();
        let trace_id = current_trace_id();
        assert_eq!(outer_trace_id, trace_id);
    }
}
