# Axum request tracing

`axum-otel` supplies three callbacks for `tower-http::TraceLayer`:

- `AxumOtelSpanCreator` creates and enriches the request span;
- `AxumOtelOnResponse` records the response status and emits the completion
  event;
- `AxumOtelOnFailure` marks classified server failures as OpenTelemetry errors.

```rust,no_run
use axum::{Router, routing::get};
use axum_otel::{
    AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level,
};
use tower_http::trace::TraceLayer;
use otel_logger::Logger;

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = Logger::new("web-api").init()?;

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

The span creator delegates shared HTTP attributes and remote-parent extraction
to `otel-http`. It owns only Axum-specific enrichment: matched route,
transport peer from `ConnectInfo<SocketAddr>`, span name, and server span kind.

To record `client.address`, serve the router with Axum connect information:

```rust,ignore
axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
    .await?;
```

The value comes from the socket peer, not `Forwarded` or `X-Forwarded-For`.
Deployments behind a proxy should interpret it accordingly.

The defaults are `TRACE` for span creation, `DEBUG` for response events, and
`ERROR` for failure events. Ensure the subscriber filter enables the selected
levels; otherwise expected events or spans can be filtered out.
