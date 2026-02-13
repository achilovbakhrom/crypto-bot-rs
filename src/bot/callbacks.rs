use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::MessageId;

use crate::enums::{ Chain, AlertKind };
use super::{BotState, DialogueState};
use super::keyboards;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Handle plain text messages for dialogue flow
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
) -> HandlerResult {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or("");

    tracing::info!("handle_text_message called: user_id={}, text={}", user_id, text);

    // Get current dialogue state
    let dialogue_state = {
        let storage = state.dialogue_storage.read().await;
        tracing::info!("Current dialogue storage keys: {:?}", storage.keys().collect::<Vec<_>>());
        storage.get(&user_id).cloned().unwrap_or(DialogueState::None)
    };

    tracing::info!("Dialogue state for user {}: {:?}", user_id, dialogue_state);

    match dialogue_state {
        DialogueState::WaitingForSendAddress { wallet_id, amount, symbol } => {
            // User entered recipient address
            let recipient = text.trim().to_string();

            // Validate address format (basic check)
            if recipient.is_empty() {
                bot.send_message(chat_id, "❌ Please enter a valid address.")
                    .await?;
                return Ok(());
            }

            // Clear dialogue state
            {
                let mut storage = state.dialogue_storage.write().await;
                storage.remove(&user_id);
            }

            // Show confirmation
            show_send_confirmation(&bot, chat_id, &wallet_id, &recipient, &amount, &symbol, &state, user_id).await?;
        }
        DialogueState::WaitingForSendAmount { wallet_id, recipient, symbol } => {
            // User entered amount
            let amount = text.trim().to_string();

            // Validate amount
            if amount.parse::<f64>().is_err() {
                bot.send_message(chat_id, "❌ Please enter a valid number.")
                    .await?;
                return Ok(());
            }

            // If recipient is empty, we need to ask for the address next
            if recipient.is_empty() {
                // Update state to wait for address
                {
                    let mut storage = state.dialogue_storage.write().await;
                    storage.insert(user_id, DialogueState::WaitingForSendAddress {
                        wallet_id: wallet_id.clone(),
                        amount: amount.clone(),
                        symbol: symbol.clone(),
                    });
                }

                // Ask for recipient address with cancel button
                let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                    vec![
                        teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("send:cancel:{}", wallet_id)),
                    ],
                ]);

                bot.send_message(chat_id, format!(
                    "📤 Send {} {}\n\n\
💰 Amount: {} {}\n\n\
📬 Now paste or type the recipient address:",
                    amount, symbol, amount, symbol
                ))
                .reply_markup(keyboard)
                .await?;
            } else {
                // Clear dialogue state
                {
                    let mut storage = state.dialogue_storage.write().await;
                    storage.remove(&user_id);
                }

                // Show confirmation
                show_send_confirmation(&bot, chat_id, &wallet_id, &recipient, &amount, &symbol, &state, user_id).await?;
            }
        }
        DialogueState::WaitingForSwapAmount { wallet_id, from_token, to_token } => {
            // User entered swap amount
            let amount = text.trim().to_string();

            if amount.parse::<f64>().is_err() {
                bot.send_message(chat_id, "❌ Please enter a valid number.")
                    .await?;
                return Ok(());
            }

            // Clear dialogue state
            {
                let mut storage = state.dialogue_storage.write().await;
                storage.remove(&user_id);
            }

            // Show swap confirmation
            show_swap_confirmation(&bot, chat_id, &wallet_id, &from_token, &to_token, &amount, &state).await?;
        }
        DialogueState::WaitingForAlertValue { token_symbol, chain, alert_kind } => {
            let value_str = text.trim();
            let value: f64 = match value_str.parse() {
                Ok(v) if v > 0.0 => v,
                _ => {
                    bot.send_message(chat_id, "❌ Please enter a valid positive number.")
                        .await?;
                    return Ok(());
                }
            };

            // Clear dialogue state
            {
                let mut storage = state.dialogue_storage.write().await;
                storage.remove(&user_id);
            }

            // Show confirmation
            show_alert_confirmation(&bot, chat_id, &token_symbol, &chain, &alert_kind, value, user_id, &state).await?;
        }
        DialogueState::PendingAlertConfirmation { .. } => {
            // Waiting for button confirmation - ignore text
        }
        DialogueState::PendingSendConfirmation { .. } => {
            // User already entered address, waiting for button confirmation - ignore text
        }
        DialogueState::None => {
            // No active dialogue - ignore the message
        }
    }

    Ok(())
}

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    state: Arc<BotState>,
) -> HandlerResult {
    // Answer callback to remove loading state
    bot.answer_callback_query(q.id.clone()).await?;

    let data = match q.data {
        Some(ref d) => d.as_str(),
        None => return Ok(()),
    };

    let chat_id = match q.message {
        Some(ref m) => m.chat().id,
        None => return Ok(()),
    };

    let message_id = match q.message {
        Some(ref m) => m.id(),
        None => return Ok(()),
    };

    let user_id = q.from.id.0 as i64;
    let user_id_str = user_id.to_string();

    // Parse callback data
    let parts: Vec<&str> = data.split(':').collect();

    match parts.as_slice() {
        // Main menu navigation
        ["menu", "main"] => {
            show_main_menu(&bot, chat_id, message_id).await?;
        }
        ["menu", "wallets"] => {
            show_wallets(&bot, chat_id, message_id, &user_id_str, &state).await?;
        }
        ["menu", "create_wallet"] => {
            show_chain_selection(&bot, chat_id, message_id).await?;
        }
        ["menu", "import_wallet"] => {
            show_import_instructions(&bot, chat_id, message_id).await?;
        }
        ["menu", "portfolio"] => {
            show_portfolio(&bot, chat_id, message_id, &user_id_str, &state).await?;
        }
        ["menu", "prices"] => {
            show_prices(&bot, chat_id, message_id, &state).await?;
        }
        ["menu", "addresses"] => {
            show_address_book_menu(&bot, chat_id, message_id).await?;
        }
        ["menu", "alerts"] => {
            show_alerts_menu(&bot, chat_id, message_id).await?;
        }
        ["menu", "security"] => {
            show_security_menu(&bot, chat_id, message_id).await?;
        }
        ["menu", "help"] => {
            show_help_menu(&bot, chat_id, message_id).await?;
        }

        // Chain selection for wallet creation
        ["chain", chain] => {
            create_wallet(&bot, chat_id, message_id, chain, &user_id_str, &state).await?;
        }

        // Wallet actions
        ["wallet", "select", wallet_id] => {
            show_wallet_actions(&bot, chat_id, message_id, wallet_id, &state).await?;
        }
        ["wallet", "balance", wallet_id] => {
            show_wallet_balance(&bot, chat_id, message_id, wallet_id, &state).await?;
        }
        ["wallet", "history", wallet_id] => {
            show_wallet_history(&bot, chat_id, message_id, wallet_id, &user_id_str, &state).await?;
        }
        ["wallet", "qr", wallet_id] => {
            show_wallet_qr(&bot, chat_id, wallet_id, &state).await?;
        }
        ["wallet", "send", wallet_id] => {
            show_send_menu(&bot, chat_id, message_id, wallet_id, &state).await?;
        }
        ["wallet", "swap", wallet_id] => {
            show_swap_menu(&bot, chat_id, message_id, wallet_id, &state).await?;
        }
        ["wallet", "receive", wallet_id] => {
            show_receive_address(&bot, chat_id, message_id, wallet_id, &state).await?;
        }
        ["wallet", "tokens", wallet_id] => {
            show_wallet_tokens(&bot, chat_id, message_id, wallet_id, 0, &state).await?;
        }
        ["wallet", "tokens", wallet_id, page] => {
            let page: usize = page.parse().unwrap_or(0);
            show_wallet_tokens(&bot, chat_id, message_id, wallet_id, page, &state).await?;
        }
        ["wallet", "explorer", wallet_id] => {
            show_wallet_explorer_link(&bot, chat_id, message_id, wallet_id, &state).await?;
        }

        // Send flow
        ["send", "native", wallet_id] => {
            show_send_native(&bot, chat_id, message_id, wallet_id, &state).await?;
        }
        ["send", "token", wallet_id] => {
            show_send_token_prompt(&bot, chat_id, message_id, wallet_id).await?;
        }
        ["send", "custom", wallet_id] => {
            show_send_custom_prompt(&bot, chat_id, message_id, wallet_id, user_id, &state).await?;
        }
        ["send", "amount", wallet_id, percent] => {
            // User selected a percentage - ask for recipient address
            show_send_ask_recipient(&bot, chat_id, message_id, wallet_id, percent, user_id, &state).await?;
        }
        ["send", "confirm"] => {
            // Read transaction details from dialogue state
            let dialogue_state = {
                let storage = state.dialogue_storage.read().await;
                storage.get(&user_id).cloned()
            };

            if let Some(DialogueState::PendingSendConfirmation { wallet_id, recipient, amount, symbol: _ }) = dialogue_state {
                // Clear the state
                {
                    let mut storage = state.dialogue_storage.write().await;
                    storage.remove(&user_id);
                }
                execute_send_with_params(&bot, chat_id, message_id, &wallet_id, &recipient, &amount, &state).await?;
            } else {
                bot.edit_message_text(chat_id, message_id, "❌ Transaction expired. Please start again.")
                    .reply_markup(keyboards::back_to_menu())
                    .await?;
            }
        }
        ["send", "confirm", wallet_id] => {
            execute_send(&bot, chat_id, message_id, wallet_id, user_id, &state).await?;
        }
        ["send", "cancel", wallet_id] => {
            cancel_send(&bot, chat_id, message_id, wallet_id, user_id, &state).await?;
        }

        // Swap flow
        ["swap", "preset1", wallet_id] => {
            show_swap_preset(&bot, chat_id, message_id, wallet_id, "preset1", &user_id_str, &state).await?;
        }
        ["swap", "preset2", wallet_id] => {
            show_swap_preset(&bot, chat_id, message_id, wallet_id, "preset2", &user_id_str, &state).await?;
        }
        ["swap", "custom", wallet_id] => {
            show_swap_custom_prompt(&bot, chat_id, message_id, wallet_id, user_id, &state).await?;
        }
        ["swap", "customamt", wallet_id, from_token, to_token] => {
            show_swap_amount_custom_prompt(&bot, chat_id, message_id, wallet_id, from_token, to_token, user_id, &state).await?;
        }
        ["swap", "amount", wallet_id, from_token, to_token, percent] => {
            show_swap_confirm(&bot, chat_id, message_id, wallet_id, from_token, to_token, percent, &state).await?;
        }
        ["swap", "confirm", wallet_id, from_token, to_token, amount] => {
            execute_swap(&bot, chat_id, message_id, wallet_id, from_token, to_token, amount, &state).await?;
        }
        ["swap", "cancel", wallet_id] => {
            cancel_swap(&bot, chat_id, message_id, wallet_id, &state).await?;
        }

        // Help categories
        ["help", "wallets"] => {
            show_help_wallets(&bot, chat_id, message_id).await?;
        }
        ["help", "transactions"] => {
            show_help_transactions(&bot, chat_id, message_id).await?;
        }
        ["help", "addressbook"] => {
            show_help_addressbook(&bot, chat_id, message_id).await?;
        }
        ["help", "alerts"] => {
            show_help_alerts(&bot, chat_id, message_id).await?;
        }
        ["help", "security"] => {
            show_help_security(&bot, chat_id, message_id).await?;
        }
        ["help", "swap"] => {
            show_help_swap(&bot, chat_id, message_id).await?;
        }

        // Address book
        ["address", "list"] => {
            show_addresses(&bot, chat_id, message_id, &user_id_str, &state).await?;
        }
        ["address", "save"] => {
            show_save_address_instructions(&bot, chat_id, message_id).await?;
        }

        // Alerts
        ["alert", "list"] => {
            show_alerts(&bot, chat_id, message_id, &user_id_str, &state).await?;
        }
        ["alert", "new"] => {
            show_alert_token_selection(&bot, chat_id, message_id).await?;
        }
        ["alert", "token", symbol] => {
            show_alert_chain_or_type(&bot, chat_id, message_id, symbol).await?;
        }
        ["alert", "chain", symbol, chain] => {
            show_alert_type_selection_screen(&bot, chat_id, message_id, symbol, chain, &state).await?;
        }
        ["alert", "type", symbol, chain, alert_kind] => {
            show_alert_value_prompt(&bot, chat_id, message_id, symbol, chain, alert_kind, user_id, &state).await?;
        }
        ["alert", "confirm"] => {
            confirm_create_alert(&bot, chat_id, message_id, user_id, &state).await?;
        }
        ["alert", "cancel"] => {
            // Clear dialogue state and go back to alerts menu
            {
                let mut storage = state.dialogue_storage.write().await;
                storage.remove(&user_id);
            }
            show_alerts_menu(&bot, chat_id, message_id).await?;
        }

        // Refresh actions
        ["refresh", "portfolio"] => {
            show_portfolio(&bot, chat_id, message_id, &user_id_str, &state).await?;
        }
        ["refresh", "prices"] => {
            show_prices(&bot, chat_id, message_id, &state).await?;
        }

        // No-op for non-interactive buttons (e.g. page indicators)
        ["noop"] => {}

        // Cancel action
        ["cancel"] => {
            show_main_menu(&bot, chat_id, message_id).await?;
        }

        _ => {
            tracing::warn!("Unknown callback data: {}", data);
        }
    }

    Ok(())
}

