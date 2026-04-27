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
    pub contract_address: String,
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

    /// Transfer USDC from lender to the loan contract (calls fundLoan()).
    /// `lender_wallet_encrypted` is the AES-256-GCM encrypted private key.
    async fn fund_loan(
        &self,
        contract_address: &str,
        lender_wallet_encrypted: &str,
        encryption_key: &str,
    ) -> Result<TxReceipt, AppError>;

    /// Record a borrower repayment on the loan contract (calls makePayment()).
    /// `borrower_wallet_encrypted` is the AES-256-GCM encrypted private key.
    /// `amount_usdc` is in USDC units with 6 decimal places (e.g. 100 USDC = 100_000_000).
    async fn record_payment(
        &self,
        contract_address: &str,
        borrower_wallet_encrypted: &str,
        encryption_key: &str,
        amount_usdc: u64,
    ) -> Result<TxReceipt, AppError>;

    /// Fetch the current on-chain state of a loan contract.
    async fn get_loan_state(&self, contract_address: &str) -> Result<OnChainLoanState, AppError>;
}
