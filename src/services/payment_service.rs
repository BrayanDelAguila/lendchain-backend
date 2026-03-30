use std::sync::Arc;

use sqlx::PgPool;

use crate::blockchain::BlockchainAdapter;
use crate::errors::AppError;

// TODO: implementar lógica de negocio de pagos

pub struct PaymentService {
    pub pool: PgPool,
    pub blockchain: Arc<dyn BlockchainAdapter>,
}

impl PaymentService {
    pub fn new(pool: PgPool, blockchain: Arc<dyn BlockchainAdapter>) -> Self {
        Self { pool, blockchain }
    }

    // TODO: process_payment, list_payments_for_loan
}

/// Placeholder to satisfy the compiler until handlers are implemented.
pub async fn process_payment(_pool: &PgPool) -> Result<(), AppError> {
    // TODO: implementar
    Ok(())
}
