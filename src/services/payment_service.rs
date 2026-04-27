use std::sync::Arc;

use bigdecimal::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::blockchain::BlockchainAdapter;
use crate::config::Config;
use crate::db::{loans as db_loans, payments as db_payments, users as db_users};
use crate::errors::AppError;
use crate::services::loan_service::polygonscan_url;

#[derive(Debug, Serialize)]
pub struct PayInstallmentResult {
    pub payment_number: i16,
    pub amount_usdc: String,
    pub tx_hash: String,
    pub polygonscan_url: String,
}

pub async fn pay_installment(
    pool: &PgPool,
    blockchain: &Arc<dyn BlockchainAdapter>,
    config: &Config,
    loan_id: Uuid,
    user_id: Uuid,
) -> Result<PayInstallmentResult, AppError> {
    let loan = db_loans::find_by_id(pool, loan_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if loan.borrower_id != user_id {
        return Err(AppError::Forbidden);
    }
    if loan.status != "FUNDED" {
        return Err(AppError::InvalidState(
            "Loan must be FUNDED to accept payments".into(),
        ));
    }

    let contract_address = loan
        .contract_address
        .as_deref()
        .ok_or_else(|| AppError::InvalidState("Loan has no deployed contract".into()))?;

    let payments = db_payments::list_by_loan(pool, loan_id).await?;
    let next_payment = payments
        .iter()
        .find(|p| p.status == "PENDING")
        .ok_or_else(|| AppError::InvalidState("No pending payments found".into()))?;

    let borrower = db_users::find_by_id(pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let amount_u64 = next_payment
        .amount_usdc
        .to_f64()
        .map(|f| (f * 1_000_000.0).round() as u64)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("BigDecimal conversion failed")))?;

    let receipt = blockchain
        .record_payment(
            contract_address,
            &borrower.encrypted_private_key,
            &config.wallet_encryption_key,
            amount_u64,
        )
        .await?;

    sqlx::query(
        "UPDATE loan_payments SET status = 'CONFIRMED', tx_hash = $1, paid_at = NOW() WHERE id = $2",
    )
    .bind(&receipt.tx_hash)
    .bind(next_payment.id)
    .execute(pool)
    .await?;

    let confirmed_count = payments.iter().filter(|p| p.status == "CONFIRMED").count() + 1;
    if confirmed_count >= payments.len() {
        db_loans::update_status(pool, loan_id, "REPAID").await?;
    }

    let tx_url = polygonscan_url(config.polygon_chain_id, &receipt.tx_hash);

    Ok(PayInstallmentResult {
        payment_number: next_payment.payment_number,
        amount_usdc: next_payment.amount_usdc.to_string(),
        tx_hash: receipt.tx_hash,
        polygonscan_url: tx_url,
    })
}
