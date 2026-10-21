# Getting started

The workspace version is `0.34.0`. The `otel-http` / `otel-logger` split is
for the upcoming release; the dependency snippets describe that target.
`Logger` is available without features; HTTP helpers require explicit features.

## Application logging

Use `otel-logger` for code-based configuration:

```toml
[dependencies]
otel-logger = "0.34"
anyhow = "1"
tracing = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust,no_run
use otel_logger::{LogFormat, Logger};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = Logger::new("inventory-api")
        .with_format(LogFormat::Json)
        .with_ansi(false)
        .init()?;

    tracing::info!(component = "startup", "service ready");
    Ok(())
}
```

Keep the returned `LoggerGuard` alive for the whole application. Dropping it
shuts down the OpenTelemetry providers and then releases any non-blocking file
writer guard.

Set `OTEL_SERVICE_NAME=inventory-api` to name the resource. The argument to
`Logger::new(...)` only names the tracer's instrumentation scope; resource
attributes otherwise use [SDK defaults](otlp.md#environment-variable-precedence).

## Environment-based configuration

Enable `env` when Logger configuration should come from environment variables:

```toml
[dependencies]
otel-logger = { version = "0.34", features = ["env"] }
```

```rust,no_run
use otel_logger::Logger;

# fn run() -> anyhow::Result<()> {
let _guard = Logger::from_env(Some("LOG"))?.init()?;
# Ok(())
# }
```

With the `LOG` prefix, fields map to names such as `LOG_SERVICE_NAME`,
`LOG_FORMAT`, and `LOG_CONSOLE_ENABLED`. See [Logger configuration](logger.md) for
the complete mapping used by the repository.

## Axum middleware

`axum-otel` supplies the request-span callbacks; it does not replace subscriber
initialization:

```toml
[dependencies]
axum-otel = "0.34"
otel-logger = "0.34"
anyhow = "1"
axum = "0.8"
tokio = { version = "1", features = ["macros", "net", "rt-multi-thread"] }
tower-http = { version = "0.6", features = ["trace"] }
```

Continue with [Axum request tracing](axum.md) for a complete layer setup.
