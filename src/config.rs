use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub enum NetworkMode {
    Testnet,
    Mainnet,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub network_mode: NetworkMode,
    pub database_url: String,
    pub encryption_key: Vec<u8>,
    pub eth_rpc_urls: Vec<String>,
    pub bsc_rpc_urls: Vec<String>,
    pub solana_rpc_urls: Vec<String>,
    pub eth_explorer_url: String,
    pub bsc_explorer_url: String,
    pub solana_explorer_url: String,
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
        let encryption_key = hex
            ::decode(&encryption_key_hex)
            .map_err(|_| "ENCRYPTION_KEY must be a valid hex string")?;

        if encryption_key.len() != 32 {
            return Err("ENCRYPTION_KEY must be 32 bytes (64 hex characters)".into());
        }

        // Select RPC URLs and Explorer URLs based on network mode
        let (eth_rpc_key, bsc_rpc_key, solana_rpc_key, eth_explorer_key, bsc_explorer_key, solana_explorer_key) = match network_mode {
            NetworkMode::Testnet => (
                "ETH_TESTNET_RPC_URLS",
                "BSC_TESTNET_RPC_URLS",
                "SOLANA_TESTNET_RPC_URLS",
                "ETH_TESTNET_EXPLORER_URL",
                "BSC_TESTNET_EXPLORER_URL",
                "SOLANA_TESTNET_EXPLORER_URL",
            ),
            NetworkMode::Mainnet => (
                "ETH_MAINNET_RPC_URLS",
                "BSC_MAINNET_RPC_URLS",
                "SOLANA_MAINNET_RPC_URLS",
                "ETH_MAINNET_EXPLORER_URL",
                "BSC_MAINNET_EXPLORER_URL",
                "SOLANA_MAINNET_EXPLORER_URL",
            ),
        };

        let eth_rpc_urls = Self::parse_rpc_urls(&env::var(eth_rpc_key)?)?;
        let bsc_rpc_urls = Self::parse_rpc_urls(&env::var(bsc_rpc_key)?)?;
        let solana_rpc_urls = Self::parse_rpc_urls(&env::var(solana_rpc_key)?)?;

        // Explorer URLs with defaults
        let eth_explorer_url = env::var(eth_explorer_key)
            .unwrap_or_else(|_| "https://etherscan.io".to_string());
        let bsc_explorer_url = env::var(bsc_explorer_key)
            .unwrap_or_else(|_| "https://bscscan.com".to_string());
        let solana_explorer_url = env::var(solana_explorer_key)
            .unwrap_or_else(|_| "https://explorer.solana.com".to_string());

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env
            ::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?;
        let rate_limit_per_user = env
            ::var("RATE_LIMIT_PER_USER")
            .unwrap_or_else(|_| "60".to_string())
            .parse()?;

        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN")?;

        Ok(Config {
            network_mode,
            database_url,
            encryption_key,
            eth_rpc_urls,
            bsc_rpc_urls,
            solana_rpc_urls,
            eth_explorer_url,
            bsc_explorer_url,
            solana_explorer_url,
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

    /// Get the explorer base URL for a specific chain
    pub fn get_explorer_url(&self, chain: &str) -> &str {
        match chain.to_uppercase().as_str() {
            "ETH" | "ETHEREUM" => &self.eth_explorer_url,
            "BSC" | "BNB" => &self.bsc_explorer_url,
            "SOLANA" | "SOL" => &self.solana_explorer_url,
            _ => &self.eth_explorer_url, // Default to ETH
        }
    }

    /// Generate a transaction explorer URL for a specific chain and tx hash
    pub fn get_tx_explorer_url(&self, chain: &str, tx_hash: &str) -> String {
        let base_url = self.get_explorer_url(chain);
        match chain.to_uppercase().as_str() {
            "SOLANA" | "SOL" => format!("{}/tx/{}", base_url, tx_hash),
            _ => format!("{}/tx/{}", base_url, tx_hash), // ETH, BSC use same format
        }
    }
}
