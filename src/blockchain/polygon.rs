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
            contract_address: "0x_stub_contract_address".to_string(),
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

    async fn get_loan_state(&self, contract_address: &str) -> Result<OnChainLoanState, AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_adapter() -> PolygonAdapter {
        PolygonAdapter::new(
            "https://rpc-mumbai.example.com".to_string(),
            80001,
            "0x0000000000000000000000000000000000000000".to_string(),
        )
    }

    #[tokio::test]
    async fn test_deploy_loan_contract_stub_ok() {
        let adapter = make_adapter();
        let result = adapter
            .deploy_loan_contract(uuid::Uuid::new_v4(), "0xBorrower", 1000.0, 12)
            .await;
        assert!(result.is_ok(), "deploy_loan_contract stub should return Ok");
        let receipt = result.unwrap();
        assert!(
            !receipt.tx_hash.is_empty(),
            "Stub receipt tx_hash should not be empty"
        );
    }

    #[tokio::test]
    async fn test_fund_loan_stub_ok() {
        let adapter = make_adapter();
        let result = adapter
            .fund_loan("0xContractAddress", "0xLenderAddress", 1000.0)
            .await;
        assert!(result.is_ok(), "fund_loan stub should return Ok");
    }

    #[tokio::test]
    async fn test_record_payment_stub_ok() {
        let adapter = make_adapter();
        let result = adapter.record_payment("0xContractAddress", 1, 85.61).await;
        assert!(result.is_ok(), "record_payment stub should return Ok");
    }

    #[tokio::test]
    async fn test_get_loan_state_stub_defaults() {
        let adapter = make_adapter();
        let contract = "0x1234567890abcdef1234567890abcdef12345678";
        let result = adapter.get_loan_state(contract).await;
        assert!(result.is_ok(), "get_loan_state stub should return Ok");
        let state = result.unwrap();
        assert!(
            !state.is_funded,
            "Stub loan state should have is_funded = false"
        );
        assert!(
            !state.is_repaid,
            "Stub loan state should have is_repaid = false"
        );
        assert_eq!(state.contract_address, contract);
    }
}
