pub mod handlers;
pub mod commands;
pub mod constants;
pub mod keyboards;
mod callbacks;
mod utils;

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use teloxide::prelude::*;
use teloxide::dispatching::{UpdateHandler, UpdateFilterExt};
use teloxide::utils::command::BotCommands;
use dptree::case;
use crate::services::{
    WalletService,
    BalanceService,
    TransferService,
    TransactionService,
    PortfolioService,
    PriceService,
    AddressBookService,
    GasEstimationService,
    scheduling_service::SchedulingService,
    price_alert_service::PriceAlertService,
    security_service::SecurityService,
    swap_service::SwapService,
};
use crate::crypto::Encryptor;
use crate::config::Config;

/// User dialogue state for interactive flows
#[derive(Clone, Debug)]
pub enum DialogueState {
    /// No active dialogue
    None,
    /// Waiting for recipient address for send
    WaitingForSendAddress {
        wallet_id: String,
        amount: String,
        symbol: String,
    },
    /// Waiting for send amount
    WaitingForSendAmount {
        wallet_id: String,
        recipient: String,
        symbol: String,
    },
    /// Pending send confirmation - stores all details for the confirm button
    PendingSendConfirmation {
        wallet_id: String,
        recipient: String,
        amount: String,
        symbol: String,
    },
    /// Waiting for swap amount
    WaitingForSwapAmount {
        wallet_id: String,
        from_token: String,
        to_token: String,
    },
}

impl Default for DialogueState {
    fn default() -> Self {
        DialogueState::None
    }
}

/// Dialogue storage for users
pub type DialogueStorage = Arc<RwLock<HashMap<i64, DialogueState>>>;

#[derive(Clone)]
pub struct BotState {
    pub wallet_service: Arc<WalletService>,
    pub balance_service: Arc<BalanceService>,
    pub transfer_service: Arc<TransferService>,
    pub transaction_service: Arc<TransactionService>,
    pub portfolio_service: Arc<PortfolioService>,
    pub price_service: Arc<PriceService>,
    pub address_book_service: Arc<AddressBookService>,
    pub gas_estimation_service: Arc<GasEstimationService>,
    pub scheduling_service: Arc<SchedulingService>,
    pub price_alert_service: Arc<PriceAlertService>,
    pub security_service: Arc<SecurityService>,
    pub swap_service: Arc<SwapService>,
    pub encryptor: Arc<Encryptor>,
    pub config: Arc<Config>,
    pub dialogue_storage: DialogueStorage,
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let command_handler = Update::filter_message()
        .filter_command::<commands::Command>()
        .endpoint(handlers::handle_command_dispatch);

    let callback_handler = Update::filter_callback_query()
        .endpoint(callbacks::handle_callback);

    // Handle plain text messages for dialogue flow
    let message_handler = Update::filter_message()
        .filter(|msg: Message| msg.text().is_some() && !msg.text().unwrap().starts_with('/'))
        .endpoint(callbacks::handle_text_message);

    dptree::entry()
        .branch(command_handler)
        .branch(callback_handler)
        .branch(message_handler)
}

pub async fn run_bot(
    bot_token: String,
    wallet_service: Arc<WalletService>,
    balance_service: Arc<BalanceService>,
    transfer_service: Arc<TransferService>,
    transaction_service: Arc<TransactionService>,
    portfolio_service: Arc<PortfolioService>,
    price_service: Arc<PriceService>,
    address_book_service: Arc<AddressBookService>,
    gas_estimation_service: Arc<GasEstimationService>,
    scheduling_service: Arc<SchedulingService>,
    price_alert_service: Arc<PriceAlertService>,
    security_service: Arc<SecurityService>,
    swap_service: Arc<SwapService>,
    encryptor: Arc<Encryptor>,
    config: Arc<Config>,
) {
    tracing::info!("Starting Telegram bot...");

    let bot = Bot::new(bot_token);

    // Set bot commands for slash menu
    if let Err(e) = bot.set_my_commands(commands::Command::bot_commands()).await {
        tracing::warn!("Failed to set bot commands: {}", e);
    } else {
        tracing::info!("Bot commands registered successfully");
    }

    let dialogue_storage: DialogueStorage = Arc::new(RwLock::new(HashMap::new()));

    let state = Arc::new(BotState {
        wallet_service,
        balance_service,
        transfer_service,
        transaction_service,
        portfolio_service,
        price_service,
        address_book_service,
        gas_estimation_service,
        scheduling_service,
        price_alert_service,
        security_service,
        swap_service,
        encryptor,
        config,
        dialogue_storage,
    });

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
