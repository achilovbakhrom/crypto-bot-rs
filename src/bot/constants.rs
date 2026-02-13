// Command descriptions
pub mod command_descriptions {
    pub const START: &str = "Start the bot and see welcome message";
    pub const CREATE_WALLET: &str = "Create a new wallet - Usage: /createwallet <chain>";
    pub const IMPORT_WALLET: &str =
        "Import existing wallet - Usage: /importwallet <chain> <mnemonic or private key>";
    pub const WALLETS: &str = "List all your wallets";
    pub const BALANCE: &str = "Check wallet balance - Usage: /balance <wallet_id> [token_address]";
    pub const SEND: &str =
        "Send transaction - Usage: /send <wallet_id> <to_address> <amount> [token_address]";
    pub const ESTIMATE_FEE: &str =
        "Estimate transaction fee - Usage: /estimatefee <wallet_id> <to_address> <amount>";
    pub const BATCH_SEND: &str =
        "Batch send - Usage: /batchsend <wallet_id> then paste CSV (to,amount)";
    pub const HISTORY: &str = "View transaction history - Usage: /history <wallet_id> [limit]";
    pub const ADDRESS: &str = "Get wallet address with QR code - Usage: /address <wallet_id>";
    pub const PORTFOLIO: &str = "Show your complete portfolio with USD values";
    pub const PRICES: &str = "Get current cryptocurrency prices";
    pub const SAVE_ADDRESS: &str =
        "Save address to address book - Usage: /saveaddress <name> <address> <chain> [notes]";
    pub const ADDRESSES: &str = "List all saved addresses";
    pub const DELETE_ADDRESS: &str = "Delete saved address - Usage: /deleteaddress <name>";
    pub const SCHEDULE: &str =
        "Schedule a transaction - Usage: /schedule <wallet_id> <to> <amount> <datetime> [token] [recurring]";
    pub const SCHEDULED: &str = "List scheduled transactions";
    pub const CANCEL_SCHEDULE: &str =
        "Cancel scheduled transaction - Usage: /cancelschedule <schedule_id>";
    pub const SET_ALERT: &str =
        "Set price alert - Usage: /setalert <symbol> <above|below> <price> [chain]";
    pub const ALERTS: &str = "List your price alerts";
    pub const DELETE_ALERT: &str = "Delete price alert - Usage: /deletealert <alert_id>";
    pub const SET_PIN: &str = "Set transaction PIN - Usage: /setpin <6-digit-pin>";
    pub const CHANGE_PIN: &str = "Change your PIN - Usage: /changepin <old-pin> <new-pin>";
    pub const DISABLE_PIN: &str = "Disable PIN protection";
    pub const SET_LIMIT: &str =
        "Set withdrawal limits - Usage: /setlimit daily <amount> or weekly <amount>";
    pub const LOCK_WALLET: &str = "Lock wallet (requires PIN to unlock)";
    pub const UNLOCK_WALLET: &str = "Unlock wallet - Usage: /unlock <pin>";
    pub const SECURITY: &str = "View security settings";
    pub const SWAP: &str =
        "Swap tokens - Usage: /swap <wallet_id> <from_token> <to_token> <amount> [slippage]";
    pub const SWAP_QUOTE: &str =
        "Get swap quote - Usage: /swapquote <chain> <from_token> <to_token> <amount> [slippage]";
    pub const SWAP_HISTORY: &str = "View swap history - Usage: /swaphistory [wallet_id]";
    pub const HELP: &str = "Show help message";
}

// Bot messages
pub mod messages {
    // Welcome messages
    pub const WELCOME_TEXT: &str =
        r#"
🔐 *Welcome to Crypto Wallet Bot\!*

I'm a multi\-chain wallet manager supporting 14\+ blockchains including Bitcoin, Ethereum, BSC, Polygon, Avalanche, Arbitrum, Optimism, Base, Fantom, Cronos, Gnosis, Solana, XRP, and Cardano\.

*Quick Start:*
1\. Create a wallet: `/createwallet ETH`
2\. Check balance: `/balance <wallet_id>`
3\. Send tokens: `/send <wallet_id> <address> <amount>`

Use /help to see all available commands\.
"#;

