use sqlx::PgPool;
use uuid::Uuid;

use crate::db::admin::{self, AdminLoanFilters, AdminUserFilters, GlobalStatsRow};
use crate::errors::AppError;
use crate::models::loan::Loan;
use crate::models::user::User;
use crate::services::loan_service::Page;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

fn clamp(limit: Option<i64>) -> i64 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

pub async fn list_loans(
    pool: &PgPool,
    status: Option<&str>,
    network: Option<&str>,
    borrower_id: Option<Uuid>,
    cursor: Option<Uuid>,
    limit: Option<i64>,
) -> Result<Page<Loan>, AppError> {
    let limit = clamp(limit);
    let mut items = admin::list_all_loans(
        pool,
        AdminLoanFilters {
            status,
            network,
            borrower_id,
            cursor_id: cursor,
            limit: limit + 1,
        },
    )
    .await?;

    let next_cursor = if items.len() as i64 > limit {
        items.truncate(limit as usize);
        items.last().map(|l| l.id)
    } else {
        None
    };
    Ok(Page { items, next_cursor })
}

pub async fn update_loan_status(pool: &PgPool, id: Uuid, status: &str) -> Result<Loan, AppError> {
    if status != "DEFAULTED" && status != "CANCELLED" {
        return Err(AppError::Validation(
            "Admin can only set status to DEFAULTED or CANCELLED".into(),
        ));
    }
    let loan = admin::update_loan_status(pool, id, status).await?;
    Ok(loan)
}

pub async fn list_users(
    pool: &PgPool,
    kyc_status: Option<&str>,
    role: Option<&str>,
    cursor: Option<Uuid>,
    limit: Option<i64>,
) -> Result<Page<User>, AppError> {
    let limit = clamp(limit);
    let mut items = admin::list_all_users(
        pool,
        AdminUserFilters {
            kyc_status,
            role,
            cursor_id: cursor,
            limit: limit + 1,
        },
    )
    .await?;

    let next_cursor = if items.len() as i64 > limit {
        items.truncate(limit as usize);
        items.last().map(|u| u.id)
    } else {
        None
    };
    Ok(Page { items, next_cursor })
}

pub async fn update_user_kyc(pool: &PgPool, id: Uuid, kyc_status: &str) -> Result<User, AppError> {
    if kyc_status != "APPROVED" && kyc_status != "REJECTED" {
        return Err(AppError::Validation(
            "kyc_status must be APPROVED or REJECTED".into(),
        ));
    }
    let user = admin::update_user_kyc(pool, id, kyc_status).await?;
    Ok(user)
}

pub async fn global_stats(pool: &PgPool) -> Result<GlobalStatsRow, AppError> {
    let stats = admin::global_stats(pool).await?;
    Ok(stats)
}
