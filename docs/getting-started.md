# Getting started

The workspace packages are version `0.33.1`. `tracing-otel` has no default
features, so applications must enable the capability they use.

## Application logging

Enable `logger` for code-based configuration:

```toml
[dependencies]
tracing-otel = { version = "0.33.1", features = ["logger"] }
anyhow = "1"
tracing = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust,no_run
use tracing_otel::{LogFormat, Logger};

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

## Environment-based configuration

Enable `env` when configuration should come from environment variables. It
includes `logger`:

```toml
[dependencies]
tracing-otel = { version = "0.33.1", features = ["env"] }
```

```rust,no_run
use tracing_otel::Logger;

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
axum-otel = "0.33.1"
tracing-otel = { version = "0.33.1", features = ["logger"] }
anyhow = "1"
axum = "0.8"
tokio = { version = "1", features = ["macros", "net", "rt-multi-thread"] }
tower-http = { version = "0.6", features = ["trace"] }
```

Continue with [Axum request tracing](axum.md) for a complete layer setup.
