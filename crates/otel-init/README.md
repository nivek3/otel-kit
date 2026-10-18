# otel-init

**Reference:** This crate is mainly organized based on the [official tracing-opentelemetry OTLP example](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs).

This crate provides enhanced OpenTelemetry integration for tracing applications. It's based on the [tracing-opentelemetry examples](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs) and provides a clean, easy-to-use API for setting up OpenTelemetry tracing and metrics.

`otel-init` is the successor to `tracing-opentelemetry-extra`. It is a separate crates.io package, so applications must update both the Cargo dependency name and Rust imports; Cargo cannot migrate the package automatically. See the [changelog](CHANGELOG.md) for the migration steps.

## Features

- Easy OpenTelemetry initialization with OTLP exporter
- Configurable sampling and resource attributes
- Automatic cleanup with guard pattern
- Support for tracing, metrics, and logs
- Clean separation of concerns from other tracing libraries

## OTLP Protocol Configuration

This crate delegates protocol selection to the native OpenTelemetry exporter builders:

```bash
# No OTLP endpoint configured, or configured as an empty string:
# providers are initialized locally, trace IDs are still created, and nothing is exported.
#
# Enable OTLP export for all signals:
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Or enable one signal only:
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4318/v1/traces

# Default protocol for all signals
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc

# Specify Protocol for Traces, Metrics, and Logs:
export OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=http/protobuf
export OTEL_EXPORTER_OTLP_METRICS_PROTOCOL=http/json
export OTEL_EXPORTER_OTLP_LOGS_PROTOCOL=grpc
```

Supported values: `grpc`, `http/protobuf`, `http/json` (exact lowercase values without surrounding whitespace).

Default behavior: when no endpoint env vars are set, or endpoint env vars are empty, traces, metrics, and logs are local-only and do not attempt to connect to `localhost:4317`. When an endpoint is configured, protocol selection uses the signal-specific variable, then the global variable, then the SDK feature default. Invalid values fall through to the next source. With this crate's enabled features, the default is `http/json`; set `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` explicitly for a gRPC receiver.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
otel-init = { version = "0.33", features = ["subscriber"] }
```

## Quick Start

### Basic Usage

```rust
use opentelemetry::KeyValue;
use otel_init::{get_resource, init_tracer_provider, init_meter_provider, init_logger_provider, OtelGuard};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create resource with service name and attributes
    let resource = get_resource(
        "my-service",
        &[
            KeyValue::new("environment", "production"),
            KeyValue::new("version", "1.0.0"),
        ],
    );

    // Initialize providers
    let tracer_provider = init_tracer_provider(&resource)?;
    let meter_provider = init_meter_provider(&resource)?;
    let logger_provider = init_logger_provider(&resource)?;

    // Create guard for automatic cleanup
    let _guard = OtelGuard::new(Some(tracer_provider), Some(meter_provider), Some(logger_provider));

    // Your application code here...
    tracing::info!("Application started");

    // Cleanup is handled automatically when the guard is dropped
    Ok(())
}
```

### With Tracing Subscriber

```rust
use opentelemetry::KeyValue;
use tracing::Level;
use otel_init::{get_resource, init_tracer_provider, init_meter_provider, init_logger_provider, init_tracing_subscriber, init_env_filter, OtelGuard};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create resource
    let service_name = "my-service";
    let resource = get_resource(service_name, &[KeyValue::new("environment", "production")]);
    
    // Initialize providers
    let tracer_provider = init_tracer_provider(&resource)?;
    let meter_provider = init_meter_provider(&resource)?;
    let logger_provider = init_logger_provider(&resource)?;

    // Set up tracing subscriber
    let env_filter = init_env_filter(&Level::INFO);
    
    // Create guard for cleanup (logger_provider is optional, pass None to disable OTel logs)
    let _guard = init_tracing_subscriber(
        service_name,
        env_filter,
        vec![Box::new(tracing_subscriber::fmt::layer())],
        tracer_provider,
        meter_provider,
        Some(logger_provider),
    )?;

    // Your application code here...
    tracing::info!("Application started with OpenTelemetry");

    Ok(())
}
```

## Configuration

### Sampling

Set `OTEL_TRACES_SAMPLER=parentbased_traceidratio` and
`OTEL_TRACES_SAMPLER_ARG=0.1` to sample 10% of root traces while honoring the
parent's sampling decision. The SDK handles these variables directly.
When `OTEL_TRACES_SAMPLER` is unset, the SDK uses `parentbased_always_on`:
root spans are sampled, and child spans honor their parent's sampling decision.

```rust
// Sampling comes from SDK environment settings or SDK defaults.
let tracer_provider = init_tracer_provider(&resource)?;
```

### Metrics Collection

Set `OTEL_METRIC_EXPORT_INTERVAL` in milliseconds (for example, `30000` for
30 seconds). When unset or invalid, the SDK uses its 60-second default:

```rust
// Collection uses SDK environment settings or SDK defaults.
let meter_provider = init_meter_provider(&resource)?;
```

### Resource Attributes

Add custom attributes to your service:

Service-name environment settings take priority: a non-empty `OTEL_SERVICE_NAME`,
then `service.name` in `OTEL_RESOURCE_ATTRIBUTES`, then explicit attributes,
then the service-name argument below. Other explicit attributes still override
the corresponding environment attributes.

```rust
let resource = get_resource(
    "my-service",
    &[
        KeyValue::new("environment", "production"),
        KeyValue::new("version", "1.0.0"),
        KeyValue::new("region", "us-west-2"),
    ],
);
```

## Features

- `subscriber` (opt-in): Enables tracing-subscriber integration

## Examples

See the [examples directory](../../examples/) for more detailed usage examples.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](../../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Related Crates

- [tracing-otel](../tracing-otel/) - HTTP, context, fields, and span utilities
- [axum-otel](../axum-otel/) - Axum web framework integration 