    // Help messages
    pub const HELP_HEADER: &str = "*Available Commands:*\n\n";
    pub const HELP_COMMANDS: &str =
        "\
        `/start` \\- Show welcome message\n\
        `/createwallet <chain>` \\- Create new wallet\n\
          Example: `/createwallet ETH`\n\n\
        `/importwallet <chain> <key>` \\- Import wallet\n\
          Example: `/importwallet ETH word1 word2\\.\\.\\.`\n\n\
        `/wallets` \\- List all your wallets\n\n\
        `/balance <wallet_id>` \\- Check balance\n\
          Example: `/balance abc123`\n\n\
        `/send <wallet_id> <to> <amount>` \\- Send transaction\n\
          Example: `/send abc123 0x\\.\\.\\. 0\\.5`\n\n\
        `/estimatefee <wallet_id> <to> <amount>` \\- Estimate fees\n\
          Example: `/estimatefee abc123 0x\\.\\.\\. 0\\.5`\n\n\
        `/batchsend <wallet_id>` \\- Batch send to multiple addresses\n\
          Then paste CSV: `address1,amount1\\naddress2,amount2`\n\n\
        `/history <wallet_id>` \\- View transactions\n\
          Example: `/history abc123`\n\n\
        `/address <wallet_id>` \\- Get wallet address\n\
          Example: `/address abc123`\n\n\
        `/portfolio` \\- View your complete portfolio\n\n\
        `/prices` \\- Get current crypto prices\n\n\
        `/saveaddress <name> <addr> <chain>` \\- Save address\n\
          Example: `/saveaddress alice 0x\\.\\.\\. ETH`\n\n\
        `/addresses` \\- List saved addresses\n\n\
        `/deleteaddress <name>` \\- Delete saved address\n\
          Example: `/deleteaddress alice`\n\n\
        *Supported Chains:* BTC, ETH, BSC, SOLANA, POLYGON, AVAX, ARBITRUM, OPTIMISM, BASE, FANTOM, CRONOS, GNOSIS, XRP, ADA";

