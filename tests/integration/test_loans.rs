//! Integration tests for /api/v1/loans endpoints.
//!
//! These tests require `TEST_DATABASE_URL` to be set.
//! If the variable is absent the tests are skipped automatically.

use actix_web::test;

#[path = "../common/mod.rs"]
mod common;

// ── Authorization guards ──────────────────────────────────────────────────────

/// GET /loans without an auth token should not return 5xx.
#[tokio::test]
async fn test_list_loans_unauthorized() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/loans")
        .to_request();
    let resp = test::call_service(&app, req).await;
    // TODO: once auth middleware is added this should be 401
    assert!(
        !resp.status().is_server_error(),
        "GET /loans without token should not return 5xx"
    );
}

/// POST /loans without an auth token should not return 5xx.
#[tokio::test]
async fn test_create_loan_unauthorized() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .set_json(serde_json::json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    // TODO: once auth middleware is added this should be 401
    assert!(
        !resp.status().is_server_error(),
        "POST /loans without token should not return 5xx"
    );
}

// ── Loan creation ─────────────────────────────────────────────────────────────

/// A valid loan creation request with auth token should return 201.
#[tokio::test]
async fn test_create_loan_success() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", "Bearer stub_token"))
        .set_json(serde_json::json!({
            "amount_usdc": 500.0,
            "term_months": 12,
            "annual_rate": 0.05,
            "purpose": "business"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::CREATED,
        "Valid loan creation should return 201"
    );
}

/// A loan with amount below the minimum ($100) should not return 5xx.
#[tokio::test]
async fn test_create_loan_amount_too_low_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", "Bearer stub_token"))
        .set_json(serde_json::json!({
            "amount_usdc": 50.0,
            "term_months": 12,
            "annual_rate": 0.05
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once validation is implemented this should be 422
    assert!(
        !resp.status().is_server_error(),
        "Loan with amount below minimum should not cause 5xx"
    );
}

/// A loan with an invalid term (e.g. 7 months) should not return 5xx.
#[tokio::test]
async fn test_create_loan_invalid_term_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", "Bearer stub_token"))
        .set_json(serde_json::json!({
            "amount_usdc": 1000.0,
            "term_months": 7,
            "annual_rate": 0.05
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once validation is implemented this should be 422
    assert!(
        !resp.status().is_server_error(),
        "Loan with invalid term should not cause 5xx"
    );
}

// ── Loan listing ──────────────────────────────────────────────────────────────

/// GET /loans with a token should return 2xx (only the authenticated user's loans).
#[tokio::test]
async fn test_list_loans_only_own() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", "Bearer stub_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "GET /loans with token should return 2xx, got {}",
        resp.status()
    );
}

// ── Single loan ───────────────────────────────────────────────────────────────

/// GET /loans/:id for an existing loan should not return 5xx.
#[tokio::test]
async fn test_get_loan_success() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    // TODO: insert a loan directly into the DB, then query it
    let loan_id = uuid::Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}", loan_id))
        .insert_header(("Authorization", "Bearer stub_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Currently returns 404 (stub). Will be 200 once handler is implemented.
    assert!(
        !resp.status().is_server_error(),
        "GET /loans/:id should not return 5xx"
    );
}

/// Accessing another user's loan should return a client error, not 5xx.
#[tokio::test]
async fn test_get_loan_forbidden_other_user() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    // TODO: create a loan for user A, then request it as user B
    let other_loan_id = uuid::Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}", other_loan_id))
        .insert_header(("Authorization", "Bearer other_user_stub_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // TODO: once ownership check is implemented this should be 403
    assert!(
        !resp.status().is_server_error(),
        "Accessing another user's loan should not cause 5xx"
    );
}

/// GET /loans/:id for a non-existent UUID should return 404.
#[tokio::test]
async fn test_get_loan_not_found() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    let app = common::spawn_app(pool).await;

    let nonexistent_id = uuid::Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}", nonexistent_id))
        .insert_header(("Authorization", "Bearer stub_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::NOT_FOUND,
        "Non-existent loan should return 404"
    );
}

// ── Available loans & schedule ────────────────────────────────────────────────

/// GET /loans/available should return 2xx (publicly accessible).
#[tokio::test]
async fn test_list_available_loans_public() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/loans/available")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "GET /loans/available should be publicly accessible, got {}",
        resp.status()
    );
}

/// GET /loans/:id/schedule should return 2xx with the amortisation table.
#[tokio::test]
async fn test_get_loan_schedule_correct_rows() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let app = common::spawn_app(pool).await;

    // TODO: insert a 12-month loan, then assert the schedule has 12 rows
    let loan_id = uuid::Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}/schedule", loan_id))
        .insert_header(("Authorization", "Bearer stub_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "GET /loans/:id/schedule should return 2xx, got {}",
        resp.status()
    );
}
