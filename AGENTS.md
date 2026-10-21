# otel-kit

This document provides context and guidelines for AI agents working with the `otel-kit` codebase.

## Project Overview

**otel-kit** is a repository of Rust crates for tracing, logging, and OpenTelemetry integration. It focuses on providing production-ready observability solutions for Axum web applications with minimal boilerplate.

### Key Goals

- Simplify OpenTelemetry setup for Rust applications
- Provide opinionated, production-oriented defaults
- Integrate tracing, metrics, and logging cohesively
- Support the Axum web framework with first-class middleware

## Repository Structure

```
otel-kit/
├── crates/
│   ├── axum-otel/           # Axum middleware for HTTP tracing
│   ├── otel-http/           # Shared inbound HTTP server spans and propagation
│   ├── otel-logger/         # Application logging/bootstrap for one process
│   └── otel-init/           # OpenTelemetry initialization layer
├── examples/
│   ├── otel/                # Basic OpenTelemetry example
│   └── microservices/       # Multi-service distributed tracing demo
├── Cargo.toml               # Workspace configuration
└── docker-compose.yml       # Development infrastructure
```

### Crate Dependencies

```
axum-otel -> otel-http
applications -> otel-logger -> otel-init
```

### Crate Boundaries

| Crate | Responsibility |
| ----- | -------------- |
| `otel-init` | OpenTelemetry provider/subscriber bootstrap (`OtelGuard`, OTLP setup) |
| `otel-http` | Shared inbound HTTP server spans, fields, context propagation, and dynamic macros |
| `otel-logger` | Application logging/bootstrap facade for one process |
| `axum-otel` | Axum/Tower HTTP middleware (`AxumOtelSpanCreator`, `AxumOtelOnResponse`, `AxumOtelOnFailure`) |

- Applications that only need Axum middleware should depend on `axum-otel`.
- Applications that only need provider-level OpenTelemetry setup can use `otel-init`.
- Applications that want the logging/bootstrap facade should use `otel-logger`, adding `env` for environment configuration.
- Applications importing shared HTTP helpers directly should depend on `otel-http`.
- `otel-http` does not depend on `otel-init` or `otel-logger`; `otel-logger` does not depend on `otel-http`.

Workspace `[workspace.dependencies]` entries must not enable crate features implicitly. Each member crate must declare the features its source code actually uses (`axum-otel` enables `otel-http/macros` and `otel-http/span`; examples enable `otel-logger/env`).

## Coding Conventions

### Rust Edition & Toolchain

- **Edition**: Rust 2024 (`edition = "2024"`)
- **Minimum Rust Version**: 1.96.0
- **Resolver**: Cargo resolver v2

### Code Style

1. **Lints**: The codebase uses strict linting (see `axum-otel/src/lib.rs` for reference):
   ```rust
   #![deny(unsafe_code)]
   #![warn(
       missing_docs,
       missing_debug_implementations,
       missing_copy_implementations,
       trivial_casts,
       trivial_numeric_casts,
       unused_import_braces,
       unused_qualifications
   )]
   ```

2. **Documentation**: All public APIs must have doc comments with examples where appropriate.

3. **Error Handling**: Use `anyhow::Result` for application-level errors. Library code should define specific error types when appropriate.

4. **Builder Pattern**: Configuration structs use the builder pattern with `with_*` methods:
   ```rust
   Logger::new("my-service")
       .with_format(LogFormat::Json)
       .with_level(Level::DEBUG)
       .init()
   ```

5. **Imports**: Prefer explicit imports over glob imports. Group imports by:
   - Standard library
   - External crates
   - Internal modules

### Feature Flags

`otel-http` has no default features:

| Feature | Provides | Enables |
| --- | --- | --- |
| `fields` | HTTP field extraction | `dep:http` only |
| `macros` | Runtime-level `dyn_span!` / `dyn_event!` | `dep:tracing` |
| `http` | Context propagation | `fields`, `dep:http`, `dep:opentelemetry`, `dep:opentelemetry-http` |
| `context` | Trace/span IDs and parent attachment | `http`, `dep:tracing`, `dep:tracing-opentelemetry` |
| `span` | Shared HTTP server span creation | `context`, `macros` |

`otel-logger` exposes Logger without features. Its only optional feature is
`env = ["dep:config"]`, disabled by default. It always enables `otel-init/subscriber`.
Do not reintroduce `logger`, `otel`, or the old implicit `opentelemetry-otlp`
feature. Within the two split crates, `opentelemetry-otlp` is only a development
dependency of `otel-http`; `otel-init` still uses it as a production dependency.

HTTP modules are at the `otel_http` root, with no `http` or `extract` compatibility
module. Logger entries, `default`, `setup_tracing`, `init_layer`, and
`init_tracing_from_logger` are at the `otel_logger` root. Logger explicitly
re-exports `OtelGuard`, `BoxLayer`, and `init_env_filter`; it does not forward the
whole `otel-init` API or SDK crates. The default instrumentation scope is
`otel_logger`; resource `service.name` still follows SDK configuration.

## Key Patterns

### 1. Resource Management

OpenTelemetry providers are managed via guard patterns that clean up on drop:

```rust
let _guard = Logger::new("service").init()?;
// Providers are automatically flushed and shut down when _guard is dropped
```

### 2. Tower Integration

The `axum-otel` crate integrates with `tower-http::TraceLayer`:

