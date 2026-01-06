use async_trait::async_trait;
use serde::{ Deserialize, Serialize };

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub private_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: String,
    pub token_address: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub gas_limit: Option<u64>,
    pub compute_units: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub estimated_gas: u64,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub total_cost_native: String,
    pub total_cost_usd: Option<f64>,
}

#[async_trait]
pub trait ChainProvider: Send + Sync {
    /// Generate a new wallet with 24-word mnemonic
    async fn generate_wallet(&self, derivation_index: u32) -> Result<WalletInfo>;

    /// Restore wallet from mnemonic (12/24 words) or private key
    async fn restore_wallet(&self, secret: &str, derivation_index: u32) -> Result<WalletInfo>;

    /// Get native token balance
    async fn get_balance(&self, address: &str) -> Result<Balance>;

    /// Get token balance (ERC20/SPL)
    async fn get_token_balance(&self, address: &str, token_address: &str) -> Result<Balance>;

    /// Send transaction (native or token)
    async fn send_transaction(
        &self,
        private_key: &str,
        request: TransactionRequest
    ) -> Result<TransactionResponse>;

    /// Estimate gas for a transaction
    async fn estimate_gas(
        &self,
        from: &str,
        to: &str,
        amount: &str,
        token_address: Option<&str>
    ) -> Result<GasEstimate>;

    /// Validate address format
    fn validate_address(&self, address: &str) -> bool;
}
