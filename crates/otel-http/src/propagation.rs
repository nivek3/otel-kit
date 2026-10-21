//! OpenTelemetry context propagation for HTTP.

use opentelemetry::{Context, global};
use opentelemetry_http::{HeaderExtractor, HeaderInjector, Request, Response};

/// Extract the context from the incoming request headers
pub fn extract_context_from_headers(headers: &http::HeaderMap) -> Context {
    global::get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(headers)))
}

/// Extract the context from the incoming request headers
pub fn extract_context_from_request<T>(request: &Request<T>) -> Context {
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    })
}

/// Inject specific context into a request for distributed tracing
pub fn inject_context_into_request<T>(context: &Context, request: &mut Request<T>) {
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(context, &mut HeaderInjector(request.headers_mut()));
    });
}

/// Inject specific context into a response for distributed tracing
pub fn inject_context_into_response<T>(context: &Context, response: &mut Response<T>) {
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(context, &mut HeaderInjector(response.headers_mut()));
    });
}

#[cfg(test)]
#[cfg(feature = "http")]
mod tests {
    use super::*;
    use opentelemetry::Context;
    use opentelemetry::global;
    use opentelemetry::trace::{
        SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState,
    };
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use std::str::FromStr;

    #[test]
    fn test_inject_context_into_request() {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let trace_id = TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        let span_id = SpanId::from_hex("00f067aa0ba902b7").unwrap();
        let span_context = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        );

        let context = Context::current().with_remote_span_context(span_context);
        let mut request = Request::builder().body(()).unwrap();
        inject_context_into_request(&context, &mut request);

        let traceparent = request
            .headers()
            .get("traceparent")
            .expect("traceparent header should be set")
            .to_str()
            .expect("traceparent header should be valid UTF-8");

        let expected_traceparent = format!("00-{trace_id}-{span_id}-01");
        assert_eq!(traceparent, expected_traceparent);
    }

    #[test]
    fn test_inject_context_into_request_with_trace_state() {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let trace_id = TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        let span_id = SpanId::from_hex("00f067aa0ba902b7").unwrap();
        let span_context = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            false,
            TraceState::from_str("key1=value1,key2=value2").unwrap(),
        );

        let context = Context::current().with_remote_span_context(span_context);
        let mut request = Request::builder().body(()).unwrap();
        inject_context_into_request(&context, &mut request);

        let tracestate = request
            .headers()
            .get("tracestate")
            .expect("tracestate header should be set")
            .to_str()
            .expect("tracestate header should be valid UTF-8");
        assert_eq!(tracestate, "key1=value1,key2=value2");
    }

    #[test]
    fn test_inject_context_into_request_without_span() {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let context = Context::current();
        let mut request = Request::builder().body(()).unwrap();
        inject_context_into_request(&context, &mut request);

        assert!(!request.headers().contains_key("traceparent"));
    }
}
