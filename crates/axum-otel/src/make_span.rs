use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, MatchedPath},
    http,
};
use opentelemetry::trace::SpanKind;
use otel_http::span::make_request_span;
use tower_http::trace::MakeSpan;
use tracing::Level;

/// An implementor of [`MakeSpan`] which creates `tracing` spans populated with information about
/// the request received by an `axum` web server.
///
/// Shared HTTP server span semantics are provided by `otel-http`;
/// this adapter adds only Axum route, peer address, span name, and span kind.
///
/// Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/main/tower-http/src/trace/make_span.rs).
///
/// The shared HTTP span module adds the following attributes when their values
/// are available:
///
/// - `http.request.method`: The HTTP method
/// - `server.address`: The `Host` header (OpenTelemetry [`server.address`](https://opentelemetry.io/docs/specs/semconv/registry/attributes/server/))
/// - `network.protocol.name`: The network protocol name
/// - `network.protocol.version`: The network protocol version
/// - `url.path`: The request path
/// - `url.query`: The request query string
/// - `url.scheme`: The request scheme
/// - `user_agent.original`: The `User-Agent` header
/// - `request_id`: A unique request identifier
/// - `trace_id`: The OpenTelemetry trace ID
///
/// This Axum adapter additionally records:
///
/// - `http.route`: The matched route
/// - `client.address`: The client IP when [`ConnectInfo`] is available (OpenTelemetry [`client.address`](https://opentelemetry.io/docs/specs/semconv/registry/attributes/client/))
/// - `otel.name`: The span name derived from the method and matched route
/// - `otel.kind`: OpenTelemetry server span kind
///
/// # Example
///
/// ```rust
/// use axum_otel::{AxumOtelSpanCreator, Level};
/// use tower_http::trace::TraceLayer;
///
/// let layer = TraceLayer::new_for_http()
///     .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelSpanCreator {
    level: Level,
}

impl AxumOtelSpanCreator {
    /// Create a new `AxumOtelSpanCreator`.
    pub fn new() -> Self {
        Self {
            level: Level::TRACE,
        }
    }

    /// Set the [`Level`] used for [tracing events].
    ///
    /// Defaults to [`Level::TRACE`].
    ///
    /// [tracing events]: https://docs.rs/tracing/latest/tracing/#events
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl Default for AxumOtelSpanCreator {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> MakeSpan<B> for AxumOtelSpanCreator {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        let http_method = request.method().as_str();
        let http_route = request
            .extensions()
            .get::<MatchedPath>()
            .map(|p| p.as_str());

        let peer = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| *addr);

        let span_name = http_route.as_ref().map_or_else(
            || http_method.to_string(),
            |route| format!("{http_method} {route}"),
        );

        make_request_span(self.level, request, |span| {
            span.record("otel.name", span_name.as_str());
            span.record("otel.kind", tracing::field::debug(SpanKind::Server));
            if let Some(route) = http_route {
                span.record("http.route", route);
            }
            if let Some(peer) = peer {
                span.record("client.address", tracing::field::display(peer.ip()));
            }
        })
    }
}
