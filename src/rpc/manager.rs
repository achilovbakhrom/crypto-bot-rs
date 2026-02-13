use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::chains::bitcoin::provider::BitcoinProvider;
use crate::chains::cardano::provider::CardanoProvider;
use crate::chains::evm::EvmProvider;
use crate::chains::solana::SolanaProvider;
use crate::chains::xrp::provider::XrpProvider;
use crate::config::Config;
use crate::enums::Chain;
use crate::error::{AppError, Result};
use crate::providers::ChainProvider;

struct ProviderPool {
    providers: Vec<Arc<dyn ChainProvider>>,
    current_index: RwLock<usize>,
}

pub struct RpcManager {
    pools: HashMap<Chain, ProviderPool>,
}

impl RpcManager {
    pub fn new(config: &Config) -> Result<Self> {
        let is_testnet = config.is_testnet();
        let mut pools = HashMap::new();

        for (chain, chain_config) in &config.chain_configs {
            let mut providers: Vec<Arc<dyn ChainProvider>> = Vec::new();

            for url in &chain_config.rpc_urls {
                if chain.is_evm() {
                    let chain_id = chain.chain_id(is_testnet).ok_or_else(|| {
                        AppError::Config(format!("No chain ID for {}", chain))
                    })?;
                    match EvmProvider::new(url, chain_id, &chain_config.native_symbol) {
                        Ok(p) => providers.push(Arc::new(p)),
                        Err(e) => tracing::warn!("Failed to create {} provider for {}: {}", chain, url, e),
                    }
                } else if *chain == Chain::Solana {
                    providers.push(Arc::new(SolanaProvider::new(url)));
                } else if *chain == Chain::Btc {
                    providers.push(Arc::new(BitcoinProvider::new(url, is_testnet)));
                } else if *chain == Chain::Xrp {
                    providers.push(Arc::new(XrpProvider::new(url)));
                } else if *chain == Chain::Cardano {
                    providers.push(Arc::new(CardanoProvider::new(url, is_testnet)));
                }
            }

            if providers.is_empty() {
                return Err(AppError::Config(format!(
                    "No valid RPC providers for {}",
                    chain
                )));
            }

            tracing::info!("Initialized {} with {} RPC provider(s)", chain, providers.len());

            pools.insert(*chain, ProviderPool {
                providers,
                current_index: RwLock::new(0),
            });
        }

        Ok(Self { pools })
    }

    /// Get a provider for the given chain (round-robin).
    pub async fn get_provider_by_chain(&self, chain: &str) -> Result<Arc<dyn ChainProvider>> {
        let parsed: Chain = chain.parse()?;
        let pool = self.pools.get(&parsed).ok_or_else(|| {
            AppError::Config(format!("Chain {} is not configured", chain))
        })?;

        let mut index = pool.current_index.write().await;
        let provider = pool.providers[*index].clone();
        *index = (*index + 1) % pool.providers.len();
        Ok(provider)
    }

    /// Rotate to the next provider for a chain (useful after failures).
    pub async fn rotate_provider(&self, chain: &str) -> Result<()> {
        let parsed: Chain = chain.parse()?;
        let pool = self.pools.get(&parsed).ok_or_else(|| {
            AppError::Config(format!("Chain {} is not configured", chain))
        })?;

        let mut index = pool.current_index.write().await;
        *index = (*index + 1) % pool.providers.len();
        tracing::info!("Rotated {} provider to index {}", chain, *index);
        Ok(())
    }

    /// Get all chains that have configured providers.
    pub fn get_configured_chains(&self) -> Vec<Chain> {
        self.pools.keys().copied().collect()
    }

    /// Check if a chain has providers configured.
    pub fn is_chain_configured(&self, chain: &Chain) -> bool {
        self.pools.contains_key(chain)
    }
}
