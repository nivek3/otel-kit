# Migration guide

## Splitting `tracing-otel`

The upcoming release replaces `tracing-otel` with `otel-http` and `otel-logger`.
There is no compatibility crate under the old name. The following `0.34`
requirements describe the workspace target, not an already published split.

```toml
# Before: Logger plus direct HTTP helpers.
tracing-otel = { version = "0.34", features = ["env", "span"] }

# After: declare only the crates your source imports.
otel-logger = { version = "0.34", features = ["env"] }
otel-http = { version = "0.34", features = ["span"] }
```

Logger requires no feature for code configuration; use `otel-logger = "0.34"`.
The three workspace examples use `otel-logger` with `env`. Axum callers that only
use the callbacks need `axum-otel`; direct HTTP helper imports additionally need
an explicit `otel-http` dependency. Its `fields`, `macros`, `http`, `context`, and
`span` features retain their dependency relationships and have no defaults.

Remove the old `logger`, `otel`, and implicit `opentelemetry-otlp` feature flags.
For provider-only initialization, depend on `otel-init`. For direct subscriber
setup, explicitly enable its feature:

```toml
otel-init = { version = "0.34", features = ["subscriber"] }
```

### Public paths

| Before | After |
| --- | --- |
| `tracing_otel::http::{fields, propagation, context, span}` | `otel_http::{fields, propagation, context, span}` |
| `tracing_otel::extract::{fields, context, span}` | Corresponding root modules in `otel_http` |
| `tracing_otel::extract::http::*` | `otel_http::propagation::*` |
| `tracing_otel::{dyn_span, dyn_event}` | `otel_http::{dyn_span, dyn_event}` |
| `tracing_otel::macros` | `otel_http::macros`; macros themselves remain exported at the root |
| Root or `logger::` Logger types, `FmtSpan`, `init_logging` | Corresponding root entries in `otel_logger` |
| `tracing_otel::logger::{init_tracing_from_logger, setup_tracing, init_layer}` | Corresponding root functions in `otel_logger` |
| `tracing_otel::logger::default::*` | `otel_logger::default::*` |
| Root or `logger::` environment helpers | Corresponding root functions in `otel_logger`, with `env` |
| Root, `otel::`, or `logger::` `init_env_filter` | `otel_logger::init_env_filter`, or `otel_init::init_env_filter` with `subscriber` |
| Root or `otel::` `OtelGuard`, `BoxLayer` | Root entries in `otel_logger`, or `otel_init` (`BoxLayer` needs `subscriber`) |
| Root or `otel::` resource/provider initialization functions | Corresponding functions in `otel_init` |
| Root or `otel::` `init_tracing_subscriber` | `otel_init::init_tracing_subscriber`, with `subscriber` |
| Root or `otel::` `opentelemetry`, `opentelemetry_sdk`, `tracing_opentelemetry` | Direct crate imports, or existing `otel_init` re-exports |

There is no `otel_http::http`, `otel_http::extract`, `otel_logger::logger`, or
`otel_logger::otel` compatibility module. The public `default` module retains all
nine helpers: `service_name`, `max_log_files`, `log_level`, `dir`,
`filename_prefix`, `filename_suffix`, `span_events`, `rotation`, and
`console_enabled`.

`setup_tracing` and `init_layer` remain public, with unchanged signatures.
`OtelGuard` and `BoxLayer` are explicitly re-exported by `otel-logger`;
`opentelemetry::KeyValue` needs the native API dependency (currently `0.33`) or
an import through `otel-init`. No SDK-wide re-export remains in `otel-logger`.

### Default instrumentation scope

The crate-name default changes from `tracing_otel` to `otel_logger` for
`Logger::default()` and deserialization without a service name. This affects the
instrumentation scope, not resource `service.name`. For a stable scope, set it
with `Logger::new(...)`, `with_service_name(...)`, or `LOG_SERVICE_NAME` when
using environment configuration. Other Logger defaults, HTTP span attributes,
and callback order are unchanged by the split.

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

`axum-otel` kept its package and import name; version `0.33.1` depended on
`tracing-otel` internally.

For that historical migration, verify explicit `tracing-otel` features. That crate has no default
features: use `logger` for code-based `Logger`, `env` for `Logger::from_env`, or
the narrower HTTP features listed in [Cargo features](features.md).

If the application is upgrading from an older HTTP-span release, migrate
dashboard and alert field names using the table in
[HTTP span attributes](http-spans.md).
