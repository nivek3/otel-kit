use axum::http;
use tower_http::trace::OnResponse;
use tracing::Level;
use tracing_otel::dyn_event;

/// An implementor of [`OnResponse`] which records the response status code and latency.
///
/// Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/main/tower-http/src/trace/on_response.rs).
///
/// This component adds the following attributes to the span:
///
/// - `http.response.status_code`: The response status code
/// - `otel.status_code`: The OpenTelemetry status code (OK for successful responses)
///
/// # Example
///
/// ```rust
/// use axum_otel::{AxumOtelOnResponse, Level};
/// use tower_http::trace::TraceLayer;
///
/// let layer = TraceLayer::new_for_http()
///     .on_response(AxumOtelOnResponse::new().level(Level::INFO));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelOnResponse {
    level: Level,
}

impl Default for AxumOtelOnResponse {
    fn default() -> Self {
        Self {
            level: Level::DEBUG,
        }
    }
}

impl AxumOtelOnResponse {
    /// Create a new `DefaultOnResponse`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the [`Level`] used for [tracing events].
    ///
    /// Please note that while this will set the level for the tracing events
    /// themselves, it might cause them to lack expected information, like
    /// request method or path. You can address this using
    /// [`AxumOtelOnResponse::level`].
    ///
    /// Defaults to [`Level::DEBUG`].
    ///
    /// [tracing events]: https://docs.rs/tracing/latest/tracing/#events
    /// [`AxumOtelOnResponse::level`]: crate::make_span::AxumOtelSpanCreator::level
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl<B> OnResponse<B> for AxumOtelOnResponse {
    fn on_response(
        self,
        response: &http::Response<B>,
        latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        let status = response.status().as_u16();
        span.record("http.response.status_code", i64::from(status));
        span.record("otel.status_code", "OK");

        dyn_event!(
            self.level,
            latency = %latency.as_millis(),
            status = %status,
            "response"
        );
    }
}
