use crate::db::entity::price_alert;
use crate::enums::{ AlertKind, AlertType };
use crate::error::Result;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue,
    ColumnTrait,
    DatabaseConnection,
    EntityTrait,
    QueryFilter,
    prelude::Decimal,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct PriceAlertService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct CreateAlertRequest {
    pub user_id: String,
    pub token_symbol: String,
    pub chain: String,
    pub token_address: Option<String>,
    pub alert_type: AlertType,
}

impl PriceAlertService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new price alert
    pub async fn create_alert(&self, req: CreateAlertRequest) -> Result<price_alert::Model> {
        let now = Utc::now();

        let (alert_kind, target_price, percent_change, base_price) = match req.alert_type {
            AlertType::Above { target_price } => {
                (
                    AlertKind::Above,
                    Some(Decimal::from_f64_retain(target_price).unwrap()),
                    None,
                    None,
                )
            }
            AlertType::Below { target_price } => {
                (
                    AlertKind::Below,
                    Some(Decimal::from_f64_retain(target_price).unwrap()),
                    None,
                    None,
                )
            }
            AlertType::PercentChange { percent, base_price } => {
                (
                    AlertKind::PercentChange,
                    None,
                    Some(Decimal::from_f64_retain(percent).unwrap()),
                    Some(Decimal::from_f64_retain(base_price).unwrap()),
                )
            }
        };

        let alert = price_alert::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(req.user_id),
            token_symbol: ActiveValue::Set(req.token_symbol),
            chain: ActiveValue::Set(req.chain),
            token_address: ActiveValue::Set(req.token_address),
            alert_type: ActiveValue::Set(alert_kind.to_string()),
            target_price: ActiveValue::Set(target_price),
            percent_change: ActiveValue::Set(percent_change),
            base_price: ActiveValue::Set(base_price),
            active: ActiveValue::Set(true),
            triggered_at: ActiveValue::Set(None),
            last_checked_at: ActiveValue::Set(None),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        let alert = alert.insert(&self.db).await?;
        Ok(alert)
    }

    /// List all alerts for a user
    pub async fn list_user_alerts(
        &self,
        user_id: &str,
        active_only: bool
    ) -> Result<Vec<price_alert::Model>> {
        let mut query = price_alert::Entity::find().filter(price_alert::Column::UserId.eq(user_id));

        if active_only {
            query = query.filter(price_alert::Column::Active.eq(true));
        }

        let alerts = query.all(&self.db).await?;
        Ok(alerts)
    }

    /// Get a specific alert by ID
    pub async fn get_alert(&self, id: Uuid, user_id: &str) -> Result<Option<price_alert::Model>> {
        let alert = price_alert::Entity
            ::find_by_id(id)
            .filter(price_alert::Column::UserId.eq(user_id))
            .one(&self.db).await?;
        Ok(alert)
    }

    /// Delete an alert
    pub async fn delete_alert(&self, id: Uuid, user_id: &str) -> Result<()> {
        price_alert::Entity
            ::delete_many()
            .filter(price_alert::Column::Id.eq(id))
            .filter(price_alert::Column::UserId.eq(user_id))
            .exec(&self.db).await?;
        Ok(())
    }

    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<price_alert::Model>> {
        let alerts = price_alert::Entity
            ::find()
            .filter(price_alert::Column::Active.eq(true))
            .all(&self.db).await?;
        Ok(alerts)
    }

    /// Mark alert as triggered
    pub async fn trigger_alert(&self, id: Uuid) -> Result<()> {
        let alert = price_alert::Entity::find_by_id(id).one(&self.db).await?;

        if let Some(alert) = alert {
            let mut active: price_alert::ActiveModel = alert.into();
            active.active = ActiveValue::Set(false);
            active.triggered_at = ActiveValue::Set(Some(Utc::now()));
            active.updated_at = ActiveValue::Set(Utc::now());
            active.update(&self.db).await?;
        }

        Ok(())
    }

    /// Update last checked time
    pub async fn update_last_checked(&self, id: Uuid) -> Result<()> {
        let alert = price_alert::Entity::find_by_id(id).one(&self.db).await?;

        if let Some(alert) = alert {
            let mut active: price_alert::ActiveModel = alert.into();
            active.last_checked_at = ActiveValue::Set(Some(Utc::now()));
            active.updated_at = ActiveValue::Set(Utc::now());
            active.update(&self.db).await?;
        }

        Ok(())
    }
}
