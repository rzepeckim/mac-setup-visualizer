use api::app;
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
async fn register_login_me_success(pool: PgPool) {
    let app = app(pool);

    let reg_req = Request::post("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "dev_john",
                "email": "john@example.com",
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(reg_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let login_req = Request::post("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "john@example.com",
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let cookie = res
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(cookie.contains("token="));

    let body = to_bytes(res.into_body(), 1024 * 16).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let token = body_json["token"].as_str().unwrap();

    let me_bearer = Request::get("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(me_bearer).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 16).await.unwrap();
    let me_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(me_json["username"], "dev_john");
    assert_eq!(me_json["email"], "john@example.com");

    let me_cookie = Request::get("/api/auth/me")
        .header(header::COOKIE, format!("token={}", token))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(me_cookie).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[sqlx::test]
async fn login_invalid_password_returns_401(pool: PgPool) {
    let app = app(pool);

    let reg = Request::post("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "user_auth",
                "email": "auth@example.com",
                "password": "correct_password"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(reg).await.unwrap();

    let login = Request::post("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "auth@example.com",
                "password": "wrong_password"
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(login).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn register_duplicate_returns_409(pool: PgPool) {
    let app = app(pool);

    let reg1 = Request::post("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "dup_user",
                "email": "dup@example.com",
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();
    let res1 = app.clone().oneshot(reg1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    let reg2 = Request::post("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "dup_user",
                "email": "other@example.com",
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();
    let res2 = app.clone().oneshot(reg2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::CONFLICT);

    let reg3 = Request::post("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "other_user",
                "email": "dup@example.com",
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();
    let res3 = app.oneshot(reg3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::CONFLICT);
}

#[sqlx::test]
async fn me_without_token_returns_401(pool: PgPool) {
    let app = app(pool);

    let req = Request::get("/api/auth/me")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let req = Request::get("/api/auth/me")
        .header(header::AUTHORIZATION, "Bearer invalid.token.payload")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
