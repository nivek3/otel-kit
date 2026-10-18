# axum-otel

A structured logging middleware for Axum web framework that integrates with OpenTelemetry.

## Features

- Structured logging middleware for Axum
- OpenTelemetry integration
- Request tracing
- Metrics collection
- Customizable span attributes
- [OpenTelemetry HTTP semantic conventions](https://opentelemetry.io/docs/specs/semconv/http/http-spans/) for span attributes (see [crate docs](https://docs.rs/axum-otel) for migration from older field names)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
axum-otel = "0.33"
axum = { version = "0.8", features = ["macros"] }
tower-http = { version = "0.6", features = ["trace"] }
tracing-otel = { version = "0.33", features = ["logger"] }
tracing = "0.1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## Quick Start

```rust
use axum::{
    routing::get,
    Router,
};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_otel::Logger;

async fn handler() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = Logger::new("my-service").init()?;

    // Build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                .on_failure(AxumOtelOnFailure::new()),
        );

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
```

To enable OTLP export, set `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317`
and `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` before starting the application.

## Examples

Check out the [examples](https://github.com/nivek3/otel-kit/tree/main/examples) directory for more usage examples:

- [Basic OpenTelemetry integration](https://github.com/nivek3/otel-kit/tree/main/examples/otel)

## Documentation

For more detailed documentation, visit [docs.rs](https://docs.rs/axum-otel/).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
