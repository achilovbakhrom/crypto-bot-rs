use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use crate::crypto::Encryptor;
use crate::db::WalletRepository;
use crate::enums::Chain;
use crate::error::Result;
use crate::providers::{Balance, TokenBalanceEntry};
use crate::rpc::RpcManager;
use crate::services::token_discovery_service::TokenDiscoveryService;

#[derive(Debug, Clone, Serialize)]
pub struct WalletBalances {
    pub wallet_id: String,
    pub chain: String,
    pub address: String,
    pub native: Balance,
    pub tokens: Vec<TokenBalanceEntry>,
}

pub struct BalanceService {
    repository: Arc<WalletRepository>,
    rpc_manager: Arc<RpcManager>,
    encryptor: Arc<Encryptor>,
    token_discovery: Option<Arc<TokenDiscoveryService>>,
    is_testnet: bool,
}

impl BalanceService {
    pub fn new(
        repository: Arc<WalletRepository>,
        rpc_manager: Arc<RpcManager>,
        encryptor: Arc<Encryptor>,
        token_discovery: Option<Arc<TokenDiscoveryService>>,
        is_testnet: bool,
    ) -> Self {
        Self {
            repository,
            rpc_manager,
            encryptor,
            token_discovery,
            is_testnet,
        }
    }

    pub async fn get_balance(
        &self,
        wallet_id: Uuid,
        token_address: Option<String>,
    ) -> Result<Balance> {
        let wallet = self.repository.find_by_id(wallet_id).await?;
        let provider = self.rpc_manager.get_provider_by_chain(&wallet.chain).await?;

        if let Some(token_addr) = token_address {
            provider.get_token_balance(&wallet.address, &token_addr).await
        } else {
            provider.get_balance(&wallet.address).await
        }
    }

    /// Get native balance + all discovered token balances for a wallet.
    pub async fn get_all_balances(&self, wallet_id: Uuid) -> Result<WalletBalances> {
        let wallet = self.repository.find_by_id(wallet_id).await?;
        let provider = self.rpc_manager.get_provider_by_chain(&wallet.chain).await?;

        let native = provider.get_balance(&wallet.address).await?;

        let tokens = if let Some(ref discovery) = self.token_discovery {
            if let Ok(chain) = wallet.chain.parse::<Chain>() {
                if discovery.is_supported(&chain) {
                    discovery
                        .get_all_token_balances(chain, &wallet.address, self.is_testnet)
                        .await
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Ok(WalletBalances {
            wallet_id: wallet.id.to_string(),
            chain: wallet.chain,
            address: wallet.address,
            native,
            tokens,
        })
    }
}
