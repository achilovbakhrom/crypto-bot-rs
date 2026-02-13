use sea_orm::{ entity::prelude::*, DatabaseConnection, Set };
use uuid::Uuid;

use crate::error::{ AppError, Result };

pub mod entity;
pub use entity::*;

mod transaction_repository;
pub use transaction_repository::TransactionRepository;

mod token_metadata_repository;
pub use token_metadata_repository::{TokenMetadataRepository, TokenMetadataInput};

pub struct WalletRepository {
    db: DatabaseConnection,
}

impl WalletRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: String,
        chain: String,
        address: String,
        encrypted_private_key: String
    ) -> Result<entity::wallet::Model> {
        let wallet = entity::wallet::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            chain: Set(chain),
            address: Set(address),
            encrypted_private_key: Set(encrypted_private_key),
            created_at: Set(chrono::Utc::now()),
        };

        let wallet = wallet.insert(&self.db).await?;
        Ok(wallet)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<entity::wallet::Model> {
        entity::wallet::Entity::find_by_id(id).one(&self.db).await?.ok_or(AppError::WalletNotFound)
    }

    pub async fn find_by_user_and_chain(
        &self,
        user_id: &str,
        chain: &str
    ) -> Result<Vec<entity::wallet::Model>> {
        let wallets = entity::wallet::Entity
            ::find()
            .filter(entity::wallet::Column::UserId.eq(user_id))
            .filter(entity::wallet::Column::Chain.eq(chain))
            .all(&self.db).await?;

        Ok(wallets)
    }

    pub async fn find_by_user(&self, user_id: &str) -> Result<Vec<entity::wallet::Model>> {
        let wallets = entity::wallet::Entity
            ::find()
            .filter(entity::wallet::Column::UserId.eq(user_id))
            .all(&self.db).await?;

        Ok(wallets)
    }

    pub async fn find_by_address(&self, address: &str) -> Result<Option<entity::wallet::Model>> {
        let wallet = entity::wallet::Entity
            ::find()
            .filter(entity::wallet::Column::Address.eq(address))
            .one(&self.db).await?;

        Ok(wallet)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        entity::wallet::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}
