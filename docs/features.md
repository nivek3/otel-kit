# Cargo features

`otel-http` and `otel-logger` have no default features. HTTP callers select the
helpers they use; Logger itself is available without any feature flags.

## `otel-http`

| Feature | Adds | Enables |
| --- | --- | --- |
| `fields` | Request/header extraction | `dep:http` only |
| `macros` | Root `dyn_span!` and `dyn_event!` macros | `dep:tracing` |
| `http` | `propagation` context extraction and injection | `fields`, `dep:http`, `dep:opentelemetry`, `dep:opentelemetry-http` |
| `context` | Current trace/span IDs and remote-parent attachment | `http`, `dep:tracing`, `dep:tracing-opentelemetry` |
| `span` | Shared HTTP server span creation | `context`, `macros` |

`dep:` denotes an optional dependency, not another crate feature. In particular,
`fields` does not enable the `http` propagation feature.

`axum-otel` explicitly enables `macros` and `span`. Applications importing
`otel_http` themselves must also declare that direct dependency and the features
they use. `otel-http` does not depend on `otel-init` or `otel-logger`.

## `otel-logger`

| Feature | Adds | Enables |
| --- | --- | --- |
| `env` | `Logger::from_env`, `init_logger_from_env`, `init_logging_from_env` | `dep:config` |

Logger always depends on `otel-init` with `subscriber`. The old `logger`, `otel`,
and implicit `opentelemetry-otlp` features are removed. Exporter construction in
application code requires a direct exporter dependency.

Dependency examples for the upcoming split release:

```toml
# Shared HTTP helpers.
otel-http = { version = "0.34", features = ["span"] }

# Code-configured application bootstrap.
otel-logger = "0.34"
```

For environment configuration, replace the Logger dependency with:

```toml
otel-logger = { version = "0.34", features = ["env"] }
```

Applications using both sets of APIs must declare both crates. Workspace
entries do not enable features implicitly; each member selects its own.

## `otel-init`

| Feature | Adds |
| --- | --- |
| `subscriber` | `BoxLayer`, `init_env_filter`, `init_tracing_subscriber`, and the OpenTelemetry log bridge |

Direct subscriber users should explicitly enable `subscriber` rather than rely
on another dependency enabling it through Cargo feature unification.

## Contributor checks

Check packages independently so workspace feature unification cannot hide a
missing feature:

```sh
cargo check -p otel-http --no-default-features
cargo check -p otel-http --no-default-features --features fields
cargo check -p otel-http --no-default-features --features macros
cargo check -p otel-http --no-default-features --features http
cargo check -p otel-http --no-default-features --features context
cargo check -p otel-http --no-default-features --features span
cargo check -p otel-logger --no-default-features
cargo check -p otel-logger --no-default-features --features env
```
