use std::sync::Arc;

use sqlx::PgPool;

use crate::blockchain::BlockchainAdapter;
use crate::errors::AppError;

// TODO: implementar lógica de negocio de préstamos

pub struct LoanService {
    pub pool: PgPool,
    pub blockchain: Arc<dyn BlockchainAdapter>,
}

impl LoanService {
    pub fn new(pool: PgPool, blockchain: Arc<dyn BlockchainAdapter>) -> Self {
        Self { pool, blockchain }
    }

    // TODO: create_loan, fund_loan, repay_loan, list_loans, get_loan
}

/// Placeholder to satisfy the compiler until handlers are implemented.
pub async fn list_loans(_pool: &PgPool) -> Result<(), AppError> {
    // TODO: implementar
    Ok(())
}