```rust
TraceLayer::new_for_http()
    .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
    .on_response(AxumOtelOnResponse::new())
    .on_failure(AxumOtelOnFailure::new())
```

### 3. OpenTelemetry Context Propagation

The `set_otel_parent` function extracts trace context from HTTP headers:

```rust
pub fn set_otel_parent(headers: &http::HeaderMap, span: &tracing::Span) {
    let remote_context = extract_context_from_headers(headers);
    span.set_parent(remote_context);
    // Record trace_id for logging
}
```

### 4. Dynamic Span Creation

Use the `dyn_span!` macro for runtime-configurable log levels:

```rust
let span = dyn_span!(
    self.level,
    "request",
    http.request.method = %method,
    http.route = route,
    trace_id = Empty
);
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p otel-logger

# Run tests with specific features
cargo test -p otel-http --features "context,http"
```

### Test Requirements

- Tests requiring OpenTelemetry exporters need a collector running:
  ```bash
  docker run -d -p 4317:4317 otel/opentelemetry-collector
  ```

- Integration tests may use `#[tokio::test]` for async context

### Test Patterns

```rust
#[cfg(test)]
#[cfg(feature = "context")]
mod tests {
    use super::*;

    fn init_tracing() {
        // Setup test tracing subscriber
    }

    #[tokio::test]
    async fn test_feature() {
        init_tracing();
        // Test implementation
    }
}
```

## Development Workflow

### Adding New Features

1. Determine if the feature needs a new feature flag
2. Add appropriate workspace dependencies to root `Cargo.toml`
3. Implement with full documentation
4. Add tests with appropriate feature gates
5. Update the crate's `CHANGELOG.md`

### Common Tasks

#### Adding a new HTTP header extraction

1. Add the field constant to `crates/otel-http/src/fields.rs`
2. Implement extraction function following existing patterns
3. Use in span creation via `AxumOtelSpanCreator`

#### Adding new log format support

1. Extend `LogFormat` enum in `crates/otel-logger/src/config.rs`
2. Implement the format layer in `crates/otel-logger/src/subscriber.rs`
3. Add environment variable mapping in `crates/otel-logger/src/env.rs` if using `env` feature

#### Modifying span attributes

1. Update the shared HTTP server span schema in `crates/otel-http/src/span.rs`; framework adapters customize spans through the `make_request_span` callback
2. Ensure compliance with [OpenTelemetry HTTP traces](https://opentelemetry.io/docs/specs/semconv/http/http-spans/) (see `axum_otel` crate docs on docs.rs for the attribute migration table and `CHANGELOG.md` for breaking renames)
3. Keep `AxumOtelSpanCreator` limited to Axum-specific enrichment such as route, peer address, span name, and span kind
4. Update documentation with new attributes

## Dependencies

### Key External Dependencies

| Crate                          | Purpose                         |
| ------------------------------ | ------------------------------- |
| `opentelemetry` (0.33)         | Core OpenTelemetry APIs         |
| `opentelemetry-otlp` (0.33)    | OTLP exporter                   |
| `tracing` (0.1)                | Rust tracing framework          |
| `tracing-subscriber` (0.3)     | Subscriber implementations      |
| `tracing-opentelemetry` (0.34) | Bridge between tracing and OTel |
| `axum` (0.8)                   | Web framework                   |
| `tower-http` (0.6)             | HTTP middleware utilities       |
| `reqwest` (0.13, examples)     | HTTP client in microservices demo |
| `reqwest-middleware` (0.5, examples) | Middleware stack for outbound HTTP |
| `reqwest-retry` (0.9, examples) | Retry middleware for outbound HTTP |
| `reqwest-tracing` (0.7, examples) | Distributed tracing for outbound HTTP |

### Version Compatibility Notes

- Examples use the workspace-pinned Git revision of `reqwest-tracing` with `opentelemetry_0_33`, aligned with workspace OpenTelemetry 0.33.
- Always check compatibility when upgrading OpenTelemetry crates as they often have breaking changes

## Environment Variables

### Standard OpenTelemetry Variables

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_RESOURCE_ATTRIBUTES='service.name=my-service,service.version=1.0.0'
RUST_LOG=debug
```

### Logger Configuration (with `env` feature)

```bash
LOG_SERVICE_NAME=my-service
LOG_FORMAT=json              # compact, pretty, json
LOG_LEVEL=info
OTEL_TRACES_SAMPLER=parentbased_traceidratio
OTEL_TRACES_SAMPLER_ARG=0.1
LOG_ANSI=false
LOG_CONSOLE_ENABLED=true
LOG_FILE_ENABLE=true
LOG_FILE_DIR=/var/log
LOG_FILE_FILENAME_PREFIX=myapp
LOG_FILE_ROTATION=daily      # daily, hourly, minutely, never
```

## Architecture Decisions

### Why builder pattern over struct initialization?

The builder pattern allows:

- Default values without requiring all fields
- Method chaining for clean configuration
- Future extensibility without breaking changes
- Compile-time validation of required fields

## Release Process

This project uses `release-plz` for automated releases. See `release-plz.toml` for configuration.

### Versioning

- All workspace crates share the same version
- Follow SemVer strictly
- Document all changes in per-crate `CHANGELOG.md` files

## Resources

- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Tracing Crate](https://docs.rs/tracing)
- [Axum Framework](https://github.com/tokio-rs/axum)
- [Tower HTTP](https://github.com/tower-rs/tower-http)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/concepts/semantic-conventions/)
