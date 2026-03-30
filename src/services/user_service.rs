use sqlx::PgPool;

use crate::errors::AppError;

// TODO: implementar lógica de negocio de usuarios

pub struct UserService {
    pub pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // TODO: register, login, refresh_token, get_profile, update_kyc
}

/// Placeholder to satisfy the compiler until handlers are implemented.
pub async fn get_user(_pool: &PgPool) -> Result<(), AppError> {
    // TODO: implementar
    Ok(())
}
