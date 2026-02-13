use std::collections::HashMap;
use std::env;

use serde::Deserialize;

use crate::enums::Chain;

#[derive(Debug, Clone, Deserialize)]
pub enum NetworkMode {
    Testnet,
    Mainnet,
}

/// Per-chain configuration resolved from environment variables.
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain: Chain,
    pub rpc_urls: Vec<String>,
    pub explorer_url: String,
    pub chain_id: Option<u64>,
    pub native_symbol: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub network_mode: NetworkMode,
    pub database_url: String,
    pub encryption_key: Vec<u8>,
    pub chain_configs: HashMap<Chain, ChainConfig>,
    pub alchemy_api_key: Option<String>,
    pub server_host: String,
    pub server_port: u16,
    pub rate_limit_per_user: u32,
    pub telegram_bot_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let network_mode = match env::var("NETWORK_MODE")?.to_lowercase().as_str() {
            "mainnet" => NetworkMode::Mainnet,
            "testnet" => NetworkMode::Testnet,
            _ => {
                return Err("NETWORK_MODE must be 'testnet' or 'mainnet'".into());
            }
        };

        let database_url = env::var("DATABASE_URL")?;

        let encryption_key_hex = env::var("ENCRYPTION_KEY")?;
        let encryption_key = hex::decode(&encryption_key_hex)
            .map_err(|_| "ENCRYPTION_KEY must be a valid hex string")?;

        if encryption_key.len() != 32 {
            return Err("ENCRYPTION_KEY must be 32 bytes (64 hex characters)".into());
        }

        let is_testnet = matches!(network_mode, NetworkMode::Testnet);
        let mode_suffix = if is_testnet { "TESTNET" } else { "MAINNET" };

        // Build chain configs dynamically from env vars
        let mut chain_configs = HashMap::new();

        for &chain in Chain::all() {
            let rpc_key = format!("{}_{}_RPC_URLS", chain.as_str(), mode_suffix);
            let explorer_key = format!("{}_{}_EXPLORER_URL", chain.as_str(), mode_suffix);

            // Only configure chains that have RPC URLs set
            if let Ok(rpc_val) = env::var(&rpc_key) {
                let rpc_urls = Self::parse_rpc_urls(&rpc_val)?;
                let explorer_url = env::var(&explorer_key)
                    .unwrap_or_else(|_| chain.explorer_url(is_testnet).to_string());

                chain_configs.insert(chain, ChainConfig {
                    chain,
                    rpc_urls,
                    explorer_url,
                    chain_id: chain.chain_id(is_testnet),
                    native_symbol: chain.native_symbol().to_string(),
                });
            }
        }

        if chain_configs.is_empty() {
            return Err("No chain RPC URLs configured. Set at least one *_RPC_URLS env var.".into());
        }

        let alchemy_api_key = env::var("ALCHEMY_API_KEY").ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?;
        let rate_limit_per_user = env::var("RATE_LIMIT_PER_USER")
            .unwrap_or_else(|_| "60".to_string())
            .parse()?;

        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN")?;

        Ok(Config {
            network_mode,
            database_url,
            encryption_key,
            chain_configs,
            alchemy_api_key,
            server_host,
            server_port,
            rate_limit_per_user,
            telegram_bot_token,
        })
    }

    fn parse_rpc_urls(urls_str: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let urls: Vec<String> = urls_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if urls.is_empty() {
            return Err("RPC URLs list cannot be empty".into());
        }

        Ok(urls)
    }

    /// Whether we are running in testnet mode.
    pub fn is_testnet(&self) -> bool {
        matches!(self.network_mode, NetworkMode::Testnet)
    }

    /// Get the explorer base URL for a specific chain.
    pub fn get_explorer_url(&self, chain: &str) -> String {
        match chain.parse::<Chain>() {
            Ok(c) => self
                .chain_configs
                .get(&c)
                .map(|cc| cc.explorer_url.clone())
                .unwrap_or_else(|| c.explorer_url(self.is_testnet()).to_string()),
            Err(_) => "https://etherscan.io".to_string(),
        }
    }

    /// Generate a transaction explorer URL for a specific chain and tx hash.
    pub fn get_tx_explorer_url(&self, chain: &str, tx_hash: &str) -> String {
        let base_url = self.get_explorer_url(chain);
        format!("{}/tx/{}", base_url, tx_hash)
    }

    /// Generate an address explorer URL for a specific chain and address.
    pub fn get_address_explorer_url(&self, chain: &str, address: &str) -> String {
        let base_url = self.get_explorer_url(chain);
        format!("{}/address/{}", base_url, address)
    }

    /// Generate a token explorer URL for a specific chain and token contract.
    pub fn get_token_explorer_url(&self, chain: &str, token_address: &str) -> String {
        let base_url = self.get_explorer_url(chain);
        format!("{}/token/{}", base_url, token_address)
    }

    /// Get list of configured chains.
    pub fn configured_chains(&self) -> Vec<Chain> {
        self.chain_configs.keys().copied().collect()
    }
}
