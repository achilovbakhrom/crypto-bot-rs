use crate::db::entity::wallet;
use crate::services::scheduling_service::SchedulingService;
use crate::services::transfer_service::{ TransferService, TransferRequest };
use sea_orm::{ DatabaseConnection, EntityTrait };
use std::sync::Arc;
use tokio::time::{ interval, Duration };

pub struct Scheduler {
    db: DatabaseConnection,
    transfer_service: Arc<TransferService>,
}

impl Scheduler {
    pub fn new(db: DatabaseConnection, transfer_service: Arc<TransferService>) -> Self {
        Self {
            db,
            transfer_service,
        }
    }

    pub async fn start(self) {
        let mut interval = interval(Duration::from_secs(60)); // Check every minute

        loop {
            interval.tick().await;

            if let Err(e) = self.process_due_transactions().await {
                eprintln!("Scheduler error: {}", e);
            }
        }
    }

    async fn process_due_transactions(&self) -> crate::error::Result<()> {
        let scheduling_service = SchedulingService::new(self.db.clone());

        let due_transactions = scheduling_service.get_due_transactions().await?;

        for schedule in due_transactions {
            println!(
                "Processing scheduled transaction: {} for user {}",
                schedule.id,
                schedule.user_id
            );

            // Get wallet
            let wallet = match wallet::Entity::find_by_id(schedule.wallet_id).one(&self.db).await? {
                Some(w) => w,
                None => {
                    scheduling_service.mark_failed(
                        schedule.id,
                        "Wallet not found".to_string()
                    ).await?;
                    continue;
                }
            };

            // Verify wallet belongs to user
            if wallet.user_id != schedule.user_id {
                scheduling_service.mark_failed(
                    schedule.id,
                    "Wallet ownership mismatch".to_string()
                ).await?;
                continue;
            }

            // Prepare transfer request
            let request = TransferRequest {
                to: schedule.to_address.clone(),
                amount: schedule.amount.clone(),
                token_address: schedule.token_address.clone(),
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                gas_limit: None,
                compute_units: None,
            };

            // Execute transfer
            match self.transfer_service.send_transaction(schedule.wallet_id, request).await {
                Ok(tx_response) => {
                    println!(
                        "✅ Scheduled transaction {} executed: {}",
                        schedule.id,
                        tx_response.tx_hash
                    );

                    if
                        let Err(e) = scheduling_service.mark_executed(
                            schedule.id,
                            tx_response.tx_hash
                        ).await
                    {
                        eprintln!("Failed to mark transaction as executed: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to execute scheduled transaction {}: {}", schedule.id, e);

                    if
                        let Err(mark_err) = scheduling_service.mark_failed(
                            schedule.id,
                            e.to_string()
                        ).await
                    {
                        eprintln!("Failed to mark transaction as failed: {}", mark_err);
                    }
                }
            }
        }

        Ok(())
    }
}
