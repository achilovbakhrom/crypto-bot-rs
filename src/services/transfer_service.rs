use std::sync::Arc;
use uuid::Uuid;

use crate::crypto::Encryptor;
use crate::db::{ WalletRepository, TransactionRepository };
use crate::enums::{ Chain, TxStatus };
use crate::error::Result;
use crate::providers::{ TransactionRequest, TransactionResponse };
use crate::rpc::RpcManager;

pub struct TransferService {
    repository: Arc<WalletRepository>,
    transaction_repo: Arc<TransactionRepository>,
    rpc_manager: Arc<RpcManager>,
    encryptor: Arc<Encryptor>,
}

impl TransferService {
    pub fn new(
        repository: Arc<WalletRepository>,
        transaction_repo: Arc<TransactionRepository>,
        rpc_manager: Arc<RpcManager>,
        encryptor: Arc<Encryptor>
    ) -> Self {
        Self {
            repository,
            transaction_repo,
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
            to: request.to.clone(),
            amount: request.amount.clone(),
            token_address: request.token_address.clone(),
            max_fee_per_gas: request.max_fee_per_gas,
            max_priority_fee_per_gas: request.max_priority_fee_per_gas,
            gas_limit: request.gas_limit,
            compute_units: request.compute_units,
        };

        // Send transaction
        let response = provider.send_transaction(&private_key, tx_request).await?;

        // Log transaction to database
        let token_symbol = if request.token_address.is_some() {
            Some("TOKEN".to_string())
        } else {
            wallet.chain.parse::<Chain>().ok().map(|c| c.native_symbol().to_string())
        };

        self.transaction_repo.create(
            wallet_id,
            response.tx_hash.clone(),
            wallet.chain.clone(),
            wallet.address.clone(),
            request.to,
            request.amount,
            request.token_address,
            token_symbol,
            response.status.clone()
        ).await?;

        Ok(response)
    }

    pub async fn send_batch_transactions(
        &self,
        wallet_id: Uuid,
        recipients: Vec<BatchRecipient>
    ) -> Result<BatchTransferResult> {
        // Get wallet from database
        let wallet = self.repository.find_by_id(wallet_id).await?;

        // Decrypt private key
        let private_key = self.encryptor.decrypt(&wallet.encrypted_private_key)?;

        // Get appropriate provider
        let provider = self.rpc_manager.get_provider_by_chain(&wallet.chain).await?;

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for (index, recipient) in recipients.iter().enumerate() {
            // Validate destination address
            if !provider.validate_address(&recipient.to) {
                results.push(BatchTransferStatus {
                    index,
                    to: recipient.to.clone(),
                    amount: recipient.amount.clone(),
                    status: TxStatus::Failed.to_string(),
                    tx_hash: None,
                    error: Some("Invalid address format".to_string()),
                });
                failed += 1;
                continue;
            }

            // Build transaction request
            let tx_request = TransactionRequest {
                from: wallet.address.clone(),
                to: recipient.to.clone(),
                amount: recipient.amount.clone(),
                token_address: recipient.token_address.clone(),
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                gas_limit: None,
                compute_units: None,
            };

            // Send transaction
            match provider.send_transaction(&private_key, tx_request).await {
                Ok(response) => {
                    // Log transaction to database
                    let token_symbol = if recipient.token_address.is_some() {
                        Some("TOKEN".to_string())
                    } else {
                        wallet.chain.parse::<Chain>().ok().map(|c| c.native_symbol().to_string())
                    };

                    let _ = self.transaction_repo.create(
                        wallet_id,
                        response.tx_hash.clone(),
                        wallet.chain.clone(),
                        wallet.address.clone(),
                        recipient.to.clone(),
                        recipient.amount.clone(),
                        recipient.token_address.clone(),
                        token_symbol,
                        response.status.clone()
                    ).await;

                    results.push(BatchTransferStatus {
                        index,
                        to: recipient.to.clone(),
                        amount: recipient.amount.clone(),
                        status: response.status,
                        tx_hash: Some(response.tx_hash),
                        error: None,
                    });
                    successful += 1;
                }
                Err(e) => {
                    results.push(BatchTransferStatus {
                        index,
                        to: recipient.to.clone(),
                        amount: recipient.amount.clone(),
                        status: TxStatus::Failed.to_string(),
                        tx_hash: None,
                        error: Some(e.to_string()),
                    });
                    failed += 1;
                }
            }

            // Small delay between transactions to avoid nonce issues
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Ok(BatchTransferResult {
            total: recipients.len(),
            successful,
            failed,
            results,
        })
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

#[derive(serde::Deserialize, Clone)]
pub struct BatchRecipient {
    pub to: String,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_address: Option<String>,
}

#[derive(serde::Serialize)]
pub struct BatchTransferResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BatchTransferStatus>,
}

#[derive(serde::Serialize)]
pub struct BatchTransferStatus {
    pub index: usize,
    pub to: String,
    pub amount: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
}
