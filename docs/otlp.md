# OTLP export and lifecycle

`otel-init` creates trace, metric, and log providers. Export is opt-in through
an endpoint environment variable.

```sh
# Enable all configured signals.
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
```

Signal-specific endpoints can enable only one signal or override the shared
endpoint for that signal:

```sh
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4318/v1/traces
export OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=http/protobuf
```

The corresponding metric and log names are
`OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` / `_PROTOCOL` and
`OTEL_EXPORTER_OTLP_LOGS_ENDPOINT` / `_PROTOCOL`. Supported protocol values in
this workspace are `grpc`, `http/protobuf`, and `http/json`, using exact
lowercase values without surrounding whitespace. Native exporter builders
resolve the signal-specific protocol first, then `OTEL_EXPORTER_OTLP_PROTOCOL`,
then the SDK feature default; invalid values fall through to the next source.
With the enabled features, the default is `http/json`. For a gRPC receiver,
set `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` explicitly.

When neither the signal-specific endpoint nor the shared endpoint contains a
non-blank value, that provider remains local-only. It can still create trace
IDs, but it does not construct an exporter or attempt a connection to
`localhost:4317`.

`Logger` always initializes trace and metric providers. It initializes the log
provider only when `otel_logs_enabled` is true, for example with
`LOG_OTEL_LOGS_ENABLED=true` through the `env` feature.

## Environment variable precedence

The OTLP exporters read standard endpoint, header, and timeout variables from
the process environment. The metric exporter also reads
`OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` (`cumulative`, `delta`, or
`lowmemory`); when unset, the SDK default is cumulative.

Sampling and metric intervals are managed entirely by the SDK. Native
service-name settings take priority over code and `LOG_*` fallbacks:

| SDK environment variable | Behavior in this library |
| --- | --- |
| `OTEL_TRACES_SAMPLER`, `OTEL_TRACES_SAMPLER_ARG` | Configured entirely by the SDK. The default is `parentbased_always_on`: sample root spans and honor the parent's sampling decision for child spans. Setting only the argument does not select a sampler. |
| `OTEL_METRIC_EXPORT_INTERVAL` | Configured entirely by the SDK, in milliseconds. The default is `60000` (60 seconds). |
| `OTEL_SERVICE_NAME`, `OTEL_RESOURCE_ATTRIBUTES` | Service-name priority: non-empty `OTEL_SERVICE_NAME`, then `service.name` in `OTEL_RESOURCE_ATTRIBUTES`, then explicit `service.name` attributes, then the `get_resource` argument. Other explicit attributes still override environment attributes. |
| `RUST_LOG` | Read by `tracing-subscriber::EnvFilter`; a missing or invalid value falls back to the configured log level. |

For example:

```sh
export OTEL_SERVICE_NAME=orders-api
export OTEL_TRACES_SAMPLER=parentbased_traceidratio
export OTEL_TRACES_SAMPLER_ARG=0.1
export OTEL_METRIC_EXPORT_INTERVAL=30000
```

An empty or invalid sampler setting is handled by the SDK, which falls back to
`parentbased_always_on`. An empty or non-numeric metric interval uses the SDK's
60-second default. These settings apply when providers are initialized, not
dynamically. The Logger service-name field retains its configured fallback value.

`LOG_*` variables configure this library's `Logger`, including console output,
formatting, and file rotation. They require the `env` feature and
`Logger::from_env(None)` (or `init_logging_from_env(None)`). The OpenTelemetry
SDK does not replace this configuration layer.

Environment support does not load a `.env` file. The examples call
`dotenvy::dotenv()` before initialization to load that file into the process
environment.

## Shutdown order

Keep `LoggerGuard` or `OtelGuard` alive until application shutdown. Explicit
shutdown is available when the application needs to surface provider shutdown
errors:

```rust,no_run
# use tracing_otel::Logger;
# fn run() -> anyhow::Result<()> {
let guard = Logger::new("worker").init()?;
tracing::info!("draining work");
guard.shutdown()?;
# Ok(())
# }
```

`LoggerGuard` shuts down telemetry providers before releasing its optional
non-blocking file writer guard. Dropping a guard also performs cleanup, but an
explicit `shutdown()` is the path that returns provider shutdown errors.
