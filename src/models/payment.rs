use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;

/// Individual loan payment record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub payment_number: i16,
    pub amount_usdc: BigDecimal,
    pub principal: BigDecimal,
    pub interest: BigDecimal,
    pub tx_hash: Option<String>,
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Valid payment statuses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Failed,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "PENDING",
            PaymentStatus::Confirmed => "CONFIRMED",
            PaymentStatus::Failed => "FAILED",
        }
    }
}
