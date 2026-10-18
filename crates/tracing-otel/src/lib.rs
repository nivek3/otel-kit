//! # Tracing Extra
//!
//! This crate provides common utilities for initializing tracing and OpenTelemetry.
//!
//! ## Features
//!
//! The crate is organized into several feature flags:
//!
//! - `otel`: OpenTelemetry integration for distributed tracing
//! - `logger`: Basic logging functionality with configurable formats
//! - `env`: Environment-based logging configuration
//! - `context`: Trace context utilities
//! - `fields`: Common tracing fields and attributes
//! - `http`: HTTP request/response tracing
//! - `span`: Span creation and management utilities
//!
//! ## Examples
//!
//! Basic usage with configuration builder:
//! ```rust,no_run,ignore
//! use tracing_otel::{Logger, LogFormat};
//! use opentelemetry::KeyValue;
//!
//! #[tokio::main]
//! async fn main() {
//!     let _guard = Logger::new("my-service")
//!         .with_format(LogFormat::Json)
//!         .with_ansi(false)
//!         .with_attributes(vec![
//!             KeyValue::new("environment", "production"),
//!             KeyValue::new("version", "1.0.0"),
//!         ])
//!         .init()
//!         .expect("Failed to initialize tracing");
//!
//!     // Your application code here
//!
//!     // Cleanup is handled automatically when the guard is dropped
//! }
//! ```
//!
//! Using environment-based configuration:
//! ```rust,no_run,ignore
//! use tracing_otel::init_logging_from_env;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Configure through environment variables:
//!     // LOG_SERVICE_NAME=my-service
//!     // LOG_FORMAT=json
//!     // OTEL_TRACES_SAMPLER=parentbased_traceidratio
//!     // OTEL_TRACES_SAMPLER_ARG=0.1
//!     let _guard = init_logging_from_env(None)
//!         .expect("Failed to initialize tracing from environment");
//!
//!     // Your application code here
//! }
//! ```
//!

// HTTP tracing modules
#[cfg(any(
    feature = "context",
    feature = "fields",
    feature = "http",
    feature = "span",
))]
pub mod http;

// OpenTelemetry integration
#[cfg(feature = "otel")]
pub mod otel {
    pub use otel_init::*;
}

// Logging functionality
#[cfg(any(feature = "logger", feature = "env"))]
pub mod logger;

// Re-exports
#[cfg(feature = "otel")]
pub use otel::*;

// Logger module exports
#[cfg(feature = "logger")]
pub use logger::{
    FmtSpan, LogFormat, LogRollingRotation, Logger, LoggerFileAppender, LoggerGuard, init_logging,
};

// Logger module exports
#[cfg(feature = "env")]
pub use logger::{init_logger_from_env, init_logging_from_env};

// Macros module exports
#[cfg(feature = "macros")]
pub mod macros;

// Extra module exports (backward compatibility)
pub mod extract {
    #[cfg(feature = "context")]
    pub use crate::http::context;

    #[cfg(feature = "fields")]
    pub use crate::http::fields;

    #[cfg(feature = "http")]
    pub use crate::http::propagation as http;

    #[cfg(feature = "span")]
    pub use crate::http::span;
}
