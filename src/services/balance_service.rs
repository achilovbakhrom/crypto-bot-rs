use std::sync::Arc;
use uuid::Uuid;

use crate::crypto::Encryptor;
use crate::db::WalletRepository;
use crate::error::Result;
use crate::providers::Balance;
use crate::rpc::RpcManager;

pub struct BalanceService {
    repository: Arc<WalletRepository>,
    rpc_manager: Arc<RpcManager>,
    encryptor: Arc<Encryptor>,
}

impl BalanceService {
    pub fn new(
        repository: Arc<WalletRepository>,
        rpc_manager: Arc<RpcManager>,
        encryptor: Arc<Encryptor>
    ) -> Self {
        Self {
            repository,
            rpc_manager,
            encryptor,
        }
    }

    pub async fn get_balance(
        &self,
        wallet_id: Uuid,
        token_address: Option<String>
    ) -> Result<Balance> {
        // Get wallet from database
        let wallet = self.repository.find_by_id(wallet_id).await?;

        // Get appropriate provider
        let provider = self.rpc_manager.get_provider_by_chain(&wallet.chain).await?;

        // Get balance
        if let Some(token_addr) = token_address {
            provider.get_token_balance(&wallet.address, &token_addr).await
        } else {
            provider.get_balance(&wallet.address).await
        }
    }
}
