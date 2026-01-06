use std::sync::Arc;
use uuid::Uuid;

use crate::db::{ TransactionRepository, WalletRepository };
use crate::error::{ AppError, Result };
use crate::db::entity::transaction;

pub struct TransactionService {
    transaction_repo: Arc<TransactionRepository>,
    wallet_repo: Arc<WalletRepository>,
}

impl TransactionService {
    pub fn new(
        transaction_repo: Arc<TransactionRepository>,
        wallet_repo: Arc<WalletRepository>
    ) -> Self {
        Self {
            transaction_repo,
            wallet_repo,
        }
    }

    pub async fn log_transaction(
        &self,
        wallet_id: Uuid,
        tx_hash: String,
        chain: String,
        from_address: String,
        to_address: String,
        amount: String,
        token_address: Option<String>,
        token_symbol: Option<String>
    ) -> Result<transaction::Model> {
        // Verify wallet exists
        self.wallet_repo.find_by_id(wallet_id).await?;

        // Create transaction record
        self.transaction_repo.create(
            wallet_id,
            tx_hash,
            chain,
            from_address,
            to_address,
            amount,
            token_address,
            token_symbol,
            "confirmed".to_string()
        ).await
    }

    pub async fn get_wallet_transactions(
        &self,
        wallet_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>
    ) -> Result<Vec<transaction::Model>> {
        // Verify wallet exists
        self.wallet_repo.find_by_id(wallet_id).await?;

        self.transaction_repo.find_by_wallet_id(wallet_id, limit, offset).await
    }

    pub async fn get_user_transactions(
        &self,
        user_id: &str,
        chain: Option<&str>,
        limit: Option<u64>,
        offset: Option<u64>
    ) -> Result<Vec<transaction::Model>> {
        // Get all wallets for user
        let wallets = if let Some(chain) = chain {
            self.wallet_repo.find_by_user_and_chain(user_id, chain).await?
        } else {
            // Need to implement find_by_user
            return Err(AppError::InvalidInput("Chain parameter required for now".to_string()));
        };

        let wallet_ids: Vec<Uuid> = wallets
            .into_iter()
            .map(|w| w.id)
            .collect();

        if wallet_ids.is_empty() {
            return Ok(vec![]);
        }

        self.transaction_repo.find_by_user_id(wallet_ids, limit, offset).await
    }

    pub async fn get_transaction_by_hash(&self, tx_hash: &str) -> Result<transaction::Model> {
        self.transaction_repo.find_by_tx_hash(tx_hash).await
    }
}
