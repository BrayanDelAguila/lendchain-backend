/// Business logic for user registration, login and profile management.

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::users as db;
use crate::errors::AppError;
use crate::models::user::UserPublic;
use crate::utils::crypto::{hash_password, verify_password};
use crate::utils::jwt::{generate_access_token, generate_refresh_token};
use crate::utils::wallet::generate_custodial_wallet;

// ─── DTOs ─────────────────────────────────────────────────────────────────────

pub struct RegisterDto {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub document_number: Option<String>,
    pub phone: Option<String>,
}

pub struct LoginDto {
    pub email: String,
    pub password: String,
}

pub struct AuthResponse {
    pub user: UserPublic,
    pub access_token: String,
    pub refresh_token: String,
}

// ─── Validation helpers ───────────────────────────────────────────────────────

fn validate_register(dto: &RegisterDto) -> Result<(), AppError> {
    if !dto.email.contains('@') || !dto.email.contains('.') {
        return Err(AppError::Validation("Invalid email format".into()));
    }
    if dto.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }
    if dto.full_name.trim().len() < 2 {
        return Err(AppError::Validation(
            "Full name must be at least 2 characters".into(),
        ));
    }
    Ok(())
}

// ─── SHA-256 hash for refresh tokens (stored in DB, never the raw token) ──────

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

// ─── Public service functions ─────────────────────────────────────────────────

/// Register a new user, generate a custodial wallet, and issue tokens.
pub async fn register(
    pool: &PgPool,
    dto: RegisterDto,
    jwt_secret: &str,
    wallet_encryption_key: &str,
) -> Result<AuthResponse, AppError> {
    validate_register(&dto)?;

    // Check for duplicate email
    if db::find_by_email(pool, &dto.email).await?.is_some() {
        return Err(AppError::Validation("Email already in use".into()));
    }

    let password_hash =
        hash_password(&dto.password).map_err(|e| AppError::Internal(e))?;

    let wallet = generate_custodial_wallet(wallet_encryption_key)
        .map_err(|e| AppError::Internal(e))?;

    let user_id = Uuid::new_v4();
    let user = db::insert_user(
        pool,
        user_id,
        &dto.email,
        &password_hash,
        &dto.full_name,
        dto.document_number.as_deref(),
        dto.phone.as_deref(),
        &wallet.address,
        &wallet.encrypted_private_key,
    )
    .await?;

    let (access_token, refresh_token) =
        issue_tokens(pool, &user.id, &user.email, &user.role, jwt_secret).await?;

    Ok(AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
    })
}

/// Authenticate an existing user and issue tokens.
///
/// Returns `Unauthorized` for both "user not found" and "wrong password"
/// so callers cannot determine whether an email exists.
pub async fn login(
    pool: &PgPool,
    dto: LoginDto,
    jwt_secret: &str,
) -> Result<AuthResponse, AppError> {
    let user = db::find_by_email(pool, &dto.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let ok =
        verify_password(&dto.password, &user.password_hash).map_err(|_| AppError::Unauthorized)?;
    if !ok {
        return Err(AppError::Unauthorized);
    }

    let (access_token, refresh_token) =
        issue_tokens(pool, &user.id, &user.email, &user.role, jwt_secret).await?;

    Ok(AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
    })
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

async fn issue_tokens(
    pool: &PgPool,
    user_id: &Uuid,
    email: &str,
    role: &str,
    jwt_secret: &str,
) -> Result<(String, String), AppError> {
    let access_token =
        generate_access_token(*user_id, email, role, jwt_secret).map_err(AppError::Internal)?;

    let refresh_token = generate_refresh_token();
    let token_hash = sha256_hex(&refresh_token);
    let expires_at = Utc::now() + chrono::Duration::days(30);

    db::insert_refresh_token(pool, *user_id, &token_hash, expires_at).await?;

    Ok((access_token, refresh_token))
}
