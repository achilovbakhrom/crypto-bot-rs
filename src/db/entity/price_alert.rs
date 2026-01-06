use sea_orm::entity::prelude::*;
use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "price_alerts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: String,
    pub token_symbol: String,
    pub chain: String,
    pub token_address: Option<String>,
    pub alert_type: String, // "above", "below", "percent_change", "portfolio_value"
    pub target_price: Option<Decimal>,
    pub percent_change: Option<Decimal>,
    pub base_price: Option<Decimal>,
    pub active: bool,
    pub triggered_at: Option<DateTimeUtc>,
    pub last_checked_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
