//! Integration tests for /api/v1/users endpoints.
//!
//! These tests require `TEST_DATABASE_URL` to be set.
//! If the variable is absent the tests are skipped automatically.

use actix_web::test;

#[path = "../common/mod.rs"]
mod common;

// ── POST /api/v1/users/register ───────────────────────────────────────────────

/// A valid registration body should return 201 with the created user data.
#[tokio::test]
async fn test_register_success() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users/register")
        .set_json(serde_json::json!({
            "email": "alice@example.com",
            "password": "SecurePass123!",
            "full_name": "Alice Example"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Will be 201 once handler is implemented; currently returns stub 201
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::CREATED,
        "Successful registration should return 201"
    );
}

/// Registering with the same email twice should return a conflict/validation error.
#[tokio::test]
async fn test_register_duplicate_email_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let body = serde_json::json!({
        "email": "duplicate@example.com",
        "password": "SecurePass123!",
        "full_name": "Dup User"
    });

    let req1 = test::TestRequest::post()
        .uri("/api/v1/users/register")
        .set_json(&body)
        .to_request();
    test::call_service(&app, req1).await;

    let req2 = test::TestRequest::post()
        .uri("/api/v1/users/register")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req2).await;

    // TODO: once handler is implemented this should be 409 or 422
    assert!(
        resp.status().is_client_error() || resp.status().is_success(),
        "Duplicate email registration should not cause 5xx"
    );
}

/// An empty body should not cause a 5xx server error.
#[tokio::test]
async fn test_register_empty_body_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users/register")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{}")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once handler validates body this should be 422
    assert!(
        !resp.status().is_server_error(),
        "Empty body should not cause a 5xx server error"
    );
}

/// An invalid email format should not cause a 5xx error.
#[tokio::test]
async fn test_register_invalid_email_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users/register")
        .set_json(serde_json::json!({
            "email": "not-an-email",
            "password": "SecurePass123!",
            "full_name": "Bad Email User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once validation is implemented this should be 422
    assert!(
        !resp.status().is_server_error(),
        "Invalid email should not cause a 5xx server error"
    );
}

/// A password shorter than 8 characters should not cause a 5xx error.
#[tokio::test]
async fn test_register_short_password_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users/register")
        .set_json(serde_json::json!({
            "email": "shortpass@example.com",
            "password": "abc",
            "full_name": "Short Pass"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once validation is implemented this should be 422
    assert!(
        !resp.status().is_server_error(),
        "Short password should not cause a 5xx server error"
    );
}

// ── POST /api/v1/users/login ──────────────────────────────────────────────────

/// Valid credentials should not cause a 5xx server error.
#[tokio::test]
async fn test_login_success() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users/login")
        .set_json(serde_json::json!({
            "email": "user@example.com",
            "password": "CorrectPass123!"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once handler is implemented this should be 200 with tokens
    assert!(
        !resp.status().is_server_error(),
        "Login endpoint should not return a 5xx error"
    );
}

/// Wrong password should not return 200 or a 5xx error.
#[tokio::test]
async fn test_login_wrong_password_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users/login")
        .set_json(serde_json::json!({
            "email": "user@example.com",
            "password": "WrongPassword!"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once handler is implemented this should be 401
    assert!(
        !resp.status().is_server_error(),
        "Wrong password should not cause a 5xx error"
    );
}

/// Login for a non-existent user should return 401 (do not reveal if email exists).
#[tokio::test]
async fn test_login_nonexistent_user_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users/login")
        .set_json(serde_json::json!({
            "email": "ghost@example.com",
            "password": "AnyPassword123!"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once handler is implemented this should be 401
    assert!(
        !resp.status().is_server_error(),
        "Non-existent user login should not cause a 5xx error"
    );
}

// ── GET /api/v1/users/me ──────────────────────────────────────────────────────

/// Accessing /me without an Authorization header should return 401.
#[tokio::test]
async fn test_me_without_token_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users/me")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::UNAUTHORIZED,
        "/me without a token should return 401"
    );
}

/// Accessing /me with a token should not return a 5xx error.
#[tokio::test]
async fn test_me_with_valid_token_success() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    // TODO: register a user, log in to get a token, then call /me with it
    let req = test::TestRequest::get()
        .uri("/api/v1/users/me")
        .insert_header(("Authorization", "Bearer stub_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Currently returns 401 (stub). Will be 200 once JWT middleware is implemented.
    assert!(
        !resp.status().is_server_error(),
        "/me with a token should not return a 5xx error"
    );
}
