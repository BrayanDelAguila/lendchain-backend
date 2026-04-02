use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;
use validator::Validate;

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

/// Request body for creating a new loan.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateLoanBody {
    /// Loan amount in USDC — minimum $100.
    #[validate(range(min = 100.0, message = "Minimum loan amount is $100 USDC"))]
    pub amount_usdc: f64,

    /// Loan term in months — must be one of: 3, 6, 12, 24.
    #[validate(custom(function = "validate_term_months"))]
    pub term_months: i16,

    /// Annual interest rate as a decimal (e.g. 0.05 for 5%).
    #[validate(range(
        min = 0.01,
        max = 1.0,
        message = "Annual rate must be between 1% and 100%"
    ))]
    pub annual_rate: f64,

    /// Optional free-text purpose of the loan.
    pub purpose: Option<String>,
}

fn validate_term_months(term: i16) -> Result<(), validator::ValidationError> {
    if [3, 6, 12, 24].contains(&term) {
        Ok(())
    } else {
        Err(validator::ValidationError::new(
            "term_months must be one of: 3, 6, 12, 24",
        ))
    }
}
