use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use lendchain_backend::api;
use lendchain_backend::blockchain::{polygon::PolygonAdapter, BlockchainAdapter};
use lendchain_backend::config::Config;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // ─── Load environment variables ───────────────────────────────────────────
    dotenvy::dotenv().ok();

    // ─── Load and validate config ─────────────────────────────────────────────
    let config = Config::from_env().unwrap_or_else(|e| panic!("Config error: {}", e));

    // ─── Initialise tracing (JSON format) ────────────────────────────────────
    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.log_level))
        .with(fmt::layer().json())
        .init();

    info!(
        environment = %config.environment,
        port = config.port,
        "LendChain backend starting"
    );

    // ─── Database pool ────────────────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations automatically at startup
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    info!("Database migrations applied successfully");

    // ─── Blockchain adapter ───────────────────────────────────────────────────
    let blockchain: Arc<dyn BlockchainAdapter> = Arc::new(PolygonAdapter::new(
        config.polygon_rpc_url.clone(),
        config.polygon_chain_id,
        config.polygon_contract_address.clone(),
    ));

    // ─── Shared state ─────────────────────────────────────────────────────────
    let pool_data = web::Data::new(pool);
    let blockchain_data: web::Data<Arc<dyn BlockchainAdapter>> = web::Data::new(blockchain);
    let config_data = web::Data::new(config.clone());

    // ─── HTTP server ──────────────────────────────────────────────────────────
    let bind_addr = format!("0.0.0.0:{}", config.port);
    info!(address = %bind_addr, "Starting HTTP server");

    let cors_origins = config.cors_origins.clone();

    HttpServer::new(move || {
        // Build CORS from configured origins
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
            ])
            .max_age(3600);

        for origin in &cors_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            // Shared state
            .app_data(pool_data.clone())
            .app_data(blockchain_data.clone())
            .app_data(config_data.clone())
            // Middleware
            .wrap(cors)
            .wrap(middleware::Logger::default())
            // Health check
            .route(
                "/health",
                web::get().to(|| async {
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "ok",
                        "service": "lendchain-backend"
                    }))
                }),
            )
            // API v1 routes
            .configure(api::v1::configure)
    })
    .bind(&bind_addr)
    .expect("Failed to bind to address")
    .run()
    .await?;

    Ok(())
}
