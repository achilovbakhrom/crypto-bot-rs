use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::db::TokenMetadataRepository;
use crate::enums::Chain;
use crate::error::{AppError, Result};
use crate::providers::TokenBalanceEntry;

/// Service for discovering all tokens held by a wallet address using Alchemy APIs.
#[derive(Clone)]
pub struct TokenDiscoveryService {
    client: reqwest::Client,
    api_key: String,
    token_repo: Arc<TokenMetadataRepository>,
}

// ── Alchemy JSON-RPC response types ────────────────────────────────

#[derive(Debug, Deserialize)]
struct AlchemyRpcResponse<T> {
    result: Option<T>,
    error: Option<AlchemyRpcError>,
}

#[derive(Debug, Deserialize)]
struct AlchemyRpcError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct AlchemyTokenBalancesResult {
    address: String,
    #[serde(rename = "tokenBalances")]
    token_balances: Vec<AlchemyTokenBalance>,
}

#[derive(Debug, Deserialize)]
struct AlchemyTokenBalance {
    #[serde(rename = "contractAddress")]
    contract_address: String,
    #[serde(rename = "tokenBalance")]
    token_balance: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlchemyTokenMetadataResult {
    name: Option<String>,
    symbol: Option<String>,
    decimals: Option<u8>,
    logo: Option<String>,
}

// ── Public types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalances {
    pub native: TokenBalanceEntry,
    pub tokens: Vec<TokenBalanceEntry>,
}

// ── Implementation ──────────────────────────────────────────────────

impl TokenDiscoveryService {
    pub fn new(api_key: String, token_repo: Arc<TokenMetadataRepository>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            api_key,
            token_repo,
        }
    }

    /// Whether Alchemy token discovery is supported for this chain.
    pub fn is_supported(&self, chain: &Chain) -> bool {
        // Alchemy supports token enumeration for EVM chains it knows about
        chain.alchemy_network_name(false).is_some() && chain.is_evm()
    }

    /// Get all ERC-20 token balances for an address on a given chain.
    pub async fn get_all_token_balances(
        &self,
        chain: Chain,
        address: &str,
        testnet: bool,
    ) -> Result<Vec<TokenBalanceEntry>> {
        let network = chain.alchemy_network_name(testnet).ok_or_else(|| {
            AppError::External(format!("Alchemy not available for {}", chain))
        })?;

        let url = format!(
            "https://{}.g.alchemy.com/v2/{}",
            network, self.api_key
        );

        // Step 1: fetch all token balances
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "alchemy_getTokenBalances",
            "params": [address, "erc20"]
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Alchemy request failed: {}", e)))?;

        let rpc_resp: AlchemyRpcResponse<AlchemyTokenBalancesResult> = response
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse Alchemy response: {}", e)))?;

        if let Some(err) = rpc_resp.error {
            return Err(AppError::External(format!("Alchemy error: {}", err.message)));
        }

        let result = match rpc_resp.result {
            Some(r) => r,
            None => return Ok(vec![]),
        };

        // Step 2: filter non-zero balances
        let non_zero: Vec<_> = result
            .token_balances
            .into_iter()
            .filter(|tb| {
                tb.token_balance
                    .as_deref()
                    .map(|b| b != "0x0" && b != "0x" && b != "0x0000000000000000000000000000000000000000000000000000000000000000")
                    .unwrap_or(false)
            })
            .collect();

        // Step 3: enrich with metadata
        let mut entries = Vec::with_capacity(non_zero.len());

        for tb in non_zero {
            let raw_balance = tb.token_balance.unwrap_or_default();

            let meta = self
                .get_or_fetch_metadata(chain, &tb.contract_address, &url)
                .await;

            let (symbol, name, decimals, logo_url) = match meta {
                Ok(m) => (m.symbol, m.name, m.decimals, m.logo_url),
                Err(_) => ("UNKNOWN".to_string(), "Unknown Token".to_string(), 18, None),
            };

            let formatted = format_hex_balance(&raw_balance, decimals);

            entries.push(TokenBalanceEntry {
                contract_address: tb.contract_address,
                symbol,
                name,
                decimals,
                balance: formatted,
                logo_url,
            });
        }

        Ok(entries)
    }

    /// Get or fetch token metadata. DB cache first, then Alchemy API, then store.
    async fn get_or_fetch_metadata(
        &self,
        chain: Chain,
        contract_address: &str,
        rpc_url: &str,
    ) -> Result<TokenMetadataInfo> {
        let address_lower = contract_address.to_lowercase();

        // Check DB cache
        if let Some(cached) = self
            .token_repo
            .find_by_chain_and_address(chain.as_str(), &address_lower)
            .await?
        {
            return Ok(TokenMetadataInfo {
                symbol: cached.symbol,
                name: cached.name,
                decimals: cached.decimals as u8,
                logo_url: cached.logo_url,
            });
        }

        // Fetch from Alchemy
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "alchemy_getTokenMetadata",
            "params": [contract_address]
        });

        let response = self
            .client
            .post(rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Alchemy metadata request failed: {}", e)))?;

        let rpc_resp: AlchemyRpcResponse<AlchemyTokenMetadataResult> = response
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse metadata response: {}", e)))?;

        let meta = rpc_resp.result.unwrap_or(AlchemyTokenMetadataResult {
            name: None,
            symbol: None,
            decimals: None,
            logo: None,
        });

        let symbol = meta.symbol.unwrap_or_else(|| "UNKNOWN".to_string());
        let name = meta.name.unwrap_or_else(|| "Unknown Token".to_string());
        let decimals = meta.decimals.unwrap_or(18);
        let logo_url = meta.logo;

        // Store in DB
        let _ = self
            .token_repo
            .upsert(
                chain.as_str(),
                &address_lower,
                &symbol,
                &name,
                decimals as i16,
                logo_url.clone(),
                None,
            )
            .await;

        Ok(TokenMetadataInfo {
            symbol,
            name,
            decimals,
            logo_url,
        })
    }
}

struct TokenMetadataInfo {
    symbol: String,
    name: String,
    decimals: u8,
    logo_url: Option<String>,
}

/// Parse a hex balance string (e.g. "0x1234") and format it with decimals.
fn format_hex_balance(hex_str: &str, decimals: u8) -> String {
    let hex_clean = hex_str.trim_start_matches("0x");
    let raw = u128::from_str_radix(hex_clean, 16).unwrap_or(0);

    if raw == 0 {
        return "0".to_string();
    }

    let divisor = 10u128.pow(decimals as u32);
    let whole = raw / divisor;
    let frac = raw % divisor;

    if frac == 0 {
        whole.to_string()
    } else {
        let frac_str = format!("{:0>width$}", frac, width = decimals as usize);
        let trimmed = frac_str.trim_end_matches('0');
        format!("{}.{}", whole, trimmed)
    }
}
