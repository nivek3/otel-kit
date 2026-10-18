# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Fixed

- Let the OTLP metric exporter read `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` instead of overriding it with the SDK default.

### Changed

- Rely on the SDK's default random trace ID generator.
- Delegate trace, metric, and log exporter protocol selection to native OpenTelemetry builders; remove the custom exporter macro and protocol parser.
- Reuse the OTLP crate's endpoint constants instead of defining duplicates.

### Breaking Changes

- Remove the sample-ratio argument from `init_tracer_provider`; sampling now uses only `OTEL_TRACES_SAMPLER` / `OTEL_TRACES_SAMPLER_ARG` or the SDK default `parentbased_always_on`.
- Remove the interval argument from `init_meter_provider` and use the SDK's `with_periodic_exporter`. The interval now uses only `OTEL_METRIC_EXPORT_INTERVAL` (milliseconds) or the SDK default of 60 seconds, replacing the Logger default of 30 seconds.
- Prefer native service-name environment settings over code configuration. The service-name argument remains a fallback; non-service resource attributes retain their existing precedence.
- Remove the public `otel_init::macros` module and `protocol_for_signal` helper. Use the native OTLP exporter builders for environment-based protocol selection.
- With no valid protocol environment setting, the enabled features now select `http/json` instead of `grpc`. Set `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` explicitly to retain gRPC.
- Protocol values now follow native parsing: `grpc`, `http/protobuf`, and `http/json`. The `http/proto` alias, case normalization, and whitespace trimming are no longer supported; invalid values fall through to the next configuration source.

## [0.33.1]

### Added

- Publish `otel-init` `0.33.1` as the successor to `tracing-opentelemetry-extra` `0.33.0`. These are separate crates.io packages; Cargo does not migrate dependencies automatically.

### Migration

- Replace `tracing-opentelemetry-extra = "0.33.0"` with `otel-init = "0.33.1"` in `Cargo.toml`.
- Replace Rust imports from `tracing_opentelemetry_extra` with `otel_init`.
- The old `tracing-opentelemetry-extra` package remains available at `0.33.0` until it is yanked after the successor is published.

## Predecessor history

The entries below were published under the `tracing-opentelemetry-extra` package name.

## [0.33.0](https://github.com/nivek3/otel-kit/compare/tracing-opentelemetry-extra-v0.32.1...tracing-opentelemetry-extra-v0.33.0)

### ⚠️ Breaking Changes

- Upgrade OpenTelemetry dependencies to `0.32` and `tracing-opentelemetry` to `0.33`. Downstream crates must use matching OpenTelemetry versions.
- Remove `Clone` from `OtelGuard` so dropping a duplicate cannot shut down shared OpenTelemetry providers early.

## [0.31.8](https://github.com/nivek3/otel-kit/compare/tracing-opentelemetry-extra-v0.31.7...tracing-opentelemetry-extra-v0.31.8)




### Fixed


- *(opentelemetry)* Skip OTLP exporters when no endpoint is configured ([#21](https://github.com/nivek3/otel-kit/pull/21)) - ([4e614fe](https://github.com/nivek3/otel-kit/commit/4e614fe45568da3682ce2e4db4eb32a7317b3dd8))

### 🐛 Bug Fixes

- Initialize local OpenTelemetry providers without OTLP exporters when no endpoint is configured or endpoint env vars are empty, avoiding default localhost:4317 connection attempts.

## [0.31.5](https://github.com/nivek3/otel-kit/compare/tracing-opentelemetry-extra-v0.31.4...tracing-opentelemetry-extra-v0.31.5)




### 🚜 Refactor

- Reorganize imports and simplify shutdown logic in tracing modules - ([0f95108](https://github.com/nivek3/otel-kit/commit/0f951082ae571380fef1c626855271d1ab74794a))


### ⚙️ Miscellaneous Tasks

- Update workspace dependencies and enhance CI configuration - ([244742d](https://github.com/nivek3/otel-kit/commit/244742d220816d3750abfd67175be04bacd057da))

- Update dependencies and refactor configuration handling - ([576ba88](https://github.com/nivek3/otel-kit/commit/576ba887424fc684aaea33a92cfc60debe36a521))

- Update dependencies and improve CI workflow - ([7b4410b](https://github.com/nivek3/otel-kit/commit/7b4410b5b8fc295e2b14c3f752b0a99d3753bb44))
