use crate::{
    auth::{create_jwt, hash_password, verify_password, AuthUser},
    error::AppError,
};
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}

#[derive(FromRow)]
struct UserWithHash {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let username = req.username.trim();
    if username.len() < 3 || username.len() > 30 {
        return Err(AppError::BadRequest("username must be between 3 and 30 characters".into()));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::BadRequest("username must be alphanumeric".into()));
    }

    let email = req.email.trim().to_lowercase();
    if !email.contains('@') || !email.contains('.') || email.len() < 5 {
        return Err(AppError::BadRequest("invalid email address".into()));
    }

    if req.password.len() < 8 {
        return Err(AppError::BadRequest("password must be at least 8 characters".into()));
    }

    let password_hash = hash_password(&req.password)?;

    let user = sqlx::query_as::<_, UserResponse>(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email",
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let identifier = req
        .email
        .or(req.username)
        .ok_or_else(|| AppError::BadRequest("missing email or username".into()))?;

    let row = sqlx::query_as::<_, UserWithHash>(
        "SELECT id, username, email, password_hash FROM users WHERE email = $1 OR username = $1",
    )
    .bind(identifier.trim().to_lowercase())
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    verify_password(&row.password_hash, &req.password)?;
    let token = create_jwt(row.id)?;

    let cookie = format!("token={}; HttpOnly; Path=/; SameSite=Lax", token);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|e| AppError::Internal(e.to_string()))?,
    );

    Ok((
        headers,
        Json(json!({
            "token": token,
            "user": {
                "id": row.id,
                "username": row.username,
                "email": row.email
            }
        })),
    ))
}

pub async fn me(auth: AuthUser) -> impl IntoResponse {
    Json(json!({
        "id": auth.id,
        "username": auth.username,
        "email": auth.email
    }))
}
