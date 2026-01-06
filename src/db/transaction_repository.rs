use sea_orm::{ DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, Set };
use uuid::Uuid;

use crate::error::{ AppError, Result };
use crate::db::entity::{ transaction, Transaction };

pub struct TransactionRepository {
    db: DatabaseConnection,
}

impl TransactionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        wallet_id: Uuid,
        tx_hash: String,
        chain: String,
        from_address: String,
        to_address: String,
        amount: String,
        token_address: Option<String>,
        token_symbol: Option<String>,
        status: String
    ) -> Result<transaction::Model> {
        let transaction_model = transaction::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_id: Set(wallet_id),
            tx_hash: Set(tx_hash),
            chain: Set(chain),
            from_address: Set(from_address),
            to_address: Set(to_address),
            amount: Set(amount),
            token_address: Set(token_address),
            token_symbol: Set(token_symbol),
            status: Set(status),
            block_number: Set(None),
            gas_used: Set(None),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };

        let transaction = Transaction::insert(transaction_model)
            .exec_with_returning(&self.db).await
            .map_err(|e| AppError::Database(e))?;

        Ok(transaction)
    }

    pub async fn find_by_wallet_id(
        &self,
        wallet_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>
    ) -> Result<Vec<transaction::Model>> {
        let query = Transaction::find()
            .filter(transaction::Column::WalletId.eq(wallet_id))
            .order_by_desc(transaction::Column::CreatedAt);

        let transactions = (
            if let (Some(limit), Some(offset)) = (limit, offset) {
                use sea_orm::QuerySelect;
                query.limit(limit).offset(offset).all(&self.db).await
            } else if let Some(limit) = limit {
                use sea_orm::QuerySelect;
                query.limit(limit).all(&self.db).await
            } else {
                query.all(&self.db).await
            }
        ).map_err(|e| AppError::Database(e))?;

        Ok(transactions)
    }

    pub async fn find_by_tx_hash(&self, tx_hash: &str) -> Result<transaction::Model> {
        Transaction::find()
            .filter(transaction::Column::TxHash.eq(tx_hash))
            .one(&self.db).await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))
    }

    pub async fn find_by_user_id(
        &self,
        wallet_ids: Vec<Uuid>,
        limit: Option<u64>,
        offset: Option<u64>
    ) -> Result<Vec<transaction::Model>> {
        let query = Transaction::find()
            .filter(transaction::Column::WalletId.is_in(wallet_ids))
            .order_by_desc(transaction::Column::CreatedAt);

        let transactions = (
            if let (Some(limit), Some(offset)) = (limit, offset) {
                use sea_orm::QuerySelect;
                query.limit(limit).offset(offset).all(&self.db).await
            } else if let Some(limit) = limit {
                use sea_orm::QuerySelect;
                query.limit(limit).all(&self.db).await
            } else {
                query.all(&self.db).await
            }
        ).map_err(|e| AppError::Database(e))?;

        Ok(transactions)
    }

    pub async fn update_status(
        &self,
        tx_hash: &str,
        status: String,
        block_number: Option<i64>,
        gas_used: Option<String>
    ) -> Result<transaction::Model> {
        let transaction = self.find_by_tx_hash(tx_hash).await?;

        let mut transaction_model: transaction::ActiveModel = transaction.into();
        transaction_model.status = Set(status);
        transaction_model.block_number = Set(block_number);
        transaction_model.gas_used = Set(gas_used);

        let updated = Transaction::update(transaction_model)
            .exec(&self.db).await
            .map_err(|e| AppError::Database(e))?;

        Ok(updated)
    }
}
