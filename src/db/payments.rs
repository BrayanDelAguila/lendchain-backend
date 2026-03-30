use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::payment::Payment;

// TODO: implementar queries de pagos

pub async fn find_by_loan_id(_pool: &PgPool, _loan_id: Uuid) -> Result<Vec<Payment>, AppError> {
    // TODO: SELECT * FROM loan_payments WHERE loan_id = $1 ORDER BY payment_number ASC
    Ok(vec![])
}

pub async fn find_by_id(_pool: &PgPool, _id: Uuid) -> Result<Option<Payment>, AppError> {
    // TODO: SELECT * FROM loan_payments WHERE id = $1
    Ok(None)
}
