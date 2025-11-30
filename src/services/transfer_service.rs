use std::sync::Arc;
use uuid::Uuid;

use crate::crypto::Encryptor;
use crate::db::WalletRepository;
use crate::error::Result;
use crate::providers::{ TransactionRequest, TransactionResponse };
use crate::rpc::RpcManager;

pub struct TransferService {
    repository: Arc<WalletRepository>,
    rpc_manager: Arc<RpcManager>,
    encryptor: Arc<Encryptor>,
}

impl TransferService {
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

    pub async fn send_transaction(
        &self,
        wallet_id: Uuid,
        request: TransferRequest
    ) -> Result<TransactionResponse> {
        // Get wallet from database
        let wallet = self.repository.find_by_id(wallet_id).await?;

        // Decrypt private key
        let private_key = self.encryptor.decrypt(&wallet.encrypted_private_key)?;

        // Get appropriate provider
        let provider = self.rpc_manager.get_provider_by_chain(&wallet.chain).await?;

        // Validate destination address
        if !provider.validate_address(&request.to) {
            return Err(crate::error::AppError::InvalidAddress);
        }

        // Build transaction request
        let tx_request = TransactionRequest {
            from: wallet.address.clone(),
            to: request.to,
            amount: request.amount,
            token_address: request.token_address,
            max_fee_per_gas: request.max_fee_per_gas,
            max_priority_fee_per_gas: request.max_priority_fee_per_gas,
            gas_limit: request.gas_limit,
            compute_units: request.compute_units,
        };

        // Send transaction
        provider.send_transaction(&private_key, tx_request).await
    }
}

#[derive(serde::Deserialize)]
pub struct TransferRequest {
    pub to: String,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_units: Option<u32>,
}
