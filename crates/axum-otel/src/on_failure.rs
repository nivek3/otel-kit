use otel_http::dyn_event;
use tower_http::{classify::ServerErrorsFailureClass, trace::OnFailure};
use tracing::Level;

/// An implementor of [`OnFailure`] which records the failure status code.
///
/// Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/main/tower-http/src/trace/on_failure.rs).
///
/// This component updates the span's `otel.status_code` and `otel.status_description`
/// when a server error occurs.
///
/// # Example
///
/// ```rust
/// use axum_otel::{AxumOtelOnFailure, Level};
/// use tower_http::trace::TraceLayer;
///
/// let layer = TraceLayer::new_for_http()
///     .on_failure(AxumOtelOnFailure::new().level(Level::INFO));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelOnFailure {
    level: Level,
}

impl Default for AxumOtelOnFailure {
    fn default() -> Self {
        Self {
            level: Level::ERROR,
        }
    }
}

impl AxumOtelOnFailure {
    /// Create a new `DefaultOnFailure`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the [`Level`] used for [tracing events].
    ///
    /// Defaults to [`Level::ERROR`].
    ///
    /// [tracing events]: https://docs.rs/tracing/latest/tracing/#events
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl OnFailure<ServerErrorsFailureClass> for AxumOtelOnFailure {
    fn on_failure(
        &mut self,
        failure_classification: ServerErrorsFailureClass,
        latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        dyn_event!(
            self.level,
            classification = %failure_classification,
            latency = %latency.as_millis(),
            "response failed"
        );
        match failure_classification {
            ServerErrorsFailureClass::StatusCode(status) if status.is_server_error() => {
                span.record("otel.status_code", "ERROR");
                span.record(
                    "otel.status_description",
                    tracing::field::display(failure_classification),
                );
            }
            _ => {}
        }
    }
}
