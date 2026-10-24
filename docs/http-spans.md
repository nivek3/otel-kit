# HTTP span attributes

`otel_http::span::make_request_span` records shared HTTP server
attributes. `axum-otel` adds framework-specific values before the remote parent
context is applied.

| Attribute | Source |
| --- | --- |
| `http.request.method` | Request method |
| `server.address` | URI authority, then `Host`; excludes port and IPv6 brackets |
| `server.port` | Explicit port from the same authority, recorded as an integer |
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

Authority extraction uses `http-extract`'s `authority` feature.
URI authority takes precedence over `Host`, including HTTP/2 and HTTP/3
`:authority` as represented by `http::Uri`. Missing or invalid authority values
are omitted; invalid, duplicate, or non-text `Host` fields are omitted when the
URI has no authority. No default port is inferred. `Forwarded` and
`X-Forwarded-Host` are not used for these two attributes. The `extract_host`
helper continues to return the raw `Host` header.

Request ID, singular header text, and URL scheme extraction directly reuse
`http-extract` 0.1.4. `fields` re-exports `extract_header_request_id`,
`extract_request_request_id`, `extract_single_header_text`, and
`extract_request_url_scheme` with their native `Result<Option<_>, Error>` APIs.
These replace `extract_request_id_from_headers`, `extract_request_id`,
`extract_field_from_headers`, and `extract_url_scheme`. The `extract_user_agent`
and raw `extract_host` helpers also return `Result<Option<&str>, Error>`.
Missing values return `Ok(None)`. Request ID and other singular-header helpers
reject duplicate or non-text selected field lines. Empty request IDs and
singular header text are preserved. Request ID falls back to `Request-Id` only
when `X-Request-Id` is absent.

URL scheme selection prefers the URI scheme, then the first protocol from the
validated `X-Forwarded-Proto` chain, then the first `Forwarded` proto assertion.
`X-Forwarded-Proto` accepts repeated field lines as one ordered protocol chain;
all lines and items are validated before selecting the first protocol.
`Forwarded` also accepts repeated field lines, searched in order for the first
`proto` assertion.
Proxy protocols are normalized to lowercase and Forwarded quoted pairs are
decoded. Malformed selected sources return an error rather than falling back;
missing metadata returns `Ok(None)`. The result is `Result<Option<String>, Error>`.
These headers do not establish proxy trust. Span creation omits invalid optional
metadata while the public extraction APIs preserve errors for callers.

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
