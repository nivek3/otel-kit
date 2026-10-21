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
//! Framework-independent HTTP server spans and context propagation.
//!
//! Enable `fields`, `macros`, `http`, `context`, or `span` for the APIs you use.
//! There are no default features. `span` includes the other four features.
//!
//! This crate provides utilities for extracting and managing trace context
//! in HTTP requests and responses.

#[cfg(feature = "context")]
pub mod context;
#[cfg(feature = "fields")]
pub mod fields;
#[cfg(feature = "http")]
pub mod propagation;
#[cfg(feature = "span")]
pub mod span;

#[cfg(feature = "macros")]
pub mod macros;
