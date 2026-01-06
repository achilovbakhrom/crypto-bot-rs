use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Crypto Wallet Bot Commands:")]
pub enum Command {
    #[command(description = "Start the bot and see welcome message")]
    Start,

    #[command(
        description = "Create a new wallet - Usage: /createwallet <ETH|BSC|SOLANA>"
    )] CreateWallet(String),

    #[command(
        description = "Import existing wallet - Usage: /importwallet <chain> <mnemonic or private key>"
    )] ImportWallet(String),

    #[command(description = "List all your wallets")]
    Wallets,

    #[command(
        description = "Check wallet balance - Usage: /balance <wallet_id> [token_address]"
    )] Balance(String),

    #[command(
        description = "Send transaction - Usage: /send <wallet_id> <to_address> <amount> [token_address]"
    )] Send(String),

    #[command(
        description = "Estimate transaction fee - Usage: /estimatefee <wallet_id> <to_address> <amount>"
    )] EstimateFee(String),

    #[command(
        description = "Batch send - Usage: /batchsend <wallet_id> then paste CSV (to,amount)"
    )] BatchSend(String),

    #[command(
        description = "View transaction history - Usage: /history <wallet_id> [limit]"
    )] History(String),

    #[command(
        description = "Get wallet address with QR code - Usage: /address <wallet_id>"
    )] Address(String),

    #[command(description = "Show your complete portfolio with USD values")]
    Portfolio,

    #[command(description = "Get current cryptocurrency prices")]
    Prices,

    #[command(
        description = "Save address to address book - Usage: /saveaddress <name> <address> <ETH|BSC|SOLANA> [notes]"
    )] SaveAddress(String),

    #[command(description = "List all saved addresses")]
    Addresses,

    #[command(description = "Delete saved address - Usage: /deleteaddress <name>")] DeleteAddress(
        String,
    ),

    #[command(
        description = "Schedule a transaction - Usage: /schedule <wallet_id> <to> <amount> <datetime> [token] [recurring]"
    )] Schedule(String),

    #[command(description = "List scheduled transactions")]
    Scheduled,

    #[command(
        description = "Cancel scheduled transaction - Usage: /cancelschedule <schedule_id>"
    )] CancelSchedule(String),

    #[command(
        description = "Set price alert - Usage: /setalert <symbol> <above|below> <price> [chain]"
    )] SetAlert(String),

    #[command(description = "List your price alerts")]
    Alerts,

    #[command(description = "Delete price alert - Usage: /deletealert <alert_id>")] DeleteAlert(
        String,
    ),

    #[command(description = "Set transaction PIN - Usage: /setpin <6-digit-pin>")] SetPin(String),

    #[command(description = "Change your PIN - Usage: /changepin <old-pin> <new-pin>")] ChangePin(
        String,
    ),

    #[command(description = "Disable PIN protection")]
    DisablePin,

    #[command(
        description = "Set withdrawal limits - Usage: /setlimit daily <amount> or weekly <amount>"
    )] SetLimit(String),

    #[command(description = "Lock wallet (requires PIN to unlock)")]
    LockWallet,

    #[command(description = "Unlock wallet - Usage: /unlock <pin>")] UnlockWallet(String),

    #[command(description = "View security settings")]
    Security,

    #[command(
        description = "Swap tokens - Usage: /swap <wallet_id> <from_token> <to_token> <amount> [slippage]"
    )] Swap(String),

    #[command(
        description = "Get swap quote - Usage: /swapquote <chain> <from_token> <to_token> <amount> [slippage]"
    )] SwapQuote(String),

    #[command(description = "View swap history - Usage: /swaphistory [wallet_id]")] SwapHistory(
        String,
    ),

    #[command(description = "Show help message")]
    Help,
}
