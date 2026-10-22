# Logger configuration

`otel_logger::Logger` is the opinionated application bootstrap. It creates the
configured console and file layers, initializes trace and metric providers,
optionally initializes an OpenTelemetry log provider, and installs the global
tracing subscriber.

## Builder configuration

```rust,no_run
use tracing::Level;
use otel_logger::{LogFormat, Logger};

# fn run() -> anyhow::Result<()> {
let _guard = Logger::new("payments-api")
    .with_format(LogFormat::Json)
    .with_level(Level::INFO)
    .with_ansi(false)
    .with_console_enabled(true)
    .init()?;
# Ok(())
# }
```

`Logger::default()` uses instrumentation scope `otel_logger`, compact output,
ANSI enabled, `INFO`, console output
enabled, no file appender, and OpenTelemetry log export disabled. Sampling and
metric intervals use SDK environment settings or SDK defaults
(`parentbased_always_on` and 60 seconds). Environment deserialization
uses `false` for an omitted `LOG_ANSI`, so set it explicitly when switching
between builder and environment configuration. `RUST_LOG`, when valid, takes
precedence over the logger level because subscriber setup reads the standard
environment filter first.

## Environment mapping

`Logger::from_env(Some("LOG"))` uses `LOG` as the prefix. Passing `None` also
defaults to `LOG`.

| Environment variable | Logger field | Example |
| --- | --- | --- |
| `LOG_SERVICE_NAME` | `service_name` (tracer instrumentation scope) | `orders-api` |
| `LOG_FORMAT` | `format` | `compact`, `pretty`, or `json` |
| `LOG_SPAN_EVENTS` | `span_events` | `FMT::NEW|FMT::CLOSE` |
| `LOG_ANSI` | `ansi` | `false` |
| `LOG_LEVEL` | `level` | `info` |
| `LOG_ATTRIBUTES` | resource attributes | `deployment.environment=prod` |
| `LOG_CONSOLE_ENABLED` | console layer | `true` |
| `LOG_OTEL_LOGS_ENABLED` | OTLP log provider/layer | `true` |

File settings use the `LOG_FILE` prefix:

| Environment variable | Meaning |
| --- | --- |
| `LOG_FILE_ENABLE` | Enable the file layer |
| `LOG_FILE_NON_BLOCKING` | Use the non-blocking writer |
| `LOG_FILE_LEVEL` | Optional file-only level filter; further restricts the global filter |
| `LOG_FILE_ANSI` | ANSI in file output |
| `LOG_FILE_FORMAT` | Optional file-specific format; otherwise inherits Logger's format at initialization |
| `LOG_FILE_ROTATION` | `minutely`, `hourly`, `daily`, or `never` |
| `LOG_FILE_DIR` | Output directory |
| `LOG_FILE_FILENAME_PREFIX` | Filename prefix |
| `LOG_FILE_FILENAME_SUFFIX` | Filename suffix |
| `LOG_FILE_MAX_LOG_FILES` | Maximum retained files |

With no `LOG_FILE_*` variables, no file appender is configured. If any are
present, `LOG_FILE_ENABLE` is required and the complete file configuration must
be valid, even when disabled. Invalid values return an error instead of silently
disabling file output.

An omitted file format inherits the Logger format when output layers are
created, for both builder and environment configuration. Changing the Logger
format after `from_env` also applies to the file unless it has an explicit format.
An omitted file level adds no filter beyond the global `RUST_LOG` or Logger
level. An explicit file level can only further restrict that global filter;
it cannot restore events already excluded globally or change console filtering.
For example, `RUST_LOG=debug` with `LOG_FILE_LEVEL=warn` keeps debug events on the
console while writing only warn/error events to the file.

Environment-based configuration requires the `env` feature. Calling
`Logger::new(...)` does not read `LOG_*`; use `Logger::from_env(...)` or
`init_logging_from_env(...)` when those variables should apply.

Sampling uses only native `OTEL_TRACES_SAMPLER` / `OTEL_TRACES_SAMPLER_ARG`, or
the SDK default `parentbased_always_on`. Metrics use `OTEL_METRIC_EXPORT_INTERVAL`
in milliseconds, defaulting to 60 seconds. Neither setting has a Logger field or
`LOG_*` fallback. `Logger::service_name` and `LOG_SERVICE_NAME` only configure
the tracer's instrumentation scope. Resource `service.name` uses SDK defaults;
set `OTEL_SERVICE_NAME` or provide explicit resource attributes to configure it.
These native settings also work without the `env` feature. See
[precedence details](otlp.md#environment-variable-precedence).
