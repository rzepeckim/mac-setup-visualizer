pub mod config;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use sqlx::PgPool;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .with_state(pool)
}

async fn health(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query("select 1").execute(&pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ok" }))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "error" })),
        ),
    }
}
