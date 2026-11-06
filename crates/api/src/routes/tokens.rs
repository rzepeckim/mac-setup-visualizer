use crate::{
    auth::{hash_api_token, ApiTokenUser, AuthUser},
    error::AppError,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TokenItem {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreatedTokenResponse {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_token(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Json(req): Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("token name cannot be empty".into()));
    }

    let raw_token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let plain_token = format!("msv_{}", raw_token);
    let token_hash = hash_api_token(&plain_token);

    #[derive(FromRow)]
    struct NewTokenRow {
        id: Uuid,
        name: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let row = sqlx::query_as::<_, NewTokenRow>(
        "INSERT INTO api_tokens (user_id, name, token_hash) VALUES ($1, $2, $3) RETURNING id, name, created_at",
    )
    .bind(auth.id)
    .bind(name)
    .bind(token_hash)
    .fetch_one(&pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedTokenResponse {
            id: row.id,
            name: row.name,
            token: plain_token,
            created_at: row.created_at,
        }),
    ))
}

pub async fn list_tokens(
    State(pool): State<PgPool>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let tokens = sqlx::query_as::<_, TokenItem>(
        "SELECT id, name, created_at, last_used_at FROM api_tokens WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(auth.id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(tokens))
}

pub async fn delete_token(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM api_tokens WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth.id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("token not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn verify_token(user: ApiTokenUser) -> impl IntoResponse {
    Json(json!({ "user_id": user.user_id }))
}
