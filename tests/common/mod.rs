use std::sync::Arc;

use actix_web::web;
use sqlx::PgPool;

use lendchain_backend::blockchain::{polygon::PolygonAdapter, BlockchainAdapter};
use lendchain_backend::config::Config;

/// Convenience alias so test files don't need to import actix internals.
pub type TestResponse = actix_web::dev::ServiceResponse;

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
pub async fn spawn_app(
    pool: PgPool,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let blockchain: Arc<dyn BlockchainAdapter> = Arc::new(PolygonAdapter::new(
        "https://rpc.stub.example.com".to_string(),
        80001,
        "0x0000000000000000000000000000000000000000".to_string(),
    ));

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
