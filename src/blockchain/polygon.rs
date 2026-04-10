use std::sync::Arc;

use async_trait::async_trait;
use ethers::{
    abi::Abi,
    contract::{Contract, ContractFactory},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, U256},
};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::blockchain::{BlockchainAdapter, OnChainLoanState, TxReceipt};
use crate::errors::AppError;

pub struct PolygonAdapter {
    pub rpc_url: String,
    pub chain_id: u64,
    /// USDC token contract address on this network.
    pub usdc_address: String,
    /// Private key (hex, no 0x prefix) of the backend deployer wallet.
    /// This wallet pays gas fees for all contract deployments.
    pub deployer_private_key: String,
}

impl PolygonAdapter {
    pub fn new(
        rpc_url: String,
        chain_id: u64,
        usdc_address: String,
        deployer_private_key: String,
    ) -> Self {
        Self {
            rpc_url,
            chain_id,
            usdc_address,
            deployer_private_key,
        }
    }

    fn provider(&self) -> Result<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>, AppError> {
        let provider = Provider::<Http>::try_from(self.rpc_url.as_str())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("RPC provider error: {}", e)))?;

        let wallet: LocalWallet = self
            .deployer_private_key
            .parse::<LocalWallet>()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Wallet parse error: {}", e)))?
            .with_chain_id(self.chain_id);

        Ok(Arc::new(SignerMiddleware::new(provider, wallet)))
    }
}

