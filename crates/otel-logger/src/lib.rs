#![deny(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
//! OpenTelemetry logging configuration and initialization.
//!
//! This crate provides a flexible and configurable logging system that integrates
//! OpenTelemetry tracing and metrics. It offers both programmatic configuration
//! through a builder pattern and environment variable-based configuration.
//!
//! # Features
//!
//! - Builder-style configuration API
//! - Environment variable support (with "env" feature)
//! - Multiple log formats (compact, pretty, json)
//! - SDK environment settings for sampling (`OTEL_TRACES_SAMPLER`) and metric export
//!   intervals (`OTEL_METRIC_EXPORT_INTERVAL`)
//! - Custom resource attributes
//! - Optional console output
//! - Optional file output
//!
//! The default instrumentation scope name is `otel_logger`.
//! The Logger service name identifies the tracer's instrumentation scope.
//! Resource `service.name` uses SDK defaults or explicit resource attributes.
//! Set `OTEL_SERVICE_NAME` to configure the resource service name.
//!
//! # Quick Start
//!
//! ```rust
//! use otel_logger::Logger;
//! use tracing::Level;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Basic initialization with console output
//!     let guard = Logger::new("my-service").init()?;
//!
//!     // Your application code here...
//!
//!     // The guard will automatically clean up when dropped
//!     Ok(())
//! }
//! ```
//!
//! # Advanced Configuration
//!
//! ```rust
//! use otel_logger::Logger;
//! use tracing::Level;
//! use opentelemetry::KeyValue;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let guard = Logger::new("my-service")
//!         .with_level(Level::DEBUG)
//!         .with_console_enabled(true)
//!         .with_attributes(vec![
//!             KeyValue::new("environment", "production"),
//!             KeyValue::new("version", "1.0.0"),
//!         ])
//!         .init()?;
//!
//!     // Your application code here...
//!
//!     Ok(())
//! }
//! ```

mod config;
mod deserialize;
#[cfg(feature = "env")]
mod env;
mod guard;
mod init;
mod subscriber;

// Re-exports
pub use config::{LogFormat, LogRollingRotation, Logger, LoggerFileAppender};
pub use deserialize::default;
#[cfg(feature = "env")]
pub use env::{init_logger_from_env, init_logging_from_env};
pub use guard::LoggerGuard;
pub use init::{init_logging, init_tracing_from_logger};
pub use otel_init::{BoxLayer, OtelGuard, init_env_filter};
pub use subscriber::{init_layer, setup_tracing};

// Re-export FmtSpan
pub use tracing_subscriber::fmt::format::FmtSpan;

#[cfg(test)]
mod tests;
