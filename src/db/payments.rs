use sqlx::types::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::payment::Payment;

/// A single row to insert into loan_payments.
pub struct PaymentRow {
    pub payment_number: i16,
    pub amount_usdc: BigDecimal,
    pub principal: BigDecimal,
    pub interest: BigDecimal,
}

/// Bulk-insert the full amortisation schedule for a loan.
pub async fn insert_payment_schedule(
    pool: &PgPool,
    loan_id: Uuid,
    rows: &[PaymentRow],
) -> Result<(), sqlx::Error> {
    if rows.is_empty() {
        return Ok(());
    }

    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO loan_payments \
         (loan_id, payment_number, amount_usdc, principal, interest, status) ",
    );

    qb.push_values(rows, |mut b, row| {
        b.push_bind(loan_id)
            .push_bind(row.payment_number)
            .push_bind(&row.amount_usdc)
            .push_bind(&row.principal)
            .push_bind(&row.interest)
            .push_bind("PENDING");
    });

    qb.build().execute(pool).await?;
    Ok(())
}

/// List all payment rows for a loan, ordered by payment_number ASC.
pub async fn list_by_loan(pool: &PgPool, loan_id: Uuid) -> Result<Vec<Payment>, sqlx::Error> {
    sqlx::query_as::<_, Payment>(
        "SELECT * FROM loan_payments WHERE loan_id = $1 ORDER BY payment_number ASC",
    )
    .bind(loan_id)
    .fetch_all(pool)
    .await
}

/// Mark a payment as CONFIRMED and record the on-chain tx hash.
pub async fn confirm_payment(
    pool: &PgPool,
    payment_id: Uuid,
    tx_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE loan_payments SET status = 'CONFIRMED', tx_hash = $1, paid_at = NOW() WHERE id = $2",
    )
    .bind(tx_hash)
    .bind(payment_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Find a single payment by id.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as::<_, Payment>("SELECT * FROM loan_payments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}
