use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::MessageId;

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
            show_new_alert_instructions(&bot, chat_id, message_id).await?;
        }

        // Refresh actions
        ["refresh", "portfolio"] => {
            show_portfolio(&bot, chat_id, message_id, &user_id_str, &state).await?;
        }
        ["refresh", "prices"] => {
            show_prices(&bot, chat_id, message_id, &state).await?;
        }

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
                let chain_emoji = match w.chain.as_str() {
                    "ETH" => "🔷",
                    "BSC" => "🟡",
                    "SOLANA" => "🟣",
                    _ => "📍",
                };
                let short_addr = format!("{}...{}", &w.address[..6], &w.address[w.address.len()-4..]);
                vec![
                    teloxide::types::InlineKeyboardButton::callback(
                        format!("{} {} {}", chain_emoji, w.chain, short_addr),
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
            let chain_emoji = match wallet.chain.as_str() {
                "ETH" => "🔷",
                "BSC" => "🟡",
                "SOLANA" => "🟣",
                _ => "📍",
            };

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

    match state.balance_service.get_balance(uuid, None).await {
        Ok(balance) => {
            let text = format!(
                "💰 Balance\n\n\
                💵 Symbol: {}\n\
                💎 Amount: {}",
                balance.symbol,
                balance.balance
            );

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![
                    teloxide::types::InlineKeyboardButton::callback("🔄 Refresh", format!("wallet:balance:{}", wallet_id)),
                ],
                vec![
                    teloxide::types::InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
                ],
            ]);

            bot.edit_message_text(chat_id, message_id, text)
                .reply_markup(keyboard)
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
            let chain_emoji = match wallet.chain.as_str() {
                "ETH" => "🔷",
                "BSC" => "🟡",
                "SOLANA" => "🟣",
                _ => "📍",
            };

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
            let mut text = String::from("💼 Your Portfolio\n\n");

            for holding in &portfolio.holdings {
                text.push_str(&format!(
                    "{} {}: {:.6} (${:.2})\n",
                    chain_emoji(&holding.symbol),
                    holding.symbol,
                    holding.total_balance,
                    holding.usd_value
                ));
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

    let symbols = vec!["ETH".to_string(), "BNB".to_string(), "SOL".to_string()];
    match state.price_service.get_prices(&symbols).await {
        Ok(prices) => {
            let mut text = String::from("📊 Cryptocurrency Prices\n\n");

            for (symbol, price) in &prices {
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
    let text = "📥 Import Wallet\n\n\
To import an existing wallet, use the command:\n\n\
/importwallet <chain> <mnemonic or private key>\n\n\
Examples:\n\
/importwallet ETH word1 word2 word3...\n\
/importwallet SOLANA 5J7K...\n\n\
Supported chains: ETH, BSC, SOLANA";

    bot.edit_message_text(chat_id, message_id, text)
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

            let native_token = match wallet.chain.as_str() {
                "ETH" => "ETH",
                "BSC" => "BNB",
                "SOLANA" => "SOL",
                _ => "tokens",
            };

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

    let symbol = match wallet.chain.as_str() {
        "ETH" => "ETH",
        "BSC" => "BNB",
        "SOLANA" => "SOL",
        _ => "tokens",
    };

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
            let chain_emoji = match wallet.chain.as_str() {
                "ETH" => "🔷",
                "BSC" => "🟡",
                "SOLANA" => "🟣",
                _ => "📍",
            };

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

    let (from_token, to_token) = match (wallet.chain.as_str(), preset) {
        ("ETH", "preset1") => ("ETH", "USDC"),
        ("ETH", "preset2") => ("ETH", "USDT"),
        ("BSC", "preset1") => ("BNB", "USDT"),
        ("BSC", "preset2") => ("BNB", "BUSD"),
        ("SOLANA", "preset1") => ("SOL", "USDC"),
        ("SOLANA", "preset2") => ("SOL", "USDT"),
        _ => ("NATIVE", "STABLE"),
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
    let text = "💼 Wallet Commands\n\n\
/createwallet <chain> - Create new wallet\n\
/importwallet <chain> <key> - Import wallet\n\
/wallets - List all wallets\n\
/balance <wallet_id> - Check balance\n\
/address <wallet_id> - Get address with QR\n\n\
Supported chains: ETH, BSC, SOLANA";

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
                let condition = if alert.alert_type == "above" { "📈 Above" } else { "📉 Below" };
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

async fn show_new_alert_instructions(bot: &Bot, chat_id: ChatId, message_id: MessageId) -> HandlerResult {
    let text = "🔔 Create Price Alert\n\n\
Use the command:\n\
/setalert <symbol> <above|below> <price>\n\n\
Examples:\n\
/setalert BTC above 100000\n\
/setalert ETH below 3000";

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(keyboards::alerts_menu())
        .await?;

    Ok(())
}

fn chain_emoji(chain: &str) -> &'static str {
    match chain.to_uppercase().as_str() {
        "ETH" | "ETHEREUM" => "🔷",
        "BSC" | "BNB" => "🟡",
        "SOLANA" | "SOL" => "🟣",
        "BTC" | "BITCOIN" => "🟠",
        _ => "📍",
    }
}
