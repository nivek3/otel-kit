use anyhow::Result;
use axum::extract::Query;
use axum::{Router, routing::get};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};
use otel_logger::Logger;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
pub struct HelloQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

#[tracing::instrument]
async fn hello(q: Query<HelloQuery>) -> &'static str {
    "Hello world!"
}

#[tracing::instrument]
async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let mut logger = Logger::from_env(Some("LOG"))?;
    logger = logger.with_ansi(true);
    let _guard = logger.init()?;

    // Setup Axum router and server
    let app = Router::new()
        .route("/hello", get(hello))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                        .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                        .on_failure(AxumOtelOnFailure::new().level(Level::ERROR)),
                )
                .layer(PropagateRequestIdLayer::x_request_id()),
        )
        .route("/health", get(health)); // without request id, the span will not be created

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("Server is running on http://0.0.0.0:8080");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
