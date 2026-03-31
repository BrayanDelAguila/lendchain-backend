use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

/// Insert a new user and return the created record.
#[allow(clippy::too_many_arguments)]
pub async fn insert_user(
    pool: &PgPool,
    id: Uuid,
    email: &str,
    password_hash: &str,
    full_name: &str,
    document_number: Option<&str>,
    phone: Option<&str>,
    wallet_address: &str,
    encrypted_private_key: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users
            (id, email, password_hash, full_name, document_number, phone,
             wallet_address, encrypted_private_key, kyc_status, role,
             is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'PENDING', 'USER', true, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(full_name)
    .bind(document_number)
    .bind(phone)
    .bind(wallet_address)
    .bind(encrypted_private_key)
    .fetch_one(pool)
    .await
}

/// Find a user by email address.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

/// Find a user by id.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Insert a hashed refresh token (never store the raw token).
pub async fn insert_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, revoked, created_at)
        VALUES ($1, $2, $3, $4, false, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Return the `user_id` if the token hash exists, is not revoked and has not expired.
pub async fn find_valid_refresh_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT user_id FROM refresh_tokens
        WHERE token_hash = $1
          AND revoked = false
          AND expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// Mark a refresh token as revoked.
pub async fn revoke_refresh_token(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE token_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await?;
    Ok(())
}
