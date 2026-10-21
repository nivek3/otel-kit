# Crates

`otel-kit` is the repository for four crates, not an all-inclusive package.

| Crate | Responsibility | Use it when |
| --- | --- | --- |
| `otel-init` | Resources, OTLP providers, subscriber setup, and provider shutdown | You own the subscriber or need provider-level initialization |
| `otel-http` | Shared inbound HTTP server spans, fields, context propagation, and dynamic macros | You need framework-independent HTTP helpers |
| `otel-logger` | Application logging and bootstrap for one process | You want the opinionated `Logger` facade |
| `axum-otel` | Axum-specific `TraceLayer` callbacks | You need inbound Axum request spans |

The dependency direction is:

```text
axum-otel -> otel-http
applications -> otel-logger -> otel-init
```

`otel-http` depends on neither `otel-init` nor `otel-logger`. Logger depends on
`otel-init` with `subscriber`, and does not depend on `otel-http`. Applications
can combine the crates or supply their own subscriber.

## Public entry points

### `otel-init`

- `get_resource`
- `init_tracer_provider`, `init_meter_provider`, `init_logger_provider`
- `OtelGuard`
- `BoxLayer`, `init_env_filter`, `init_tracing_subscriber` with `subscriber`
- Existing `opentelemetry`, `opentelemetry_sdk`, `tracing_opentelemetry` re-exports

### `otel-http`

- `fields`, `propagation`, `context`, `span` modules at the crate root
- Root `dyn_span!` and `dyn_event!` macros, with the `macros` feature
- No nested `http` module or `extract` aliases

### `otel-logger`

- `Logger`, `LoggerGuard`, `LoggerFileAppender`, `LogFormat`, `LogRollingRotation`
- `FmtSpan`, `default`
- `init_logging`, `init_tracing_from_logger`, `setup_tracing`, `init_layer`
- Explicit `OtelGuard`, `BoxLayer`, `init_env_filter` re-exports from `otel-init`
- `Logger::from_env`, `init_logger_from_env`, `init_logging_from_env` with `env`

These entries are at the crate root. There is no nested `logger` or `otel`
module and no blanket forwarding of `otel-init` or SDK crates.

### `axum-otel`

- `AxumOtelSpanCreator`
- `AxumOtelOnResponse`
- `AxumOtelOnFailure`
- `Level` re-export from `tracing`

See [features](features.md) for dependency choices and
[migration](migration.md#splitting-tracing-otel) for the upcoming package split.
