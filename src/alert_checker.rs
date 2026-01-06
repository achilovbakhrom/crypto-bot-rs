use crate::services::price_alert_service::PriceAlertService;
use crate::services::price_service::PriceService;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::time::{ interval, Duration };

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

            // Check if alert should trigger
            let should_trigger = match alert.alert_type.as_str() {
                "above" => {
                    if let Some(target) = alert.target_price {
                        let target_f64 = target.to_string().parse::<f64>().unwrap_or(0.0);
                        current_price >= target_f64
                    } else {
                        false
                    }
                }
                "below" => {
                    if let Some(target) = alert.target_price {
                        let target_f64 = target.to_string().parse::<f64>().unwrap_or(0.0);
                        current_price <= target_f64
                    } else {
                        false
                    }
                }
                "percent_change" => {
                    if let (Some(percent), Some(base)) = (alert.percent_change, alert.base_price) {
                        let percent_f64 = percent.to_string().parse::<f64>().unwrap_or(0.0);
                        let base_f64 = base.to_string().parse::<f64>().unwrap_or(0.0);
                        let change = ((current_price - base_f64) / base_f64) * 100.0;

                        if percent_f64 > 0.0 {
                            change >= percent_f64
                        } else {
                            change <= percent_f64
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            // Update last checked time
            let _ = alert_service.update_last_checked(alert.id).await;

            if should_trigger {
                // Send notification
                let message = self.format_alert_message(&alert, current_price);

                if let Ok(user_id) = alert.user_id.parse::<i64>() {
                    let chat_id = ChatId(user_id);
                    let _ = self.bot.send_message(chat_id, message).await;
                }

                // Mark alert as triggered
                let _ = alert_service.trigger_alert(alert.id).await;

                println!(
                    "🔔 Alert triggered for user {} - {} {} at ${:.4}",
                    alert.user_id,
                    alert.token_symbol,
                    alert.alert_type,
                    current_price
                );
            }
        }

        Ok(())
    }

    fn format_alert_message(
        &self,
        alert: &crate::db::entity::price_alert::Model,
        current_price: f64
    ) -> String {
        let emoji = match alert.alert_type.as_str() {
            "above" => "📈",
            "below" => "📉",
            "percent_change" => "⚡",
            _ => "🔔",
        };

        let condition = match alert.alert_type.as_str() {
            "above" => {
                if let Some(target) = alert.target_price {
                    format!("above ${:.4}", target.to_string().parse::<f64>().unwrap_or(0.0))
                } else {
                    "triggered".to_string()
                }
            }
            "below" => {
                if let Some(target) = alert.target_price {
                    format!("below ${:.4}", target.to_string().parse::<f64>().unwrap_or(0.0))
                } else {
                    "triggered".to_string()
                }
            }
            "percent_change" => {
                if let (Some(percent), Some(base)) = (alert.percent_change, alert.base_price) {
                    let percent_f64 = percent.to_string().parse::<f64>().unwrap_or(0.0);
                    let base_f64 = base.to_string().parse::<f64>().unwrap_or(0.0);
                    let change = ((current_price - base_f64) / base_f64) * 100.0;
                    format!("{:+.2}% change (from ${:.4})", change, base_f64)
                } else {
                    "triggered".to_string()
                }
            }
            _ => "triggered".to_string(),
        };

        format!(
            "{} *Price Alert Triggered\\!*\n\n\
            🪙 Token: {}\n\
            ⛓️ Chain: {}\n\
            💰 Current Price: \\${:.4}\n\
            📊 Condition: {}\n\n\
            This alert has been deactivated\\. Use /setalert to create a new one\\.",
            emoji,
            alert.token_symbol.replace('-', "\\-").replace('.', "\\."),
            alert.chain.replace('-', "\\-").replace('.', "\\."),
            current_price,
            condition.replace('-', "\\-").replace('.', "\\.").replace('+', "\\+")
        )
    }
}
