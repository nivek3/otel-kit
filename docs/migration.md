# Migration guide

## Native SDK resource configuration

Replace `get_resource("my-service", attributes)` with `get_resource(attributes)`.
Resource detection now uses SDK defaults, with explicit attributes merged last.
Set `OTEL_SERVICE_NAME=my-service` or an explicit `service.name` attribute to
name the resource. `Logger::service_name`, `with_service_name`, and
`LOG_SERVICE_NAME` only name the tracer's instrumentation scope and no longer
provide a resource service-name fallback. Without a configured resource service
name, the SDK uses `unknown_service:<executable>`.

See [resource configuration](otlp.md#environment-variable-precedence) for the
SDK's environment merge order and explicit attribute overrides.

## OpenTelemetry 0.33

The next release uses OpenTelemetry `0.33` and `tracing-opentelemetry` `0.34`.
Align direct dependencies on the API, SDK, HTTP helpers, and exporters with
these versions when passing OpenTelemetry types to this library. Dependency
requirements allow compatible patch releases; applications control resolution
through their own `Cargo.lock`.

The OTLP `tls` feature alias has been replaced with `tls-ring`. OTLP 0.33 enables
exporter retries by default and tightens endpoint validation. Check export
failure and shutdown behavior, and use explicit `http://` or `https://` endpoint
schemes. Local-only operation without a configured endpoint remains unchanged.
Protocol selection now uses SDK defaults; set `OTEL_EXPORTER_OTLP_PROTOCOL=grpc`
explicitly for a gRPC receiver. See [OTLP configuration](otlp.md).

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
