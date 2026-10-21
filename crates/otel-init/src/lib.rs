//!
//! # Tracing OpenTelemetry Extra
//!
//! **Reference:** This crate is mainly organized based on the [official tracing-opentelemetry OTLP example](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs).
//!
//! This crate provides enhanced OpenTelemetry integration for tracing applications.
//! It's based on the [tracing-opentelemetry examples](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs)
//! and provides a clean, easy-to-use API for setting up OpenTelemetry tracing and metrics.
//!
//! ## Features
//!
//! - Easy OpenTelemetry initialization with OTLP exporter
//! - Configurable sampling and resource attributes
//! - Automatic cleanup with guard pattern
//! - Support for both tracing and metrics
//!
//! ## Examples
//!
//! Basic usage with manual setup:
//! ```rust,ignore
//! use opentelemetry::KeyValue;
//! use otel_init::{get_resource, init_tracer_provider, init_env_filter, init_tracing_subscriber, init_meter_provider, init_logger_provider};
//! use tracing::Level;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create resource using SDK defaults and additional attributes
//!     let resource = get_resource(&[
//!         KeyValue::new("environment", "production"),
//!         KeyValue::new("version", "1.0.0"),
//!     ]);
//!
//!     // Initialize providers
//!     let tracer_provider = init_tracer_provider(&resource)?;
//!     let meter_provider = init_meter_provider(&resource)?;
//!     let logger_provider = init_logger_provider(&resource)?;
//!
//!     // initialize tracing subscriber with otel layers
//!     let _guard = init_tracing_subscriber(
//!         "my-service",
//!         init_env_filter(&Level::INFO),
//!         vec![Box::new(tracing_subscriber::fmt::layer())],
//!         tracer_provider,
//!         meter_provider,
//!         Some(logger_provider),
//!     )?;
//!     // Your application code here...
//!
//!     // Cleanup is handled automatically when the guard is dropped
//!     Ok(())
//! }
//! ```

mod guard;
mod otel;
mod resource;
#[cfg(feature = "subscriber")]
mod subscriber;

// Re-exports
pub use guard::OtelGuard;
pub use otel::{init_logger_provider, init_meter_provider, init_tracer_provider};
pub use resource::get_resource;
#[cfg(feature = "subscriber")]
pub use subscriber::{BoxLayer, init_env_filter, init_tracing_subscriber};

// Re-exports opentelemetry crates
pub use opentelemetry;
pub use opentelemetry_sdk;
pub use tracing_opentelemetry;
