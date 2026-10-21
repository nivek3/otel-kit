# otel-kit

`otel-kit` is the repository for four Rust crates that cover
different observability boundaries:

- `otel-init` initializes OpenTelemetry providers and tracing subscribers;
- `otel-http` provides shared inbound HTTP server spans and context propagation;
- `otel-logger` provides application logging and bootstrap for one process;
- `axum-otel` adapts the shared HTTP span behavior to Axum and
`tower-http::TraceLayer`.

Choose the smallest crate that owns the behavior your application needs. An
Axum application can use `axum-otel` with its own subscriber, while an
application that wants the repository's complete logging setup can combine it
with `otel-logger`, adding `env` for environment configuration. Provider-only
users can depend directly on `otel-init`.

The workspace version is `0.34.0`; the package split is for the upcoming release.
`Cargo.toml` declares Rust 1.96.0 as the minimum supported Rust version. The repository toolchain may
pin a newer compiler for local development.

## Where to start

- [Getting started](getting-started.md) — install the right crates and build a
minimal application.
- [Crates](crates.md) — choose the smallest crate for each observability
responsibility.
- [Cargo features](features.md) — keep optional dependencies and capabilities
explicit.
- [Comparison with](comparison.md) `axum-tracing-opentelemetry` — choose between
focused request middleware and this workspace's broader observability setup.
- [OTLP export and lifecycle](otlp.md) — configure exporters and shut providers
down safely.
- [HTTP span attributes](http-spans.md) — understand emitted fields before
writing dashboards or alerts.

The complete type-level references are on
[docs.rs for](https://docs.rs/otel-http) `otel-http`,
[docs.rs for](https://docs.rs/otel-logger) `otel-logger`,
[docs.rs for](https://docs.rs/otel-init) `otel-init`, and
[docs.rs for](https://docs.rs/axum-otel) `axum-otel`.

The new crate references become available when the split packages are published.
