//! Runtime-level tracing event and span macros.

/// A macro to emit tracing events with dynamic level.
///
/// This macro is used to emit tracing events with a runtime-determined log level.
/// It solves the limitation where tracing::event! requires a constant level at compile time.
///
/// This implementation is based on the discussion in [tracing issue #2730](https://github.com/tokio-rs/tracing/issues/2730).
///
/// # Motivation
///
/// The standard `tracing::event!` macro requires the level to be a constant at compile time.
/// This can be limiting when you need to determine the log level at runtime, for example:
/// - Based on error severity
/// - Based on configuration
/// - Based on runtime conditions
///
/// # Performance
///
/// The macro expands to a match statement on the level, which has minimal runtime overhead.
/// The actual event emission is still handled by tracing's efficient filtering system.
///
/// # Example
///
/// ```rust
/// use otel_http::dyn_event;
/// use tracing::Level;
///
/// let is_critical_error = true;
/// // Determine level at runtime
/// let level = if is_critical_error {
///     Level::ERROR
/// } else {
///     Level::DEBUG
/// };
/// let request_id = "uuid";
/// // Emit event with dynamic level
/// dyn_event!(level, request_id = %request_id, "request");
/// ```
///
/// # Comparison with log crate
///
/// The `log` crate allows dynamic levels by default, but `tracing` requires static metadata
/// for performance reasons. This macro provides a similar experience to `log` while maintaining
/// `tracing`'s performance benefits.
#[macro_export]
macro_rules! dyn_event {
    ($lvl:expr, $($tt:tt)*) => {
        match $lvl {
            tracing::Level::ERROR => tracing::event!(tracing::Level::ERROR, $($tt)*),
            tracing::Level::WARN => tracing::event!(tracing::Level::WARN, $($tt)*),
            tracing::Level::INFO => tracing::event!(tracing::Level::INFO, $($tt)*),
            tracing::Level::DEBUG => tracing::event!(tracing::Level::DEBUG, $($tt)*),
            tracing::Level::TRACE => tracing::event!(tracing::Level::TRACE, $($tt)*),
        }
    };
}

/// A macro to create spans with dynamic level.
///
/// Similar to `dyn_event!`, this macro allows creating spans with a runtime-determined level.
/// This implementation is based on the discussion in [tracing issue #2730](https://github.com/tokio-rs/tracing/issues/2730).
///
/// # Performance
///
/// Like `dyn_event!`, this macro expands to a match statement with minimal runtime overhead.
/// The span creation is still handled by tracing's efficient filtering system.
///
/// # Example
///
/// ```rust
/// use otel_http::dyn_span;
/// use tracing::Level;
///
/// let is_important_operation = true;
/// let level = if is_important_operation {
///     Level::INFO
/// } else {
///     Level::DEBUG
/// };
/// let op = "important operation";
/// let span = dyn_span!(level, "processing", operation = %op);
/// let _guard = span.enter();
/// // ... do work ...
/// ```
#[macro_export]
macro_rules! dyn_span {
    ($lvl:expr, $($tt:tt)*) => {
        match $lvl {
            tracing::Level::ERROR => tracing::span!(tracing::Level::ERROR, $($tt)*),
            tracing::Level::WARN => tracing::span!(tracing::Level::WARN, $($tt)*),
            tracing::Level::INFO => tracing::span!(tracing::Level::INFO, $($tt)*),
            tracing::Level::DEBUG => tracing::span!(tracing::Level::DEBUG, $($tt)*),
            tracing::Level::TRACE => tracing::span!(tracing::Level::TRACE, $($tt)*),
        }
    };
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use tracing::Level;

    #[test]
    fn test_dyn_event_basic_usage() {
        let level = Level::INFO;
        // Test all log levels
        dyn_event!(level, "error message");
        dyn_event!(level, "warning message");
        dyn_event!(level, "info message");
        dyn_event!(level, "debug message");
        dyn_event!(level, "trace message");
    }

    #[test]
    fn test_dyn_event_with_fields() {
        let level = Level::INFO;
        dyn_event!(level, field1 = "value1", field2 = 42, "message with fields");
        let request_id = "uuid";
        dyn_event!(level, request_id = %request_id, "processing request");
        dyn_event!(level, request_id = %request_id);
    }

    #[test]
    fn test_dyn_span() {
        let level = Level::INFO;
        let _span = dyn_span!(level, "span message");
    }

    #[test]
    fn test_dyn_span_with_fields() {
        let level = Level::DEBUG;
        let _span = dyn_span!(
            level,
            "operation",
            operation_type = "database_query",
            duration_ms = 150
        );
    }
}
