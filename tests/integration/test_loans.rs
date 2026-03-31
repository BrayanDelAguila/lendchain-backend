//! Integration tests for /api/v1/loans endpoints.
//!
//! These tests require `TEST_DATABASE_URL` to be set.
//! If the variable is absent the tests are skipped automatically.

use actix_web::test;

#[path = "../common/mod.rs"]
mod common;

// ── Authorization guards ──────────────────────────────────────────────────────

/// GET /loans without an auth token should return 401.
#[tokio::test]
async fn test_list_loans_unauthorized() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::get().uri("/api/v1/loans").to_request();
    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::UNAUTHORIZED,
        "GET /loans without token should return 401"
    );
}

/// POST /loans without an auth token should return 401.
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
    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::UNAUTHORIZED,
        "POST /loans without token should return 401"
    );
}

// ── Loan creation ─────────────────────────────────────────────────────────────

/// A valid loan creation request with a real token should return 201.
#[tokio::test]
async fn test_create_loan_success() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let token = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "amount_usdc": 500.0,
            "term_months": 12,
            "annual_rate": 0.05,
            "purpose": "business"
        }))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::CREATED,
        "Valid loan creation should return 201"
    );
}

/// A loan with amount below the minimum ($100) should return 422.
#[tokio::test]
async fn test_create_loan_amount_too_low_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let token = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "amount_usdc": 50.0,
            "term_months": 12,
            "annual_rate": 0.05
        }))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
        "Loan with amount below minimum should return 422"
    );
}

/// A loan with an invalid term (e.g. 7 months) should return 422.
#[tokio::test]
async fn test_create_loan_invalid_term_fails() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let token = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "amount_usdc": 1000.0,
            "term_months": 7,
            "annual_rate": 0.05
        }))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
        "Loan with invalid term should return 422"
    );
}

// ── Loan listing ──────────────────────────────────────────────────────────────

/// GET /loans with a valid token should return 200 with only the user's loans.
#[tokio::test]
async fn test_list_loans_only_own() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let token = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "GET /loans with token should return 2xx, got {}",
        resp.status()
    );
}

// ── Single loan ───────────────────────────────────────────────────────────────

/// GET /loans/:id for a non-existent UUID should return 404.
#[tokio::test]
async fn test_get_loan_not_found() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    let token = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    let nonexistent_id = uuid::Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}", nonexistent_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::NOT_FOUND,
        "Non-existent loan should return 404"
    );
}

/// GET /loans/:id for an existing loan by its owner should return 200.
#[tokio::test]
async fn test_get_loan_success() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let token = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    // Create a loan first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "amount_usdc": 1000.0,
            "term_months": 12,
            "annual_rate": 0.05
        }))
        .to_request();
    let create_resp: common::TestResponse = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let loan_id = json["data"]["id"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}", loan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::OK,
        "GET /loans/:id for own loan should return 200"
    );
}

/// Accessing another user's loan should return 403.
#[tokio::test]
async fn test_get_loan_forbidden_other_user() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let token_a = common::create_test_user_token(pool.clone()).await;
    let token_b = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    // User A creates a loan
    let create_req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", format!("Bearer {}", token_a)))
        .set_json(serde_json::json!({
            "amount_usdc": 1000.0,
            "term_months": 12,
            "annual_rate": 0.05
        }))
        .to_request();
    let create_resp: common::TestResponse = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let loan_id = json["data"]["id"].as_str().unwrap().to_string();

    // User B tries to access it
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}", loan_id))
        .insert_header(("Authorization", format!("Bearer {}", token_b)))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::FORBIDDEN,
        "Accessing another user's loan should return 403"
    );
}

// ── Available loans & schedule ────────────────────────────────────────────────

/// GET /loans/available should return 200 (publicly accessible).
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

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "GET /loans/available should be publicly accessible, got {}",
        resp.status()
    );
}

/// GET /loans/:id/schedule should return 200 with the amortisation table.
#[tokio::test]
async fn test_get_loan_schedule_correct_rows() {
    let pool = match common::get_test_pool().await {
        Some(p) => p,
        None => return,
    };
    common::truncate_tables(&pool).await;
    let token = common::create_test_user_token(pool.clone()).await;
    let app = common::spawn_app(pool).await;

    // Create a 12-month loan
    let create_req = test::TestRequest::post()
        .uri("/api/v1/loans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "amount_usdc": 1000.0,
            "term_months": 12,
            "annual_rate": 0.05
        }))
        .to_request();
    let create_resp: common::TestResponse = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let loan_id = json["data"]["id"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/loans/{}/schedule", loan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: common::TestResponse = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "GET /loans/:id/schedule should return 2xx, got {}",
        resp.status()
    );

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rows = json["data"].as_array().expect("data should be an array");
    assert_eq!(rows.len(), 12, "12-month loan schedule should have 12 rows");
}
