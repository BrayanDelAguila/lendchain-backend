use async_trait::async_trait;
use uuid::Uuid;

use crate::blockchain::{BlockchainAdapter, OnChainLoanState, TxReceipt};
use crate::errors::AppError;

/// Stub implementation of BlockchainAdapter for Polygon.
/// All methods return a hardcoded success receipt until the real Ethers/Alloy integration
/// is implemented.
pub struct PolygonAdapter {
    pub rpc_url: String,
    pub chain_id: u64,
    pub contract_address: String,
}

impl PolygonAdapter {
    pub fn new(rpc_url: String, chain_id: u64, contract_address: String) -> Self {
        Self {
            rpc_url,
            chain_id,
            contract_address,
        }
    }

    fn stub_receipt() -> TxReceipt {
        TxReceipt {
            tx_hash: "0x_stub_polygon_tx_hash".to_string(),
            block_number: Some(0),
            gas_used: Some(0),
        }
    }
}

#[async_trait]
impl BlockchainAdapter for PolygonAdapter {
    async fn deploy_loan_contract(
        &self,
        _loan_id: Uuid,
        _borrower_address: &str,
        _amount_usdc: f64,
        _term_months: u32,
    ) -> Result<TxReceipt, AppError> {
        // TODO: deploy LendChain contract via Ethers / Alloy
        tracing::info!(
            chain_id = self.chain_id,
            rpc = %self.rpc_url,
            "deploy_loan_contract stub called"
        );
        Ok(Self::stub_receipt())
    }

    async fn fund_loan(
        &self,
        _contract_address: &str,
        _lender_address: &str,
        _amount_usdc: f64,
    ) -> Result<TxReceipt, AppError> {
        // TODO: call fundLoan() on the deployed contract
        tracing::info!("fund_loan stub called");
        Ok(Self::stub_receipt())
    }

    async fn record_payment(
        &self,
        _contract_address: &str,
        _payment_number: u32,
        _amount_usdc: f64,
    ) -> Result<TxReceipt, AppError> {
        // TODO: call recordPayment() on the deployed contract
        tracing::info!("record_payment stub called");
        Ok(Self::stub_receipt())
    }

    async fn get_loan_state(
        &self,
        contract_address: &str,
    ) -> Result<OnChainLoanState, AppError> {
        // TODO: query contract state from the chain
        tracing::info!(contract = %contract_address, "get_loan_state stub called");
        Ok(OnChainLoanState {
            contract_address: contract_address.to_string(),
            is_funded: false,
            is_repaid: false,
            total_repaid_usdc: 0.0,
        })
    }
}
