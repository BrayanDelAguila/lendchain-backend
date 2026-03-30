pub mod polygon;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

// ─── Value objects ────────────────────────────────────────────────────────────

/// Receipt returned after submitting a transaction to the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxReceipt {
    pub tx_hash: String,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
}

/// Snapshot of an on-chain loan contract state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainLoanState {
    pub contract_address: String,
    pub is_funded: bool,
    pub is_repaid: bool,
    pub total_repaid_usdc: f64,
}

// ─── Trait ────────────────────────────────────────────────────────────────────

/// Abstraction over a blockchain network.
/// Implement this trait to support additional networks (e.g. Ethereum, Avalanche).
#[async_trait]
pub trait BlockchainAdapter: Send + Sync {
    /// Deploy a new loan smart contract and return its transaction receipt.
    async fn deploy_loan_contract(
        &self,
        loan_id: Uuid,
        borrower_address: &str,
        amount_usdc: f64,
        term_months: u32,
    ) -> Result<TxReceipt, AppError>;

    /// Transfer USDC from lender to the loan contract.
    async fn fund_loan(
        &self,
        contract_address: &str,
        lender_address: &str,
        amount_usdc: f64,
    ) -> Result<TxReceipt, AppError>;

    /// Record a borrower repayment on the loan contract.
    async fn record_payment(
        &self,
        contract_address: &str,
        payment_number: u32,
        amount_usdc: f64,
    ) -> Result<TxReceipt, AppError>;

    /// Fetch the current on-chain state of a loan contract.
    async fn get_loan_state(
        &self,
        contract_address: &str,
    ) -> Result<OnChainLoanState, AppError>;
}
