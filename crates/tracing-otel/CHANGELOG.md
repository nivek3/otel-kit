# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Changed

- Re-export `otel-init::init_env_filter` instead of duplicating `RUST_LOG` parsing; existing public paths and fallback behavior are preserved.
- Reuse the environment configuration reader for file-appender settings.
- Native OpenTelemetry service-name settings now take priority over Logger fields and `LOG_*` values at initialization.
- Use tracing-subscriber's native JSON layer configuration (including disabling ANSI escapes) and Serde's derived log-format parser, preserving case-insensitive values and optional empty values. Invalid formats now use Serde's standard error messages.
- Re-export the standard `http::header::FORWARDED` constant instead of defining a duplicate.

### Fixed

- Update README feature requirements and builder examples, and document native environment priority and code configuration fallbacks.

### Breaking Changes

- Remove `Logger::sample_ratio`, `with_sample_ratio`, `default::sample_ratio`, and the sample-ratio argument to `setup_tracing`. `LOG_SAMPLE_RATIO` no longer configures sampling. Use `OTEL_TRACES_SAMPLER=parentbased_traceidratio` and `OTEL_TRACES_SAMPLER_ARG` instead; without configuration, the SDK uses `parentbased_always_on`.
- Remove `Logger::metrics_interval_secs`, `with_metrics_interval_secs`, `default::metrics_interval_secs`, and the interval argument to `setup_tracing`. `LOG_METRICS_INTERVAL_SECS` no longer configures collection. Use `OTEL_METRIC_EXPORT_INTERVAL` in milliseconds; the SDK default is 60 seconds instead of the previous Logger default of 30 seconds.

## [0.33.1]

### Added

- Publish `tracing-otel` `0.33.1` as the successor to `tracing-otel-extra` `0.33.0`. These are separate crates.io packages; Cargo does not migrate dependencies automatically.

### Migration

- Replace `tracing-otel-extra = "0.33.0"` with `tracing-otel = "0.33.1"` in `Cargo.toml`.
- Replace Rust imports from `tracing_otel_extra` with `tracing_otel`.
- The old `tracing-otel-extra` package remains available at `0.33.0` until it is yanked after the successor is published.

## Predecessor history

The entries below were published under the `tracing-otel-extra` package name.

## [0.33.0](https://github.com/nivek3/otel-kit/compare/tracing-otel-extra-v0.32.1...tracing-otel-extra-v0.33.0)

### ⚠️ Breaking Changes

- Upgrade OpenTelemetry dependencies to `0.32` and `tracing-opentelemetry` to `0.33`. Downstream crates must use matching OpenTelemetry versions.
- Change `make_request_span` to accept a customization callback that runs before remote parent context is applied.
- Return `LoggerGuard` from `Logger::init`, `init_logging`, and `init_logging_from_env`.
- Remove the public `logger::create_output_layers` and `logger::set_nonblocking_appender_guard` APIs so writer ownership cannot escape the returned guard. Applications should initialize logging through `Logger::init`, `init_logging`, or `init_logging_from_env` and retain the returned `LoggerGuard` for the lifetime of logging.

### 🐛 Bug Fixes

- Keep the non-blocking file writer guard in `LoggerGuard` so dropping or shutting down the logger triggers tracing-appender's writer shutdown and flush path instead of leaving the guard in a process-level `OnceLock`; the writer does not report the flush outcome.

## [0.31.8](https://github.com/nivek3/otel-kit/compare/tracing-otel-extra-v0.31.7...tracing-otel-extra-v0.31.8)




### Fixed


- *(opentelemetry)* Skip OTLP exporters when no endpoint is configured ([#21](https://github.com/nivek3/otel-kit/pull/21)) - ([4e614fe](https://github.com/nivek3/otel-kit/commit/4e614fe45568da3682ce2e4db4eb32a7317b3dd8))


## [0.31.7](https://github.com/nivek3/otel-kit/compare/tracing-otel-extra-v0.31.6...tracing-otel-extra-v0.31.7)




### Fixed

- Align HTTP spans with latest OpenTelemetry semantics ([#19](https://github.com/nivek3/otel-kit/pull/19)) - ([c5acf3f](https://github.com/nivek3/otel-kit/commit/c5acf3f3c0ad4584431733264acef245506a51bc))


### ⚠️ Breaking Changes

- [`make_request_span`](https://docs.rs/tracing-otel-extra/latest/tracing_otel_extra/http/span/fn.make_request_span.html) emits the same OpenTelemetry-aligned HTTP attribute names as `axum-otel` (for example `server.address` and `user_agent.original` instead of `http.host` and `http.user_agent`). Update any code or tests that depend on the previous field names.

### 📚 Documentation

- Document HTTP span attributes in `http::span` and link to the OpenTelemetry HTTP spec and `axum-otel` migration notes; README mentions `make_request_span`.

### 🐛 Bug Fixes

- Allow logger initialization without an OTLP endpoint, or with empty endpoint env vars, by using local OpenTelemetry providers with no exporters.

- Change default environment variable prefix from `LOG_` to `LOG` to avoid double underscores in variable names (e.g., `LOG__SERVICE_NAME` → `LOG_SERVICE_NAME`)

## [0.31.5](https://github.com/nivek3/otel-kit/compare/tracing-otel-extra-v0.31.4...tracing-otel-extra-v0.31.5)


### 🚜 Refactor

- Reorganize imports and simplify shutdown logic in tracing modules - ([0f95108](https://github.com/nivek3/otel-kit/commit/0f951082ae571380fef1c626855271d1ab74794a))

- Reorder imports and enhance test safety in logger.rs - ([171c58d](https://github.com/nivek3/otel-kit/commit/171c58d07ba727b60cbead34d92804871cc52c2f))


### 📚 Documentation

- Add comprehensive AGENTS.md for tracing-otel-extra project - ([630e018](https://github.com/nivek3/otel-kit/commit/630e0182880560076c271134920c7f2529461947))


### ⚙️ Miscellaneous Tasks

- Update workspace dependencies and enhance CI configuration - ([244742d](https://github.com/nivek3/otel-kit/commit/244742d220816d3750abfd67175be04bacd057da))

- Update dependencies and refactor configuration handling - ([576ba88](https://github.com/nivek3/otel-kit/commit/576ba887424fc684aaea33a92cfc60debe36a521))
