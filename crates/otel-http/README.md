# otel-http

Framework-independent inbound HTTP server spans and context propagation.
`axum-otel` adapts these helpers to Axum. Application logging and provider
initialization belong to `otel-logger` and `otel-init` respectively.

This crate is split out of `tracing-otel` in the upcoming release. See the
[migration guide](../../docs/migration.md#splitting-tracing-otel) for dependency,
feature, and import changes.

## Features

There are no default features. `dep:` denotes an optional dependency; other
names in the table denote features of this crate.

| Feature | Public APIs | Enables |
| --- | --- | --- |
| `fields` | `fields` request/header helpers and constants | `dep:http`, `dep:http-extract` (`request-id`, `scheme`) |
| `macros` | Root `dyn_span!` and `dyn_event!` macros | `dep:tracing` |
| `http` | `propagation` context extraction and injection | `fields`, `dep:http`, `dep:opentelemetry`, `dep:opentelemetry-http` |
| `context` | Current trace/span IDs and remote-parent attachment | `http`, `dep:tracing`, `dep:tracing-opentelemetry` |
| `span` | `span::make_request_span` | `context`, `macros`, `http-extract/authority` |

## Request spans

Dependency configuration for the upcoming release:

```toml
[dependencies]
otel-http = { version = "0.34", features = ["span"] }
http = "1"
tracing = "0.1"
```

```rust
use http::Request;
use otel_http::span::make_request_span;
use tracing::Level;

let request = Request::builder().uri("/items").body(()).unwrap();
let span = make_request_span(Level::INFO, &request, |span| {
    span.record("http.route", "/items");
});
```

The customization callback runs once before the remote parent is applied.
Framework adapters set route, peer address, span name, and span kind there.
See [HTTP span attributes](../../docs/http-spans.md) for the shared schema.

The caller owns its tracing subscriber and OpenTelemetry propagator setup.
These helpers do not initialize providers or depend on `otel-logger` or
`otel-init`. The optional dependency feature `opentelemetry-otlp` from the old
package is removed; exporter construction belongs in the caller's dependencies.

## License

MIT or Apache-2.0; see the repository's [MIT](../../LICENSE-MIT) and
[Apache](../../LICENSE-APACHE) licenses.
