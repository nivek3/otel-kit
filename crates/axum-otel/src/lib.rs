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
#![doc(html_root_url = "https://docs.rs/axum-otel/latest")]

//! OpenTelemetry tracing for axum based on tower-http.
//!
//! This crate provides a middleware for Axum web framework that automatically instruments HTTP requests
//! and responses, and adds OpenTelemetry tracing to the request and response spans.
//!
//! ## Features
//!
//! - Automatic request and response tracing
//! - OpenTelemetry integration
//! - Request ID tracking
//! - Customizable span attributes
//! - Error tracking
//!
//! ## Usage
//!
//! ```rust
//! use axum::{
//!     routing::get,
//!     Router,
//! };
//! use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};
//! use tower_http::trace::TraceLayer;
//!
//! async fn handler() -> &'static str {
//!     "Hello, world!"
//! }
//!
//! // Build our application with a route
//! let app: Router<()> = Router::new()
//!     .route("/", get(handler))
//!     .layer(
//!         TraceLayer::new_for_http()
//!             .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
//!             .on_response(AxumOtelOnResponse::new().level(Level::INFO))
//!             .on_failure(AxumOtelOnFailure::new()),
//!     );
//! ```
//!
//! ## Components
//!
//! - [`AxumOtelSpanCreator`] - Creates spans for each request with relevant HTTP information
//! - [`AxumOtelOnResponse`] - Records response status and latency
//! - [`AxumOtelOnFailure`] - Handles error cases and updates span status
//!
//! ## HTTP span attributes
//!
//! Field names follow the [OpenTelemetry HTTP traces](https://opentelemetry.io/docs/specs/semconv/http/http-spans/)
//! semantic conventions where applicable. If you upgrade from older releases, update queries and dashboards:
//!
//! | Previous attribute | Replacement |
//! |--------------------|-------------|
//! | `http.host` | `server.address` |
//! | `http.user_agent` | `user_agent.original` |
//! | `http.client_ip` | `client.address` |
//!
//! `http.response.status_code` is recorded as an **integer** when the response is sent ([`AxumOtelOnResponse`]).
//!
//! See the [examples](https://github.com/nivek3/otel-kit/tree/main/examples) directory for complete examples.
//!
mod make_span;
mod on_failure;
mod on_response;

// Exports for the tower-http::trace::TraceLayer based middleware
pub use make_span::AxumOtelSpanCreator;
pub use on_failure::AxumOtelOnFailure;
pub use on_response::AxumOtelOnResponse;

// Re-export the Level enum from tracing crate
pub use tracing::Level;
