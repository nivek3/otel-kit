# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking Changes

- Upgrade HTTP tracing to OpenTelemetry `0.33` and `tracing-opentelemetry` `0.34`. Applications that integrate OpenTelemetry directly must use compatible versions.

## [0.33.1](https://github.com/nivek-ph/otel-kit/compare/axum-otel-v0.33.0...axum-otel-v0.33.1)

### Changed

- Replace the internal `tracing-otel-extra` dependency with its successor, `tracing-otel`. Applications that also depend on the shared tracing crate directly must replace `tracing-otel-extra = "0.33.0"` with `tracing-otel = "0.33.1"` and update Rust imports from `tracing_otel_extra` to `tracing_otel`.

## [0.33.0](https://github.com/nivek-ph/otel-kit/compare/axum-otel-v0.32.1...axum-otel-v0.33.0)

### ⚠️ Breaking Changes

- Upgrade OpenTelemetry dependencies to `0.32` and `tracing-opentelemetry` to `0.33`. Downstream crates must use matching OpenTelemetry versions.

### Changed

- Delegate shared HTTP server span creation to `tracing-otel-extra`, leaving `AxumOtelSpanCreator` responsible only for Axum-specific enrichment.

## [0.31.7](https://github.com/nivek-ph/otel-kit/compare/axum-otel-v0.31.6...axum-otel-v0.31.7)




### Fixed

- Align HTTP spans with latest OpenTelemetry semantics ([#19](https://github.com/nivek-ph/otel-kit/pull/19)) - ([c5acf3f](https://github.com/nivek-ph/otel-kit/commit/c5acf3f3c0ad4584431733264acef245506a51bc))


### ⚠️ Breaking Changes

- HTTP span attributes were renamed to align with [OpenTelemetry HTTP semantic conventions](https://opentelemetry.io/docs/specs/semconv/http/http-spans/): `http.host` → `server.address`, `http.user_agent` → `user_agent.original`, `http.client_ip` → `client.address` (when `[ConnectInfo](https://docs.rs/axum/latest/axum/extract/struct.ConnectInfo.html)` is present). Update dashboards, alerts, and sampling rules that referenced the old attribute keys.
- `http.response.status_code` is recorded as an integer (OpenTelemetry `int`), not a string.

### 📚 Documentation

- Expand crate-level documentation with attribute migration and links to the OpenTelemetry HTTP spec.

## [0.31.5](https://github.com/nivek-ph/otel-kit/compare/axum-otel-v0.31.4...axum-otel-v0.31.5)

### 🚜 Refactor

- Reorganize imports and simplify shutdown logic in tracing modules - ([0f95108](https://github.com/nivek-ph/otel-kit/commit/0f951082ae571380fef1c626855271d1ab74794a))

### ⚙️ Miscellaneous Tasks

- Update workspace dependencies and enhance CI configuration - ([244742d](https://github.com/nivek-ph/otel-kit/commit/244742d220816d3750abfd67175be04bacd057da))
