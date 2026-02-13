use async_trait::async_trait;
use serde::Deserialize;

use crate::error::{AppError, Result};
use crate::providers::{
    Balance, ChainProvider, GasEstimate, TransactionRequest, TransactionResponse, WalletInfo,
};

use super::wallet;

#[derive(Clone)]
pub struct XrpProvider {
    client: reqwest::Client,
    rpc_url: String,
}

// ── XRP JSON-RPC response types ─────────────────────────────────────

#[derive(Debug, Deserialize)]
struct XrpRpcResponse<T> {
    result: T,
}

#[derive(Debug, Deserialize)]
struct AccountInfoResult {
    account_data: Option<AccountData>,
    status: String,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AccountData {
    #[serde(rename = "Balance")]
    balance: String,
    #[serde(rename = "Sequence")]
    sequence: u32,
}

#[derive(Debug, Deserialize)]
struct FeeResult {
    drops: FeeDrops,
}

#[derive(Debug, Deserialize)]
struct FeeDrops {
    open_ledger_fee: String,
    minimum_fee: String,
}

// ── Implementation ──────────────────────────────────────────────────

impl XrpProvider {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            rpc_url: rpc_url.to_string(),
        }
    }

    async fn rpc_call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "method": method,
            "params": [params]
        });

        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("XRP RPC request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::External(format!(
                "XRP RPC error: {}",
                resp.status()
            )));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse XRP RPC response: {}", e)))?;

        Ok(result)
    }

    fn drops_to_xrp(drops: &str) -> String {
        let drops_u64: u64 = drops.parse().unwrap_or(0);
        let xrp = drops_u64 as f64 / 1_000_000.0;
        format!("{:.6}", xrp)
    }
}

#[async_trait]
impl ChainProvider for XrpProvider {
    async fn generate_wallet(&self, derivation_index: u32) -> Result<WalletInfo> {
        wallet::generate_wallet(derivation_index)
    }

    async fn restore_wallet(&self, secret: &str, derivation_index: u32) -> Result<WalletInfo> {
        wallet::detect_and_restore(secret, derivation_index)
    }

    async fn get_balance(&self, address: &str) -> Result<Balance> {
        let resp = self
            .rpc_call(
                "account_info",
                serde_json::json!({
                    "account": address,
                    "strict": true,
                    "ledger_index": "current"
                }),
            )
            .await?;

        let result: XrpRpcResponse<AccountInfoResult> = serde_json::from_value(resp)
            .map_err(|e| AppError::External(format!("Failed to parse account_info: {}", e)))?;

        if let Some(error) = &result.result.error {
            if error == "actNotFound" {
                // Account not activated yet (needs 10 XRP reserve)
                return Ok(Balance {
                    balance: "0.000000".to_string(),
                    symbol: "XRP".to_string(),
                    decimals: 6,
                });
            }
            return Err(AppError::External(format!("XRP error: {}", error)));
        }

        let balance_drops = result
            .result
            .account_data
            .map(|d| d.balance)
            .unwrap_or_else(|| "0".to_string());

        Ok(Balance {
            balance: Self::drops_to_xrp(&balance_drops),
            symbol: "XRP".to_string(),
            decimals: 6,
        })
    }

    async fn get_token_balance(&self, _address: &str, _token_address: &str) -> Result<Balance> {
        Err(AppError::Validation(
            "XRP trust line tokens are not yet supported".to_string(),
        ))
    }

    async fn send_transaction(
        &self,
        _private_key: &str,
        _request: TransactionRequest,
    ) -> Result<TransactionResponse> {
        // Full XRP transaction flow:
        // 1. Get account_info for sequence number
        // 2. Get fee estimate
        // 3. Build Payment transaction
        // 4. Sign with Ed25519
        // 5. Submit signed blob
        Err(AppError::Validation(
            "XRP send_transaction not yet fully implemented. Coming soon.".to_string(),
        ))
    }

    async fn estimate_gas(
        &self,
        _from: &str,
        _to: &str,
        _amount: &str,
        _token_address: Option<&str>,
    ) -> Result<GasEstimate> {
        let resp = self
            .rpc_call("fee", serde_json::json!({}))
            .await?;

        let result: XrpRpcResponse<FeeResult> = serde_json::from_value(resp)
            .map_err(|e| AppError::External(format!("Failed to parse fee response: {}", e)))?;

        let fee_drops: u64 = result
            .result
            .drops
            .open_ledger_fee
            .parse()
            .unwrap_or(12);

        Ok(GasEstimate {
            estimated_gas: fee_drops,
            gas_price: Some(format!("{} drops", fee_drops)),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            total_cost_native: Self::drops_to_xrp(&fee_drops.to_string()),
            total_cost_usd: None,
        })
    }

    fn validate_address(&self, address: &str) -> bool {
        // XRP classic addresses start with 'r', are 25-35 characters, and are base58-encoded
        if !address.starts_with('r') || address.len() < 25 || address.len() > 35 {
            return false;
        }

        // Check valid base58 (Ripple alphabet)
        bs58::decode(address)
            .with_alphabet(bs58::Alphabet::RIPPLE)
            .into_vec()
            .is_ok()
    }
}
