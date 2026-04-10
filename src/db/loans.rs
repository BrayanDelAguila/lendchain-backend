use chrono::{DateTime, Utc};
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::loan::Loan;

/// Insert a new loan record and return the created row.
#[allow(clippy::too_many_arguments)]
pub async fn insert_loan(
    pool: &PgPool,
    borrower_id: Uuid,
    amount_usdc: &BigDecimal,
    annual_rate: &BigDecimal,
    term_months: i16,
    monthly_payment: &BigDecimal,
    purpose: Option<&str>,
    contract_address: Option<&str>,
    deploy_tx_hash: Option<&str>,
) -> Result<Loan, sqlx::Error> {
    sqlx::query_as::<_, Loan>(
        r#"
        INSERT INTO loans
            (borrower_id, amount_usdc, annual_rate, term_months, monthly_payment,
             purpose, contract_address, deploy_tx_hash, status, network)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'PENDING', 'polygon')
        RETURNING *
        "#,
    )
    .bind(borrower_id)
    .bind(amount_usdc)
    .bind(annual_rate)
    .bind(term_months)
    .bind(monthly_payment)
    .bind(purpose)
    .bind(contract_address)
    .bind(deploy_tx_hash)
    .fetch_one(pool)
    .await
}

/// Find a loan by its primary key.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Loan>, sqlx::Error> {
    sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// List loans for a borrower with cursor-based pagination (newest first).
/// `cursor_id` is the `id` of the last item returned on the previous page.
pub async fn list_by_borrower(
    pool: &PgPool,
    borrower_id: Uuid,
    cursor_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Loan>, sqlx::Error> {
    if let Some(cursor) = cursor_id {
        sqlx::query_as::<_, Loan>(
            r#"
            SELECT * FROM loans
            WHERE borrower_id = $1
              AND (created_at, id) < (
                  SELECT created_at, id FROM loans WHERE id = $2
              )
            ORDER BY created_at DESC, id DESC
            LIMIT $3
            "#,
        )
        .bind(borrower_id)
        .bind(cursor)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Loan>(
            r#"
            SELECT * FROM loans
            WHERE borrower_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2
            "#,
        )
        .bind(borrower_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

/// List loans in PENDING status available to fund, excluding the caller's own loans.
pub async fn list_available(
    pool: &PgPool,
    exclude_user_id: Uuid,
    cursor_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Loan>, sqlx::Error> {
    if let Some(cursor) = cursor_id {
        sqlx::query_as::<_, Loan>(
            r#"
            SELECT * FROM loans
            WHERE status = 'PENDING'
              AND borrower_id != $1
              AND (created_at, id) < (
                  SELECT created_at, id FROM loans WHERE id = $2
              )
            ORDER BY created_at DESC, id DESC
            LIMIT $3
            "#,
        )
        .bind(exclude_user_id)
        .bind(cursor)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Loan>(
            r#"
            SELECT * FROM loans
            WHERE status = 'PENDING'
              AND borrower_id != $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2
            "#,
        )
        .bind(exclude_user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

/// List loans where the user is the lender (portfolio view), newest first.
pub async fn list_by_lender(
    pool: &PgPool,
    lender_id: Uuid,
    cursor_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Loan>, sqlx::Error> {
    if let Some(cursor) = cursor_id {
        sqlx::query_as::<_, Loan>(
            r#"
            SELECT * FROM loans
            WHERE lender_id = $1
              AND (created_at, id) < (
                  SELECT created_at, id FROM loans WHERE id = $2
              )
            ORDER BY created_at DESC, id DESC
            LIMIT $3
            "#,
        )
        .bind(lender_id)
        .bind(cursor)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Loan>(
            r#"
            SELECT * FROM loans
            WHERE lender_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2
            "#,
        )
        .bind(lender_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

/// Update the status of a loan.
pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE loans SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update contract info after a successful blockchain deployment.
pub async fn update_contract_info(
    pool: &PgPool,
    id: Uuid,
    contract_address: &str,
    deploy_tx_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE loans
        SET contract_address = $1, deploy_tx_hash = $2, updated_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(contract_address)
    .bind(deploy_tx_hash)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update fund info after a lender funds the loan.
pub async fn update_fund_info(
    pool: &PgPool,
    id: Uuid,
    lender_id: Uuid,
    fund_tx_hash: &str,
    funded_at: DateTime<Utc>,
    due_date: DateTime<Utc>,
) -> Result<Loan, sqlx::Error> {
    sqlx::query_as::<_, Loan>(
        r#"
        UPDATE loans
        SET lender_id = $1, fund_tx_hash = $2, funded_at = $3, due_date = $4,
            status = 'FUNDED', updated_at = NOW()
        WHERE id = $5
        RETURNING *
        "#,
    )
    .bind(lender_id)
    .bind(fund_tx_hash)
    .bind(funded_at)
    .bind(due_date)
    .bind(id)
    .fetch_one(pool)
    .await
}
