use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;

/// Loan record as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Loan {
    pub id: Uuid,
    pub borrower_id: Uuid,
    pub lender_id: Option<Uuid>,
    pub amount_usdc: BigDecimal,
    pub annual_rate: BigDecimal,
    pub term_months: i16,
    pub monthly_payment: BigDecimal,
    pub status: String,
    pub network: String,
    pub contract_address: Option<String>,
    pub deploy_tx_hash: Option<String>,
    pub fund_tx_hash: Option<String>,
    pub purpose: Option<String>,
    pub funded_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Valid loan status transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoanStatus {
    Pending,
    Active,
    Funded,
    Repaid,
    Defaulted,
    Cancelled,
}

impl LoanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoanStatus::Pending => "PENDING",
            LoanStatus::Active => "ACTIVE",
            LoanStatus::Funded => "FUNDED",
            LoanStatus::Repaid => "REPAID",
            LoanStatus::Defaulted => "DEFAULTED",
            LoanStatus::Cancelled => "CANCELLED",
        }
    }
}
