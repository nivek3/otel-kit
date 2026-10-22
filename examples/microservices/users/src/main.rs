use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator};
use otel_logger::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::Level;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Clone, Debug)]
struct AppState {
    users: Arc<tokio::sync::RwLock<Vec<User>>>,
}

#[tracing::instrument(skip(state))]
async fn get_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let users = state.users.read().await;
    Json(users.clone())
}

#[tracing::instrument]
async fn health() -> &'static str {
    "OK"
}

#[tracing::instrument(skip(state))]
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<User>, (axum::http::StatusCode, String)> {
    let users = state.users.read().await;
    let user = users.iter().find(|u| u.id == id);
    match user {
        Some(user) => Ok(Json(user.clone())),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            "User not found".to_string(),
        )),
    }
}

#[tracing::instrument(skip(state, payload))]
async fn create_user(State(state): State<AppState>, Json(payload): Json<CreateUser>) -> Json<User> {
    let mut users = state.users.write().await;
    let id = users.len() as u64 + 1;
    let user = User {
        id,
        name: payload.name,
        email: payload.email,
    };
    users.push(user.clone());
    Json(user)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let mut logger = Logger::from_env(Some("LOG"))?;
    logger = logger.with_ansi(true);
    let _guard = logger.init()?;

    let state = AppState {
        users: Arc::new(tokio::sync::RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/{id}", get(get_user))
        .route("/users", post(create_user))
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
        .route("/health", get(health)) // without request id, the span will not be created
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8081").await?;
    tracing::info!("Users service listening on 0.0.0.0:8081");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
