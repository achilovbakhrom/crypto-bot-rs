use std::sync::Arc;
use uuid::Uuid;

use crate::crypto::Encryptor;
use crate::db::WalletRepository;
use crate::error::Result;
use crate::rpc::RpcManager;

pub struct WalletService {
    repository: Arc<WalletRepository>,
    rpc_manager: Arc<RpcManager>,
    encryptor: Arc<Encryptor>,
}

impl WalletService {
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

    pub async fn generate_wallet(
        &self,
        user_id: String,
        chain: String,
        derivation_index: Option<u32>
    ) -> Result<GeneratedWalletResponse> {
        let provider = self.rpc_manager.get_provider_by_chain(&chain).await?;

        let wallet_info = provider.generate_wallet(derivation_index.unwrap_or(0)).await?;

        // Encrypt private key
        let encrypted_private_key = self.encryptor.encrypt(&wallet_info.private_key)?;

        // Save to database
        let wallet = self.repository.create(
            user_id,
            chain.clone(),
            wallet_info.address.clone(),
            encrypted_private_key
        ).await?;

        Ok(GeneratedWalletResponse {
            id: wallet.id,
            address: wallet_info.address,
            chain,
            mnemonic: wallet_info.mnemonic,
        })
    }

    pub async fn restore_wallet(
        &self,
        user_id: String,
        chain: String,
        secret: String,
        derivation_index: Option<u32>
    ) -> Result<RestoredWalletResponse> {
        let provider = self.rpc_manager.get_provider_by_chain(&chain).await?;

        let wallet_info = provider.restore_wallet(&secret, derivation_index.unwrap_or(0)).await?;

        // Encrypt private key
        let encrypted_private_key = self.encryptor.encrypt(&wallet_info.private_key)?;

        // Save to database
        let wallet = self.repository.create(
            user_id,
            chain.clone(),
            wallet_info.address.clone(),
            encrypted_private_key
        ).await?;

        Ok(RestoredWalletResponse {
            id: wallet.id,
            address: wallet_info.address,
            chain,
        })
    }

    pub async fn get_wallet(&self, wallet_id: Uuid) -> Result<crate::db::entity::wallet::Model> {
        self.repository.find_by_id(wallet_id).await
    }

    pub async fn list_user_wallets(
        &self,
        user_id: &str,
        chain: Option<&str>
    ) -> Result<Vec<crate::db::entity::wallet::Model>> {
        if let Some(chain) = chain {
            self.repository.find_by_user_and_chain(user_id, chain).await
        } else {
            self.repository.find_by_user(user_id).await
        }
    }
}

#[derive(serde::Serialize)]
pub struct GeneratedWalletResponse {
    pub id: Uuid,
    pub address: String,
    pub chain: String,
    pub mnemonic: Option<String>,
}

#[derive(serde::Serialize)]
pub struct RestoredWalletResponse {
    pub id: Uuid,
    pub address: String,
    pub chain: String,
}
