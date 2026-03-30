use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::user::User;

// TODO: implementar queries de usuarios

pub async fn find_by_id(_pool: &PgPool, _id: Uuid) -> Result<Option<User>, AppError> {
    // TODO: SELECT * FROM users WHERE id = $1
    Ok(None)
}

pub async fn find_by_email(_pool: &PgPool, _email: &str) -> Result<Option<User>, AppError> {
    // TODO: SELECT * FROM users WHERE email = $1
    Ok(None)
}

pub async fn find_by_wallet_address(
    _pool: &PgPool,
    _address: &str,
) -> Result<Option<User>, AppError> {
    // TODO: SELECT * FROM users WHERE wallet_address = $1
    Ok(None)
}
