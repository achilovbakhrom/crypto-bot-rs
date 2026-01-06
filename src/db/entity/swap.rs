use sea_orm::entity::prelude::*;
use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "swaps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: String,
    pub wallet_id: Uuid,
    pub chain: String,
    pub dex: String,
    pub from_token: String,
    pub from_token_address: Option<String>,
    pub to_token: String,
    pub to_token_address: Option<String>,
    pub from_amount: Decimal,
    pub to_amount: Decimal,
    pub expected_to_amount: Option<Decimal>,
    pub price_impact: Option<Decimal>,
    pub slippage: Decimal,
    pub tx_hash: Option<String>,
    pub status: String, // "pending", "success", "failed"
    pub error_message: Option<String>,
    pub gas_fee: Option<Decimal>,
    pub route: Option<Json>, // Route information for multi-hop swaps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
