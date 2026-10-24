# Logger configuration

`otel_logger::Logger` is the opinionated application bootstrap. It creates the
shared formatting layer for the configured console and file outputs, initializes trace and metric providers,
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
| `LOG_CONSOLE_ENABLED` | console output | `true` |
| `LOG_OTEL_LOGS_ENABLED` | OTLP log provider/layer | `true` |

File settings use the `LOG_FILE` prefix:

| Environment variable | Meaning |
| --- | --- |
| `LOG_FILE_ENABLE` | Enable file output |
| `LOG_FILE_NON_BLOCKING` | Use the non-blocking writer |
| `LOG_FILE_ROTATION` | `minutely`, `hourly`, `daily`, or `never` |
| `LOG_FILE_DIR` | Output directory |
| `LOG_FILE_FILENAME_PREFIX` | Filename prefix |
| `LOG_FILE_FILENAME_SUFFIX` | Filename suffix |
| `LOG_FILE_MAX_LOG_FILES` | Maximum retained files |

With no `LOG_FILE_*` variables, no file appender is configured. If any are
present, `LOG_FILE_ENABLE` is required and the complete file configuration must
be valid, even when disabled. Invalid values return an error instead of silently
disabling file output.

Console and file output share one formatting layer, so both receive the same
formatted bytes, ANSI setting, span events, and global filter. Use `LOG_FORMAT`,
`LOG_ANSI`, and `LOG_LEVEL` (or `RUST_LOG`) to configure both. Set `LOG_ANSI=false`
when text files should not contain terminal colors; JSON always disables ANSI.
Changing the Logger after `from_env` changes both outputs at initialization.
The global filter applies to spans as well as events: `RUST_LOG=warn` excludes
INFO request spans, so neither output includes their fields on WARN events.

The removed `LOG_FILE_FORMAT`, `LOG_FILE_LEVEL`, and `LOG_FILE_ANSI` settings are
rejected as unknown file settings. Remove them and use the shared settings above.
The corresponding `LoggerFileAppender` fields and the `merge_with_logger` /
`format_or_default` helpers have also been removed. File configuration now only
controls enabling, non-blocking writes, rotation, path, filename, and retention.

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