async fn show_main_menu(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🏠 Main Menu\n\nSelect an option:";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::main_menu())
        .await?;

    Ok(())
}

async fn show_chain_selection(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🔗 Select Blockchain\n\nChoose a chain for your new wallet:";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::chain_selection())
        .await?;

    Ok(())
}

async fn show_wallets(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    user_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    match state.wallet_service.list_user_wallets(user_id, None).await {
        Ok(wallets) if wallets.is_empty() => {
            let text = "📭 No Wallets Found\n\nYou don't have any wallets yet.\nCreate one to get started!";

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::wallets_menu())
                .await?;
        }
        Ok(wallets) => {
            let mut buttons: Vec<Vec<teloxide::types::InlineKeyboardButton>> = wallets.iter().map(|w| {
                let chain_emoji = chain_emoji(&w.chain);
                let short_addr = if w.address.len() >= 10 {
                    format!("{}...{}", &w.address[..6], &w.address[w.address.len()-4..])
                } else {
                    w.address.clone()
                };
                let chain_display = w.chain.parse::<Chain>()
                    .map(|c| c.display_name().to_string())
                    .unwrap_or_else(|_| w.chain.clone());
                vec![
                    teloxide::types::InlineKeyboardButton::callback(
                        format!("{} {} {}", chain_emoji, chain_display, short_addr),
                        format!("wallet:select:{}", w.id)
                    )
                ]
            }).collect();

            buttons.push(vec![
                teloxide::types::InlineKeyboardButton::callback("➕ Create New", "menu:create_wallet"),
                teloxide::types::InlineKeyboardButton::callback("📥 Import", "menu:import_wallet"),
            ]);
            buttons.push(vec![
                teloxide::types::InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
            ]);

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(buttons);

            let text = format!("💼 Your Wallets ({})\n\nSelect a wallet to manage:", wallets.len());

            bot.edit_message_text(chat_id, message_id, &text)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get wallets: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Error loading wallets: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_wallet_actions(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            let _ = bot.delete_message(chat_id, message_id).await;
            bot.send_message(chat_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    match state.wallet_service.get_wallet(uuid).await {
        Ok(wallet) => {
            let chain_emoji = chain_emoji(&wallet.chain);

            let text = format!(
                "{} {} Wallet\n\n\
📬 Address:\n`{}`\n\n\
Tap address to copy\\. What would you like to do?",
                chain_emoji,
                wallet.chain,
                wallet.address
            );

            // Try to edit the message, if it fails (e.g., it's a photo), delete and send new
            let edit_result = bot.edit_message_text(chat_id, message_id, &text)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_markup(keyboards::wallet_actions(wallet_id))
                .await;

            if edit_result.is_err() {
                // Message might be a photo, delete it and send a new text message
                let _ = bot.delete_message(chat_id, message_id).await;
                bot.send_message(chat_id, text)
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .reply_markup(keyboards::wallet_actions(wallet_id))
                    .await?;
            }
        }
        Err(e) => {
            let _ = bot.delete_message(chat_id, message_id).await;
            bot.send_message(chat_id, format!("❌ Failed to load wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn create_wallet(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    chain: &str,
    user_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    // Show loading
    bot.edit_message_text(chat_id, message_id, "⏳ Creating wallet...")
        .await?;

    match state.wallet_service.generate_wallet(user_id.to_string(), chain.to_string(), Some(0)).await {
        Ok(response) => {
            let text = format!(
                "✅ Wallet Created Successfully!\n\n\
📍 Chain: {}\n\
🆔 Wallet ID: {}\n\
📬 Address: {}\n\n\
🔑 SAVE YOUR MNEMONIC SECURELY:\n\
{}\n\n\
⚠️ IMPORTANT: Never share your mnemonic. Save it now!",
                chain,
                response.id,
                response.address,
                response.mnemonic.unwrap_or_default()
            );

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to create wallet: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to create wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_wallet_balance(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    bot.edit_message_text(chat_id, message_id, "⏳ Fetching balance...")
        .await?;

    match state.balance_service.get_all_balances(uuid).await {
        Ok(balances) => {
            let emoji = chain_emoji(&balances.chain);
            let chain_name = balances.chain.parse::<Chain>()
                .map(|c| c.display_name())
                .unwrap_or("Unknown");

            let mut text = format!(
                "{} {} Wallet Balance\n\n\
                💎 {} {}\n",
                emoji,
                chain_name,
                balances.native.balance,
                balances.native.symbol
            );

            if !balances.tokens.is_empty() {
                text.push_str(&format!("\n🪙 Tokens ({}):\n", balances.tokens.len()));
                for token in balances.tokens.iter().take(10) {
                    text.push_str(&format!("  • {} {}\n", token.balance, token.symbol));
                }
                if balances.tokens.len() > 10 {
                    text.push_str(&format!("  ... and {} more\n", balances.tokens.len() - 10));
                }
            }

            // Explorer link
            let addr_short = if balances.address.len() > 12 {
                format!("{}...{}", &balances.address[..6], &balances.address[balances.address.len()-4..])
            } else {
                balances.address.clone()
            };
            let explorer_url = state.config.get_address_explorer_url(&balances.chain, &balances.address);
            text.push_str(&format!("\n📬 {}\n🔍 {}", addr_short, explorer_url));

            let mut buttons = vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback("🔄 Refresh", format!("wallet:balance:{}", wallet_id)),
                    teloxide::types::InlineKeyboardButton::callback("🪙 All Tokens", format!("wallet:tokens:{}", wallet_id)),
                ],
            ];
            buttons.push(vec![
                teloxide::types::InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
            ]);

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(teloxide::types::InlineKeyboardMarkup::new(buttons))
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get balance: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to get balance: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_wallet_history(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    user_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    bot.edit_message_text(chat_id, message_id, "⏳ Fetching transaction history...")
        .await?;

    match state.transaction_service.get_wallet_transactions(uuid, Some(10), None).await {
        Ok(transactions) if transactions.is_empty() => {
            let text = "📭 No Transaction History\n\n\
This wallet has no transactions yet.\n\n\
Transactions will appear here after you:\n\
• Send or receive crypto\n\
• Perform swaps";

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::wallet_actions(wallet_id))
                .await?;
        }
        Ok(transactions) => {
            let mut text = String::from("📋 Transaction History\n\n");

            for tx in transactions.iter().take(5) {
                let symbol = tx.token_symbol.as_deref().unwrap_or(&tx.chain);
                let tx_hash_short = if tx.tx_hash.len() > 16 { &tx.tx_hash[..16] } else { &tx.tx_hash };
                let to_addr_short = if tx.to_address.len() > 10 { &tx.to_address[..10] } else { &tx.to_address };
                let explorer_url = state.config.get_tx_explorer_url(&tx.chain, &tx.tx_hash);
                text.push_str(&format!(
                    "🔸 {}...\n   {} {} → {}...\n   🔍 {}\n\n",
                    tx_hash_short,
                    tx.amount,
                    symbol,
                    to_addr_short,
                    explorer_url
                ));
            }

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
                ],
            ]);

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get history: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to get history: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_wallet_qr(
    bot: &Bot,
    chat_id: ChatId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Invalid wallet ID").await?;
            return Ok(());
        }
    };

    match state.wallet_service.get_wallet(uuid).await {
        Ok(wallet) => {
            let chain_emoji = chain_emoji(&wallet.chain);

            // Generate QR code
            let qr = qrcode::QrCode::new(&wallet.address)?;
            let image = qr.render::<image::Luma<u8>>().build();

            let mut bytes: Vec<u8> = Vec::new();
            image::DynamicImage::ImageLuma8(image)
                .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;

            let input_file = teloxide::types::InputFile::memory(bytes).file_name("qr.png");

            let caption = format!(
                "📥 Receive {}\n\n\
{} Address:\n`{}`\n\n\
Scan QR or tap address to copy\\.",
                wallet.chain,
                chain_emoji,
                wallet.address
            );

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
                ],
            ]);

            bot.send_photo(chat_id, input_file)
                .caption(caption)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get wallet: {:?}", e);
            bot.send_message(chat_id, format!("❌ Failed to get wallet: {}", e)).await?;
        }
    }

    Ok(())
}

async fn show_wallet_tokens(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    page: usize,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    bot.edit_message_text(chat_id, message_id, "⏳ Discovering tokens...")
        .await?;

    match state.balance_service.get_all_balances(uuid).await {
        Ok(balances) => {
            let emoji = chain_emoji(&balances.chain);
            let chain_name = balances.chain.parse::<Chain>()
                .map(|c| c.display_name())
                .unwrap_or("Unknown");

            if balances.tokens.is_empty() {
                let text = format!(
                    "{} {} Tokens\n\n\
                    No ERC-20/SPL tokens found in this wallet.\n\n\
                    💎 Native: {} {}",
                    emoji, chain_name,
                    balances.native.balance, balances.native.symbol
                );

                bot.edit_message_text(chat_id, message_id, text)
                    .reply_markup(teloxide::types::InlineKeyboardMarkup::new(vec![
                        vec![
                            teloxide::types::InlineKeyboardButton::callback("🔄 Refresh", format!("wallet:tokens:{}", wallet_id)),
                            teloxide::types::InlineKeyboardButton::callback("« Back", format!("wallet:select:{}", wallet_id)),
                        ],
                    ]))
                    .await?;
                return Ok(());
            }

            let tokens_per_page = 8;
            let total_pages = (balances.tokens.len() + tokens_per_page - 1) / tokens_per_page;
            let page = page.min(total_pages.saturating_sub(1));
            let start = page * tokens_per_page;
            let page_tokens = &balances.tokens[start..(start + tokens_per_page).min(balances.tokens.len())];

            let mut text = format!(
                "{} {} Tokens ({})\n\n",
                emoji, chain_name, balances.tokens.len()
            );

            for token in page_tokens {
                text.push_str(&format!("🪙 {} — {}\n", token.symbol, token.balance));
                if let Some(ref name) = token.logo_url {
                    // Just show name, not the URL
                    let _ = name;
                }
            }

            text.push_str(&format!("\n💎 Native: {} {}", balances.native.balance, balances.native.symbol));

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::token_list(wallet_id, page, total_pages))
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get tokens: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to discover tokens: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_wallet_explorer_link(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    match state.wallet_service.get_wallet(uuid).await {
        Ok(wallet) => {
            let emoji = chain_emoji(&wallet.chain);
            let explorer_url = state.config.get_address_explorer_url(&wallet.chain, &wallet.address);
            let chain_name = wallet.chain.parse::<Chain>()
                .map(|c| c.display_name())
                .unwrap_or("Unknown");

            let text = format!(
                "🔍 {} {} Explorer\n\n\
                📬 Address:\n{}\n\n\
                🔗 View on Explorer:\n{}",
                emoji, chain_name,
                wallet.address,
                explorer_url
            );

            let keyboard = if let Ok(parsed_url) = reqwest::Url::parse(&explorer_url) {
                teloxide::types::InlineKeyboardMarkup::new(vec![
                    vec![
                        teloxide::types::InlineKeyboardButton::url("🌐 Open Explorer", parsed_url),
                    ],
                    vec![
                        teloxide::types::InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
                    ],
                ])
            } else {
                teloxide::types::InlineKeyboardMarkup::new(vec![
                    vec![
                        teloxide::types::InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
                    ],
                ])
            };

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_portfolio(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    user_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    bot.edit_message_text(chat_id, message_id, "⏳ Fetching portfolio data...")
        .await?;

    match state.portfolio_service.get_portfolio(user_id).await {
        Ok(portfolio) => {
            let mut text = format!(
                "💼 Your Portfolio\n📊 {} wallets across {} chains\n\n",
                portfolio.wallet_count,
                portfolio.chains.len()
            );

            for holding in &portfolio.holdings {
                let change_str = holding.price_change_24h
                    .map(|c| {
                        let arrow = if c >= 0.0 { "📈" } else { "📉" };
                        format!(" {} {:.1}%", arrow, c.abs())
                    })
                    .unwrap_or_default();

                text.push_str(&format!(
                    "{} {} {:.6} (${:.2}){}\n",
                    chain_emoji(&holding.symbol),
                    holding.symbol,
                    holding.total_balance,
                    holding.usd_value,
                    change_str,
                ));

                // Show per-wallet breakdown if multiple wallets hold this token
                if holding.wallets.len() > 1 {
                    for wh in &holding.wallets {
                        let short_addr = if wh.address.len() >= 10 {
                            format!("{}...{}", &wh.address[..6], &wh.address[wh.address.len()-4..])
                        } else {
                            wh.address.clone()
                        };
                        text.push_str(&format!("   └ {} {}\n", short_addr, wh.balance));
                    }
                }
            }

            text.push_str(&format!("\n💰 Total Value: ${:.2}", portfolio.total_usd_value));

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::refresh_button("portfolio"))
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get portfolio: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load portfolio: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_prices(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    state: &Arc<BotState>,
) -> HandlerResult {
    bot.edit_message_text(chat_id, message_id, "⏳ Fetching current prices...")
        .await?;

    // Fetch prices for all configured chain native tokens
    let symbols: Vec<String> = state.config.configured_chains()
        .iter()
        .map(|c| c.native_symbol().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    match state.price_service.get_prices(&symbols).await {
        Ok(prices) => {
            let mut text = String::from("📊 Cryptocurrency Prices\n\n");

            // Display in a deterministic order
            let mut sorted_prices: Vec<_> = prices.iter().collect();
            sorted_prices.sort_by(|a, b| {
                b.1.usd_price.partial_cmp(&a.1.usd_price).unwrap_or(std::cmp::Ordering::Equal)
            });

            for (symbol, price) in &sorted_prices {
                let change = price.price_change_24h.unwrap_or(0.0);
                let change_emoji = if change >= 0.0 { "📈" } else { "📉" };
                text.push_str(&format!(
                    "{} {}: ${:.2} {} {:.2}%\n",
                    chain_emoji(symbol),
                    symbol,
                    price.usd_price,
                    change_emoji,
                    change.abs()
                ));
            }

            text.push_str("\n_Updated just now_");

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::refresh_button("prices"))
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get prices: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load prices: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_import_instructions(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let chain_list: String = Chain::all()
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let text = format!(
        "📥 Import Wallet\n\n\
To import an existing wallet, use the command:\n\n\
/importwallet <chain> <mnemonic or private key>\n\n\
Examples:\n\
/importwallet ETH word1 word2 word3...\n\
/importwallet SOLANA 5J7K...\n\
/importwallet POLYGON word1 word2 word3...\n\n\
Supported chains: {}",
        chain_list
    );

    bot.edit_message_text(chat_id, message_id, &text)
        .reply_markup(keyboards::back_to_menu())
        .await?;

    Ok(())
}

async fn show_send_menu(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    // Get wallet to know the chain
    match state.wallet_service.get_wallet(uuid).await {
        Ok(wallet) => {
            // Get balance
            let balance_str = match state.balance_service.get_balance(uuid, None).await {
                Ok(b) => format!("{} {}", b.balance, b.symbol),
                Err(_) => "Unknown".to_string(),
            };

            let native_token = wallet.chain.parse::<Chain>()
                .map(|c| c.native_symbol())
                .unwrap_or("tokens");

            let text = format!(
                "📤 Send Crypto\n\n\
💰 Available: {}\n\n\
What would you like to send?",
                balance_str
            );

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::send_menu(wallet_id, &wallet.chain))
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get wallet: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_send_native(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    match state.balance_service.get_balance(uuid, None).await {
        Ok(balance) => {
            let text = format!(
                "📤 Send {}\n\n\
💰 Available: {} {}\n\n\
Select the amount you want to send:",
                balance.symbol,
                balance.balance,
                balance.symbol
            );

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::send_amount_presets(wallet_id, &balance.balance))
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to get balance: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_send_ask_recipient(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    percent: &str,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    let percent_val: f64 = percent.parse().unwrap_or(0.0);

    match state.balance_service.get_balance(uuid, None).await {
        Ok(balance) => {
            let balance_num: f64 = balance.balance.parse().unwrap_or(0.0);
            let amount = balance_num * (percent_val / 100.0);
            let amount_str = format!("{:.6}", amount);

            // Set dialogue state to wait for recipient address
            {
                tracing::info!("Setting dialogue state for user_id: {}", user_id);
                let mut storage = state.dialogue_storage.write().await;
                storage.insert(user_id, DialogueState::WaitingForSendAddress {
                    wallet_id: wallet_id.to_string(),
                    amount: amount_str.clone(),
                    symbol: balance.symbol.clone(),
                });
                tracing::info!("Dialogue state set successfully. Storage now has {} entries", storage.len());
            }

            let text = format!(
                "📤 Send {} {}\n\n\
💰 Amount: {} {} ({}%)\n\n\
📬 Now paste or type the recipient address:",
                percent, balance.symbol,
                amount_str, balance.symbol, percent
            );

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("send:cancel:{}", wallet_id)),
                ],
            ]);

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to get balance: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_send_custom_prompt(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    match state.balance_service.get_balance(uuid, None).await {
        Ok(balance) => {
            // Set dialogue state to wait for amount first, then recipient
            {
                let mut storage = state.dialogue_storage.write().await;
                storage.insert(user_id, DialogueState::WaitingForSendAmount {
                    wallet_id: wallet_id.to_string(),
                    recipient: String::new(), // Will ask for this after amount
                    symbol: balance.symbol.clone(),
                });
            }

            let text = format!(
                "📤 Send {}\n\n\
💰 Available: {} {}\n\n\
Type the amount you want to send:",
                balance.symbol,
                balance.balance,
                balance.symbol
            );

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("send:cancel:{}", wallet_id)),
                ],
            ]);

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to get balance: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_send_token_prompt(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
) -> HandlerResult {
    let text = "🪙 Send Token\n\n\
Token sending coming soon!\n\n\
For now, please send native tokens only.";

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback("« Back", format!("wallet:send:{}", wallet_id)),
        ],
    ]);

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn show_send_confirmation(
    bot: &Bot,
    chat_id: ChatId,
    wallet_id: &str,
    recipient: &str,
    amount: &str,
    symbol: &str,
    state: &Arc<BotState>,
    user_id: i64,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Invalid wallet ID").await?;
            return Ok(());
        }
    };

    let wallet = match state.wallet_service.get_wallet(uuid).await {
        Ok(w) => w,
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to load wallet: {}", e)).await?;
            return Ok(());
        }
    };

    let short_recipient = if recipient.len() > 16 {
        format!("{}...{}", &recipient[..8], &recipient[recipient.len()-6..])
    } else {
        recipient.to_string()
    };

    let text = format!(
        "📤 Confirm Transaction\n\n\
From: {} Wallet\n\
{}\n\n\
To: {}\n\n\
Amount: {} {}\n\n\
⚠️ Please verify all details before confirming.",
        wallet.chain,
        wallet.address,
        short_recipient,
        amount,
        symbol
    );

    // Store transaction details in dialogue state for the confirm button
    // (Telegram callback data has 64-byte limit, can't fit wallet_id + address + amount)
    {
        let mut storage = state.dialogue_storage.write().await;
        storage.insert(user_id, DialogueState::PendingSendConfirmation {
            wallet_id: wallet_id.to_string(),
            recipient: recipient.to_string(),
            amount: amount.to_string(),
            symbol: symbol.to_string(),
        });
    }

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback("✅ Confirm & Send", "send:confirm"),
        ],
        vec![
            teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("wallet:select:{}", wallet_id)),
        ],
    ]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn execute_send(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    _wallet_id: &str,
    _user_id: i64,
    _state: &Arc<BotState>,
) -> HandlerResult {
    // This would be called from confirm button
    bot.edit_message_text(chat_id, message_id, "⏳ Processing transaction...")
        .await?;

    // TODO: Implement actual transfer
    bot.edit_message_text(chat_id, message_id, "✅ Transaction submitted!\n\nYou will receive a notification when it's confirmed.")
        .reply_markup(keyboards::back_to_menu())
        .await?;

    Ok(())
}

async fn execute_send_with_params(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    recipient: &str,
    amount: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    use crate::services::transfer_service::TransferRequest;

    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    bot.edit_message_text(chat_id, message_id, "⏳ Processing transaction...")
        .await?;

    // Get wallet to know the chain/symbol
    let wallet = match state.wallet_service.get_wallet(uuid).await {
        Ok(w) => w,
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    let symbol = wallet.chain.parse::<Chain>()
        .map(|c| c.native_symbol())
        .unwrap_or("tokens");

    // Create transfer request
    let transfer_request = TransferRequest {
        to: recipient.to_string(),
        amount: amount.to_string(),
        token_address: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        gas_limit: None,
        compute_units: None,
    };

    // Execute the transfer
    match state.transfer_service.send_transaction(uuid, transfer_request).await {
        Ok(result) => {
            let explorer_url = state.config.get_tx_explorer_url(&wallet.chain, &result.tx_hash);
            let text = format!(
                "✅ Transaction Submitted!\n\n\
📤 Sent: {} {}\n\
📬 To: {}\n\
🔗 TX Hash:\n{}\n\n\
🔍 View on Explorer:\n{}\n\n\
Your transaction is being processed.",
                amount,
                symbol,
                if recipient.len() > 20 {
                    format!("{}...{}", &recipient[..10], &recipient[recipient.len()-8..])
                } else {
                    recipient.to_string()
                },
                result.tx_hash,
                explorer_url
            );

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
        Err(e) => {
            tracing::error!("Transfer failed: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Transaction Failed\n\n{}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn cancel_send(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    // Clear dialogue state
    {
        let mut storage = state.dialogue_storage.write().await;
        storage.remove(&user_id);
    }

    // Return to wallet
    show_wallet_actions(bot, chat_id, message_id, wallet_id, state).await
}

async fn show_receive_address(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    match state.wallet_service.get_wallet(uuid).await {
        Ok(wallet) => {
            let chain_emoji = chain_emoji(&wallet.chain);

            // Generate QR code
            let qr = qrcode::QrCode::new(&wallet.address)?;
            let image = qr.render::<image::Luma<u8>>().build();

            let mut bytes: Vec<u8> = Vec::new();
            image::DynamicImage::ImageLuma8(image)
                .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;

            let input_file = teloxide::types::InputFile::memory(bytes).file_name("qr.png");

            let caption = format!(
                "📥 Receive {}\n\n\
{} Address:\n`{}`\n\n\
Scan QR or tap address to copy\\.",
                wallet.chain,
                chain_emoji,
                wallet.address
            );

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
                ],
            ]);

            // Delete the old message and send photo
            let _ = bot.delete_message(chat_id, message_id).await;

            bot.send_photo(chat_id, input_file)
                .caption(caption)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_swap_menu(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    match state.wallet_service.get_wallet(uuid).await {
        Ok(wallet) => {
            let balance_str = match state.balance_service.get_balance(uuid, None).await {
                Ok(b) => format!("{} {}", b.balance, b.symbol),
                Err(_) => "Unknown".to_string(),
            };

            let text = format!(
                "💱 Swap Tokens\n\n\
💰 Available: {}\n\n\
Select a swap pair or create custom swap:",
                balance_str
            );

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::swap_menu(wallet_id, &wallet.chain))
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get wallet: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_swap_preset(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    preset: &str,
    user_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    let wallet = match state.wallet_service.get_wallet(uuid).await {
        Ok(w) => w,
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load wallet: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    let chain = wallet.chain.parse::<Chain>().ok();
    let native = chain.map(|c| c.native_symbol()).unwrap_or("NATIVE");

    let (from_token, to_token) = match (chain, preset) {
        (Some(Chain::Bsc), "preset1") => (native, "USDT"),
        (Some(Chain::Bsc), "preset2") => (native, "BUSD"),
        (_, "preset1") => (native, "USDC"),
        (_, "preset2") => (native, "USDT"),
        _ => (native, "USDC"),
    };

    let balance_str = match state.balance_service.get_balance(uuid, None).await {
        Ok(b) => format!("{} {}", b.balance, b.symbol),
        Err(_) => "Unknown".to_string(),
    };

    let text = format!(
        "💱 Swap {} → {}\n\n\
💰 Available: {}\n\n\
Select amount to swap:",
        from_token, to_token, balance_str
    );

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::swap_amount_presets(wallet_id, from_token, to_token))
        .await?;

    Ok(())
}

async fn show_swap_custom_prompt(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    let balance_str = match state.balance_service.get_balance(uuid, None).await {
        Ok(b) => b.symbol,
        Err(_) => "tokens".to_string(),
    };

    // Set dialogue to wait for swap amount
    {
        let mut storage = state.dialogue_storage.write().await;
        storage.insert(user_id, DialogueState::WaitingForSwapAmount {
            wallet_id: wallet_id.to_string(),
            from_token: balance_str.clone(),
            to_token: "USDC".to_string(),
        });
    }

    let text = format!(
        "💱 Custom Swap\n\n\
Type the amount of {} you want to swap:\n\n\
(You can swap to stablecoins)",
        balance_str
    );

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("swap:cancel:{}", wallet_id)),
        ],
    ]);

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn show_swap_amount_custom_prompt(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    from_token: &str,
    to_token: &str,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    // Set dialogue to wait for swap amount
    {
        let mut storage = state.dialogue_storage.write().await;
        storage.insert(user_id, DialogueState::WaitingForSwapAmount {
            wallet_id: wallet_id.to_string(),
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
        });
    }

    let text = format!(
        "💱 Swap {} → {}\n\n\
Type the amount of {} you want to swap:",
        from_token, to_token, from_token
    );

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("swap:cancel:{}", wallet_id)),
        ],
    ]);

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn show_swap_confirm(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    from_token: &str,
    to_token: &str,
    percent: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let uuid = match uuid::Uuid::parse_str(wallet_id) {
        Ok(id) => id,
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid wallet ID")
                .reply_markup(keyboards::back_to_menu())
                .await?;
            return Ok(());
        }
    };

    let percent_val: f64 = percent.parse().unwrap_or(0.0);

    match state.balance_service.get_balance(uuid, None).await {
        Ok(balance) => {
            let balance_num: f64 = balance.balance.parse().unwrap_or(0.0);
            let amount = balance_num * (percent_val / 100.0);
            let amount_str = format!("{:.6}", amount);

            let text = format!(
                "💱 Confirm Swap\n\n\
From: {} {}\n\
To: {} (estimated)\n\n\
Amount: {}%\n\n\
⚠️ Slippage: 0.5%\n\
Final amount may vary.",
                amount_str, from_token,
                to_token,
                percent
            );

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback(
                        "✅ Confirm Swap",
                        format!("swap:confirm:{}:{}:{}:{}", wallet_id, from_token, to_token, amount_str)
                    ),
                ],
                vec![
                    teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("wallet:select:{}", wallet_id)),
                ],
            ]);

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to get balance: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_swap_confirmation(
    bot: &Bot,
    chat_id: ChatId,
    wallet_id: &str,
    from_token: &str,
    to_token: &str,
    amount: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    let text = format!(
        "💱 Confirm Swap\n\n\
Swap: {} {}\n\
To: {} (estimated)\n\n\
⚠️ Slippage: 0.5%\n\
Final amount may vary.",
        amount, from_token, to_token
    );

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback(
                "✅ Confirm Swap",
                format!("swap:confirm:{}:{}:{}:{}", wallet_id, from_token, to_token, amount)
            ),
        ],
        vec![
            teloxide::types::InlineKeyboardButton::callback("❌ Cancel", format!("wallet:select:{}", wallet_id)),
        ],
    ]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn execute_swap(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    from_token: &str,
    to_token: &str,
    amount: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    bot.edit_message_text(chat_id, message_id, "⏳ Processing swap...")
        .await?;

    // TODO: Implement actual swap
    bot.edit_message_text(chat_id, message_id, "✅ Swap submitted!\n\nYou will receive a notification when it's confirmed.")
        .reply_markup(keyboards::back_to_menu())
        .await?;

    Ok(())
}

async fn cancel_swap(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    wallet_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    show_wallet_actions(bot, chat_id, message_id, wallet_id, state).await
}

async fn show_help_menu(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "❓ Help Center\n\nSelect a category to learn more:";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::help_menu())
        .await?;

    Ok(())
}

async fn show_help_wallets(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let chain_list: String = Chain::all()
        .iter()
        .map(|c| format!("{} {}", c.emoji(), c.as_str()))
        .collect::<Vec<_>>()
        .join(", ");

    let text = format!(
        "💼 Wallet Commands\n\n\
/createwallet <chain> - Create new wallet\n\
/importwallet <chain> <key> - Import wallet\n\
/wallets - List all wallets\n\
/balance <wallet_id> - Check balance\n\
/address <wallet_id> - Get address with QR\n\n\
Supported chains:\n{}",
        chain_list
    );

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::help_menu())
        .await?;

    Ok(())
}

async fn show_help_transactions(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "💸 Transaction Commands\n\n\
/send <wallet_id> <to> <amount> - Send tokens\n\
/estimatefee <wallet_id> <to> <amount> - Estimate fees\n\
/batchsend <wallet_id> - Send to multiple addresses\n\
/history <wallet_id> - View transaction history";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::help_menu())
        .await?;

    Ok(())
}

async fn show_help_addressbook(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "📖 Address Book Commands\n\n\
/saveaddress <name> <addr> <chain> - Save address\n\
/addresses - List saved addresses\n\
/deleteaddress <name> - Delete saved address\n\n\
Use saved names instead of addresses when sending!";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::help_menu())
        .await?;

    Ok(())
}

async fn show_help_alerts(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🔔 Alerts & Scheduling\n\n\
/setalert <symbol> <above|below> <price> - Set price alert\n\
/alerts - List your alerts\n\
/deletealert <id> - Delete alert\n\n\
/schedule <wallet_id> <to> <amount> <datetime> - Schedule tx\n\
/scheduled - List scheduled transactions\n\
/cancelschedule <id> - Cancel scheduled tx";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::help_menu())
        .await?;

    Ok(())
}

async fn show_help_security(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🔐 Security Commands\n\n\
/setpin <6-digit-pin> - Set transaction PIN\n\
/changepin <old> <new> - Change PIN\n\
/disablepin - Disable PIN protection\n\
/setlimit daily|weekly <amount> - Set limits\n\
/lockwallet - Lock your wallet\n\
/unlock <pin> - Unlock wallet\n\
/security - View security settings";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::help_menu())
        .await?;

    Ok(())
}

async fn show_help_swap(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "💱 Swap Commands\n\n\
/swapquote <chain> <from> <to> <amount> - Get quote\n\
/swap <wallet_id> <from> <to> <amount> - Execute swap\n\
/swaphistory - View swap history";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::help_menu())
        .await?;

    Ok(())
}

async fn show_security_menu(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🔐 Security Settings\n\nManage your wallet security:";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::security_menu())
        .await?;

    Ok(())
}

async fn show_alerts_menu(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🔔 Price Alerts\n\nGet notified when prices hit your targets:";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::alerts_menu())
        .await?;

    Ok(())
}

async fn show_address_book_menu(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "📖 Address Book\n\nSave frequently used addresses:";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::address_book_menu())
        .await?;

    Ok(())
}

async fn show_addresses(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    user_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    match state.address_book_service.list_addresses(user_id, None).await {
        Ok(addresses) if addresses.is_empty() => {
            bot.edit_message_text(chat_id, message_id, "📭 No saved addresses yet.\n\nSave one with /saveaddress")
                .reply_markup(keyboards::address_book_menu())
                .await?;
        }
        Ok(addresses) => {
            let mut text = String::from("📖 Saved Addresses\n\n");

            for addr in &addresses {
                text.push_str(&format!(
                    "📇 {} ({})\n   {}\n\n",
                    addr.name,
                    addr.chain,
                    addr.address
                ));
            }

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::address_book_menu())
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get addresses: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load addresses: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_alerts(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    user_id: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    match state.price_alert_service.list_user_alerts(user_id, true).await {
        Ok(alerts) if alerts.is_empty() => {
            bot.edit_message_text(chat_id, message_id, "📭 No price alerts set.\n\nCreate one with /setalert")
                .reply_markup(keyboards::alerts_menu())
                .await?;
        }
        Ok(alerts) => {
            let mut text = String::from("🔔 Your Price Alerts\n\n");

            for alert in &alerts {
                let condition = match alert.alert_type.parse::<AlertKind>() {
                    Ok(AlertKind::Above) => "📈 Above",
                    Ok(AlertKind::Below) => "📉 Below",
                    Ok(AlertKind::PercentChange) => "⚡ Change",
                    Err(_) => "🔔 Alert",
                };
                let price_str = alert.target_price
                    .map(|p| format!("${}", p))
                    .unwrap_or_else(|| "N/A".to_string());
                let id_short = &alert.id.to_string()[..8];
                text.push_str(&format!(
                    "🔸 {} {} {}\n   ID: {}\n\n",
                    alert.token_symbol,
                    condition,
                    price_str,
                    id_short
                ));
            }

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::alerts_menu())
                .await?;
        }
        Err(e) => {
            tracing::error!("Failed to get alerts: {:?}", e);
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to load alerts: {}", e))
                .reply_markup(keyboards::back_to_menu())
                .await?;
        }
    }

    Ok(())
}

async fn show_save_address_instructions(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "📝 Save Address\n\n\
Use the command:\n\
/saveaddress <name> <address> <chain>\n\n\
Example:\n\
/saveaddress alice 0x742d... ETH";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::address_book_menu())
        .await?;

    Ok(())
}

// ==================== INTERACTIVE ALERT FLOW ====================

fn chains_for_symbol(symbol: &str) -> Vec<Chain> {
    let chains: Vec<Chain> = Chain::all()
        .iter()
        .filter(|c| c.native_symbol().eq_ignore_ascii_case(symbol))
        .copied()
        .collect();
    if chains.is_empty() {
        vec![Chain::Eth]
    } else {
        chains
    }
}

async fn show_alert_token_selection(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🔔 Create Price Alert\n\nSelect a token to watch:";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::alert_token_selection())
        .await?;

    Ok(())
}

async fn show_alert_chain_or_type(bot: &Bot, chat_id: ChatId, message_id: MessageId, symbol: &str) -> HandlerResult {
    let chains = chains_for_symbol(symbol);

    if chains.len() == 1 {
        // Single chain — skip chain picker, go directly to alert type
        let chain_str = chains[0].as_str();
        let text = format!(
            "🔔 Alert for {}\n\nSelect alert type:",
            symbol
        );

        bot.edit_message_text(chat_id, message_id, text)
            .reply_markup(keyboards::alert_type_selection(symbol, chain_str))
            .await?;
    } else {
        // Multiple chains — show chain picker
        let text = format!(
            "🔔 Alert for {}\n\n{} is available on multiple chains.\nSelect which chain:",
            symbol, symbol
        );

        bot.edit_message_text(chat_id, message_id, text)
            .reply_markup(keyboards::alert_chain_selection(symbol, &chains))
            .await?;
    }

    Ok(())
}

async fn show_alert_type_selection_screen(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    symbol: &str,
    chain: &str,
    state: &Arc<BotState>,
) -> HandlerResult {
    // Fetch current price to show context
    let price_str = match state.price_service.get_price(symbol).await {
        Ok(p) => format!("Current price: ${:.2}", p.usd_price),
        Err(_) => String::new(),
    };

    let chain_name = chain.parse::<Chain>()
        .map(|c| c.display_name())
        .unwrap_or(chain);

    let text = format!(
        "🔔 Alert for {} on {}\n{}\n\nSelect alert type:",
        symbol, chain_name, price_str
    );

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::alert_type_selection(symbol, chain))
        .await?;

    Ok(())
}

async fn show_alert_value_prompt(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    symbol: &str,
    chain: &str,
    alert_kind: &str,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    // Fetch current price
    let price_info = match state.price_service.get_price(symbol).await {
        Ok(p) => Some(p),
        Err(_) => None,
    };

    let current_price_str = price_info
        .as_ref()
        .map(|p| format!("\n💰 Current price: ${:.2}", p.usd_price))
        .unwrap_or_default();

    let prompt = match alert_kind {
        "above" => format!(
            "📈 Alert: {} Price Above\n{}\n\nEnter the target price in USD:",
            symbol, current_price_str
        ),
        "below" => format!(
            "📉 Alert: {} Price Below\n{}\n\nEnter the target price in USD:",
            symbol, current_price_str
        ),
        "percent" => format!(
            "⚡ Alert: {} Percent Change\n{}\n\nEnter the percent change (e.g. 10 for +10%, -5 for -5%):",
            symbol, current_price_str
        ),
        _ => return Ok(()),
    };

    // Set dialogue state
    {
        let mut storage = state.dialogue_storage.write().await;
        storage.insert(user_id, DialogueState::WaitingForAlertValue {
            token_symbol: symbol.to_string(),
            chain: chain.to_string(),
            alert_kind: alert_kind.to_string(),
        });
    }

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback("❌ Cancel", "alert:cancel"),
        ],
    ]);

    bot.edit_message_text(chat_id, message_id, prompt)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn show_alert_confirmation(
    bot: &Bot,
    chat_id: ChatId,
    token_symbol: &str,
    chain: &str,
    alert_kind: &str,
    value: f64,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    let chain_name = chain.parse::<Chain>()
        .map(|c| c.display_name())
        .unwrap_or(chain);

    let condition = match alert_kind {
        "above" => format!("Above ${:.2}", value),
        "below" => format!("Below ${:.2}", value),
        "percent" => format!("{:+.1}% change", value),
        _ => "Unknown".to_string(),
    };

    // Fetch current price for context
    let current_price_str = match state.price_service.get_price(token_symbol).await {
        Ok(p) => format!("\n💰 Current Price: ${:.2}", p.usd_price),
        Err(_) => String::new(),
    };

    let text = format!(
        "🔔 Confirm Price Alert\n\n\
Token: {}\n\
Chain: {}\n\
Condition: {}{}\n\n\
Create this alert?",
        token_symbol, chain_name, condition, current_price_str
    );

    // Store confirmation state
    {
        let mut storage = state.dialogue_storage.write().await;
        storage.insert(user_id, DialogueState::PendingAlertConfirmation {
            token_symbol: token_symbol.to_string(),
            chain: chain.to_string(),
            alert_kind: alert_kind.to_string(),
            value,
        });
    }

    let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
        vec![
            teloxide::types::InlineKeyboardButton::callback("✅ Create Alert", "alert:confirm"),
        ],
        vec![
            teloxide::types::InlineKeyboardButton::callback("❌ Cancel", "alert:cancel"),
        ],
    ]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn confirm_create_alert(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    user_id: i64,
    state: &Arc<BotState>,
) -> HandlerResult {
    use crate::enums::AlertType;
    use crate::services::price_alert_service::CreateAlertRequest;

    let dialogue_state = {
        let storage = state.dialogue_storage.read().await;
        storage.get(&user_id).cloned()
    };

    let (token_symbol, chain, alert_kind, value) = match dialogue_state {
        Some(DialogueState::PendingAlertConfirmation { token_symbol, chain, alert_kind, value }) => {
            (token_symbol, chain, alert_kind, value)
        }
        _ => {
            bot.edit_message_text(chat_id, message_id, "❌ Alert expired. Please start again.")
                .reply_markup(keyboards::alerts_menu())
                .await?;
            return Ok(());
        }
    };

    // Clear dialogue state
    {
        let mut storage = state.dialogue_storage.write().await;
        storage.remove(&user_id);
    }

    bot.edit_message_text(chat_id, message_id, "⏳ Creating alert...")
        .await?;

    // Build alert type
    let alert_type = match alert_kind.as_str() {
        "above" => AlertType::Above { target_price: value },
        "below" => AlertType::Below { target_price: value },
        "percent" => {
            match state.price_service.get_price(&token_symbol).await {
                Ok(current_price) => AlertType::PercentChange {
                    percent: value,
                    base_price: current_price.usd_price,
                },
                Err(e) => {
                    bot.edit_message_text(chat_id, message_id, format!("❌ Could not get current price: {}", e))
                        .reply_markup(keyboards::alerts_menu())
                        .await?;
                    return Ok(());
                }
            }
        }
        _ => {
            bot.edit_message_text(chat_id, message_id, "❌ Invalid alert type")
                .reply_markup(keyboards::alerts_menu())
                .await?;
            return Ok(());
        }
    };

    let request = CreateAlertRequest {
        user_id: user_id.to_string(),
        token_symbol: token_symbol.clone(),
        chain: chain.clone(),
        token_address: None,
        alert_type,
    };

    match state.price_alert_service.create_alert(request).await {
        Ok(_) => {
            let chain_name = chain.parse::<Chain>()
                .map(|c| c.display_name())
                .unwrap_or(&chain);

            let condition = match alert_kind.as_str() {
                "above" => format!("Above ${:.2}", value),
                "below" => format!("Below ${:.2}", value),
                "percent" => format!("{:+.1}% change", value),
                _ => "Unknown".to_string(),
            };

            let text = format!(
                "✅ Alert Created!\n\n\
Token: {}\n\
Chain: {}\n\
Condition: {}\n\n\
You'll be notified when triggered.",
                token_symbol, chain_name, condition
            );

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboards::alerts_menu())
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(chat_id, message_id, format!("❌ Failed to create alert: {}", e))
                .reply_markup(keyboards::alerts_menu())
                .await?;
        }
    }

    Ok(())
}

fn chain_emoji(chain: &str) -> &'static str {
    chain.parse::<Chain>().map(|c| c.emoji()).unwrap_or("📍")
}
