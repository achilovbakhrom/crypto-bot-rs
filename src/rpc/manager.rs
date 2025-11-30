use std::sync::Arc;
use tokio::sync::RwLock;

use crate::chains::{ evm::EvmProvider, solana::SolanaProvider };
use crate::config::{ Config, NetworkMode };
use crate::error::{ AppError, Result };
use crate::providers::ChainProvider;

pub struct RpcManager {
    eth_providers: Arc<RwLock<Vec<Arc<EvmProvider>>>>,
    bsc_providers: Arc<RwLock<Vec<Arc<EvmProvider>>>>,
    solana_providers: Arc<RwLock<Vec<Arc<SolanaProvider>>>>,
    eth_current_index: Arc<RwLock<usize>>,
    bsc_current_index: Arc<RwLock<usize>>,
    solana_current_index: Arc<RwLock<usize>>,
}

impl RpcManager {
    pub fn new(config: &Config) -> Result<Self> {
        // Initialize ETH providers
        let mut eth_providers = Vec::new();
        let (eth_chain_id, eth_symbol) = match config.network_mode {
            NetworkMode::Testnet => (11155111u64, "ETH"), // Sepolia
            NetworkMode::Mainnet => (1u64, "ETH"), // Ethereum Mainnet
        };

        for url in &config.eth_rpc_urls {
            match EvmProvider::new(url, eth_chain_id, eth_symbol) {
                Ok(provider) => eth_providers.push(Arc::new(provider)),
                Err(e) => tracing::warn!("Failed to create ETH provider for {}: {}", url, e),
            }
        }

        if eth_providers.is_empty() {
            return Err(AppError::Config("No valid ETH RPC providers configured".to_string()));
        }

        // Initialize BSC providers
        let mut bsc_providers = Vec::new();
        let (bsc_chain_id, bsc_symbol) = match config.network_mode {
            NetworkMode::Testnet => (97u64, "BNB"), // BSC Testnet
            NetworkMode::Mainnet => (56u64, "BNB"), // BSC Mainnet
        };

        for url in &config.bsc_rpc_urls {
            match EvmProvider::new(url, bsc_chain_id, bsc_symbol) {
                Ok(provider) => bsc_providers.push(Arc::new(provider)),
                Err(e) => tracing::warn!("Failed to create BSC provider for {}: {}", url, e),
            }
        }

        if bsc_providers.is_empty() {
            return Err(AppError::Config("No valid BSC RPC providers configured".to_string()));
        }

        // Initialize Solana providers
        let mut solana_providers = Vec::new();
        for url in &config.solana_rpc_urls {
            solana_providers.push(Arc::new(SolanaProvider::new(url)));
        }

        if solana_providers.is_empty() {
            return Err(AppError::Config("No valid Solana RPC providers configured".to_string()));
        }

        Ok(Self {
            eth_providers: Arc::new(RwLock::new(eth_providers)),
            bsc_providers: Arc::new(RwLock::new(bsc_providers)),
            solana_providers: Arc::new(RwLock::new(solana_providers)),
            eth_current_index: Arc::new(RwLock::new(0)),
            bsc_current_index: Arc::new(RwLock::new(0)),
            solana_current_index: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn get_eth_provider(&self) -> Result<Arc<EvmProvider>> {
        self.get_provider(&self.eth_providers, &self.eth_current_index, "ETH").await
    }

    pub async fn get_bsc_provider(&self) -> Result<Arc<EvmProvider>> {
        self.get_provider(&self.bsc_providers, &self.bsc_current_index, "BSC").await
    }

    pub async fn get_solana_provider(&self) -> Result<Arc<SolanaProvider>> {
        self.get_provider(&self.solana_providers, &self.solana_current_index, "Solana").await
    }

    async fn get_provider<T: Clone>(
        &self,
        providers: &Arc<RwLock<Vec<Arc<T>>>>,
        current_index: &Arc<RwLock<usize>>,
        chain_name: &str
    ) -> Result<Arc<T>> {
        let providers_guard = providers.read().await;
        let mut index_guard = current_index.write().await;

        if providers_guard.is_empty() {
            return Err(AppError::Rpc(format!("No {} providers available", chain_name)));
        }

        let provider = providers_guard[*index_guard].clone();

        // Round-robin to next provider for next request
        *index_guard = (*index_guard + 1) % providers_guard.len();

        Ok(provider)
    }

    pub async fn get_provider_by_chain(&self, chain: &str) -> Result<Arc<dyn ChainProvider>> {
        match chain.to_uppercase().as_str() {
            "ETH" | "ETHEREUM" => {
                let provider = self.get_eth_provider().await?;
                Ok(provider as Arc<dyn ChainProvider>)
            }
            "BSC" | "BNB" => {
                let provider = self.get_bsc_provider().await?;
                Ok(provider as Arc<dyn ChainProvider>)
            }
            "SOLANA" | "SOL" => {
                let provider = self.get_solana_provider().await?;
                Ok(provider as Arc<dyn ChainProvider>)
            }
            _ => Err(AppError::InvalidInput(format!("Unsupported chain: {}", chain))),
        }
    }

    // Rotate to next provider (useful when current one fails)
    pub async fn rotate_provider(&self, chain: &str) -> Result<()> {
        match chain.to_uppercase().as_str() {
            "ETH" | "ETHEREUM" => {
                let mut index = self.eth_current_index.write().await;
                let providers = self.eth_providers.read().await;
                *index = (*index + 1) % providers.len();
                tracing::info!("Rotated ETH provider to index {}", *index);
            }
            "BSC" | "BNB" => {
                let mut index = self.bsc_current_index.write().await;
                let providers = self.bsc_providers.read().await;
                *index = (*index + 1) % providers.len();
                tracing::info!("Rotated BSC provider to index {}", *index);
            }
            "SOLANA" | "SOL" => {
                let mut index = self.solana_current_index.write().await;
                let providers = self.solana_providers.read().await;
                *index = (*index + 1) % providers.len();
                tracing::info!("Rotated Solana provider to index {}", *index);
            }
            _ => {
                return Err(AppError::InvalidInput(format!("Unsupported chain: {}", chain)));
            }
        }
        Ok(())
    }
}
