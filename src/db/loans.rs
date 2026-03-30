use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::loan::Loan;

// TODO: implementar queries de préstamos

pub async fn find_by_id(_pool: &PgPool, _id: Uuid) -> Result<Option<Loan>, AppError> {
    // TODO: SELECT * FROM loans WHERE id = $1
    Ok(None)
}

pub async fn find_by_borrower(
    _pool: &PgPool,
    _borrower_id: Uuid,
    _limit: i64,
    _offset: i64,
) -> Result<Vec<Loan>, AppError> {
    // TODO: SELECT * FROM loans WHERE borrower_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3
    Ok(vec![])
}

pub async fn count_by_borrower(_pool: &PgPool, _borrower_id: Uuid) -> Result<i64, AppError> {
    // TODO: SELECT COUNT(*) FROM loans WHERE borrower_id = $1
    Ok(0)
}
