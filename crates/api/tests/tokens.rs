use api::app;
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

async fn create_user_and_token(app: &axum::Router, username: &str, email: &str) -> String {
    let reg = Request::post("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(reg).await.unwrap();

    let login = Request::post("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(login).await.unwrap();
    let body = to_bytes(res.into_body(), 1024 * 16).await.unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap();
    val["token"].as_str().unwrap().to_string()
}

#[sqlx::test]
async fn api_token_lifecycle(pool: PgPool) {
    let app = app(pool);
    let jwt = create_user_and_token(&app, "token_user", "token@example.com").await;

    let create_req = Request::post("/api/tokens")
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "name": "my_macbook" }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = to_bytes(res.into_body(), 1024 * 16).await.unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let token_id = created["id"].as_str().unwrap();
    let api_token = created["token"].as_str().unwrap();
    assert!(api_token.starts_with("msv_"));

    let list_req = Request::get("/api/tokens")
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 16).await.unwrap();
    let list: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["name"], "my_macbook");

    let verify_req = Request::get("/api/tokens/verify")
        .header(header::AUTHORIZATION, format!("Bearer {}", api_token))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(verify_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let del_req = Request::delete(format!("/api/tokens/{}", token_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(del_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let verify_after_del = Request::get("/api/tokens/verify")
        .header(header::AUTHORIZATION, format!("Bearer {}", api_token))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(verify_after_del).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let del_again = Request::delete(format!("/api/tokens/{}", token_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(del_again).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