#[async_trait]
impl BlockchainAdapter for PolygonAdapter {
    /// Deploy a new LendChain contract for the given loan.
    /// Returns a TxReceipt with the deployed contract address and deploy tx hash.
    async fn deploy_loan_contract(
        &self,
        loan_id: Uuid,
        borrower_address: &str,
        amount_usdc: f64,
        term_months: u32,
    ) -> Result<TxReceipt, AppError> {
        let client = self.provider()?;

        let abi: Abi = serde_json::from_str(include_str!("contracts/lendchain_abi.json"))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("ABI parse error: {}", e)))?;

        let bytecode_str = include_str!("contracts/lendchain_bytecode.hex").trim();

        // If bytecode is still a placeholder, return a stub receipt (pre-Hardhat compilation).
        if bytecode_str == "placeholder" {
            tracing::warn!(loan_id = %loan_id, "Using stub receipt — bytecode not compiled yet");
            return Ok(TxReceipt {
                tx_hash: format!("0x_stub_{}", loan_id),
                contract_address: format!("0x_stub_contract_{}", loan_id),
                block_number: Some(0),
                gas_used: Some(0),
            });
        }

        let bytecode =
            Bytes::from(hex::decode(bytecode_str).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Bytecode decode error: {}", e))
            })?);

        let borrower: Address = borrower_address
            .parse()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid borrower address")))?;

        let usdc_addr: Address = self
            .usdc_address
            .parse()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid USDC address")))?;

        // USDC has 6 decimals — round to avoid floating-point truncation errors
        let amount_raw = (amount_usdc * 1_000_000.0).round() as u64;
        let amount_u256 = U256::from(amount_raw);
        let annual_rate_bps = U256::from(500u64); // 5.00% = 500 bps
        let term_u256 = U256::from(term_months);

        let factory = ContractFactory::new(abi, bytecode, client);

        let mut deployer = factory
            .deploy((
                usdc_addr,
                borrower,
                amount_u256,
                term_u256,
                annual_rate_bps,
                // No lender arg — assigned when someone calls fundLoan()
            ))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Deploy build error: {}", e)))?;
        deployer.tx.set_gas(U256::from(1_500_000u64));
        deployer.tx.set_gas_price(U256::from(35_000_000_000u64)); // 35 gwei

        let deploy_result = timeout(Duration::from_secs(60), deployer.send_with_receipt()).await;

        let (contract, receipt) = match deploy_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                return Err(AppError::BlockchainTxFailed(format!(
                    "Transaction failed: {}",
                    e
                )))
            }
            Err(_) => return Err(AppError::BlockchainTimeout),
        };

        tracing::info!(
            loan_id = %loan_id,
            contract = %contract.address(),
            tx = ?receipt.transaction_hash,
            "LendChain contract deployed"
        );

        Ok(TxReceipt {
            tx_hash: format!("{:?}", receipt.transaction_hash),
            contract_address: format!("{:?}", contract.address()),
            block_number: receipt.block_number.map(|b| b.as_u64()),
            gas_used: receipt.gas_used.map(|g| g.as_u64()),
        })
    }

    /// Transfer USDC from lender to the loan contract.
    /// TODO Fase 4b: call fundLoan() on the deployed contract once lender wallets are integrated.
    async fn fund_loan(
        &self,
        contract_address: &str,
        _lender_address: &str,
        _amount_usdc: f64,
    ) -> Result<TxReceipt, AppError> {
        tracing::info!(contract = %contract_address, "fund_loan stub called");
        Ok(TxReceipt {
            tx_hash: "0x_stub_fund_tx".to_string(),
            contract_address: contract_address.to_string(),
            block_number: Some(0),
            gas_used: Some(0),
        })
    }

    /// Record a borrower repayment on the loan contract.
    /// TODO Fase 4b: call makePayment() on the deployed contract.
    async fn record_payment(
        &self,
        contract_address: &str,
        payment_number: u32,
        _amount_usdc: f64,
    ) -> Result<TxReceipt, AppError> {
        tracing::info!(
            contract = %contract_address,
            payment = payment_number,
            "record_payment stub called"
        );
        Ok(TxReceipt {
            tx_hash: "0x_stub_payment_tx".to_string(),
            contract_address: contract_address.to_string(),
            block_number: Some(0),
            gas_used: Some(0),
        })
    }

    /// Fetch the current on-chain state of a loan contract.
    async fn get_loan_state(&self, contract_address: &str) -> Result<OnChainLoanState, AppError> {
        let client = self.provider()?;

        let abi: Abi = serde_json::from_str(include_str!("contracts/lendchain_abi.json"))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("ABI parse error: {}", e)))?;

        let addr: Address = contract_address
            .parse()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid contract address")))?;

        let contract = Contract::new(addr, abi, client);

        let state_result = timeout(
            Duration::from_secs(15),
            contract
                .method::<_, (u8, U256, U256, U256)>("getState", ())
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Method lookup error: {}", e)))?
                .call(),
        )
        .await;

        let (status, total_repaid, _funded_at, _amount) = match state_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => return Err(AppError::BlockchainTxFailed(e.to_string())),
            Err(_) => return Err(AppError::BlockchainTimeout),
        };

        Ok(OnChainLoanState {
            contract_address: contract_address.to_string(),
            is_funded: status >= 1,
            is_repaid: status >= 2,
            total_repaid_usdc: total_repaid.as_u64() as f64 / 1_000_000.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_adapter() -> PolygonAdapter {
        PolygonAdapter::new(
            "https://rpc-amoy.example.com".to_string(),
            80002,
            "0x0000000000000000000000000000000000000000".to_string(),
            // Valid-format private key (all zeroes + 1) — no funds, never touches network
            "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        )
    }

    #[test]
    fn test_provider_builds_with_valid_key() {
        let adapter = make_adapter();
        let result = adapter.provider();
        assert!(
            result.is_ok(),
            "provider() should succeed with a valid private key"
        );
    }

    #[test]
    fn test_provider_fails_with_invalid_key() {
        let adapter = PolygonAdapter::new(
            "https://rpc-amoy.example.com".to_string(),
            80002,
            "0x0000000000000000000000000000000000000000".to_string(),
            "not_a_valid_hex_private_key".to_string(),
        );
        let result = adapter.provider();
        assert!(
            result.is_err(),
            "provider() should fail with an invalid private key"
        );
    }

    #[test]
    fn test_abi_parses_correctly() {
        let abi: Result<Abi, _> =
            serde_json::from_str(include_str!("contracts/lendchain_abi.json"));
        assert!(abi.is_ok(), "lendchain_abi.json should be valid ABI JSON");
    }
}
