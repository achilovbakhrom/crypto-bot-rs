use crate::db::entity::{ security_settings, withdrawal_tracking };
use crate::error::Result;
use chrono::{ DateTime, Duration, Utc };
use sea_orm::{
    ActiveModelTrait,
    ActiveValue,
    ColumnTrait,
    DatabaseConnection,
    EntityTrait,
    QueryFilter,
    QueryOrder,
    prelude::Decimal,
};
use uuid::Uuid;
use argon2::{ Argon2, PasswordHash, PasswordHasher, PasswordVerifier };
use argon2::password_hash::{ SaltString, rand_core::OsRng };

#[derive(Clone)]
pub struct SecurityService {
    db: DatabaseConnection,
}

impl SecurityService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get or create security settings for user
    pub async fn get_or_create_settings(&self, user_id: &str) -> Result<security_settings::Model> {
        if
            let Some(settings) = security_settings::Entity
                ::find()
                .filter(security_settings::Column::UserId.eq(user_id))
                .one(&self.db).await?
        {
            return Ok(settings);
        }

        // Create default settings
        let now = Utc::now();
        let settings = security_settings::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user_id.to_string()),
            pin_hash: ActiveValue::Set(None),
            pin_enabled: ActiveValue::Set(false),
            confirmation_delay_seconds: ActiveValue::Set(0),
            daily_withdrawal_limit: ActiveValue::Set(None),
            weekly_withdrawal_limit: ActiveValue::Set(None),
            require_confirmation_above: ActiveValue::Set(None),
            session_timeout: ActiveValue::Set(3600),
            last_activity: ActiveValue::Set(Some(now)),
            wallet_locked: ActiveValue::Set(false),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        let settings = settings.insert(&self.db).await?;
        Ok(settings)
    }

    /// Set or change PIN
    pub async fn set_pin(&self, user_id: &str, pin: &str) -> Result<()> {
        let settings = self.get_or_create_settings(user_id).await?;

        // Hash PIN with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pin_hash = argon2
            .hash_password(pin.as_bytes(), &salt)
            .map_err(|e| crate::error::AppError::Internal(format!("Failed to hash PIN: {}", e)))?
            .to_string();

        let mut active: security_settings::ActiveModel = settings.into();
        active.pin_hash = ActiveValue::Set(Some(pin_hash));
        active.pin_enabled = ActiveValue::Set(true);
        active.updated_at = ActiveValue::Set(Utc::now());
        active.update(&self.db).await?;

        Ok(())
    }

    /// Verify PIN
    pub async fn verify_pin(&self, user_id: &str, pin: &str) -> Result<bool> {
        let settings = self.get_or_create_settings(user_id).await?;

        if !settings.pin_enabled {
            return Ok(true); // PIN not enabled, always pass
        }

        let Some(pin_hash) = settings.pin_hash else {
            return Ok(false);
        };

        let parsed_hash = PasswordHash::new(&pin_hash).map_err(|e|
            crate::error::AppError::Internal(format!("Invalid hash: {}", e))
        )?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(pin.as_bytes(), &parsed_hash).is_ok())
    }

    /// Disable PIN
    pub async fn disable_pin(&self, user_id: &str) -> Result<()> {
        let settings = self.get_or_create_settings(user_id).await?;

        let mut active: security_settings::ActiveModel = settings.into();
        active.pin_enabled = ActiveValue::Set(false);
        active.pin_hash = ActiveValue::Set(None);
        active.updated_at = ActiveValue::Set(Utc::now());
        active.update(&self.db).await?;

        Ok(())
    }

    /// Set withdrawal limits
    pub async fn set_limits(
        &self,
        user_id: &str,
        daily_limit: Option<f64>,
        weekly_limit: Option<f64>
    ) -> Result<()> {
        let settings = self.get_or_create_settings(user_id).await?;

        let mut active: security_settings::ActiveModel = settings.into();
        active.daily_withdrawal_limit = ActiveValue::Set(
            daily_limit.map(|l| Decimal::from_f64_retain(l).unwrap())
        );
        active.weekly_withdrawal_limit = ActiveValue::Set(
            weekly_limit.map(|l| Decimal::from_f64_retain(l).unwrap())
        );
        active.updated_at = ActiveValue::Set(Utc::now());
        active.update(&self.db).await?;

        Ok(())
    }

    /// Check if withdrawal is within limits
    pub async fn check_withdrawal_limit(
        &self,
        user_id: &str,
        amount_usd: f64
    ) -> Result<(bool, String)> {
        let settings = self.get_or_create_settings(user_id).await?;
        let now = Utc::now();

        // Check daily limit
        if let Some(daily_limit) = settings.daily_withdrawal_limit {
            let daily_total = self.get_withdrawal_sum(user_id, now - Duration::days(1)).await?;
            let daily_limit_f64 = daily_limit.to_string().parse::<f64>().unwrap_or(0.0);

            if daily_total + amount_usd > daily_limit_f64 {
                return Ok((
                    false,
                    format!(
                        "Daily withdrawal limit exceeded. Limit: ${:.2}, Used: ${:.2}, Requested: ${:.2}",
                        daily_limit_f64,
                        daily_total,
                        amount_usd
                    ),
                ));
            }
        }

        // Check weekly limit
        if let Some(weekly_limit) = settings.weekly_withdrawal_limit {
            let weekly_total = self.get_withdrawal_sum(user_id, now - Duration::weeks(1)).await?;
            let weekly_limit_f64 = weekly_limit.to_string().parse::<f64>().unwrap_or(0.0);

            if weekly_total + amount_usd > weekly_limit_f64 {
                return Ok((
                    false,
                    format!(
                        "Weekly withdrawal limit exceeded. Limit: ${:.2}, Used: ${:.2}, Requested: ${:.2}",
                        weekly_limit_f64,
                        weekly_total,
                        amount_usd
                    ),
                ));
            }
        }

        Ok((true, String::new()))
    }

    /// Record a withdrawal
    pub async fn record_withdrawal(
        &self,
        user_id: &str,
        amount: f64,
        token_symbol: &str,
        usd_value: f64
    ) -> Result<()> {
        let withdrawal = withdrawal_tracking::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user_id.to_string()),
            amount: ActiveValue::Set(Decimal::from_f64_retain(amount).unwrap()),
            token_symbol: ActiveValue::Set(token_symbol.to_string()),
            usd_value: ActiveValue::Set(Decimal::from_f64_retain(usd_value).unwrap()),
            timestamp: ActiveValue::Set(Utc::now()),
        };

        withdrawal.insert(&self.db).await?;
        Ok(())
    }

    /// Get total withdrawals since a given time
    async fn get_withdrawal_sum(&self, user_id: &str, since: DateTime<Utc>) -> Result<f64> {
        let withdrawals = withdrawal_tracking::Entity
            ::find()
            .filter(withdrawal_tracking::Column::UserId.eq(user_id))
            .filter(withdrawal_tracking::Column::Timestamp.gte(since))
            .all(&self.db).await?;

        let total: f64 = withdrawals
            .iter()
            .map(|w| w.usd_value.to_string().parse::<f64>().unwrap_or(0.0))
            .sum();

        Ok(total)
    }

    /// Lock wallet
    pub async fn lock_wallet(&self, user_id: &str) -> Result<()> {
        let settings = self.get_or_create_settings(user_id).await?;

        let mut active: security_settings::ActiveModel = settings.into();
        active.wallet_locked = ActiveValue::Set(true);
        active.updated_at = ActiveValue::Set(Utc::now());
        active.update(&self.db).await?;

        Ok(())
    }

    /// Unlock wallet (requires PIN verification)
    pub async fn unlock_wallet(&self, user_id: &str, pin: &str) -> Result<bool> {
        if !self.verify_pin(user_id, pin).await? {
            return Ok(false);
        }

        let settings = self.get_or_create_settings(user_id).await?;

        let mut active: security_settings::ActiveModel = settings.into();
        active.wallet_locked = ActiveValue::Set(false);
        active.last_activity = ActiveValue::Set(Some(Utc::now()));
        active.updated_at = ActiveValue::Set(Utc::now());
        active.update(&self.db).await?;

        Ok(true)
    }

    /// Check if wallet is locked
    pub async fn is_wallet_locked(&self, user_id: &str) -> Result<bool> {
        let settings = self.get_or_create_settings(user_id).await?;
        Ok(settings.wallet_locked)
    }

    /// Update last activity
    pub async fn update_activity(&self, user_id: &str) -> Result<()> {
        let settings = self.get_or_create_settings(user_id).await?;

        let mut active: security_settings::ActiveModel = settings.into();
        active.last_activity = ActiveValue::Set(Some(Utc::now()));
        active.update(&self.db).await?;

        Ok(())
    }
}
