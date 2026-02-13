use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::enums::Chain;

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

// Chain selection keyboard for wallet creation — dynamic from Chain::all()
pub fn chain_selection() -> InlineKeyboardMarkup {
    let chains = Chain::all();
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    for chain in chains {
        let label = format!("{} {} ({})", chain.emoji(), chain.display_name(), chain.native_symbol());
        let btn = InlineKeyboardButton::callback(label, format!("chain:{}", chain.as_str()));
        row.push(btn);
        if row.len() == 2 {
            rows.push(row);
            row = Vec::new();
        }
    }
    if !row.is_empty() {
        rows.push(row);
    }

    rows.push(vec![
        InlineKeyboardButton::callback("« Back to Menu", "menu:main"),
    ]);

    InlineKeyboardMarkup::new(rows)
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
            InlineKeyboardButton::callback("🪙 Tokens", format!("wallet:tokens:{}", wallet_id)),
        ],
        vec![
            InlineKeyboardButton::callback("🔍 View on Explorer", format!("wallet:explorer:{}", wallet_id)),
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

// Send menu - choose what to send, using Chain::emoji() for label
pub fn send_menu(wallet_id: &str, chain: &str) -> InlineKeyboardMarkup {
    let native_label = chain
        .parse::<Chain>()
        .map(|c| format!("{} Send {}", c.emoji(), c.native_symbol()))
        .unwrap_or_else(|_| "📤 Send Native Token".to_string());

    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(&native_label, format!("send:native:{}", wallet_id)),
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
pub fn send_amount_presets(wallet_id: &str, _balance: &str) -> InlineKeyboardMarkup {
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

// Swap menu - choose swap type, dynamic per chain
pub fn swap_menu(wallet_id: &str, chain: &str) -> InlineKeyboardMarkup {
    let parsed = chain.parse::<Chain>().ok();
    let native = parsed.map(|c| c.native_symbol()).unwrap_or("NATIVE");

    let (label1, label2) = match chain {
        "BSC" => (format!("{} → USDT", native), format!("{} → BUSD", native)),
        _ => (format!("{} → USDC", native), format!("{} → USDT", native)),
    };

    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(&label1, format!("swap:preset1:{}", wallet_id)),
            InlineKeyboardButton::callback(&label2, format!("swap:preset2:{}", wallet_id)),
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

// Alert token selection keyboard
pub fn alert_token_selection() -> InlineKeyboardMarkup {
    let tokens = [
        ("₿ BTC", "BTC"), ("🔷 ETH", "ETH"),
        ("🟣 SOL", "SOL"), ("🟡 BNB", "BNB"),
        ("✕ XRP", "XRP"), ("₳ ADA", "ADA"),
        ("🔴 AVAX", "AVAX"), ("🐕 DOGE", "DOGE"),
        ("⚫ DOT", "DOT"), ("🔗 LINK", "LINK"),
        ("🟣 MATIC", "MATIC"), ("🦄 UNI", "UNI"),
    ];

    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    for (label, symbol) in tokens {
        row.push(InlineKeyboardButton::callback(label, format!("alert:token:{}", symbol)));
        if row.len() == 3 {
            rows.push(row);
            row = Vec::new();
        }
    }
    if !row.is_empty() {
        rows.push(row);
    }

    rows.push(vec![
        InlineKeyboardButton::callback("« Back to Alerts", "menu:alerts"),
    ]);

    InlineKeyboardMarkup::new(rows)
}

// Alert chain selection keyboard for multi-chain tokens
pub fn alert_chain_selection(symbol: &str, chains: &[Chain]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    for chain in chains {
        let label = format!("{} {}", chain.emoji(), chain.display_name());
        row.push(InlineKeyboardButton::callback(label, format!("alert:chain:{}:{}", symbol, chain.as_str())));
        if row.len() == 2 {
            rows.push(row);
            row = Vec::new();
        }
    }
    if !row.is_empty() {
        rows.push(row);
    }

    rows.push(vec![
        InlineKeyboardButton::callback("« Back", "alert:new"),
    ]);

    InlineKeyboardMarkup::new(rows)
}

// Alert type selection keyboard
pub fn alert_type_selection(symbol: &str, chain: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📈 Price Above", format!("alert:type:{}:{}:above", symbol, chain)),
        ],
        vec![
            InlineKeyboardButton::callback("📉 Price Below", format!("alert:type:{}:{}:below", symbol, chain)),
        ],
        vec![
            InlineKeyboardButton::callback("⚡ Percent Change", format!("alert:type:{}:{}:percent", symbol, chain)),
        ],
        vec![
            InlineKeyboardButton::callback("« Back", "alert:new"),
        ],
    ])
}

/// Token list keyboard with pagination for wallet token view
pub fn token_list(wallet_id: &str, page: usize, total_pages: usize) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    // Pagination row
    if total_pages > 1 {
        let mut nav = Vec::new();
        if page > 0 {
            nav.push(InlineKeyboardButton::callback(
                "◀️ Prev",
                format!("wallet:tokens:{}:{}", wallet_id, page - 1),
            ));
        }
        nav.push(InlineKeyboardButton::callback(
            format!("Page {}/{}", page + 1, total_pages),
            "noop",
        ));
        if page + 1 < total_pages {
            nav.push(InlineKeyboardButton::callback(
                "Next ▶️",
                format!("wallet:tokens:{}:{}", wallet_id, page + 1),
            ));
        }
        rows.push(nav);
    }

    rows.push(vec![
        InlineKeyboardButton::callback("🔄 Refresh", format!("wallet:tokens:{}", wallet_id)),
        InlineKeyboardButton::callback("« Back to Wallet", format!("wallet:select:{}", wallet_id)),
    ]);

    InlineKeyboardMarkup::new(rows)
}
