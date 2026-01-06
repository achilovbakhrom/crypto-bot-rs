use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

// Main menu keyboard
pub fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("💼 My Wallets", "menu:wallets"),
            InlineKeyboardButton::callback("➕ Create Wallet", "menu:create_wallet"),
        ],
        vec![
            InlineKeyboardButton::callback("💰 Portfolio", "menu:portfolio"),
            InlineKeyboardButton::callback("📊 Prices", "menu:prices"),
        ],
        vec![
            InlineKeyboardButton::callback("📖 Address Book", "menu:addresses"),
            InlineKeyboardButton::callback("🔔 Alerts", "menu:alerts"),
        ],
        vec![
            InlineKeyboardButton::callback("🔐 Security", "menu:security"),
            InlineKeyboardButton::callback("❓ Help", "menu:help"),
        ],
    ])
}

// Chain selection keyboard for wallet creation
pub fn chain_selection() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🔷 Ethereum (ETH)", "chain:ETH"),
            InlineKeyboardButton::callback("🟡 BSC (BNB)", "chain:BSC"),
        ],
        vec![
            InlineKeyboardButton::callback("🟣 Solana (SOL)", "chain:SOLANA"),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Wallet actions keyboard
pub fn wallet_actions(wallet_id: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("💰 Balance", format!("wallet:balance:{}", wallet_id)),
            InlineKeyboardButton::callback("📤 Send", format!("wallet:send:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("💱 Swap", format!("wallet:swap:{}", wallet_id)),
            InlineKeyboardButton::callback("📥 Receive", format!("wallet:receive:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("📋 History", format!("wallet:history:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Wallets", "menu:wallets"),
        ],
    ])
}

// Back to main menu button
pub fn back_to_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Wallets list with action buttons
pub fn wallets_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("➕ Create New Wallet", "menu:create_wallet"),
            InlineKeyboardButton::callback("📥 Import Wallet", "menu:import_wallet"),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Help menu with categories
pub fn help_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("💼 Wallet Commands", "help:wallets"),
            InlineKeyboardButton::callback("💸 Transaction Commands", "help:transactions"),
        ],
        vec![
            InlineKeyboardButton::callback("📖 Address Book", "help:addressbook"),
            InlineKeyboardButton::callback("🔔 Alerts & Scheduling", "help:alerts"),
        ],
        vec![
            InlineKeyboardButton::callback("🔐 Security", "help:security"),
            InlineKeyboardButton::callback("💱 Swap", "help:swap"),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Security menu
pub fn security_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🔒 Set PIN", "security:setpin"),
            InlineKeyboardButton::callback("🔄 Change PIN", "security:changepin"),
        ],
        vec![
            InlineKeyboardButton::callback("📊 Set Limits", "security:limits"),
            InlineKeyboardButton::callback("🔐 Lock Wallet", "security:lock"),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Alerts menu
pub fn alerts_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("➕ New Alert", "alert:new"),
            InlineKeyboardButton::callback("📋 My Alerts", "alert:list"),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Address book menu
pub fn address_book_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("➕ Save Address", "address:save"),
            InlineKeyboardButton::callback("📋 My Addresses", "address:list"),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Confirmation keyboard
pub fn confirm_action(action: &str, data: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("✅ Confirm", format!("confirm:{}:{}", action, data)),
            InlineKeyboardButton::callback("❌ Cancel", "cancel"),
        ],
    ])
}

// Refresh button for balance/portfolio
pub fn refresh_button(action: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🔄 Refresh", format!("refresh:{}", action)),
            InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
        ],
    ])
}

// Send menu - choose what to send
pub fn send_menu(wallet_id: &str, chain: &str) -> InlineKeyboardMarkup {
    let native_token = match chain {
        "ETH" => "🔷 Send ETH",
        "BSC" => "🟡 Send BNB",
        "SOLANA" => "🟣 Send SOL",
        _ => "📤 Send Native Token",
    };

    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(native_token, format!("send:native:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("🪙 Send Token (ERC20/SPL)", format!("send:token:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
        ],
    ])
}

// Send amount presets
pub fn send_amount_presets(wallet_id: &str, balance: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("25%", format!("send:amount:{}:25", wallet_id)),
            InlineKeyboardButton::callback("50%", format!("send:amount:{}:50", wallet_id)),
            InlineKeyboardButton::callback("75%", format!("send:amount:{}:75", wallet_id)),
            InlineKeyboardButton::callback("100%", format!("send:amount:{}:100", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("✏️ Enter Custom Amount", format!("send:custom:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("« Back", format!("wallet:send:{}", wallet_id)),
        ],
    ])
}

// Swap menu - choose swap type
pub fn swap_menu(wallet_id: &str, chain: &str) -> InlineKeyboardMarkup {
    let (token1, token2) = match chain {
        "ETH" => ("ETH → USDC", "ETH → USDT"),
        "BSC" => ("BNB → USDT", "BNB → BUSD"),
        "SOLANA" => ("SOL → USDC", "SOL → USDT"),
        _ => ("Native → Stable", "Native → Other"),
    };

    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(token1, format!("swap:preset1:{}", wallet_id)),
            InlineKeyboardButton::callback(token2, format!("swap:preset2:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("🔄 Custom Swap", format!("swap:custom:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
        ],
    ])
}

// Swap amount presets
pub fn swap_amount_presets(wallet_id: &str, from_token: &str, to_token: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("25%", format!("swap:amount:{}:{}:{}:25", wallet_id, from_token, to_token)),
            InlineKeyboardButton::callback("50%", format!("swap:amount:{}:{}:{}:50", wallet_id, from_token, to_token)),
            InlineKeyboardButton::callback("75%", format!("swap:amount:{}:{}:{}:75", wallet_id, from_token, to_token)),
            InlineKeyboardButton::callback("100%", format!("swap:amount:{}:{}:{}:100", wallet_id, from_token, to_token)),
        ],
        vec![
            InlineKeyboardButton::callback("✏️ Enter Custom Amount", format!("swap:customamt:{}:{}:{}", wallet_id, from_token, to_token)),
        ],
        vec![
            InlineKeyboardButton::callback("« Back", format!("wallet:swap:{}", wallet_id)),
        ],
    ])
}
