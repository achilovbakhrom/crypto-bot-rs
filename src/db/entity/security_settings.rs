use sea_orm::entity::prelude::*;
use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "security_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: String,
    pub pin_hash: Option<String>,
    pub pin_enabled: bool,
    pub confirmation_delay_seconds: i32,
    pub daily_withdrawal_limit: Option<Decimal>,
    pub weekly_withdrawal_limit: Option<Decimal>,
    pub require_confirmation_above: Option<Decimal>,
    pub session_timeout: i32,
    pub last_activity: Option<DateTimeUtc>,
    pub wallet_locked: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
