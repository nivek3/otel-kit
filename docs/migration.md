# Migration guide

## OpenTelemetry 0.33

The next release uses OpenTelemetry `0.33` and `tracing-opentelemetry` `0.34`.
Align direct dependencies on the API, SDK, HTTP helpers, and exporters with
these versions when passing OpenTelemetry types to this library. Dependency
requirements allow compatible patch releases; applications control resolution
through their own `Cargo.lock`.

The OTLP `tls` feature alias has been replaced with `tls-ring`. OTLP 0.33 enables
exporter retries by default and tightens endpoint validation. Check export
failure and shutdown behavior, and use explicit `http://` or `https://` endpoint
schemes. Local-only operation without a configured endpoint, signal-specific
configuration precedence, and the default gRPC protocol remain unchanged.

The microservices example enables `reqwest-tracing`'s `opentelemetry_0_33` feature
from upstream commit `75197ef573cc5f9aec9b0c934e3114950c5cf1e4`. Its client span
and context propagation use the same OpenTelemetry generation as the server.
The three reqwest middleware crates share this Git revision so their internal
path dependencies resolve to the same source. Switch them back to compatible
crates.io releases once OpenTelemetry 0.33 support is published.

## Migrating from 0.33.0

Version `0.33.1` introduced new crates.io package names. Cargo package names do
not migrate automatically.

```toml
# Before
tracing-otel-extra = "0.33.0"
tracing-opentelemetry-extra = "0.33.0"

# After
tracing-otel = "0.33.1"
otel-init = "0.33.1"
```

Update Rust import paths at the same time:

```rust,ignore
// Before
use tracing_otel_extra::Logger;
use tracing_opentelemetry_extra::OtelGuard;

// After
use tracing_otel::Logger;
use otel_init::OtelGuard;
```

`axum-otel` keeps its package and import name, but version `0.33.1` depends on
`tracing-otel` internally.

Also verify explicit `tracing-otel` features. The crate has no default
features: use `logger` for code-based `Logger`, `env` for `Logger::from_env`, or
the narrower HTTP features listed in [Cargo features](features.md).

If the application is upgrading from an older HTTP-span release, migrate
dashboard and alert field names using the table in
[HTTP span attributes](http-spans.md).
