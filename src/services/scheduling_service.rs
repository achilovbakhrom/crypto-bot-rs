use crate::db::entity::scheduled_transaction;
use crate::enums::{ ScheduleStatus, RecurringType };
use crate::error::Result;
use chrono::{ DateTime, Duration, Utc };
use sea_orm::{
    ActiveModelTrait,
    ActiveValue,
    ColumnTrait,
    DatabaseConnection,
    EntityTrait,
    QueryFilter,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct SchedulingService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct ScheduleRequest {
    pub user_id: String,
    pub wallet_id: Uuid,
    pub to_address: String,
    pub amount: String,
    pub token_address: Option<String>,
    pub scheduled_for: DateTime<Utc>,
    pub recurring_type: Option<RecurringType>,
}

impl SchedulingService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Schedule a new transaction (one-time or recurring)
    pub async fn schedule_transaction(
        &self,
        req: ScheduleRequest
    ) -> Result<scheduled_transaction::Model> {
        let now = Utc::now();

        let schedule = scheduled_transaction::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(req.user_id),
            wallet_id: ActiveValue::Set(req.wallet_id),
            to_address: ActiveValue::Set(req.to_address),
            amount: ActiveValue::Set(req.amount),
            token_address: ActiveValue::Set(req.token_address),
            scheduled_for: ActiveValue::Set(req.scheduled_for),
            recurring_type: ActiveValue::Set(req.recurring_type.map(|r| r.to_string())),
            status: ActiveValue::Set(ScheduleStatus::Pending.to_string()),
            executed_at: ActiveValue::Set(None),
            tx_hash: ActiveValue::Set(None),
            error_message: ActiveValue::Set(None),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        let schedule = schedule.insert(&self.db).await?;
        Ok(schedule)
    }

    /// Get all scheduled transactions for a user
    pub async fn list_scheduled(
        &self,
        user_id: &str,
        status: Option<&str>
    ) -> Result<Vec<scheduled_transaction::Model>> {
        let mut query = scheduled_transaction::Entity
            ::find()
            .filter(scheduled_transaction::Column::UserId.eq(user_id));

        if let Some(s) = status {
            query = query.filter(scheduled_transaction::Column::Status.eq(s));
        }

        let schedules = query.all(&self.db).await?;
        Ok(schedules)
    }

    /// Get a specific scheduled transaction by ID
    pub async fn get_schedule(&self, id: Uuid) -> Result<Option<scheduled_transaction::Model>> {
        let schedule = scheduled_transaction::Entity::find_by_id(id).one(&self.db).await?;
        Ok(schedule)
    }

    /// Cancel a scheduled transaction
    pub async fn cancel_schedule(&self, id: Uuid, user_id: &str) -> Result<()> {
        let schedule = scheduled_transaction::Entity
            ::find_by_id(id)
            .filter(scheduled_transaction::Column::UserId.eq(user_id))
            .one(&self.db).await?;

        if let Some(schedule) = schedule {
            if schedule.status == ScheduleStatus::Pending.as_str() {
                let mut active: scheduled_transaction::ActiveModel = schedule.into();
                active.status = ActiveValue::Set(ScheduleStatus::Cancelled.to_string());
                active.updated_at = ActiveValue::Set(Utc::now());
                active.update(&self.db).await?;
            }
        }

        Ok(())
    }

    /// Get all pending transactions that are due for execution
    pub async fn get_due_transactions(&self) -> Result<Vec<scheduled_transaction::Model>> {
        let now = Utc::now();

        let schedules = scheduled_transaction::Entity
            ::find()
            .filter(scheduled_transaction::Column::Status.eq(ScheduleStatus::Pending.as_str()))
            .filter(scheduled_transaction::Column::ScheduledFor.lte(now))
            .all(&self.db).await?;

        Ok(schedules)
    }

    /// Mark a transaction as executed
    pub async fn mark_executed(&self, id: Uuid, tx_hash: String) -> Result<()> {
        let schedule = scheduled_transaction::Entity::find_by_id(id).one(&self.db).await?;

        if let Some(schedule) = schedule {
            let mut active: scheduled_transaction::ActiveModel = schedule.clone().into();
            active.status = ActiveValue::Set(ScheduleStatus::Executed.to_string());
            active.executed_at = ActiveValue::Set(Some(Utc::now()));
            active.tx_hash = ActiveValue::Set(Some(tx_hash));
            active.updated_at = ActiveValue::Set(Utc::now());
            active.update(&self.db).await?;

            // If recurring, create next schedule
            if let Some(recurring_type) = &schedule.recurring_type {
                self.create_next_recurring_schedule(&schedule, recurring_type).await?;
            }
        }

        Ok(())
    }

    /// Mark a transaction as failed
    pub async fn mark_failed(&self, id: Uuid, error: String) -> Result<()> {
        let schedule = scheduled_transaction::Entity::find_by_id(id).one(&self.db).await?;

        if let Some(schedule) = schedule {
            let mut active: scheduled_transaction::ActiveModel = schedule.into();
            active.status = ActiveValue::Set(ScheduleStatus::Failed.to_string());
            active.executed_at = ActiveValue::Set(Some(Utc::now()));
            active.error_message = ActiveValue::Set(Some(error));
            active.updated_at = ActiveValue::Set(Utc::now());
            active.update(&self.db).await?;
        }

        Ok(())
    }

    /// Create the next recurring schedule
    async fn create_next_recurring_schedule(
        &self,
        schedule: &scheduled_transaction::Model,
        recurring_type: &str
    ) -> Result<()> {
        let parsed = match recurring_type.parse::<RecurringType>() {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };

        let next_time = match parsed {
            RecurringType::Daily => schedule.scheduled_for + Duration::days(1),
            RecurringType::Weekly => schedule.scheduled_for + Duration::weeks(1),
            RecurringType::Monthly => schedule.scheduled_for + Duration::days(30),
        };

        let next_schedule = scheduled_transaction::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(schedule.user_id.clone()),
            wallet_id: ActiveValue::Set(schedule.wallet_id),
            to_address: ActiveValue::Set(schedule.to_address.clone()),
            amount: ActiveValue::Set(schedule.amount.clone()),
            token_address: ActiveValue::Set(schedule.token_address.clone()),
            scheduled_for: ActiveValue::Set(next_time),
            recurring_type: ActiveValue::Set(Some(parsed.to_string())),
            status: ActiveValue::Set(ScheduleStatus::Pending.to_string()),
            executed_at: ActiveValue::Set(None),
            tx_hash: ActiveValue::Set(None),
            error_message: ActiveValue::Set(None),
            created_at: ActiveValue::Set(Utc::now()),
            updated_at: ActiveValue::Set(Utc::now()),
        };

        next_schedule.insert(&self.db).await?;
        Ok(())
    }
}
