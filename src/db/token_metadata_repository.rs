use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::db::entity::token_metadata;
use crate::error::Result;

#[derive(Clone)]
pub struct TokenMetadataRepository {
    db: DatabaseConnection,
}

pub struct TokenMetadataInput {
    pub chain: String,
    pub contract_address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i16,
    pub logo_url: Option<String>,
    pub coingecko_id: Option<String>,
}

impl TokenMetadataRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_chain_and_address(
        &self,
        chain: &str,
        address: &str,
    ) -> Result<Option<token_metadata::Model>> {
        let result = token_metadata::Entity::find()
            .filter(token_metadata::Column::Chain.eq(chain))
            .filter(token_metadata::Column::ContractAddress.eq(address.to_lowercase()))
            .one(&self.db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_chain(&self, chain: &str) -> Result<Vec<token_metadata::Model>> {
        let results = token_metadata::Entity::find()
            .filter(token_metadata::Column::Chain.eq(chain))
            .all(&self.db)
            .await?;
        Ok(results)
    }

    pub async fn find_by_chain_and_symbol(
        &self,
        chain: &str,
        symbol: &str,
    ) -> Result<Option<token_metadata::Model>> {
        let result = token_metadata::Entity::find()
            .filter(token_metadata::Column::Chain.eq(chain))
            .filter(token_metadata::Column::Symbol.eq(symbol.to_uppercase()))
            .one(&self.db)
            .await?;
        Ok(result)
    }

    pub async fn upsert(
        &self,
        chain: &str,
        address: &str,
        symbol: &str,
        name: &str,
        decimals: i16,
        logo_url: Option<String>,
        coingecko_id: Option<String>,
    ) -> Result<token_metadata::Model> {
        let address_lower = address.to_lowercase();
        let now = Utc::now();

        if let Some(existing) = self
            .find_by_chain_and_address(chain, &address_lower)
            .await?
        {
            let mut active: token_metadata::ActiveModel = existing.into();
            active.symbol = ActiveValue::Set(symbol.to_string());
            active.name = ActiveValue::Set(name.to_string());
            active.decimals = ActiveValue::Set(decimals);
            if logo_url.is_some() {
                active.logo_url = ActiveValue::Set(logo_url);
            }
            if coingecko_id.is_some() {
                active.coingecko_id = ActiveValue::Set(coingecko_id);
            }
            active.updated_at = ActiveValue::Set(now);
            let model = active.update(&self.db).await?;
            Ok(model)
        } else {
            let model = token_metadata::ActiveModel {
                id: ActiveValue::Set(Uuid::new_v4()),
                chain: ActiveValue::Set(chain.to_string()),
                contract_address: ActiveValue::Set(address_lower),
                symbol: ActiveValue::Set(symbol.to_string()),
                name: ActiveValue::Set(name.to_string()),
                decimals: ActiveValue::Set(decimals),
                logo_url: ActiveValue::Set(logo_url),
                coingecko_id: ActiveValue::Set(coingecko_id),
                is_verified: ActiveValue::Set(false),
                discovered_at: ActiveValue::Set(now),
                updated_at: ActiveValue::Set(now),
            };
            let model = model.insert(&self.db).await?;
            Ok(model)
        }
    }

    pub async fn bulk_upsert(&self, tokens: Vec<TokenMetadataInput>) -> Result<()> {
        for token in tokens {
            self.upsert(
                &token.chain,
                &token.contract_address,
                &token.symbol,
                &token.name,
                token.decimals,
                token.logo_url,
                token.coingecko_id,
            )
            .await?;
        }
        Ok(())
    }
}
