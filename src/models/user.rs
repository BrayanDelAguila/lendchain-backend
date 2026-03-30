use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User record as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub document_number: Option<String>,
    pub phone: Option<String>,
    pub wallet_address: String,
    pub encrypted_private_key: String,
    pub kyc_status: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public representation of a user (no sensitive fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub wallet_address: String,
    pub kyc_status: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            wallet_address: u.wallet_address,
            kyc_status: u.kyc_status,
            role: u.role,
            is_active: u.is_active,
            created_at: u.created_at,
        }
    }
}
