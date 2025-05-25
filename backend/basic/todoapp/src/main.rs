mod storage;

use std::sync::Arc;

use crate::storage::DB_URL;
use axum::{
    Extension, Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "todoapp=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database = storage::init_db(DB_URL).await.unwrap();
    let state: Arc<sqlx::Pool<sqlx::Sqlite>> = Arc::new(database);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(|| async { "OK" }))
        .route("/todos", get(get_todos))
        .route("/todos", post(create_todo))
        .route("/todos/{id}", get(get_todo_by_id))
        .route("/todos/{id}", put(update_todo))
        .route("/todos/{id}", delete(delete_todo))
        .route("/todos/complete", get(get_complete_todos))
        .route("/todos/incomplete", get(get_incomplete_todos))
        .route("/todos/time-range", post(get_todos_by_time_range))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route not found") })
        .layer(Extension(state))
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
    println!("Server running on http://0.0.0.0:3000");
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateTodoBody {
    title: String,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UpdateTodoBody {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimeRange {
    start: String, // ISO 8601 format
    end: String,   // ISO 8601 format
}

async fn get_todos(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
) -> Json<Vec<storage::Todo>> {
    let todos = storage::get_todos(&pool).await;

    todos.map(Json).unwrap_or_else(|_| Json(vec![])) // Return an empty vector on error
}

async fn create_todo(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
    Json(payload): Json<CreateTodoBody>,
) -> Result<Json<storage::Todo>, (StatusCode, String)> {
    let todo = storage::create_todo(&pool, payload.title, payload.description).await;

    match todo {
        Ok(todo) => Ok(Json(todo)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create todo item: {e}"),
        )),
    }
}

async fn update_todo(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodoBody>,
) -> Result<Json<storage::Todo>, (StatusCode, String)> {
    let todo = storage::update_todo(
        &pool,
        id,
        payload.title,
        payload.description,
        payload.completed,
    )
    .await;

    match todo {
        Ok(todo) => Ok(Json(todo)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update todo item: {e}"),
        )),
    }
}

async fn delete_todo(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = storage::delete_todo(&pool, id).await;

    match result {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete todo item: {e}"),
        )),
    }
}

async fn get_todo_by_id(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
    Path(id): Path<i64>,
) -> Result<Json<storage::Todo>, (StatusCode, String)> {
    let todo = storage::get_todo_by_id(&pool, id).await;

    match todo {
        Ok(todo) => Ok(Json(todo)),
        Err(e) => Err((StatusCode::NOT_FOUND, format!("Todo item not found: {e}"))),
    }
}

async fn get_complete_todos(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
) -> Result<Json<Vec<storage::Todo>>, (StatusCode, String)> {
    let todos = storage::get_todos_by_completion(&pool, true).await;

    match todos {
        Ok(todos) => Ok(Json(todos)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch todos: {e}"),
        )),
    }
}

async fn get_incomplete_todos(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
) -> Result<Json<Vec<storage::Todo>>, (StatusCode, String)> {
    let todos = storage::get_todos_by_completion(&pool, false).await;

    match todos {
        Ok(todos) => Ok(Json(todos)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch todos: {e}"),
        )),
    }
}

async fn get_todos_by_time_range(
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
    Json(time_range): Json<TimeRange>,
) -> Result<Json<Vec<storage::Todo>>, (StatusCode, String)> {
    let start_time = time_range
        .start
        .parse::<chrono::NaiveDateTime>()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid start time format".to_string(),
            )
        })?;
    let end_time = time_range
        .end
        .parse::<chrono::NaiveDateTime>()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid end time format".to_string(),
            )
        })?;
    let todos = storage::get_todos_by_time_range(&pool, start_time, end_time).await;

    match todos {
        Ok(todos) => Ok(Json(todos)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch todos: {e}"),
        )),
    }
}
