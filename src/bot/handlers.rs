use teloxide::prelude::*;
use teloxide::types::ParseMode;
use crate::bot::{ BotState, commands::Command, keyboards };
use super::constants::{ messages as msg, chains };
use crate::services::*;
use crate::services::scheduling_service::{ SchedulingService, ScheduleRequest };
use crate::services::price_alert_service;
use uuid::Uuid;
use std::sync::Arc;

// Handler for dispatcher-based command handling
pub async fn handle_command_dispatch(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<BotState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    handle_command(bot, msg, cmd, state).await?;
    Ok(())
}

// Helper function to format numbers with thousand separators
fn format_currency(value: f64) -> String {
    let formatted = format!("{:.2}", value);
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_part = parts[0];
    let dec_part = if parts.len() > 1 { parts[1] } else { "00" };

    let mut result = String::new();
    let chars: Vec<char> = int_part.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    format!("{}.{}", result, dec_part)
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let user_id = chat_id.0.to_string();

    match cmd {
        Command::Start => handle_start(bot, msg).await,
        Command::Help => handle_help(bot, msg).await,
        Command::CreateWallet(args) => handle_create_wallet(bot, msg, args, user_id, state).await,
        Command::ImportWallet(args) => handle_import_wallet(bot, msg, args, user_id, state).await,
        Command::Wallets => handle_list_wallets(bot, msg, user_id, state).await,
        Command::Balance(args) => handle_balance(bot, msg, args, user_id, state).await,
        Command::Send(args) => handle_send(bot, msg, args, user_id, state).await,
        Command::EstimateFee(args) => handle_estimate_fee(bot, msg, args, user_id, state).await,
        Command::BatchSend(args) => handle_batch_send(bot, msg, args, user_id, state).await,
        Command::History(args) => handle_history(bot, msg, args, user_id, state).await,
        Command::Address(args) => handle_address(bot, msg, args, user_id, state).await,
        Command::Portfolio => handle_portfolio(bot, msg, user_id, state).await,
        Command::Prices => handle_prices(bot, msg, state).await,
        Command::SaveAddress(args) => handle_save_address(bot, msg, args, user_id, state).await,
        Command::Addresses => handle_list_addresses(bot, msg, user_id, state).await,
        Command::DeleteAddress(args) => handle_delete_address(bot, msg, args, user_id, state).await,
        Command::Schedule(args) => handle_schedule(bot, msg, args, user_id, state).await,
        Command::Scheduled => handle_list_scheduled(bot, msg, user_id, state).await,
        Command::CancelSchedule(args) =>
            handle_cancel_schedule(bot, msg, args, user_id, state).await,
        Command::SetAlert(args) => handle_set_alert(bot, msg, args, user_id, state).await,
        Command::Alerts => handle_list_alerts(bot, msg, user_id, state).await,
        Command::DeleteAlert(args) => handle_delete_alert(bot, msg, args, user_id, state).await,
        Command::SetPin(args) => handle_set_pin(bot, msg, args, user_id, state).await,
        Command::ChangePin(args) => handle_change_pin(bot, msg, args, user_id, state).await,
        Command::DisablePin => handle_disable_pin(bot, msg, user_id, state).await,
        Command::SetLimit(args) => handle_set_limit(bot, msg, args, user_id, state).await,
        Command::LockWallet => handle_lock_wallet(bot, msg, user_id, state).await,
        Command::UnlockWallet(args) => handle_unlock_wallet(bot, msg, args, user_id, state).await,
        Command::Security => handle_security_info(bot, msg, user_id, state).await,
        Command::Swap(args) => handle_swap(bot, msg, args, user_id, state).await,
        Command::SwapQuote(args) => handle_swap_quote(bot, msg, args, state).await,
        Command::SwapHistory(args) => handle_swap_history(bot, msg, args, user_id, state).await,
    }
}

async fn handle_start(bot: Bot, msg: Message) -> ResponseResult<()> {
    let welcome = r#"🔐 *Welcome to Crypto Wallet Bot\!*

Your secure multi\-chain wallet manager\.

*Supported Blockchains:*
🔷 Ethereum \(ETH\)
🟡 Binance Smart Chain \(BSC\)
🟣 Solana \(SOL\)

*What I can do:*
• Create and manage wallets
• Send and receive crypto
• Track your portfolio
• Set price alerts
• Swap tokens

Select an option below to get started:"#;

    bot.send_message(msg.chat.id, welcome)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(keyboards::main_menu())
        .await?;
    Ok(())
}

async fn handle_help(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = "❓ *Help Center*\n\nSelect a category to learn more about available commands:";

    bot.send_message(msg.chat.id, help_text)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(keyboards::help_menu())
        .await?;
    Ok(())
}

