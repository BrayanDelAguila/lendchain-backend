use sqlx::PgPool;

use crate::errors::AppError;

// TODO: implementar gestión de wallets (creación, derivación de keys)

pub struct WalletService {
    pub pool: PgPool,
    pub encryption_key: String,
}

impl WalletService {
    pub fn new(pool: PgPool, encryption_key: String) -> Self {
        Self {
            pool,
            encryption_key,
        }
    }

    // TODO: create_wallet, derive_address, sign_transaction
}

/// Placeholder to satisfy the compiler until handlers are implemented.
pub async fn create_wallet(_pool: &PgPool) -> Result<(), AppError> {
    // TODO: implementar
    Ok(())
}
