mod storage;
use std::sync::Arc;

use crate::storage::{DB_URL, Url, init_db};
use axum::{extract::{Path, Query}, http::StatusCode, response::Redirect, Extension, Json, Router};
use base_62::encode;
use serde::{Deserialize, Serialize};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[derive(Debug)]
pub struct AppState {
    pub db_pool: sqlx::SqlitePool,
    pub root_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let db_pool = init_db(DB_URL).await.unwrap();
    let app_state = Arc::new(AppState {
        db_pool,
        root_url: "http://localhost:3000".into(),
    });

    let app = Router::new()
        .route(
            "/",
            axum::routing::get(|| async { "Welcome to the URL Shortener!" }),
        )
        .route("/create", axum::routing::post(create_url))
        .route("/{short_url}", axum::routing::get(redirect))
        .route("/urls", axum::routing::get(get_urls))
        .route(
            "/clicks/{short_url}",
            axum::routing::get(get_url_click_count),
        )
        .route("/cleanup", axum::routing::delete(cleanup_not_used_urls))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route not found") })
        .layer(Extension(app_state))
        .layer(
            TraceLayer::new_for_http()
                // Customize the level for different events
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(|request: &axum::extract::Request, _span: &tracing::Span| {
                    tracing::info!(
                        "Incoming request: {} {}",
                        request.method(),
                        request.uri().path()
                    );
                })
                .on_response(
                    |response: &axum::response::Response,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!("Response: {} (latency: {:?})", response.status(), latency);
                    },
                )
                .on_failure(
                    |error: tower_http::classify::ServerErrorsFailureClass,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::error!("Request failed: {:?} (latency: {:?})", error, latency);
                    },
                ),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateURLBody {
    pub original_url: String,
}

fn id_to_base62(id: i64) -> String {
    let bytes = id.to_be_bytes();
    encode(&bytes)
}

fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }
    Ok(())
}

async fn create_url(
    Extension(state): Extension<Arc<AppState>>,
    Json(body): Json<CreateURLBody>,
) -> Result<String, (StatusCode, String)> {
    match validate_url(&body.original_url) {
        Ok(_) => (),
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    }

    let url = storage::create_url(&state.db_pool, body.original_url.clone()).await;

    match url {
        Ok(url) => {
            let short_url = id_to_base62(url.id.unwrap_or(0));
            match storage::update_short_url(&state.db_pool, url.id.unwrap_or(0), &short_url).await {
                Ok(_) => Ok(short_url),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn redirect(
    Extension(state): Extension<Arc<AppState>>,
    Path(short_url): Path<String>,
) -> Result<Redirect, (StatusCode, String)> {
    let url = storage::get_url_by_short(&state.db_pool, &short_url).await;

    match url {
        Ok(Some(url)) => match storage::increment_click_count(&state.db_pool, &short_url).await {
            Ok(_) => Ok(Redirect::temporary(&url.original_url)),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        },
        Ok(None) => Err((StatusCode::NOT_FOUND, "URL not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetURLQuery {
    limit: i64,
    offset: i64,
}

async fn get_urls(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<GetURLQuery>,
) -> Result<Json<Vec<Url>>, (StatusCode, String)> {
    let urls = storage::get_urls(&state.db_pool, query.limit, query.offset).await;

    match urls {
        Ok(urls) => Ok(Json(urls)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_url_click_count(
    Extension(state): Extension<Arc<AppState>>,
    Path(short_url): Path<String>,
) -> Result<Json<i64>, (StatusCode, String)> {
    let url = storage::get_url_by_short(&state.db_pool, &short_url).await;

    match url {
        Ok(Some(url)) => Ok(Json(url.click_count)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "URL not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CleanupQuery {
    days: i64,
}

async fn cleanup_not_used_urls(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<CleanupQuery>,
) -> Result<Json<u64>, (StatusCode, String)> {
    let result = storage::cleanup_not_used_urls(&state.db_pool, query.days).await;

    match result {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
