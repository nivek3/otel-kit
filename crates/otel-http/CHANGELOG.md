# Changelog

## [Unreleased]

### Fixed

- Upgrade to `http-extract` 0.1.4 and directly re-export its Result-based request ID, singular header text, and URL scheme APIs. Remove the forwarding wrappers and local `Forwarded` proto parser; preserve errors for callers and omit invalid optional metadata only at span recording. Proxy schemes are now validated and normalized, with the first validated XFP protocol selected.
- Use `http-extract`'s authority extraction for URI/Host precedence and validation; split `server.address` from explicit integer `server.port`, without proxy-header handling or default-port inference.
- Skip parent attachment for disabled spans or missing OpenTelemetry layers without emitting warnings or recording invalid trace IDs.

## [0.34.1]

### Added

- Split the shared HTTP server span, field extraction, context propagation, and dynamic tracing macros out of `tracing-otel` into `otel-http`. HTTP modules are public at the crate root; the five HTTP features retain their existing dependency relationships and have no defaults.
- Retain the existing HTTP semantics, including the customization callback before parent attachment and the standard `http::header::FORWARDED` re-export.

### Fixed

- Inherit the workspace Rust 1.96 minimum in published package metadata.

- Apply the repository lint header, document public request-header constants, declare the test exporter feature explicitly, and list the complete feature dependencies in the README.

### Migration

- Replace `tracing_otel::http::*` with the corresponding `otel_http::*` modules and import dynamic macros from `otel_http`.
- Replace `tracing_otel::extract::http::*` with `otel_http::propagation::*`; other `extract` module aliases map directly to root modules. No `extract` or nested `http` module is provided.
- Remove the old implicit `opentelemetry-otlp` feature. The exporter is only a development dependency here; callers constructing exporters must depend on it directly.

This is a new package. Earlier HTTP API releases were published as
`tracing-otel` / `tracing-otel-extra`, not as `otel-http`.