    // Error messages
    pub const ERR_CHAIN_REQUIRED: &str =
        "❌ Please specify a chain: /createwallet <chain>\nSupported: BTC, ETH, BSC, SOLANA, POLYGON, AVAX, ARBITRUM, OPTIMISM, BASE, FANTOM, CRONOS, GNOSIS, XRP, ADA";
    pub const ERR_INVALID_CHAIN: &str = "❌ Invalid chain. Supported: BTC, ETH, BSC, SOLANA, POLYGON, AVAX, ARBITRUM, OPTIMISM, BASE, FANTOM, CRONOS, GNOSIS, XRP, ADA";
    pub const ERR_INVALID_WALLET_ID: &str = "❌ Invalid wallet ID format";
    pub const ERR_IMPORT_USAGE: &str =
        "❌ Usage: /importwallet <chain> <mnemonic or private key>\nExample: /importwallet ETH word1 word2 word3...";
    pub const ERR_BALANCE_USAGE: &str = "❌ Usage: /balance <wallet_id> [token_address]";
    pub const ERR_SEND_USAGE: &str =
        "❌ Usage: /send <wallet_id> <to_address|name> <amount> [token_address]\n\
            You can use saved address names instead of full addresses!";
    pub const ERR_ESTIMATE_FEE_USAGE: &str =
        "Usage: /estimatefee <wallet_id> <to_address> <amount> [token_address]\n\n\
            Example: /estimatefee abc123 0x742d35Cc... 0.1";
    pub const ERR_BATCH_SEND_USAGE: &str =
        "❌ Usage: /batchsend <wallet_id>\n\nThen send CSV data:\naddress1,amount1\naddress2,amount2";
    pub const ERR_HISTORY_USAGE: &str =
        "❌ Usage: /history <wallet_id> [limit]\nExample: /history abc123 50";
    pub const ERR_ADDRESS_USAGE: &str = "❌ Usage: /address <wallet_id>";
    pub const ERR_SAVE_ADDRESS_USAGE: &str =
        "❌ Usage: /saveaddress <name> <address> <ETH|BSC|SOLANA> [notes]\nExample: /saveaddress alice 0x742d35Cc... ETH My friend";
    pub const ERR_DELETE_ADDRESS_USAGE: &str = "❌ Usage: /deleteaddress <name>";
    pub const ERR_SCHEDULE_USAGE: &str =
        "❌ Usage: /schedule <wallet_id> <to> <amount> <datetime> [token] [recurring]\nExample: /schedule abc123 0x742d... 0.1 2024-12-31T23:59:00 - daily";
    pub const ERR_CANCEL_SCHEDULE_USAGE: &str = "❌ Usage: /cancelschedule <schedule_id>";
    pub const ERR_SET_ALERT_USAGE: &str =
        "❌ Usage: /setalert <symbol> <above|below> <price> [chain]\nExample: /setalert BTC above 100000 ETH";
    pub const ERR_DELETE_ALERT_USAGE: &str = "❌ Usage: /deletealert <alert_id>";
    pub const ERR_SET_PIN_USAGE: &str = "❌ Usage: /setpin <6-digit-pin>\nExample: /setpin 123456";
    pub const ERR_CHANGE_PIN_USAGE: &str =
        "❌ Usage: /changepin <old-pin> <new-pin>\nExample: /changepin 123456 654321";
    pub const ERR_SET_LIMIT_USAGE: &str =
        "❌ Usage: /setlimit daily <amount> OR /setlimit weekly <amount>\nExample: /setlimit daily 1000";
    pub const ERR_UNLOCK_USAGE: &str = "❌ Usage: /unlock <pin>";
    pub const ERR_SWAP_USAGE: &str =
        "❌ Usage: /swap <wallet_id> <from_token> <to_token> <amount> [slippage]\n\
            Example: /swap abc123 USDC SOL 100 1.0\n\
            Tokens: Use contract address or native (ETH/BNB/SOL)\n\
            Slippage: Optional, default 1.0%";
    pub const ERR_SWAP_QUOTE_USAGE: &str =
        "❌ Usage: /swapquote <chain> <from_token> <to_token> <amount> [slippage]\n\
            Example: /swapquote ETH USDC ETH 1000 1.0";
    pub const ERR_SWAP_HISTORY_USAGE: &str =
        "❌ Usage: /swaphistory [wallet_id]\nExample: /swaphistory abc123";

    // Status messages
    pub const STATUS_CREATING_WALLET: &str = "⏳ Creating wallet...";
    pub const STATUS_IMPORTING_WALLET: &str = "⏳ Importing wallet...";
    pub const STATUS_FETCHING_BALANCE: &str = "⏳ Fetching balance...";
    pub const STATUS_SENDING_TX: &str = "⏳ Sending transaction...";
    pub const STATUS_ESTIMATING_FEE: &str = "⏳ Estimating transaction fee...";
    pub const STATUS_PROCESSING_BATCH: &str = "⏳ Processing batch transactions...";
    pub const STATUS_FETCHING_HISTORY: &str = "⏳ Fetching transaction history...";
    pub const STATUS_GENERATING_QR: &str = "⏳ Generating QR code...";
    pub const STATUS_FETCHING_PORTFOLIO: &str = "⏳ Fetching portfolio data...";
    pub const STATUS_FETCHING_PRICES: &str = "⏳ Fetching current prices...";
    pub const STATUS_SAVING_ADDRESS: &str = "⏳ Saving address...";
    pub const STATUS_DELETING_ADDRESS: &str = "⏳ Deleting address...";
    pub const STATUS_SCHEDULING: &str = "⏳ Scheduling transaction...";
    pub const STATUS_CANCELING_SCHEDULE: &str = "⏳ Canceling scheduled transaction...";
    pub const STATUS_SETTING_ALERT: &str = "⏳ Setting price alert...";
    pub const STATUS_DELETING_ALERT: &str = "⏳ Deleting alert...";
    pub const STATUS_SETTING_PIN: &str = "⏳ Setting PIN...";
    pub const STATUS_CHANGING_PIN: &str = "⏳ Changing PIN...";
    pub const STATUS_DISABLING_PIN: &str = "⏳ Disabling PIN...";
    pub const STATUS_SETTING_LIMIT: &str = "⏳ Setting withdrawal limit...";
    pub const STATUS_LOCKING_WALLET: &str = "⏳ Locking wallet...";
    pub const STATUS_UNLOCKING_WALLET: &str = "⏳ Unlocking wallet...";
    pub const STATUS_GETTING_QUOTE: &str = "⏳ Getting swap quote...";
    pub const STATUS_EXECUTING_SWAP: &str = "⏳ Executing swap...";
    pub const STATUS_FETCHING_SWAP_HISTORY: &str = "⏳ Fetching swap history...";

