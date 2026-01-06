use sea_orm::entity::prelude::*;
use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "scheduled_transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: String,
    pub wallet_id: Uuid,
    pub to_address: String,
    pub amount: String,
    pub token_address: Option<String>,
    pub scheduled_for: DateTimeUtc,
    pub recurring_type: Option<String>, // "daily", "weekly", "monthly", or null for one-time
    pub status: String, // "pending", "executed", "failed", "cancelled"
    pub executed_at: Option<DateTimeUtc>,
    pub tx_hash: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
