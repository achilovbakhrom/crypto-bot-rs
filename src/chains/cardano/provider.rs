use async_trait::async_trait;
use serde::Deserialize;

use crate::error::{AppError, Result};
use crate::providers::{
    Balance, ChainProvider, GasEstimate, TransactionRequest, TransactionResponse, WalletInfo,
};

use super::wallet;

#[derive(Clone)]
pub struct CardanoProvider {
    client: reqwest::Client,
    base_url: String,
    testnet: bool,
}

// ── Koios API response types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct KoiosAddressInfo {
    balance: String,
}

// ── Implementation ──────────────────────────────────────────────────

impl CardanoProvider {
    pub fn new(base_url: &str, testnet: bool) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            base_url: base_url.trim_end_matches('/').to_string(),
            testnet,
        }
    }

    fn lovelace_to_ada(lovelace: u64) -> String {
        let ada = lovelace as f64 / 1_000_000.0;
        format!("{:.6}", ada)
    }
}

#[async_trait]
impl ChainProvider for CardanoProvider {
    async fn generate_wallet(&self, derivation_index: u32) -> Result<WalletInfo> {
        wallet::generate_wallet(self.testnet, derivation_index)
    }

    async fn restore_wallet(&self, secret: &str, derivation_index: u32) -> Result<WalletInfo> {
        wallet::detect_and_restore(secret, self.testnet, derivation_index)
    }

    async fn get_balance(&self, address: &str) -> Result<Balance> {
        let url = format!("{}/address_info", self.base_url);

        let body = serde_json::json!({
            "_addresses": [address]
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Koios request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::External(format!(
                "Koios API error: {}",
                resp.status()
            )));
        }

        let data: Vec<KoiosAddressInfo> = resp
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse Koios response: {}", e)))?;

        // Empty array = unfunded address
        let lovelace: u64 = data
            .first()
            .map(|info| info.balance.parse::<u64>().unwrap_or(0))
            .unwrap_or(0);

        Ok(Balance {
            balance: Self::lovelace_to_ada(lovelace),
            symbol: "ADA".to_string(),
            decimals: 6,
        })
    }

    async fn get_token_balance(&self, _address: &str, _token_address: &str) -> Result<Balance> {
        Err(AppError::Validation(
            "Cardano native assets are not yet supported".to_string(),
        ))
    }

    async fn send_transaction(
        &self,
        _private_key: &str,
        _request: TransactionRequest,
    ) -> Result<TransactionResponse> {
        Err(AppError::Validation(
            "Cardano send_transaction not yet fully implemented. Coming soon.".to_string(),
        ))
    }

    async fn estimate_gas(
        &self,
        _from: &str,
        _to: &str,
        _amount: &str,
        _token_address: Option<&str>,
    ) -> Result<GasEstimate> {
        // Cardano has a protocol-level minimum fee formula:
        // fee = a * tx_size + b (currently a=44, b=155381 lovelace)
        // A typical simple transaction is ~300 bytes → ~168,581 lovelace ≈ 0.17 ADA
        let min_fee_lovelace: u64 = 170_000;

        Ok(GasEstimate {
            estimated_gas: min_fee_lovelace,
            gas_price: Some("~0.17 ADA min fee".to_string()),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            total_cost_native: Self::lovelace_to_ada(min_fee_lovelace),
            total_cost_usd: None,
        })
    }

    fn validate_address(&self, address: &str) -> bool {
        if self.testnet {
            address.starts_with("addr_test1") && address.len() > 20
        } else {
            address.starts_with("addr1") && !address.starts_with("addr_test") && address.len() > 20
        }
    }
}
