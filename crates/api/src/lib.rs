pub mod auth;
pub mod config;
pub mod error;
pub mod routes;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/register", post(routes::auth::register))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/me", get(routes::auth::me))
        .route(
            "/api/tokens",
            get(routes::tokens::list_tokens).post(routes::tokens::create_token),
        )
        .route("/api/tokens/{id}", delete(routes::tokens::delete_token))
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
