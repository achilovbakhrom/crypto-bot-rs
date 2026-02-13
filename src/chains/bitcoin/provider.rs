use async_trait::async_trait;
use serde::Deserialize;

use crate::error::{AppError, Result};
use crate::providers::{
    Balance, ChainProvider, GasEstimate, TransactionRequest, TransactionResponse, WalletInfo,
};

use super::wallet;

#[derive(Clone)]
pub struct BitcoinProvider {
    client: reqwest::Client,
    base_url: String,
    testnet: bool,
}

// ── Esplora API response types ──────────────────────────────────────

#[derive(Debug, Deserialize)]
struct EsploraAddressStats {
    funded_txo_sum: u64,
    spent_txo_sum: u64,
}

#[derive(Debug, Deserialize)]
struct EsploraAddress {
    chain_stats: EsploraAddressStats,
    mempool_stats: EsploraAddressStats,
}

#[derive(Debug, Deserialize)]
struct EsploraUtxo {
    txid: String,
    vout: u32,
    value: u64,
    status: EsploraUtxoStatus,
}

#[derive(Debug, Deserialize)]
struct EsploraUtxoStatus {
    confirmed: bool,
}

// ── Implementation ──────────────────────────────────────────────────

impl BitcoinProvider {
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

    async fn get_utxos(&self, address: &str) -> Result<Vec<EsploraUtxo>> {
        let url = format!("{}/address/{}/utxo", self.base_url, address);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Esplora request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::External(format!(
                "Esplora API error: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse UTXOs: {}", e)))
    }

    async fn get_fee_rate(&self) -> Result<f64> {
        let url = format!("{}/fee-estimates", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Fee estimate request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(10.0); // fallback: 10 sat/vbyte
        }

        let estimates: std::collections::HashMap<String, f64> = resp
            .json()
            .await
            .unwrap_or_default();

        // Target 6-block confirmation (~1 hour)
        Ok(estimates
            .get("6")
            .copied()
            .unwrap_or(10.0))
    }

    fn satoshis_to_btc(sats: u64) -> String {
        let btc = sats as f64 / 100_000_000.0;
        format!("{:.8}", btc)
    }
}

#[async_trait]
impl ChainProvider for BitcoinProvider {
    async fn generate_wallet(&self, derivation_index: u32) -> Result<WalletInfo> {
        wallet::generate_wallet(self.testnet, derivation_index)
    }

    async fn restore_wallet(&self, secret: &str, derivation_index: u32) -> Result<WalletInfo> {
        wallet::detect_and_restore(secret, self.testnet, derivation_index)
    }

    async fn get_balance(&self, address: &str) -> Result<Balance> {
        let url = format!("{}/address/{}", self.base_url, address);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Esplora request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::External(format!(
                "Esplora API error: {}",
                resp.status()
            )));
        }

        let addr_info: EsploraAddress = resp
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse address info: {}", e)))?;

        let confirmed_sats =
            addr_info.chain_stats.funded_txo_sum - addr_info.chain_stats.spent_txo_sum;
        let unconfirmed_sats =
            addr_info.mempool_stats.funded_txo_sum - addr_info.mempool_stats.spent_txo_sum;
        let total_sats = confirmed_sats + unconfirmed_sats;

        Ok(Balance {
            balance: Self::satoshis_to_btc(total_sats),
            symbol: "BTC".to_string(),
            decimals: 8,
        })
    }

    async fn get_token_balance(&self, _address: &str, _token_address: &str) -> Result<Balance> {
        Err(AppError::Validation(
            "Bitcoin does not support tokens".to_string(),
        ))
    }

    async fn send_transaction(
        &self,
        _private_key: &str,
        _request: TransactionRequest,
    ) -> Result<TransactionResponse> {
        // Full UTXO transaction construction:
        // 1. Fetch UTXOs for the sender address
        // 2. Select UTXOs to cover amount + fee
        // 3. Build transaction with inputs/outputs/change
        // 4. Sign each input (SegWit)
        // 5. Broadcast via POST /tx
        Err(AppError::Validation(
            "Bitcoin send_transaction not yet fully implemented. Coming soon.".to_string(),
        ))
    }

    async fn estimate_gas(
        &self,
        _from: &str,
        _to: &str,
        _amount: &str,
        _token_address: Option<&str>,
    ) -> Result<GasEstimate> {
        let fee_rate = self.get_fee_rate().await?;

        // Typical SegWit P2WPKH transaction: ~140 vbytes for 1 input, 2 outputs
        let estimated_vsize: u64 = 140;
        let fee_sats = (estimated_vsize as f64 * fee_rate).ceil() as u64;

        Ok(GasEstimate {
            estimated_gas: estimated_vsize,
            gas_price: Some(format!("{:.1} sat/vB", fee_rate)),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            total_cost_native: Self::satoshis_to_btc(fee_sats),
            total_cost_usd: None,
        })
    }

    fn validate_address(&self, address: &str) -> bool {
        use std::str::FromStr;
        bitcoin::Address::from_str(address)
            .map(|addr| {
                let network = if self.testnet {
                    bitcoin::Network::Testnet
                } else {
                    bitcoin::Network::Bitcoin
                };
                addr.is_valid_for_network(network)
            })
            .unwrap_or(false)
    }
}