async fn handle_create_wallet(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let chain = args.trim().to_uppercase();

    if chain.is_empty() {
        bot.send_message(msg.chat.id, msg::ERR_CHAIN_REQUIRED).await?;
        return Ok(());
    }

    // Validate chain
    if !chains::is_valid_chain(&chain) {
        bot.send_message(msg.chat.id, msg::ERR_INVALID_CHAIN).await?;
        return Ok(());
    }

    let normalized_chain = chains::normalize_chain(&chain);

    bot.send_message(msg.chat.id, msg::STATUS_CREATING_WALLET).await?;

    match
        state.wallet_service.generate_wallet(user_id, normalized_chain.to_string(), Some(0)).await
    {
        Ok(response) => {
            let safe_msg = format!(
                "{}

📍 Chain: `{}`
🆔 Wallet ID: `{}`
📬 Address: `{}`

🔑 *SAVE YOUR MNEMONIC SECURELY:*
`{}`

⚠️ *IMPORTANT:* Never share your mnemonic\\. I will send it once\\. Save it now\\!",
                msg::SUCCESS_WALLET_CREATED,
                escape_markdown(&response.chain),
                escape_markdown(&response.id.to_string()),
                escape_markdown(&response.address),
                escape_markdown(&response.mnemonic.unwrap_or_default())
            );

            bot.send_message(msg.chat.id, safe_msg).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            tracing::error!("Failed to create wallet: {:?}", e);
            bot.send_message(msg.chat.id, format!("❌ Failed to create wallet: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_import_wallet(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let parts: Vec<&str> = args.trim().split_whitespace().collect();

    if parts.len() < 2 {
        bot.send_message(msg.chat.id, msg::ERR_IMPORT_USAGE).await?;
        return Ok(());
    }

    let chain = parts[0].to_uppercase();
    let key = parts[1..].join(" ");

    if !chains::is_valid_chain(&chain) {
        bot.send_message(msg.chat.id, msg::ERR_INVALID_CHAIN).await?;
        return Ok(());
    }

    let normalized_chain = chains::normalize_chain(&chain);

    bot.send_message(msg.chat.id, msg::STATUS_IMPORTING_WALLET).await?;

    match
        state.wallet_service.restore_wallet(
            user_id,
            normalized_chain.to_string(),
            key,
            Some(0)
        ).await
    {
        Ok(response) => {
            let safe_msg = format!(
                "{}

📍 Chain: `{}`
🆔 Wallet ID: `{}`
📬 Address: `{}`",
                msg::SUCCESS_WALLET_IMPORTED,
                escape_markdown(&response.chain),
                escape_markdown(&response.id.to_string()),
                escape_markdown(&response.address)
            );

            bot.send_message(msg.chat.id, safe_msg).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            tracing::error!("Failed to import wallet: {:?}", e);
            bot.send_message(msg.chat.id, format!("❌ Failed to import wallet: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_list_wallets(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    match state.wallet_service.list_user_wallets(&user_id, None).await {
        Ok(wallets) => {
            if wallets.is_empty() {
                bot.send_message(msg.chat.id, msg::INFO_NO_WALLETS).await?;
                return Ok(());
            }

            let mut response = String::from(msg::HEADER_YOUR_WALLETS);

            for wallet in wallets {
                response.push_str(
                    &format!(
                        "🔸 *{}*\n🆔 ID: `{}`\n📬 Address: `{}`\n📅 Created: {}\n\n",
                        escape_markdown(&wallet.chain),
                        escape_markdown(&wallet.id.to_string()),
                        escape_markdown(&wallet.address),
                        escape_markdown(&format!("{}", wallet.created_at.format("%Y-%m-%d %H:%M")))
                    )
                );
            }

            bot.send_message(msg.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            tracing::error!("Failed to list wallets: {:?}", e);
            bot.send_message(msg.chat.id, format!("❌ Failed to list wallets: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_balance(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let parts: Vec<&str> = args.trim().split_whitespace().collect();

    if parts.is_empty() {
        bot.send_message(msg.chat.id, msg::ERR_BALANCE_USAGE).await?;
        return Ok(());
    }

    let wallet_id = match Uuid::parse_str(parts[0]) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, msg::ERR_INVALID_WALLET_ID).await?;
            return Ok(());
        }
    };

    let token_address = parts.get(1).map(|s| s.to_string());

    bot.send_message(msg.chat.id, msg::STATUS_FETCHING_BALANCE).await?;

    match state.balance_service.get_balance(wallet_id, token_address).await {
        Ok(balance) => {
            let msg_text = format!(
                "💰 *Balance*\n\n💵 Symbol: *{}*\n💎 Amount: `{}`",
                escape_markdown(&balance.symbol),
                escape_markdown(&balance.balance)
            );

            bot.send_message(msg.chat.id, msg_text).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            tracing::error!("Failed to get balance: {:?}", e);
            bot.send_message(msg.chat.id, format!("❌ Failed to get balance: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_send(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let parts: Vec<&str> = args.trim().split_whitespace().collect();

    if parts.len() < 3 {
        bot.send_message(msg.chat.id, msg::ERR_SEND_USAGE).await?;
        return Ok(());
    }

    let wallet_id = match Uuid::parse_str(parts[0]) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, msg::ERR_INVALID_WALLET_ID).await?;
            return Ok(());
        }
    };

    let to_input = parts[1].to_string();
    let amount = parts[2].to_string();
    let token_address = parts.get(3).map(|s| s.to_string());

    // Check if to_input is an address or a saved name
    let to_address = if to_input.starts_with("0x") || to_input.len() > 40 {
        // Looks like an address
        to_input
    } else {
        // Try to find it in address book
        match state.address_book_service.get_address(&user_id, &to_input).await {
            Ok(saved_addr) => {
                bot.send_message(
                    msg.chat.id,
                    format!("📖 Using saved address: {} ({})", saved_addr.name, saved_addr.address)
                ).await?;
                saved_addr.address
            }
            Err(_) => {
                // Not in address book, treat as address
                to_input
            }
        }
    };

    bot.send_message(msg.chat.id, msg::STATUS_SENDING_TX).await?;

    let request = transfer_service::TransferRequest {
        to: to_address,
        amount,
        token_address,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        gas_limit: None,
        compute_units: None,
    };

    match state.transfer_service.send_transaction(wallet_id, request).await {
        Ok(response) => {
            let msg_text = format!(
                "✅ *Transaction Sent\\!*\n\n🔗 Hash: `{}`\n📊 Status: `{}`",
                escape_markdown(&response.tx_hash),
                escape_markdown(&response.status)
            );

            bot.send_message(msg.chat.id, msg_text).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Transaction failed: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_estimate_fee(
    bot: Bot,
    msg: Message,
    args: String,
    _user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let parts: Vec<&str> = args.trim().split_whitespace().collect();

    if parts.len() < 3 {
        bot.send_message(
            msg.chat.id,
            "Usage: /estimatefee <wallet_id> <to_address> <amount> [token_address]\n\n\
            Example: /estimatefee abc123 0x742d35Cc... 0.1"
        ).await?;
        return Ok(());
    }

    let wallet_id = match Uuid::parse_str(parts[0]) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid wallet ID format").await?;
            return Ok(());
        }
    };

    let to_address = parts[1];
    let amount = parts[2];
    let token_address = parts.get(3).copied();

    bot.send_message(msg.chat.id, "⏳ Estimating transaction fee...").await?;

    match
        state.gas_estimation_service.estimate_transaction_fee(
            wallet_id,
            to_address,
            amount,
            token_address
        ).await
    {
        Ok(estimate) => {
            let mut response = format!(
                "⛽ *Gas Estimation*\n\n\
                ⛓️ Chain: {}\n\
                📊 Gas Limit: {}\n",
                escape_markdown(&estimate.chain),
                escape_markdown(&estimate.gas_estimate.estimated_gas.to_string())
            );

            if let Some(gas_price) = &estimate.gas_estimate.gas_price {
                response.push_str(&format!("💵 Gas Price: {}\n", escape_markdown(gas_price)));
            }

            if let Some(max_fee) = &estimate.gas_estimate.max_fee_per_gas {
                response.push_str(&format!("🔝 Max Fee: {} gwei\n", escape_markdown(max_fee)));
            }

            if let Some(priority_fee) = &estimate.gas_estimate.max_priority_fee_per_gas {
                response.push_str(
                    &format!("⚡ Priority Fee: {} gwei\n", escape_markdown(priority_fee))
                );
            }

            response.push_str(
                &format!(
                    "\n💰 *Total Cost:* {} {}\n",
                    escape_markdown(&estimate.gas_estimate.total_cost_native),
                    escape_markdown(&estimate.chain)
                )
            );

            if let Some(usd_cost) = estimate.gas_estimate.total_cost_usd {
                response.push_str(&format!("💵 \\~${:.4} USD\n", usd_cost));
            }

            response.push_str("\n💡 _This is an estimate\\. Actual cost may vary\\._");

            bot.send_message(msg.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed to estimate fee: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_batch_send(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    // Parse wallet_id from first line, then CSV data from remaining lines
    let lines: Vec<&str> = args.trim().lines().collect();

    if lines.is_empty() {
        bot.send_message(
            msg.chat.id,
            "Usage:\n\
            `/batchsend <wallet_id>`\n\
            `address1,amount1`\n\
            `address2,amount2`\n\n\
            Example:\n\
            `/batchsend abc123-def456-...`\n\
            `0x742d35Cc...,0.1`\n\
            `alice,0.2`\n\
            `0x1234567...,0.15`"
        ).await?;
        return Ok(());
    }

    let wallet_id = match Uuid::parse_str(lines[0].trim()) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid wallet ID format").await?;
            return Ok(());
        }
    };

    if lines.len() < 2 {
        bot.send_message(
            msg.chat.id,
            "❌ No recipients provided. Add CSV lines:\n\
            `address,amount` (one per line)"
        ).await?;
        return Ok(());
    }

    // Parse CSV lines into recipients
    let mut recipients = Vec::new();
    for (i, line) in lines[1..].iter().enumerate() {
        let parts: Vec<&str> = line
            .split(',')
            .map(|s| s.trim())
            .collect();
        if parts.len() < 2 {
            bot.send_message(
                msg.chat.id,
                format!("❌ Invalid CSV format on line {}: {}", i + 2, line)
            ).await?;
            return Ok(());
        }

        let mut to = parts[0].to_string();
        let amount = parts[1].to_string();
        let token_address = parts.get(2).map(|s| s.to_string());

        // Check if 'to' is a saved address name
        if !to.starts_with("0x") && !to.len() > 32 {
            // Might be a saved address name, try to resolve it
            match state.address_book_service.get_address(&user_id, &to).await {
                Ok(saved_addr) => {
                    to = saved_addr.address;
                }
                Err(_) => {
                    // Not found, treat as regular address (will be validated later)
                }
            }
        }

        recipients.push(transfer_service::BatchRecipient {
            to,
            amount,
            token_address,
        });
    }

    bot.send_message(
        msg.chat.id,
        format!("⏳ Processing batch transfer for {} recipients...", recipients.len())
    ).await?;

    match state.transfer_service.send_batch_transactions(wallet_id, recipients).await {
        Ok(result) => {
            let mut response = format!(
                "📊 *Batch Transfer Complete*\n\n\
                ✅ Successful: {}/{}\n\
                ❌ Failed: {}\n\n\
                *Details:*\n",
                result.successful,
                result.total,
                result.failed
            );

            for status in result.results.iter().take(10) {
                let status_icon = if status.status == "pending" || status.status == "confirmed" {
                    "✅"
                } else {
                    "❌"
                };

                if let Some(ref tx_hash) = status.tx_hash {
                    response.push_str(
                        &format!(
                            "{} {} {} → `{}`\n",
                            status_icon,
                            escape_markdown(&status.amount),
                            escape_markdown(&status.to[..8]),
                            escape_markdown(&tx_hash[..16])
                        )
                    );
                } else if let Some(ref error) = status.error {
                    response.push_str(
                        &format!(
                            "{} {} {} → Error: {}\n",
                            status_icon,
                            escape_markdown(&status.amount),
                            escape_markdown(&status.to[..8]),
                            escape_markdown(error)
                        )
                    );
                }
            }

            if result.results.len() > 10 {
                response.push_str(
                    &format!("\n_\\.\\.\\. and {} more_\n", result.results.len() - 10)
                );
            }

            bot.send_message(msg.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Batch transfer failed: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_history(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let parts: Vec<&str> = args.trim().split_whitespace().collect();

    if parts.is_empty() {
        bot.send_message(msg.chat.id, "❌ Usage: /history <wallet_id> [limit]").await?;
        return Ok(());
    }

    let wallet_id = match Uuid::parse_str(parts[0]) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid wallet ID format").await?;
            return Ok(());
        }
    };

    let limit = parts
        .get(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(10);

    bot.send_message(msg.chat.id, "⏳ Fetching transaction history...").await?;

    match state.transaction_service.get_wallet_transactions(wallet_id, Some(limit), Some(0)).await {
        Ok(transactions) => {
            if transactions.is_empty() {
                bot.send_message(msg.chat.id, "📭 No transactions found for this wallet.").await?;
                return Ok(());
            }

            let mut response = format!(
                "*Transaction History* \\(Last {}\\)\n\n",
                transactions.len()
            );

            for tx in transactions.iter().take(10) {
                response.push_str(
                    &format!(
                        "🔸 `{}`\n\
                      📊 Status: {}\n\
                      💎 Amount: {} {}\n\
                      📅 {}\n\n",
                        escape_markdown(&tx.tx_hash[..16]),
                        escape_markdown(&tx.status),
                        escape_markdown(&tx.amount),
                        escape_markdown(tx.token_symbol.as_deref().unwrap_or("UNKNOWN")),
                        escape_markdown(&format!("{}", tx.created_at.format("%Y-%m-%d %H:%M")))
                    )
                );
            }

            bot.send_message(msg.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed to fetch history: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_address(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let wallet_id = match Uuid::parse_str(args.trim()) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                "❌ Invalid wallet ID format\nUsage: /address <wallet_id>"
            ).await?;
            return Ok(());
        }
    };

    match state.wallet_service.list_user_wallets(&user_id, None).await {
        Ok(wallets) => {
            if let Some(wallet) = wallets.iter().find(|w| w.id == wallet_id) {
                let msg_text = format!(
                    "📬 *Wallet Address*\n\n\
                    📍 Chain: `{}`\n\
                    🆔 ID: `{}`\n\
                    📬 Address:\n`{}`\n\n\
                    You can use this address to receive {} tokens\\.",
                    escape_markdown(&wallet.chain),
                    escape_markdown(&wallet.id.to_string()),
                    escape_markdown(&wallet.address),
                    escape_markdown(&wallet.chain)
                );

                bot.send_message(msg.chat.id, msg_text).parse_mode(ParseMode::MarkdownV2).await?;
            } else {
                bot.send_message(msg.chat.id, "❌ Wallet not found").await?;
            }
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed to get wallet: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_portfolio(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "⏳ Fetching your portfolio...").await?;

    match state.portfolio_service.get_portfolio(&user_id).await {
        Ok(portfolio) => {
            if portfolio.holdings.is_empty() {
                bot.send_message(
                    msg.chat.id,
                    "📭 Your portfolio is empty.\n\nCreate a wallet with: /createwallet <chain>"
                ).await?;
                return Ok(());
            }

            let mut response = String::from("💼 *Your Portfolio*\n\n");

            for holding in &portfolio.holdings {
                let change_emoji = match holding.price_change_24h {
                    Some(change) if change > 0.0 => "📈",
                    Some(change) if change < 0.0 => "📉",
                    _ => "➖",
                };

                let change_text = if let Some(change) = holding.price_change_24h {
                    format!(" \\({}{:.2}%\\)", if change > 0.0 { "+" } else { "" }, change)
                } else {
                    String::new()
                };

                response.push_str(
                    &format!(
                        "*{}:* {} {} \\(${}\\) {}{}\n",
                        escape_markdown(&holding.symbol),
                        escape_markdown(
                            &format!("{:.6}", holding.total_balance)
                                .trim_end_matches('0')
                                .trim_end_matches('.')
                        ),
                        escape_markdown(&holding.symbol),
                        format_currency(holding.usd_value),
                        change_emoji,
                        change_text
                    )
                );

                response.push_str(
                    &format!(
                        "  💵 ${} per {}\n",
                        format_currency(holding.usd_price),
                        escape_markdown(&holding.symbol)
                    )
                );

                if holding.wallets.len() > 1 {
                    response.push_str(&format!("  📦 {} wallets\n", holding.wallets.len()));
                }

                response.push_str("\n");
            }

            response.push_str(
                &format!(
                    "━━━━━━━━━━━━━━━━\n\
                💰 *Total Value:* ${}\n\
                📊 {} chains \\| {} wallets",
                    format_currency(portfolio.total_usd_value),
                    portfolio.chains.len(),
                    portfolio.wallet_count
                )
            );

            bot.send_message(msg.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed to fetch portfolio: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_prices(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "⏳ Fetching prices...").await?;

    let symbols = vec!["ETH".to_string(), "BNB".to_string(), "SOL".to_string()];

    match state.price_service.get_prices(&symbols).await {
        Ok(prices) => {
            let mut response = String::from("💵 *Cryptocurrency Prices*\n\n");

            for symbol in &["ETH", "BNB", "SOL"] {
                if let Some(price) = prices.get(*symbol) {
                    let change_emoji = match price.price_change_24h {
                        Some(change) if change > 0.0 => "📈",
                        Some(change) if change < 0.0 => "📉",
                        _ => "➖",
                    };

                    let change_text = if let Some(change) = price.price_change_24h {
                        format!(" \\({}{:.2}%\\)", if change > 0.0 { "+" } else { "" }, change)
                    } else {
                        String::new()
                    };

                    response.push_str(
                        &format!(
                            "*{}:* ${} {}{}\n",
                            escape_markdown(symbol),
                            format_currency(price.usd_price),
                            change_emoji,
                            change_text
                        )
                    );

                    if let Some(market_cap) = price.market_cap {
                        response.push_str(
                            &format!(
                                "  📊 Cap: ${}B\n",
                                escape_markdown(&format!("{:.1}", market_cap / 1_000_000_000.0))
                            )
                        );
                    }

                    if let Some(volume) = price.volume_24h {
                        response.push_str(
                            &format!(
                                "  📈 Vol: ${}M\n",
                                escape_markdown(&format!("{:.1}", volume / 1_000_000.0))
                            )
                        );
                    }

                    response.push_str("\n");
                }
            }

            response.push_str("_Updated just now_");

            bot.send_message(msg.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed to fetch prices: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_save_address(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let parts: Vec<&str> = args.trim().split_whitespace().collect();

    if parts.len() < 3 {
        bot.send_message(
            msg.chat.id,
            "Usage: /saveaddress <name> <address> <chain> [notes]\n\
            Example: /saveaddress alice 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb ETH"
        ).await?;
        return Ok(());
    }

    let name = parts[0].to_string();
    let address = parts[1].to_string();
    let chain = parts[2].to_uppercase();
    let notes = if parts.len() > 3 { Some(parts[3..].join(" ")) } else { None };

    // Validate chain
    if !["ETH", "BSC", "SOLANA"].contains(&chain.as_str()) {
        bot.send_message(msg.chat.id, "Invalid chain. Supported chains: ETH, BSC, SOLANA").await?;
        return Ok(());
    }

    match
        state.address_book_service.save_address(
            user_id,
            name.clone(),
            address.clone(),
            chain.clone(),
            notes
        ).await
    {
        Ok(_) => {
            let response = format!(
                "✅ Address saved successfully!\n\n\
                Name: {}\n\
                Address: {}\n\
                Chain: {}\n\n\
                Use /addresses to view all saved addresses",
                name,
                address,
                chain
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_list_addresses(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    match state.address_book_service.list_addresses(&user_id, None).await {
        Ok(addresses) => {
            if addresses.is_empty() {
                bot.send_message(
                    msg.chat.id,
                    "📪 You don't have any saved addresses yet.\n\n\
                    Use /saveaddress to add one!"
                ).await?;
                return Ok(());
            }

            let mut response = String::from("📖 *Your Address Book*\n\n");

            for addr in addresses {
                let notes_text = addr.notes
                    .as_ref()
                    .map(|n| format!("\n  📝 {}", escape_markdown(n)))
                    .unwrap_or_default();

                response.push_str(
                    &format!(
                        "*{}*\n\
                    📍 {}\n\
                    ⛓️ {}{}\n\n",
                        escape_markdown(&addr.name),
                        escape_markdown(&addr.address),
                        escape_markdown(&addr.chain),
                        notes_text
                    )
                );
            }

            response.push_str(
                "💡 Use `/send <wallet_id> <name> <amount>` to send to saved addresses"
            );

            bot.send_message(msg.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_delete_address(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let name = args.trim();

    if name.is_empty() {
        bot.send_message(
            msg.chat.id,
            "Usage: /deleteaddress <name>\n\
            Example: /deleteaddress alice"
        ).await?;
        return Ok(());
    }

    match state.address_book_service.delete_address(&user_id, name).await {
        Ok(_) => {
            bot.send_message(
                msg.chat.id,
                format!("✅ Address '{}' deleted successfully!", name)
            ).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

// Helper function to escape markdown special characters
fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

async fn handle_schedule(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    // Parse: /schedule <wallet_id> <to> <amount> <datetime> [token] [recurring]
    // datetime format: "2024-01-15 14:30" or "2024-01-15T14:30:00"
    // recurring: "daily" | "weekly" | "monthly"

    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 4 {
        bot.send_message(
            msg.chat.id,
            "❌ Usage: /schedule <wallet_id> <to_address> <amount> <datetime> [token_address] [recurring]\n\n\
            Examples:\n\
            • /schedule abc123 0x742d... 1.5 \"2024-01-15 14:30\"\n\
            • /schedule abc123 Alice 100 \"2024-01-15 14:30\" daily\n\
            • /schedule abc123 0x742d... 50 \"2024-01-15T14:30:00\" 0xdac... weekly"
        ).await?;
        return Ok(());
    }

    let wallet_id = match Uuid::parse_str(parts[0]) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid wallet ID format").await?;
            return Ok(());
        }
    };

    let mut to_address = parts[1].to_string();

    // Check if it's an address book name
    if !to_address.starts_with("0x") && !to_address.contains('.') {
        if let Ok(saved) = state.address_book_service.get_address(&user_id, &to_address).await {
            to_address = saved.address;
        }
    }

    let amount = parts[2].to_string();

    // Parse datetime (parts[3] and possibly parts[4] if it has space)
    let datetime_str = if parts[3].contains('T') {
        parts[3].to_string()
    } else if parts.len() > 4 && (parts[4].contains(':') || parts[4].parse::<u8>().is_ok()) {
        format!("{} {}", parts[3], parts[4])
    } else {
        parts[3].to_string()
    };

    let scheduled_for = match
        chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M")
    {
        Ok(dt) => chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc),
        Err(_) =>
            match chrono::DateTime::parse_from_rfc3339(&datetime_str) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(_) => {
                    bot.send_message(
                        msg.chat.id,
                        "❌ Invalid datetime format. Use: YYYY-MM-DD HH:MM or ISO8601"
                    ).await?;
                    return Ok(());
                }
            }
    };

    // Check if datetime is in the future
    if scheduled_for <= chrono::Utc::now() {
        bot.send_message(msg.chat.id, "❌ Scheduled time must be in the future").await?;
        return Ok(());
    }

    // Parse optional token and recurring type
    let remaining_parts: Vec<&str> = if datetime_str.contains(' ') {
        parts[5..].to_vec()
    } else {
        parts[4..].to_vec()
    };

    let mut token_address = None;
    let mut recurring_type = None;

    for part in remaining_parts {
        if part.starts_with("0x") || part.starts_with("0X") {
            token_address = Some(part.to_string());
        } else if ["daily", "weekly", "monthly"].contains(&part.to_lowercase().as_str()) {
            recurring_type = Some(part.to_lowercase());
        }
    }

    // Verify wallet ownership
    match state.wallet_service.get_wallet(wallet_id).await {
        Ok(wallet) => {
            if wallet.user_id != user_id {
                bot.send_message(msg.chat.id, "❌ Wallet not found").await?;
                return Ok(());
            }
        }
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Wallet not found").await?;
            return Ok(());
        }
    }

    // Schedule the transaction
    let schedule_req = ScheduleRequest {
        user_id,
        wallet_id,
        to_address,
        amount,
        token_address,
        scheduled_for,
        recurring_type: recurring_type.clone(),
    };

    match state.scheduling_service.schedule_transaction(schedule_req).await {
        Ok(schedule) => {
            let recurring_text = if let Some(rec) = recurring_type {
                format!(" \\({}\\)", escape_markdown(&rec))
            } else {
                String::new()
            };

            bot
                .send_message(
                    msg.chat.id,
                    format!(
                        "✅ *Transaction Scheduled*\n\n\
                    📅 Schedule ID: `{}`\n\
                    ⏰ Scheduled for: {}{}\n\
                    💸 Amount: {}\n\
                    📍 To: `{}`\n\n\
                    The transaction will be executed automatically at the scheduled time\\.",
                        escape_markdown(&schedule.id.to_string()),
                        escape_markdown(&scheduled_for.format("%Y-%m-%d %H:%M UTC").to_string()),
                        recurring_text,
                        escape_markdown(&schedule.amount),
                        escape_markdown(&schedule.to_address)
                    )
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed to schedule: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_list_scheduled(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    match state.scheduling_service.list_scheduled(&user_id, Some("pending")).await {
        Ok(schedules) => {
            if schedules.is_empty() {
                bot
                    .send_message(msg.chat.id, "📅 No scheduled transactions\\.")
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
                return Ok(());
            }

            let mut response = String::from("📅 *Scheduled Transactions*\n\n");

            for schedule in schedules {
                let recurring = if let Some(rec) = schedule.recurring_type {
                    format!(" \\({}\\)", escape_markdown(&rec))
                } else {
                    String::new()
                };

                response.push_str(
                    &format!(
                        "• ID: `{}`\n\
                     ⏰ {}{}\n\
                     💸 {} → `{}`\n\
                     🆔 Wallet: `{}`\n\n",
                        escape_markdown(&schedule.id.to_string()),
                        escape_markdown(
                            &schedule.scheduled_for.format("%Y-%m-%d %H:%M UTC").to_string()
                        ),
                        recurring,
                        escape_markdown(&schedule.amount),
                        escape_markdown(&schedule.to_address),
                        escape_markdown(&schedule.wallet_id.to_string())
                    )
                );
            }

            response.push_str("Use /cancelschedule <id> to cancel\\.");

            bot
                .send_message(msg.chat.id, response)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_cancel_schedule(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let schedule_id = match Uuid::parse_str(args.trim()) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid schedule ID format").await?;
            return Ok(());
        }
    };

    match state.scheduling_service.cancel_schedule(schedule_id, &user_id).await {
        Ok(_) => {
            bot
                .send_message(msg.chat.id, "✅ *Scheduled transaction cancelled*")
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

// ==================== PHASE 7: PRICE ALERT HANDLERS ====================

async fn handle_set_alert(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    // Parse: /setalert <symbol> <above|below|percent> <value> [chain]
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 3 {
        bot.send_message(
            msg.chat.id,
            "❌ Usage: /setalert <symbol> <above|below|percent> <value> [chain]\n\n\
            Examples:\n\
            • /setalert ETH above 3000\n\
            • /setalert BTC below 40000\n\
            • /setalert SOL percent 10 SOLANA"
        ).await?;
        return Ok(());
    }

    let symbol = parts[0].to_uppercase();
    let alert_type_str = parts[1].to_lowercase();
    let value_str = parts[2];
    let chain = if parts.len() > 3 { parts[3].to_uppercase() } else { "ETH".to_string() };

    let value: f64 = match value_str.parse() {
        Ok(v) => v,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid value format").await?;
            return Ok(());
        }
    };

    // Get current price for percent alerts
    let alert_type = match alert_type_str.as_str() {
        "above" => price_alert_service::AlertType::Above { target_price: value },
        "below" => price_alert_service::AlertType::Below { target_price: value },
        "percent" => {
            match state.price_service.get_price(&symbol).await {
                Ok(current_price) => {
                    price_alert_service::AlertType::PercentChange {
                        percent: value,
                        base_price: current_price.usd_price,
                    }
                }
                Err(e) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("❌ Could not get current price: {}", e)
                    ).await?;
                    return Ok(());
                }
            }
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "❌ Alert type must be 'above', 'below', or 'percent'"
            ).await?;
            return Ok(());
        }
    };

    let request = price_alert_service::CreateAlertRequest {
        user_id: user_id.clone(),
        token_symbol: symbol.clone(),
        chain: chain.clone(),
        token_address: None,
        alert_type,
    };

    match state.price_alert_service.create_alert(request).await {
        Ok(_) => {
            let msg_text = format!(
                "✅ *Alert Created*\n\n\
                Symbol: {}\n\
                Chain: {}\n\
                Type: {}\n\n\
                You'll be notified when triggered\\.",
                escape_markdown(&symbol),
                escape_markdown(&chain),
                escape_markdown(&alert_type_str)
            );
            bot
                .send_message(msg.chat.id, msg_text)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_list_alerts(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    match state.price_alert_service.list_user_alerts(&user_id, true).await {
        Ok(alerts) => {
            if alerts.is_empty() {
                bot
                    .send_message(
                        msg.chat.id,
                        "📭 You have no active alerts\\.\\n\\nUse /setalert to create one\\."
                    )
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
            } else {
                let mut response = String::from("*📊 Your Active Alerts*\n\n");

                for alert in alerts {
                    let alert_desc = match alert.alert_type.as_str() {
                        "above" => {
                            if let Some(price) = alert.target_price {
                                format!("Above ${:.2}", price)
                            } else {
                                "Above (price not set)".to_string()
                            }
                        }
                        "below" => {
                            if let Some(price) = alert.target_price {
                                format!("Below ${:.2}", price)
                            } else {
                                "Below (price not set)".to_string()
                            }
                        }
                        "percent_change" => {
                            let percent = alert.percent_change
                                .map(|p| format!("{:.2}", p))
                                .unwrap_or("0".to_string());
                            let base = alert.base_price
                                .map(|p| format!("{:.2}", p))
                                .unwrap_or("0".to_string());
                            format!("{}% change from ${}", percent, base)
                        }
                        _ => "Unknown alert type".to_string(),
                    };

                    response.push_str(
                        &format!(
                            "🔔 *{}* \\({}\\)\n\
                        └ {}\n\
                        └ ID: `{}`\n\n",
                            escape_markdown(&alert.token_symbol),
                            escape_markdown(&alert.chain),
                            escape_markdown(&alert_desc),
                            escape_markdown(&alert.id.to_string())
                        )
                    );
                }

                response.push_str("Use /deletealert <id> to remove an alert\\.");

                bot
                    .send_message(msg.chat.id, response)
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
            }
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_delete_alert(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let alert_id = match Uuid::parse_str(args.trim()) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid alert ID format").await?;
            return Ok(());
        }
    };

    match state.price_alert_service.delete_alert(alert_id, &user_id).await {
        Ok(_) => {
            bot
                .send_message(msg.chat.id, "✅ *Alert deleted successfully*")
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

// ==================== PHASE 8: SECURITY HANDLERS ====================

async fn handle_set_pin(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let pin = args.trim();

    // Validate PIN format (6 digits)
    if pin.len() != 6 || !pin.chars().all(|c| c.is_ascii_digit()) {
        bot.send_message(msg.chat.id, "❌ PIN must be exactly 6 digits").await?;
        return Ok(());
    }

    match state.security_service.set_pin(&user_id, pin).await {
        Ok(_) => {
            bot
                .send_message(
                    msg.chat.id,
                    "✅ *PIN Set Successfully*\n\n\
                Your PIN will be required for:\n\
                • Large transfers\n\
                • Changing security settings\n\
                • Unlocking wallet"
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_change_pin(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() != 2 {
        bot.send_message(msg.chat.id, "❌ Usage: /changepin <old_pin> <new_pin>").await?;
        return Ok(());
    }

    let old_pin = parts[0];
    let new_pin = parts[1];

    // Validate new PIN format
    if new_pin.len() != 6 || !new_pin.chars().all(|c| c.is_ascii_digit()) {
        bot.send_message(msg.chat.id, "❌ New PIN must be exactly 6 digits").await?;
        return Ok(());
    }

    // Verify old PIN
    match state.security_service.verify_pin(&user_id, old_pin).await {
        Ok(true) => {
            match state.security_service.set_pin(&user_id, new_pin).await {
                Ok(_) => {
                    bot
                        .send_message(msg.chat.id, "✅ *PIN changed successfully*")
                        .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
                }
                Err(e) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("❌ Error setting new PIN: {}", e)
                    ).await?;
                }
            }
        }
        Ok(false) => {
            bot.send_message(msg.chat.id, "❌ Incorrect PIN").await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_disable_pin(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    match state.security_service.disable_pin(&user_id).await {
        Ok(_) => {
            bot
                .send_message(
                    msg.chat.id,
                    "✅ *PIN Disabled*\n\n\
                PIN protection has been removed\\."
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_set_limit(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    // Parse: /setlimit <daily|weekly> <amount_usd>
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() != 2 {
        bot.send_message(
            msg.chat.id,
            "❌ Usage: /setlimit <daily|weekly> <amount_usd>\n\n\
            Examples:\n\
            • /setlimit daily 1000\n\
            • /setlimit weekly 5000"
        ).await?;
        return Ok(());
    }

    let period = parts[0].to_lowercase();
    let amount: f64 = match parts[1].parse() {
        Ok(a) => a,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid amount format").await?;
            return Ok(());
        }
    };

    let (daily_limit, weekly_limit) = match period.as_str() {
        "daily" => (Some(amount), None),
        "weekly" => (None, Some(amount)),
        _ => {
            bot.send_message(msg.chat.id, "❌ Period must be 'daily' or 'weekly'").await?;
            return Ok(());
        }
    };

    match state.security_service.set_limits(&user_id, daily_limit, weekly_limit).await {
        Ok(_) => {
            let msg_text = format!(
                "✅ *Withdrawal Limit Set*\n\n\
                {} limit: ${}\n\n\
                Transactions exceeding this limit will be blocked\\.",
                if daily_limit.is_some() {
                    "Daily"
                } else {
                    "Weekly"
                },
                format_currency(amount)
            );
            bot
                .send_message(msg.chat.id, msg_text)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_lock_wallet(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    match state.security_service.lock_wallet(&user_id).await {
        Ok(_) => {
            bot
                .send_message(
                    msg.chat.id,
                    "🔒 *Wallet Locked*\n\n\
                All transactions are now disabled\\.\n\
                Use /unlockwallet <pin> to unlock\\."
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_unlock_wallet(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    let pin = args.trim();

    match state.security_service.unlock_wallet(&user_id, pin).await {
        Ok(_) => {
            bot
                .send_message(
                    msg.chat.id,
                    "🔓 *Wallet Unlocked*\n\n\
                Transactions are now enabled\\."
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn handle_security_info(
    bot: Bot,
    msg: Message,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    match state.security_service.get_or_create_settings(&user_id).await {
        Ok(settings) => {
            let pin_status = if settings.pin_hash.is_some() { "✅ Enabled" } else { "❌ Disabled" };

            let daily_limit = settings.daily_withdrawal_limit
                .map(|l|
                    format!("${}", format_currency(l.to_string().parse::<f64>().unwrap_or(0.0)))
                )
                .unwrap_or_else(|| "None".to_string());

            let weekly_limit = settings.weekly_withdrawal_limit
                .map(|l|
                    format!("${}", format_currency(l.to_string().parse::<f64>().unwrap_or(0.0)))
                )
                .unwrap_or_else(|| "None".to_string());

            let wallet_status = if settings.wallet_locked {
                "🔒 *Locked*"
            } else {
                "🔓 *Unlocked*"
            };

            let msg_text = format!(
                "🔐 *Security Settings*\n\n\
                *PIN Protection:* {}\n\
                *Daily Limit:* {}\n\
                *Weekly Limit:* {}\n\
                *Wallet Status:* {}\n\n\
                Commands:\n\
                • /setpin \\- Set PIN protection\n\
                • /changepin \\- Change PIN\n\
                • /disablepin \\- Disable PIN\n\
                • /setlimit \\- Set withdrawal limits\n\
                • /lockwallet \\- Lock wallet\n\
                • /unlockwallet \\- Unlock wallet",
                escape_markdown(pin_status),
                escape_markdown(&daily_limit),
                escape_markdown(&weekly_limit),
                wallet_status
            );

            bot
                .send_message(msg.chat.id, msg_text)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}

// ==================== PHASE 9: SWAP HANDLERS (STUBS) ====================

async fn handle_swap(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        "🔄 Swap feature coming soon!\n\nThis will allow you to swap tokens using Uniswap, PancakeSwap, and Jupiter."
    ).await?;
    Ok(())
}

async fn handle_swap_quote(
    bot: Bot,
    msg: Message,
    args: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "💱 Swap quote feature coming soon!").await?;
    Ok(())
}

async fn handle_swap_history(
    bot: Bot,
    msg: Message,
    args: String,
    user_id: String,
    state: Arc<BotState>
) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "📊 Swap history feature coming soon!").await?;
    Ok(())
}
