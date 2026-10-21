# otel-logger

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE-MIT)

A tracing and OpenTelemetry integration utility library for Rust applications, providing easy-to-use configuration and initialization capabilities.

`otel-logger` contains the application logging facade split from `tracing-otel` in the upcoming release. HTTP helpers are in `otel-http`. See the [migration guide](../../docs/migration.md#splitting-tracing-otel) and [changelog](CHANGELOG.md).

## Features

- **Easy to Use** - Simple configuration of tracing and OpenTelemetry through Builder pattern
- **Multiple Output Formats** - Support for Compact, Pretty, and JSON formats
- **Flexible Configuration** - Configure log levels through Logger; sampling and metric export intervals use the SDK environment settings `OTEL_TRACES_SAMPLER` and `OTEL_METRIC_EXPORT_INTERVAL`.
- **Automatic Resource Cleanup** - Automatic management of TracerProvider and MeterProvider through RAII pattern
- **Built-in Metrics Support** - Integrated OpenTelemetry metrics collection and export
- **Environment Detection** - Automatic detection of operating system and process information
- **OTLP Export** - Built-in OTLP protocol support, can directly export to Jaeger, OTEL Collector, etc.

## Crate Scope

`Logger` configures console/file logging, a tracing subscriber, trace and metric
providers, and optional OTLP logs for one process. It is available without any
feature flags. Enable `env` for `Logger::from_env` and environment helpers.

`otel-logger` depends on `otel-init` with `subscriber`, and does not depend on
`otel-http`. Provider-only users can depend directly on `otel-init`.

Public bootstrap helpers (`setup_tracing`, `init_layer`, `init_env_filter`,
`init_tracing_from_logger`), `default`, `OtelGuard`, and `BoxLayer` are available
at the crate root. SDK crates are no longer re-exported; use their direct
imports or the existing re-exports in `otel-init`.

## Quick Start

Dependency configuration for the upcoming release:

```toml
[dependencies]
otel-logger = "0.34"
opentelemetry = "0.33"
tracing = "0.1"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1"
```

### Basic Usage

```rust
use otel_logger::Logger;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with default configuration
    let _guard = Logger::default().init()?;
    
    info!("Application started");
    warn!("This is a warning message");
    
    // _guard automatically cleans up resources when it goes out of scope
    Ok(())
}
```

### Advanced Configuration

```rust
use otel_logger::{Logger, LogFormat};
use opentelemetry::KeyValue;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = Logger::new("my-awesome-service")
        .with_format(LogFormat::Json)           // Use JSON format
        .with_level(Level::DEBUG)               // Set log level
        .with_ansi(false)                       // Disable ANSI colors
        .with_attributes(vec![                  // Add custom attributes
            KeyValue::new("environment", "production"),
            KeyValue::new("version", "1.2.3"),
        ])
        .init()?;
    
    // Your application code
    tracing::info!(
        user_id = 12345,
        action = "login",
        "User logged in successfully"
    );
    
    Ok(())
}
```

### Legacy API (Backward Compatibility)

```rust
use otel_logger::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_logging("legacy-service")?;
    
    tracing::info!("Initialized using legacy API");
    
    Ok(())
}
```

## Configuration Options

| Option                  | Type            | Default    | Description                                            |
| ----------------------- | --------------- | ---------- | ------------------------------------------------------ |
| `service_name`          | `String`        | `otel_logger` | Tracer instrumentation scope name; does not set resource `service.name` |
| `format`                | `LogFormat`     | `Compact`  | Log output format: `Compact`, `Pretty`, `Json`         |
| `ansi`                  | `bool`          | `true`     | Whether to enable ANSI color output                    |
| `level`                 | `Level`         | `INFO`     | Log level filtering                                    |
| `attributes`            | `Vec<KeyValue>` | `[]`       | Custom OpenTelemetry attributes                        |
| `otel_logs_enabled`     | `bool`          | `false`    | Whether to enable OpenTelemetry logs export            |

## Environment Variable Configuration

Enable environment-based Logger configuration with:

```toml
otel-logger = { version = "0.34", features = ["env"] }
```

This library supports standard OpenTelemetry environment variables:

```bash
# OTLP export endpoint
# If no OTLP endpoint is configured, or it is configured as an empty string,
# tracing and metrics providers are still initialized locally so request spans
# can produce trace IDs, but nothing is exported and no connection to
# localhost:4317 is attempted.
#
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
# HTTP OTLP options:
# export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
# export OTEL_EXPORTER_OTLP_PROTOCOL=http/json
#
# Specify Protocol for Traces and Metrics:
# export OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=http/protobuf
# export OTEL_EXPORTER_OTLP_METRICS_PROTOCOL=http/json
#
# Without protocol env vars, the native SDK feature default is http/json.
# Set grpc explicitly when using a gRPC receiver (typically port 4317).

# Log level (takes precedence over code configuration)
export RUST_LOG=debug

# Resource attributes
export OTEL_SERVICE_NAME=my-service
export OTEL_RESOURCE_ATTRIBUTES=service.version=1.0.0
```

`LOG_*` settings require the `env` feature and `Logger::from_env(None)`; they are
library-specific settings, not OpenTelemetry SDK variables. See
[environment variable precedence](../../docs/otlp.md#environment-variable-precedence)
for SDK resource detection, explicit attribute overrides, and `.env` loading.

## Integration with Axum

Use with `axum-otel` to achieve complete web service observability:

```rust
use axum::{routing::get, Router};
use axum_otel::{AxumOtelSpanCreator, AxumOtelOnResponse, Level};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use otel_logger::{Logger, LogFormat};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let _guard = Logger::new("web-service")
        .with_format(LogFormat::Json)
        .init()?;
    
    // Build Axum application
    let app = Router::new()
        .route("/api/health", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                        .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                )
        );
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
```

## Resource Cleanup

`LoggerGuard` implements the RAII pattern and automatically shuts down OpenTelemetry providers, then releases non-blocking file writer guards to trigger their shutdown and flush path. The writer does not report the flush outcome:

```rust
{
    let _guard = Logger::new("temp-service").init()?;
    // Use tracing
    tracing::info!("Temporary service started");
} // <- guard automatically cleans up resources here

// Manual cleanup is also possible
let guard = Logger::new("manual-cleanup").init()?;
// ... use tracing
guard.shutdown()?; // Manual cleanup
```

## Requirements

- **Rust Version**: 1.96.0+
- **Tokio Runtime**: Requires tokio async runtime
- **OTLP Receiver**: Need to configure an OTLP-compatible receiver (such as Jaeger, OTEL Collector)

## Troubleshooting

### Common Issues

1. **Failed to connect to OTLP receiver**
   ```bash
   # Check endpoint configuration
   export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
   export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
   
   # Quickly start Jaeger using Docker
   docker run -d --name jaeger \
     -p 16686:16686 \
     -p 14250:14250 \
     -p 4317:4317 \
     jaegertracing/all-in-one:latest
   ```

2. **Log level filtering not working**
   ```bash
   # Ensure environment variable is set correctly
   export RUST_LOG=debug
   # Or explicitly set level in code
   .with_level(Level::DEBUG)
   ```

3. **Metrics collection issues**
   ```bash
   # Collect every 10 seconds; the SDK uses milliseconds.
   export OTEL_METRIC_EXPORT_INTERVAL=10000
   ```

## License

Licensed under MIT or Apache-2.0; see [MIT](../../LICENSE-MIT) and
[Apache-2.0](../../LICENSE-APACHE).

## Contributing

Issues and Pull Requests are welcome! Please ensure:

1. Code passes all tests
2. Add appropriate documentation
3. Follow the project's code style

## Related Projects

- [axum-otel](../axum-otel) - OpenTelemetry middleware for Axum Web framework
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust) - OpenTelemetry Rust implementation
- [tracing](https://github.com/tokio-rs/tracing) - Structured logging framework for Rust
