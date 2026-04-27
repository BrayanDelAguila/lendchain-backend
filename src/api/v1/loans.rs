use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::blockchain::BlockchainAdapter;
use crate::config::Config;
use crate::db::users as db_users;
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::loan::CreateLoanBody;
use crate::services::loan_service;
use crate::services::loan_service::polygonscan_url;

// ─── Query params ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub cursor: Option<Uuid>,
    pub limit: Option<i64>,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/v1/loans — list the authenticated user's loans.
pub async fn list_loans(
    pool: web::Data<PgPool>,
    auth: AuthenticatedUser,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;

    let page = loan_service::list_loans(&pool, user_id, query.cursor, query.limit).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": page.items,
        "next_cursor": page.next_cursor
    })))
}

/// GET /api/v1/loans/available — authenticated list of loans available to fund (excludes own).
pub async fn list_available_loans(
    pool: web::Data<PgPool>,
    auth: AuthenticatedUser,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let page = loan_service::list_available(&pool, user_id, query.cursor, query.limit).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": page.items,
        "next_cursor": page.next_cursor
    })))
}

/// GET /api/v1/loans/portfolio — loans where the authenticated user is the lender.
pub async fn list_portfolio(
    pool: web::Data<PgPool>,
    auth: AuthenticatedUser,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let page = loan_service::list_portfolio(&pool, user_id, query.cursor, query.limit).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": page.items,
        "next_cursor": page.next_cursor
    })))
}

/// POST /api/v1/loans — create a new loan request.
pub async fn create_loan(
    pool: web::Data<PgPool>,
    blockchain: web::Data<Arc<dyn BlockchainAdapter>>,
    _config: web::Data<Config>,
    auth: AuthenticatedUser,
    body: web::Json<CreateLoanBody>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;

    // Fetch borrower's wallet address for the blockchain call
    let user = db_users::find_by_id(&pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let loan =
        loan_service::create_loan(&pool, &blockchain, user_id, &user.wallet_address, &body).await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "data": loan
    })))
}

/// GET /api/v1/loans/history — unified history of loans (borrower + lender).
pub async fn list_history(
    pool: web::Data<PgPool>,
    auth: AuthenticatedUser,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let page = loan_service::list_history(&pool, user_id, query.cursor, query.limit).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": page.items,
        "next_cursor": page.next_cursor
    })))
}

/// GET /api/v1/loans/{id} — get a specific loan (borrower or lender only).
pub async fn get_loan(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let loan_id = path.into_inner();

    let loan = loan_service::get_loan(&pool, loan_id, user_id).await?;

    let polygonscan_url_deploy = loan
        .deploy_tx_hash
        .as_deref()
        .map(|h| polygonscan_url(config.polygon_chain_id, h));
    let polygonscan_url_fund = loan
        .fund_tx_hash
        .as_deref()
        .map(|h| polygonscan_url(config.polygon_chain_id, h));

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": loan,
        "polygonscan_url_deploy": polygonscan_url_deploy,
        "polygonscan_url_fund": polygonscan_url_fund
    })))
}

/// GET /api/v1/loans/{id}/schedule — amortisation table (borrower or lender only).
pub async fn get_loan_schedule(
    pool: web::Data<PgPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let loan_id = path.into_inner();

    // Verify the loan exists and the caller is borrower or lender
    let loan = crate::db::loans::find_by_id(&pool, loan_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if loan.borrower_id != user_id && loan.lender_id != Some(user_id) {
        return Err(AppError::Forbidden);
    }

    let schedule = loan_service::get_schedule(&pool, loan_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": schedule
    })))
}

/// POST /api/v1/loans/{id}/fund — lender funds a PENDING loan.
pub async fn fund_loan(
    pool: web::Data<PgPool>,
    blockchain: web::Data<Arc<dyn BlockchainAdapter>>,
    config: web::Data<Config>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let lender_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let loan_id = path.into_inner();

    let loan = crate::db::loans::find_by_id(&pool, loan_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if loan.status != "PENDING" {
        return Err(AppError::InvalidState(
            "Loan is not in PENDING status".into(),
        ));
    }
    if loan.lender_id.is_some() {
        return Err(AppError::InvalidState(
            "Loan has already been funded".into(),
        ));
    }
    if loan.borrower_id == lender_id {
        return Err(AppError::InvalidState(
            "Borrower cannot fund their own loan".into(),
        ));
    }

    let lender = db_users::find_by_id(&pool, lender_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let contract_address = loan
        .contract_address
        .as_deref()
        .ok_or_else(|| AppError::InvalidState("Loan has no deployed contract".into()))?;

    let receipt = blockchain
        .fund_loan(
            contract_address,
            &lender.encrypted_private_key,
            &config.wallet_encryption_key,
        )
        .await?;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30 * loan.term_months as i64);

    let updated = crate::db::loans::update_fund_info(
        &pool,
        loan_id,
        lender_id,
        &receipt.tx_hash,
        now,
        due_date,
    )
    .await?;

    let tx_url = loan_service::polygonscan_url(config.polygon_chain_id, &receipt.tx_hash);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": updated,
        "tx_url": tx_url
    })))
}

/// POST /api/v1/loans/{id}/pay — borrower pays the next pending installment.
pub async fn pay_loan_installment(
    pool: web::Data<PgPool>,
    blockchain: web::Data<Arc<dyn BlockchainAdapter>>,
    config: web::Data<Config>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let loan_id = path.into_inner();

    let result = crate::services::payment_service::pay_installment(
        &pool,
        &blockchain,
        &config,
        loan_id,
        user_id,
    )
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "data": result
    })))
}

/// Configure loan routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/loans")
            .route("", web::get().to(list_loans))
            .route("", web::post().to(create_loan))
            .route("/available", web::get().to(list_available_loans))
            .route("/portfolio", web::get().to(list_portfolio))
            .route("/history", web::get().to(list_history))
            .route("/{id}", web::get().to(get_loan))
            .route("/{id}/schedule", web::get().to(get_loan_schedule))
            .route("/{id}/fund", web::post().to(fund_loan))
            .route("/{id}/pay", web::post().to(pay_loan_installment)),
    );
}
