use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::admin::AdminUser;
use crate::models::user::UserPublic;
use crate::services::admin_service;

// ─── Query params ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoanFilter {
    pub status: Option<String>,
    pub network: Option<String>,
    pub borrower_id: Option<Uuid>,
    pub cursor: Option<Uuid>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct UserFilter {
    pub kyc_status: Option<String>,
    pub role: Option<String>,
    pub cursor: Option<Uuid>,
    pub limit: Option<i64>,
}

// ─── Request bodies ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateLoanStatusBody {
    pub status: String,
}

#[derive(Deserialize)]
pub struct UpdateKycBody {
    pub kyc_status: String,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/v1/admin/loans
pub async fn list_loans(
    pool: web::Data<PgPool>,
    _admin: AdminUser,
    query: web::Query<LoanFilter>,
) -> Result<HttpResponse, AppError> {
    let page = admin_service::list_loans(
        &pool,
        query.status.as_deref(),
        query.network.as_deref(),
        query.borrower_id,
        query.cursor,
        query.limit,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": page.items,
        "next_cursor": page.next_cursor
    })))
}

/// PATCH /api/v1/admin/loans/:id
pub async fn patch_loan(
    pool: web::Data<PgPool>,
    _admin: AdminUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateLoanStatusBody>,
) -> Result<HttpResponse, AppError> {
    let loan = admin_service::update_loan_status(&pool, path.into_inner(), &body.status).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": loan
    })))
}

/// GET /api/v1/admin/users
pub async fn list_users(
    pool: web::Data<PgPool>,
    _admin: AdminUser,
    query: web::Query<UserFilter>,
) -> Result<HttpResponse, AppError> {
    let page = admin_service::list_users(
        &pool,
        query.kyc_status.as_deref(),
        query.role.as_deref(),
        query.cursor,
        query.limit,
    )
    .await?;

    let public: Vec<UserPublic> = page.items.into_iter().map(UserPublic::from).collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": public,
        "next_cursor": page.next_cursor
    })))
}

/// PATCH /api/v1/admin/users/:id/kyc
pub async fn patch_user_kyc(
    pool: web::Data<PgPool>,
    _admin: AdminUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateKycBody>,
) -> Result<HttpResponse, AppError> {
    let user = admin_service::update_user_kyc(&pool, path.into_inner(), &body.kyc_status).await?;
    let public = UserPublic::from(user);
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": public
    })))
}

/// GET /api/v1/admin/stats
pub async fn stats(pool: web::Data<PgPool>, _admin: AdminUser) -> Result<HttpResponse, AppError> {
    let s = admin_service::global_stats(&pool).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "total_users": s.total_users,
            "total_loans": s.total_loans,
            "loans_by_status": {
                "PENDING": s.pending_loans,
                "FUNDED": s.funded_loans,
                "REPAID": s.repaid_loans,
                "DEFAULTED": s.defaulted_loans
            },
            "total_volume_usdc": s.total_volume_usdc,
            "total_active_usdc": s.total_active_usdc
        }
    })))
}

/// Configure admin routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/loans", web::get().to(list_loans))
            .route("/loans/{id}", web::patch().to(patch_loan))
            .route("/users", web::get().to(list_users))
            .route("/users/{id}/kyc", web::patch().to(patch_user_kyc))
            .route("/stats", web::get().to(stats)),
    );
}
