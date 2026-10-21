# HTTP span attributes

`otel_http::span::make_request_span` records shared HTTP server
attributes. `axum-otel` adds framework-specific values before the remote parent
context is applied.

| Attribute | Source |
| --- | --- |
| `http.request.method` | Request method |
| `server.address` | `Host` header |
| `network.protocol.name` | HTTP protocol family |
| `network.protocol.version` | HTTP version |
| `url.path` | URI path |
| `url.query` | URI query |
| `url.scheme` | URI scheme or forwarding field used by the shared extractor |
| `user_agent.original` | `User-Agent` header |
| `request_id` | `X-Request-Id`, then `Request-Id` |
| `trace_id` | Active OpenTelemetry trace context |
| `http.route` | Axum matched route |
| `client.address` | Axum `ConnectInfo<SocketAddr>` peer IP |
| `otel.name` | Method plus matched route when available |
| `otel.kind` | OpenTelemetry server span kind |
| `http.response.status_code` | Integer response status recorded on response |
| `otel.status_code` / `otel.status_description` | Response/failure callbacks |

The names follow the OpenTelemetry
[HTTP span semantic conventions](https://opentelemetry.io/docs/specs/semconv/http/http-spans/)
where applicable. The library does not promise to emit every attribute in that
specification; values are recorded only when the request or framework adapter
provides them.

## Attribute migration

Recent releases replaced older field names:

| Previous attribute | Current attribute |
| --- | --- |
| `http.host` | `server.address` |
| `http.user_agent` | `user_agent.original` |
| `http.client_ip` | `client.address` |

Update dashboards, alerts, queries, and sampling rules together with the
library upgrade. `http.response.status_code` is recorded as an integer.

Adapter authors can call `make_request_span(level, request, callback)`. The
callback runs exactly once before remote-parent application. Record
adapter-owned `otel.name` and `otel.kind` inside that callback because the
OpenTelemetry span is materialized when the parent is applied.
