# Comparison with `axum-tracing-opentelemetry`

Both `axum-otel` and `axum-tracing-opentelemetry` help Axum applications create
OpenTelemetry-aware HTTP request spans. The difference is mainly the intended
scope.

## `axum-tracing-opentelemetry`

`axum-tracing-opentelemetry` is a focused choice when you:

- want request tracing and context propagation with a small middleware surface;
- already own the `tracing` subscriber, exporters, metrics, and logging setup;
- prefer to assemble the observability stack from independent components.

For applications with an established observability bootstrap, that narrower
scope can be exactly what is needed.

## `axum-otel` and this workspace

`axum-otel` itself stays focused on three `tower-http::TraceLayer` callbacks:

- `AxumOtelSpanCreator` creates and enriches request spans;
- `AxumOtelOnResponse` records response status and emits completion events;
- `AxumOtelOnFailure` marks classified server failures as OpenTelemetry errors.

The broader, opinionated setup comes from composing it with the other workspace
crates. `otel-http` owns the shared HTTP span helpers. `otel-logger` adds
structured console/file logging and application bootstrap. `otel-init` adds
trace, metric, and optional log providers plus RAII lifecycle management.

## Choosing between them

Choose `axum-tracing-opentelemetry` when you primarily need focused middleware
and already own the rest of the stack.

Choose this workspace when you want the same HTTP tracing boundary plus reusable
HTTP span helpers or a cohesive tracing, metrics, logging, and shutdown setup.

The distinction is about composition and ownership, not a claim that
`axum-otel` alone provides every capability in the workspace.