    // Success messages
    pub const SUCCESS_WALLET_CREATED: &str = "✅ *Wallet Created Successfully\\!*";
    pub const SUCCESS_WALLET_IMPORTED: &str = "✅ *Wallet Imported Successfully\\!*";
    pub const SUCCESS_TX_SENT: &str = "✅ *Transaction Sent\\!*";
    pub const SUCCESS_ADDRESS_SAVED: &str = "✅ Address saved successfully!";
    pub const SUCCESS_ADDRESS_DELETED: &str = "✅ Address deleted successfully!";
    pub const SUCCESS_SCHEDULED: &str = "✅ Transaction scheduled successfully!";
    pub const SUCCESS_SCHEDULE_CANCELED: &str = "✅ Scheduled transaction canceled!";
    pub const SUCCESS_ALERT_SET: &str = "✅ Price alert set successfully!";
    pub const SUCCESS_ALERT_DELETED: &str = "✅ Alert deleted successfully!";
    pub const SUCCESS_PIN_SET: &str =
        "✅ PIN set successfully! Your transactions are now protected.";
    pub const SUCCESS_PIN_CHANGED: &str = "✅ PIN changed successfully!";
    pub const SUCCESS_PIN_DISABLED: &str = "✅ PIN protection disabled.";
    pub const SUCCESS_LIMIT_SET: &str = "✅ Withdrawal limit set successfully!";
    pub const SUCCESS_WALLET_LOCKED: &str = "✅ Wallet locked. Use /unlock <pin> to unlock.";
    pub const SUCCESS_WALLET_UNLOCKED: &str = "✅ Wallet unlocked successfully!";
    pub const SUCCESS_SWAP_COMPLETED: &str = "✅ *Swap completed successfully\\!*";

    // Info messages
    pub const INFO_NO_WALLETS: &str =
        "📭 You don't have any wallets yet.\n\nCreate one with: /createwallet <chain>";
    pub const INFO_NO_HISTORY: &str = "📭 No transaction history found for this wallet.";
    pub const INFO_NO_ADDRESSES: &str =
        "📭 You don't have any saved addresses yet.\n\nSave one with: /saveaddress <name> <address> <chain>";
    pub const INFO_NO_SCHEDULED: &str = "📭 You don't have any scheduled transactions.";
    pub const INFO_NO_ALERTS: &str = "📭 You don't have any active price alerts.";
    pub const INFO_NO_SWAP_HISTORY: &str = "📭 No swap history found.";
    pub const INFO_USING_SAVED_ADDRESS: &str = "📖 Using saved address: {} ({})";
    pub const INFO_MNEMONIC_WARNING: &str =
        "⚠️ *IMPORTANT:* Never share your mnemonic\\. I will send it once\\. Save it now\\!";

    // Field labels
    pub const LABEL_CHAIN: &str = "📍 Chain";
    pub const LABEL_WALLET_ID: &str = "🆔 Wallet ID";
    pub const LABEL_ADDRESS: &str = "📬 Address";
    pub const LABEL_MNEMONIC: &str = "🔑 *SAVE YOUR MNEMONIC SECURELY:*";
    pub const LABEL_SYMBOL: &str = "💵 Symbol";
    pub const LABEL_AMOUNT: &str = "💎 Amount";
    pub const LABEL_BALANCE: &str = "💰 *Balance*";
    pub const LABEL_TX_HASH: &str = "🔗 Hash";
    pub const LABEL_STATUS: &str = "📊 Status";
    pub const LABEL_CREATED: &str = "📅 Created";
    pub const LABEL_GAS_LIMIT: &str = "📊 Gas Limit";
    pub const LABEL_GAS_PRICE: &str = "💵 Gas Price";
    pub const LABEL_MAX_FEE: &str = "🔝 Max Fee";
    pub const LABEL_PRIORITY_FEE: &str = "⚡ Priority Fee";
    pub const LABEL_TOTAL_COST: &str = "💰 *Total Cost:*";

