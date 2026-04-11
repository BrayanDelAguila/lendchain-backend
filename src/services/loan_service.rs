use std::sync::Arc;

use bigdecimal::FromPrimitive;
use serde::Serialize;
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::blockchain::BlockchainAdapter;
use crate::db::{loans as db_loans, payments as db_payments};
use crate::errors::AppError;
use crate::models::loan::{CreateLoanBody, Loan};
use crate::models::payment::Payment;
use crate::utils::calculator;

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

/// A single row in the amortisation schedule response.
#[derive(Debug, Serialize)]
pub struct ScheduleRow {
    pub payment_number: u32,
    pub payment_usdc: f64,
    pub principal: f64,
    pub interest: f64,
    pub remaining_balance: f64,
}

/// Paginated list result.
pub struct Page<T> {
    pub items: Vec<T>,
    /// ID of the last item — use as `cursor` in the next request. `None` means end of list.
    pub next_cursor: Option<Uuid>,
}

// ── Public service functions ──────────────────────────────────────────────────

/// Create a new loan, insert its payment schedule, and deploy the blockchain contract (stub).
pub async fn create_loan(
    pool: &PgPool,
    blockchain: &Arc<dyn BlockchainAdapter>,
    borrower_id: Uuid,
    borrower_wallet: &str,
    body: &CreateLoanBody,
) -> Result<Loan, AppError> {
    let monthly_f64 =
        calculator::monthly_payment(body.amount_usdc, body.annual_rate, body.term_months as u32);

    let amount_bd = BigDecimal::from_f64(body.amount_usdc)
        .ok_or_else(|| AppError::Validation("Invalid amount_usdc value".into()))?;
    let rate_bd = BigDecimal::from_f64(body.annual_rate)
        .ok_or_else(|| AppError::Validation("Invalid annual_rate value".into()))?;
    let monthly_bd = BigDecimal::from_f64(monthly_f64)
        .ok_or_else(|| AppError::Validation("Invalid computed monthly_payment".into()))?;

    // Insert loan with PENDING status (no contract info yet)
    let loan = db_loans::insert_loan(
        pool,
        borrower_id,
        &amount_bd,
        &rate_bd,
        body.term_months,
        &monthly_bd,
        body.purpose.as_deref(),
        None,
        None,
    )
    .await?;

    // Build and insert payment schedule
    let schedule = calculator::amortisation_schedule(
        body.amount_usdc,
        body.annual_rate,
        body.term_months as u32,
    );
    let payment_rows: Vec<db_payments::PaymentRow> = schedule
        .iter()
        .map(|row| -> Result<db_payments::PaymentRow, AppError> {
            Ok(db_payments::PaymentRow {
                payment_number: row.payment_number as i16,
                amount_usdc: BigDecimal::from_f64(row.payment)
                    .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Invalid payment amount")))?,
                principal: BigDecimal::from_f64(row.principal).ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!("Invalid principal amount"))
                })?,
                interest: BigDecimal::from_f64(row.interest).ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!("Invalid interest amount"))
                })?,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    db_payments::insert_payment_schedule(pool, loan.id, &payment_rows).await?;

    // Deploy contract (stub — no real network call)
    let receipt = blockchain
        .deploy_loan_contract(
            loan.id,
            borrower_wallet,
            body.amount_usdc,
            body.term_months as u32,
        )
        .await?;

    // Persist contract info
    db_loans::update_contract_info(pool, loan.id, &receipt.contract_address, &receipt.tx_hash)
        .await?;

    // Re-fetch updated loan
    let updated = db_loans::find_by_id(pool, loan.id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(updated)
}

/// Retrieve a loan by ID. Only the borrower or lender may access it.
pub async fn get_loan(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Loan, AppError> {
    let loan = db_loans::find_by_id(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if loan.borrower_id != user_id && loan.lender_id != Some(user_id) {
        return Err(AppError::Forbidden);
    }

    Ok(loan)
}

/// List loans belonging to a borrower (cursor-based pagination).
pub async fn list_loans(
    pool: &PgPool,
    borrower_id: Uuid,
    cursor: Option<Uuid>,
    limit: Option<i64>,
) -> Result<Page<Loan>, AppError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    // fetch one extra to determine if there is a next page
    let mut items = db_loans::list_by_borrower(pool, borrower_id, cursor, limit + 1).await?;

    let next_cursor = if items.len() as i64 > limit {
        items.truncate(limit as usize);
        items.last().map(|l| l.id)
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// List loans available to fund (status = PENDING, excluding caller's own loans).
pub async fn list_available(
    pool: &PgPool,
    exclude_user_id: Uuid,
    cursor: Option<Uuid>,
    limit: Option<i64>,
) -> Result<Page<Loan>, AppError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let mut items = db_loans::list_available(pool, exclude_user_id, cursor, limit + 1).await?;

    let next_cursor = if items.len() as i64 > limit {
        items.truncate(limit as usize);
        items.last().map(|l| l.id)
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// List loans where the user is the lender (portfolio view).
pub async fn list_portfolio(
    pool: &PgPool,
    lender_id: Uuid,
    cursor: Option<Uuid>,
    limit: Option<i64>,
) -> Result<Page<Loan>, AppError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let mut items = db_loans::list_by_lender(pool, lender_id, cursor, limit + 1).await?;

    let next_cursor = if items.len() as i64 > limit {
        items.truncate(limit as usize);
        items.last().map(|l| l.id)
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// Return the full amortisation schedule for a loan.
/// Any authenticated user may view the schedule (no ownership check).
pub async fn get_schedule(pool: &PgPool, id: Uuid) -> Result<Vec<ScheduleRow>, AppError> {
    let loan = db_loans::find_by_id(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    let amount = to_f64(&loan.amount_usdc)?;
    let rate = to_f64(&loan.annual_rate)?;
    let rows = calculator::amortisation_schedule(amount, rate, loan.term_months as u32)
        .into_iter()
        .map(|r| ScheduleRow {
            payment_number: r.payment_number,
            payment_usdc: r.payment,
            principal: r.principal,
            interest: r.interest,
            remaining_balance: r.remaining_balance,
        })
        .collect();

    Ok(rows)
}

/// Return the saved payment rows for a loan.
pub async fn list_payments(pool: &PgPool, loan_id: Uuid) -> Result<Vec<Payment>, AppError> {
    let payments = db_payments::list_by_loan(pool, loan_id).await?;
    Ok(payments)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn to_f64(bd: &BigDecimal) -> Result<f64, AppError> {
    use bigdecimal::ToPrimitive;
    bd.to_f64()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("BigDecimal conversion failed")))
}

/// Build a Polygonscan URL for a given tx hash.
pub fn polygonscan_url(chain_id: u64, tx_hash: &str) -> String {
    let base = if chain_id == 137 {
        "https://polygonscan.com"
    } else {
        "https://amoy.polygonscan.com"
    };
    format!("{}/tx/{}", base, tx_hash)
}
