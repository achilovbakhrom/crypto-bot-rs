use crate::enums::AlertKind;
use crate::services::price_alert_service::PriceAlertService;
use crate::services::price_service::PriceService;
use sea_orm::DatabaseConnection;
use sea_orm::prelude::Decimal;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::time::{ interval, Duration };

/// Safely convert a Decimal to f64, returning None on parse failure
fn decimal_to_f64(d: Decimal) -> Option<f64> {
    d.to_string().parse::<f64>().ok()
}

pub struct AlertChecker {
    db: DatabaseConnection,
    price_service: Arc<PriceService>,
    bot: Bot,
}

impl AlertChecker {
    pub fn new(db: DatabaseConnection, price_service: Arc<PriceService>, bot: Bot) -> Self {
        Self {
            db,
            price_service,
            bot,
        }
    }

    /// Start the background alert checker that runs every 60 seconds
    pub async fn start(self) {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_alerts().await {
                eprintln!("Alert checker error: {}", e);
            }
        }
    }

    /// Check all active alerts
    async fn check_alerts(&self) -> crate::error::Result<()> {
        let alert_service = PriceAlertService::new(self.db.clone());
        let alerts = alert_service.get_active_alerts().await?;

        for alert in alerts {
            // Get current price
            let current_price = if let Some(ref token_addr) = alert.token_address {
                // Get price by token address
                match self.price_service.get_token_price_by_address(&alert.chain, token_addr).await {
                    Ok(price_info) => price_info.usd_price,
                    Err(_) => {
                        continue;
                    }
                }
            } else {
                // Get price by symbol
                match self.price_service.get_price(&alert.token_symbol).await {
                    Ok(price_info) => price_info.usd_price,
                    Err(_) => {
                        continue;
                    }
                }
            };

            // Parse the alert kind from the DB string
            let alert_kind = match alert.alert_type.parse::<AlertKind>() {
                Ok(kind) => kind,
                Err(_) => continue,
            };

            // Check if alert should trigger
            let should_trigger = match alert_kind {
                AlertKind::Above => {
                    alert.target_price
                        .and_then(decimal_to_f64)
                        .map(|target| current_price >= target)
                        .unwrap_or(false)
                }
                AlertKind::Below => {
                    alert.target_price
                        .and_then(decimal_to_f64)
                        .map(|target| current_price <= target)
                        .unwrap_or(false)
                }
                AlertKind::PercentChange => {
                    match (
                        alert.percent_change.and_then(decimal_to_f64),
                        alert.base_price.and_then(decimal_to_f64),
                    ) {
                        (Some(percent_f64), Some(base_f64)) if base_f64 != 0.0 => {
                            let change = ((current_price - base_f64) / base_f64) * 100.0;
                            if percent_f64 > 0.0 {
                                change >= percent_f64
                            } else {
                                change <= percent_f64
                            }
                        }
                        _ => false,
                    }
                }
            };

            // Update last checked time
            let _ = alert_service.update_last_checked(alert.id).await;

            if should_trigger {
                // Send notification
                let message = self.format_alert_message(&alert, current_price, alert_kind);

                if let Ok(user_id) = alert.user_id.parse::<i64>() {
                    let chat_id = ChatId(user_id);
                    let _ = self.bot.send_message(chat_id, message).await;
                }

                // Mark alert as triggered
                let _ = alert_service.trigger_alert(alert.id).await;

                println!(
                    "Alert triggered for user {} - {} {} at ${:.4}",
                    alert.user_id,
                    alert.token_symbol,
                    alert_kind,
                    current_price
                );
            }
        }

        Ok(())
    }

    fn format_alert_message(
        &self,
        alert: &crate::db::entity::price_alert::Model,
        current_price: f64,
        kind: AlertKind,
    ) -> String {
        let emoji = match kind {
            AlertKind::Above => "📈",
            AlertKind::Below => "📉",
            AlertKind::PercentChange => "⚡",
        };

        let condition = match kind {
            AlertKind::Above => {
                alert.target_price
                    .and_then(decimal_to_f64)
                    .map(|t| format!("above ${:.4}", t))
                    .unwrap_or_else(|| "triggered".to_string())
            }
            AlertKind::Below => {
                alert.target_price
                    .and_then(decimal_to_f64)
                    .map(|t| format!("below ${:.4}", t))
                    .unwrap_or_else(|| "triggered".to_string())
            }
            AlertKind::PercentChange => {
                match (
                    alert.percent_change.and_then(decimal_to_f64),
                    alert.base_price.and_then(decimal_to_f64),
                ) {
                    (Some(_), Some(base_f64)) if base_f64 != 0.0 => {
                        let change = ((current_price - base_f64) / base_f64) * 100.0;
                        format!("{:+.2}% change (from ${:.4})", change, base_f64)
                    }
                    _ => "triggered".to_string(),
                }
            }
        };

        format!(
            "{emoji} Price Alert Triggered!\n\n\
            Token: {symbol}\n\
            Chain: {chain}\n\
            Current Price: ${price:.4}\n\
            Condition: {condition}\n\n\
            This alert has been deactivated. Use /setalert to create a new one.",
            emoji = emoji,
            symbol = alert.token_symbol,
            chain = alert.chain,
            price = current_price,
            condition = condition,
        )
    }
}