    // Headers
    pub const HEADER_YOUR_WALLETS: &str = "*Your Wallets:*\n\n";
    pub const HEADER_TX_HISTORY: &str = "*Transaction History*\n";
    pub const HEADER_PORTFOLIO: &str = "*Your Portfolio*\n\n";
    pub const HEADER_CRYPTO_PRICES: &str = "*Cryptocurrency Prices*\n\n";
    pub const HEADER_SAVED_ADDRESSES: &str = "*Your Saved Addresses:*\n\n";
    pub const HEADER_SCHEDULED_TXS: &str = "*Your Scheduled Transactions:*\n\n";
    pub const HEADER_PRICE_ALERTS: &str = "*Your Price Alerts:*\n\n";
    pub const HEADER_SECURITY_SETTINGS: &str = "*Security Settings*\n\n";
    pub const HEADER_GAS_ESTIMATION: &str = "⛽ *Gas Estimation*\n\n";
    pub const HEADER_SWAP_QUOTE: &str = "💱 *Swap Quote*\n\n";
    pub const HEADER_SWAP_HISTORY: &str = "*Swap History*\n\n";
    pub const HEADER_BATCH_RESULTS: &str = "*Batch Send Results:*\n\n";
}

// Re-export Chain enum for convenience
pub use crate::enums::Chain;

pub mod chains {
    use crate::enums::Chain;

    pub fn is_valid_chain(chain: &str) -> bool {
        chain.parse::<Chain>().is_ok()
    }
}

// Format strings
pub mod formats {
    pub const WALLET_ITEM: &str =
        "🔸 *{}*\n\
      🆔 ID: `{}`\n\
      📬 Address: `{}`\n\
      📅 Created: {}\n\n";

    pub const WALLET_DETAILS: &str =
        "\n\n\
                📍 Chain: `{}`\n\
                🆔 Wallet ID: `{}`\n\
                📬 Address: `{}`";

    pub const WALLET_WITH_MNEMONIC: &str =
        "\n\n\
                📍 Chain: `{}`\n\
                🆔 Wallet ID: `{}`\n\
                📬 Address: `{}`\n\n\
                🔑 *SAVE YOUR MNEMONIC SECURELY:*\n\
                `{}`\n\n\
                ⚠️ *IMPORTANT:* Never share your mnemonic\\. \
                I will send it once\\. Save it now\\!";

    pub const BALANCE_INFO: &str =
        "💰 *Balance*\n\n\
                💵 Symbol: *{}*\n\
                💎 Amount: `{}`";

    pub const TX_RESULT: &str =
        "✅ *Transaction Sent\\!*\n\n\
                🔗 Hash: `{}`\n\
                📊 Status: `{}`";

    pub const ADDRESS_ITEM: &str =
        "📇 *{}*\n\
            📬 Address: `{}`\n\
            ⛓️ Chain: {}\n\
            📝 Notes: {}\n\
            📅 Added: {}\n\n";

    pub const SCHEDULE_ITEM: &str =
        "🔸 *ID:* `{}`\n\
            💼 Wallet: `{}`\n\
            📬 To: `{}`\n\
            💰 Amount: {}\n\
            🪙 Token: {}\n\
            📅 Execute at: {}\n\
            🔄 Recurring: {}\n\
            📊 Status: {}\n\n";

    pub const ALERT_ITEM: &str =
        "🔸 *ID:* `{}`\n\
            🪙 Symbol: {}\n\
            📊 Condition: {} {}\n\
            ⛓️ Chain: {}\n\
            ✅ Active: {}\n\n";

    pub const SWAP_ITEM: &str =
        "🔸 *Swap*\n\
            📅 {}\n\
            From: {} {}\n\
            To: {} {}\n\
            📊 Status: {}\n\
            🔗 Hash: `{}`\n\n";
}
