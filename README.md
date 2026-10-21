# otel-kit

### Axum request tracing and opinionated OpenTelemetry bootstrap for Rust.

[![Crates.io](https://img.shields.io/crates/v/axum-otel.svg)](https://crates.io/crates/axum-otel)
[![Documentation](https://docs.rs/axum-otel/badge.svg)](https://docs.rs/axum-otel)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

[Repository guide](docs/README.md) ·
[`axum-otel`](https://docs.rs/axum-otel) ·
[`otel-http`](crates/otel-http/README.md) ·
[`otel-logger`](crates/otel-logger/README.md) ·
[`otel-init`](https://docs.rs/otel-init) ·
[Examples](examples/)

`otel-kit` is a workspace of four crates with separate
observability responsibilities:

- `axum-otel` provides `tower-http::TraceLayer` callbacks for inbound Axum
  request spans;
- `otel-http` provides shared HTTP server spans and context propagation;
- `otel-logger` provides the application logging/bootstrap facade for one process;
- `otel-init` initializes OpenTelemetry providers and tracing subscribers and
  owns their shutdown lifecycle.

Choose the smallest crate that owns the behavior your application needs. Use
`axum-otel` with an existing subscriber for focused request tracing, or combine
it with `otel-logger` when you want tracing, metrics, structured logs, and
provider lifecycle management configured together.

## 📦 Crates

| Crate | Responsibility | Use it when |
| --- | --- | --- |
| [`axum-otel`](crates/axum-otel/README.md) | Axum-specific request span creation, response events, and failure status | You need inbound Axum request tracing |
| [`otel-http`](crates/otel-http/README.md) | Shared HTTP server spans, context propagation, fields, and dynamic macros | You need framework-independent HTTP helpers |
| [`otel-logger`](crates/otel-logger/README.md) | Console/file logging and application bootstrap | You need the `Logger` facade |
| [`otel-init`](crates/otel-init/README.md) | OTLP tracer, meter, and logger providers; subscriber setup; provider shutdown | You own subscriber setup or need provider-level initialization |

The dependency direction is:

```text
axum-otel -> otel-http
applications -> otel-logger -> otel-init
```

`otel-http` has no default features. `otel-logger` exposes `Logger` by default;
only its environment configuration requires `env`. See the
[Cargo feature guide](docs/features.md).

## 🚀 Quick start

This example combines the Axum callbacks with the opinionated `Logger` facade.
The split packages are for the upcoming release; snippets show target versions:

```toml
[dependencies]
anyhow = "1"
axum = "0.8"
axum-otel = "0.34"
tokio = { version = "1", features = ["macros", "net", "rt-multi-thread"] }
tower-http = { version = "0.6", features = ["trace"] }
otel-logger = "0.34"
```

```rust
use axum::{Router, routing::get};
use axum_otel::{
    AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level,
};
use tower_http::trace::TraceLayer;
use otel_logger::{LogFormat, Logger};

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = Logger::new("web-api")
        .with_format(LogFormat::Json)
        .with_ansi(false)
        .init()?;

    let app = Router::new().route("/health", get(health)).layer(
        TraceLayer::new_for_http()
            .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
            .on_response(AxumOtelOnResponse::new().level(Level::INFO))
            .on_failure(AxumOtelOnFailure::new().level(Level::ERROR)),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

Keep the returned `LoggerGuard` alive for the application lifetime. It owns the
OpenTelemetry providers and any non-blocking file writer guard, and shuts them
down in the required order.

OTLP export is opt-in. Without a configured endpoint, providers remain local
and no connection to `localhost:4317` is attempted:

```sh
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_SERVICE_NAME=web-api
```

`OTEL_SERVICE_NAME` names the resource. The argument to `Logger::new(...)`
only names the tracer's instrumentation scope.

See [Getting started](docs/getting-started.md),
[Logger configuration](docs/logger.md), and
[OTLP export and lifecycle](docs/otlp.md) for the complete setup.

## 🔍 Comparison with `axum-tracing-opentelemetry`

Both `axum-otel` and `axum-tracing-opentelemetry` help Axum applications create
OpenTelemetry-aware HTTP request spans. The difference is mainly the intended
scope.

### `axum-tracing-opentelemetry`

`axum-tracing-opentelemetry` is a focused choice when you:

- want request tracing and context propagation with a small middleware surface;
- already own the `tracing` subscriber, exporters, metrics, and logging setup;
- prefer to assemble the observability stack from independent components.

For applications with an established observability bootstrap, that narrower
scope can be exactly what is needed.

### `axum-otel` and this workspace

`axum-otel` itself stays focused on three `TraceLayer` callbacks:

- `AxumOtelSpanCreator` creates and enriches request spans;
- `AxumOtelOnResponse` records response status and emits completion events;
- `AxumOtelOnFailure` marks classified server failures as OpenTelemetry errors.

The broader, opinionated setup comes from composing it with the other workspace
crates. `otel-logger` adds structured console/file logging and application
bootstrap, while `otel-init` adds trace, metric, and optional log providers plus
RAII lifecycle management.

Choose `axum-tracing-opentelemetry` when you primarily need focused middleware
and already own the rest of the stack. Choose this workspace when you want the
same HTTP tracing boundary plus reusable HTTP span helpers or a cohesive
tracing, metrics, logging, and shutdown setup.

## 🔭 HTTP request spans

`axum-otel` builds on `tower-http::TraceLayer` and records OpenTelemetry-aligned
HTTP attributes including method, route, URL parts, protocol, `server.address`,
`client.address` when `ConnectInfo<SocketAddr>` is available,
`user_agent.original`, `request_id`, and `trace_id`.

The shared span schema lives in `otel-http`; the Axum adapter owns matched
route, transport peer, span name, and server span kind. See
[HTTP span attributes](docs/http-spans.md) before writing dashboards, alerts,
or sampling rules.

## 📚 Examples

- [`examples/otel`](examples/otel/README.md) demonstrates one Axum service with
  request IDs and environment-based logger configuration.
- [`examples/microservices`](examples/microservices/README.md) demonstrates
  cross-service propagation with Grafana, Loki, and Tempo-oriented local
  infrastructure.

Run the basic service with:

```sh
cargo run -p axum-otel-demo
```

## 🔄 Migrating from `tracing-otel`

The upcoming release splits `tracing-otel` into `otel-http` and `otel-logger`.
Use `otel-logger` for application bootstrap and `otel-http` for shared HTTP
helpers. Applications using both must declare both dependencies. No compatibility
crate is provided under the old name.

See the [migration guide](docs/migration.md#splitting-tracing-otel) for every
public path and feature change, including the default instrumentation scope
name changing to `otel_logger`. Earlier package migrations remain documented
there as history.

## 🧩 Compatibility

- Rust 1.96.0 is the workspace compatibility baseline.
- The repository toolchain may use a newer compiler for local development.
- OpenTelemetry export supports gRPC, HTTP/Protobuf, and HTTP/JSON according to
  the configured endpoint and protocol environment variables.

## 📄 License

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE));
- MIT License ([LICENSE-MIT](LICENSE-MIT)).
