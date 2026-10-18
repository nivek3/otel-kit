use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing_otel::Logger;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Article {
    id: u64,
    title: String,
    content: String,
    author_id: u64,
}

#[derive(Debug, Deserialize)]
struct CreateArticle {
    title: String,
    content: String,
    author_id: u64,
}

#[derive(Clone, Debug)]
struct AppState {
    articles: Arc<tokio::sync::RwLock<Vec<Article>>>,
    http_client: ClientWithMiddleware,
}

#[tracing::instrument]
async fn get_articles(State(state): State<AppState>) -> Json<Vec<Article>> {
    let articles = state.articles.read().await;
    Json(articles.clone())
}

#[tracing::instrument]
async fn get_article(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Article>, (axum::http::StatusCode, String)> {
    let articles = state.articles.read().await;
    let article = articles.iter().find(|a| a.id == id);
    match article {
        Some(article) => Ok(Json(article.clone())),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            "Article not found".to_string(),
        )),
    }
}

#[tracing::instrument(skip(state))]
async fn get_articles_by_author(
    State(state): State<AppState>,
    Path(author_id): Path<u64>,
) -> Result<Json<Vec<Article>>, (axum::http::StatusCode, String)> {
    let base_url =
        std::env::var("USERS_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    // First verify that the author exists by calling the users service
    let user_url = format!("{base_url}/users/{author_id}");
    match state.http_client.get(&user_url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return Err((
                    axum::http::StatusCode::NOT_FOUND,
                    format!("Author with id {author_id} not found"),
                ));
            }
        }
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to verify author: {e}"),
            ));
        }
    }

    // If we get here, the author exists, so we can return their articles
    let articles = state.articles.read().await;
    let author_articles: Vec<Article> = articles
        .iter()
        .filter(|a| a.author_id == author_id)
        .cloned()
        .collect();
    Ok(Json(author_articles))
}

#[tracing::instrument(skip(state))]
async fn create_article(
    State(state): State<AppState>,
    Json(payload): Json<CreateArticle>,
) -> Json<Article> {
    let mut articles = state.articles.write().await;
    let id = articles.len() as u64 + 1;
    let article = Article {
        id,
        title: payload.title,
        content: payload.content,
        author_id: payload.author_id,
    };
    articles.push(article.clone());
    Json(article)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let logger = Logger::from_env(Some("LOG"))?;
    let _guard = logger.init()?;

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client: ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with(TracingMiddleware::default())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let state = AppState {
        articles: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        http_client: client,
    };

    let app = Router::new()
        .route("/articles", get(get_articles))
        .route("/articles/{id}", get(get_article))
        .route("/articles/author/{author_id}", get(get_articles_by_author))
        .route("/articles", post(create_article))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                        .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                        .on_failure(AxumOtelOnFailure::new()),
                )
                .layer(PropagateRequestIdLayer::x_request_id()),
        )
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8082").await?;
    tracing::info!("Articles service listening on 0.0.0.0:8082");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
