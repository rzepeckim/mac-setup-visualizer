use crate::error::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_password(hash: &str, password: &str) -> Result<(), AppError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|_| AppError::Unauthorized("invalid credentials".into()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("invalid credentials".into()))
}

pub fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
}

pub fn create_jwt(user_id: Uuid) -> Result<String, AppError> {
    let exp = chrono::Utc::now().timestamp() as usize + 24 * 3600;
    let claims = Claims { sub: user_id, exp };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_jwt(token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))
}

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

impl FromRequestParts<PgPool> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, pool: &PgPool) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts)?;
        let claims = verify_jwt(&token)?;

        sqlx::query_as::<_, AuthUser>("SELECT id, username, email FROM users WHERE id = $1")
            .bind(claims.sub)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user not found".into()))
    }
}

fn extract_token(parts: &Parts) -> Result<String, AppError> {
    if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
        if let Ok(header_str) = auth_header.to_str() {
            if let Some(token) = header_str.strip_prefix("Bearer ") {
                return Ok(token.to_string());
            }
        }
    }

    if let Some(cookie_header) = parts.headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "token" {
                    return Ok(parts[1].to_string());
                }
            }
        }
    }

    Err(AppError::Unauthorized("missing authorization token".into()))
}
