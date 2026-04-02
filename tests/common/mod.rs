use std::sync::Arc;

use actix_web::web;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use lendchain_backend::blockchain::{BlockchainAdapter, OnChainLoanState, TxReceipt};
use lendchain_backend::config::Config;

/// Convenience alias so test files don't need to import actix internals.
pub type TestResponse = actix_web::dev::ServiceResponse;

/// Stub blockchain adapter used in all integration tests.
/// All methods return hardcoded receipts — no real network calls are made.
struct StubBlockchainAdapter;

#[async_trait]
impl BlockchainAdapter for StubBlockchainAdapter {
    async fn deploy_loan_contract(
        &self,
        _loan_id: Uuid,
        _borrower_address: &str,
        _amount_usdc: f64,
        _term_months: u32,
    ) -> Result<TxReceipt, lendchain_backend::errors::AppError> {
        Ok(TxReceipt {
            tx_hash: "0x_stub_deploy_tx".to_string(),
            contract_address: "0x_stub_contract_address".to_string(),
            block_number: Some(0),
            gas_used: Some(0),
        })
    }

    async fn fund_loan(
        &self,
        contract_address: &str,
        _lender_address: &str,
        _amount_usdc: f64,
    ) -> Result<TxReceipt, lendchain_backend::errors::AppError> {
        Ok(TxReceipt {
            tx_hash: "0x_stub_fund_tx".to_string(),
            contract_address: contract_address.to_string(),
            block_number: Some(0),
            gas_used: Some(0),
        })
    }

    async fn record_payment(
        &self,
        contract_address: &str,
        _payment_number: u32,
        _amount_usdc: f64,
    ) -> Result<TxReceipt, lendchain_backend::errors::AppError> {
        Ok(TxReceipt {
            tx_hash: "0x_stub_payment_tx".to_string(),
            contract_address: contract_address.to_string(),
            block_number: Some(0),
            gas_used: Some(0),
        })
    }

    async fn get_loan_state(
        &self,
        contract_address: &str,
    ) -> Result<OnChainLoanState, lendchain_backend::errors::AppError> {
        Ok(OnChainLoanState {
            contract_address: contract_address.to_string(),
            is_funded: false,
            is_repaid: false,
            total_repaid_usdc: 0.0,
        })
    }
}

pub fn test_encryption_key() -> &'static str {
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
}

pub fn test_config() -> Config {
    Config {
        database_url: "postgres://lendchain:lendchain@localhost:5432/lendchain_test".into(),
        redis_url: "redis://localhost:6379".into(),
        jwt_secret: "ci_test_jwt_secret_minimum_32_chars_ok".into(),
        wallet_encryption_key: test_encryption_key().into(),
        polygon_rpc_url: "https://rpc.stub.example.com".into(),
        polygon_chain_id: 80001,
        polygon_contract_address: "0x0000000000000000000000000000000000000000".into(),
        usdc_contract_address_polygon: "0x0000000000000000000000000000000000000000".into(),
        port: 8080,
        cors_origins: vec!["http://localhost:3000".into()],
        environment: "test".into(),
        log_level: "info".into(),
        deployer_private_key: "0000000000000000000000000000000000000000000000000000000000000001"
            .into(),
    }
}

/// Connect to the test database and run migrations.
/// Returns `None` if `TEST_DATABASE_URL` is not set — callers should skip the test.
pub async fn get_test_pool() -> Option<PgPool> {
    let db_url = std::env::var("TEST_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to test database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");
    Some(pool)
}

/// Truncate all application tables at the start of each test.
/// Data is left intact on failure so it can be inspected for debugging.
pub async fn truncate_tables(pool: &PgPool) {
    sqlx::query("TRUNCATE users, loans, loan_payments, refresh_tokens, audit_log CASCADE")
        .execute(pool)
        .await
        .expect("Failed to truncate test tables");
}

/// Build and initialise an in-memory Actix-web service backed by the given pool.
/// Uses StubBlockchainAdapter — no real blockchain calls are made during tests.
pub async fn spawn_app(
    pool: PgPool,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let blockchain: Arc<dyn BlockchainAdapter> = Arc::new(StubBlockchainAdapter);

    actix_web::test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(pool))
            .app_data(web::Data::new(blockchain))
            .app_data(web::Data::new(test_config()))
            .route(
                "/health",
                web::get().to(|| async {
                    actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
                }),
            )
            .configure(lendchain_backend::api::v1::configure),
    )
    .await
}

/// Register a fresh user and return a valid JWT access token.
/// The email is randomised so parallel tests don't collide.
pub async fn create_test_user_token(pool: PgPool) -> String {
    let app = spawn_app(pool).await;
    let email = format!("testuser_{}@example.com", uuid::Uuid::new_v4());

    let req = actix_web::test::TestRequest::post()
        .uri("/api/v1/users/register")
        .set_json(serde_json::json!({
            "email": email,
            "password": "TestPass123!",
            "full_name": "Test User"
        }))
        .to_request();

    let resp: TestResponse = actix_web::test::call_service(&app, req).await;
    let body = actix_web::test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("register response should be valid JSON");
    json["data"]["access_token"]
        .as_str()
        .expect("access_token missing from register response")
        .to_string()
}
