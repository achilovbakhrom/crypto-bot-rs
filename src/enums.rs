use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

// ─── Chain ───────────────────────────────────────────────────────────

/// Supported blockchain networks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chain {
    Eth,
    Bsc,
    Solana,
    Polygon,
    Avalanche,
    Arbitrum,
    Optimism,
    Base,
    Fantom,
    Cronos,
    Gnosis,
    Btc,
    Xrp,
    Cardano,
}

impl Chain {
    /// Canonical string stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Eth => "ETH",
            Chain::Bsc => "BSC",
            Chain::Solana => "SOLANA",
            Chain::Polygon => "POLYGON",
            Chain::Avalanche => "AVALANCHE",
            Chain::Arbitrum => "ARBITRUM",
            Chain::Optimism => "OPTIMISM",
            Chain::Base => "BASE",
            Chain::Fantom => "FANTOM",
            Chain::Cronos => "CRONOS",
            Chain::Gnosis => "GNOSIS",
            Chain::Btc => "BTC",
            Chain::Xrp => "XRP",
            Chain::Cardano => "ADA",
        }
    }

    /// Native token symbol for the chain.
    pub fn native_symbol(&self) -> &'static str {
        match self {
            Chain::Eth => "ETH",
            Chain::Bsc => "BNB",
            Chain::Solana => "SOL",
            Chain::Polygon => "POL",
            Chain::Avalanche => "AVAX",
            Chain::Arbitrum => "ETH",
            Chain::Optimism => "ETH",
            Chain::Base => "ETH",
            Chain::Fantom => "FTM",
            Chain::Cronos => "CRO",
            Chain::Gnosis => "xDAI",
            Chain::Btc => "BTC",
            Chain::Xrp => "XRP",
            Chain::Cardano => "ADA",
        }
    }

    /// EVM chain ID. Returns None for non-EVM chains (Solana).
    pub fn chain_id(&self, testnet: bool) -> Option<u64> {
        match (self, testnet) {
            (Chain::Eth, false) => Some(1),
            (Chain::Eth, true) => Some(11155111),        // Sepolia
            (Chain::Bsc, false) => Some(56),
            (Chain::Bsc, true) => Some(97),
            (Chain::Polygon, false) => Some(137),
            (Chain::Polygon, true) => Some(80002),       // Amoy
            (Chain::Avalanche, false) => Some(43114),
            (Chain::Avalanche, true) => Some(43113),     // Fuji
            (Chain::Arbitrum, false) => Some(42161),
            (Chain::Arbitrum, true) => Some(421614),     // Sepolia
            (Chain::Optimism, false) => Some(10),
            (Chain::Optimism, true) => Some(11155420),   // Sepolia
            (Chain::Base, false) => Some(8453),
            (Chain::Base, true) => Some(84532),          // Sepolia
            (Chain::Fantom, false) => Some(250),
            (Chain::Fantom, true) => Some(4002),
            (Chain::Cronos, false) => Some(25),
            (Chain::Cronos, true) => Some(338),
            (Chain::Gnosis, false) => Some(100),
            (Chain::Gnosis, true) => Some(10200),        // Chiado
            (Chain::Solana, _) => None,
            (Chain::Btc, _) => None,
            (Chain::Xrp, _) => None,
            (Chain::Cardano, _) => None,
        }
    }

    /// Whether this chain uses the EVM (Ethereum Virtual Machine).
    pub fn is_evm(&self) -> bool {
        !matches!(self, Chain::Solana | Chain::Btc | Chain::Xrp | Chain::Cardano)
    }

    /// Whether this chain uses the UTXO model.
    pub fn is_utxo(&self) -> bool {
        matches!(self, Chain::Btc | Chain::Cardano)
    }

    /// A well-known dummy address for gas estimation on this chain.
    pub fn dummy_address(&self) -> &'static str {
        if self.is_evm() {
            "0x0000000000000000000000000000000000000001"
        } else {
            match self {
                Chain::Solana => "11111111111111111111111111111111",
                Chain::Btc => "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
                Chain::Xrp => "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                Chain::Cardano => "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3jcu5d8ps7zex2k2xt3uqxgjqnnj83ws8lhrn648jjxtwq2ytjc7",
                _ => unreachable!(),
            }
        }
    }

    /// Default block explorer URL.
    pub fn explorer_url(&self, testnet: bool) -> &'static str {
        match (self, testnet) {
            (Chain::Eth, false) => "https://etherscan.io",
            (Chain::Eth, true) => "https://sepolia.etherscan.io",
            (Chain::Bsc, false) => "https://bscscan.com",
            (Chain::Bsc, true) => "https://testnet.bscscan.com",
            (Chain::Polygon, false) => "https://polygonscan.com",
            (Chain::Polygon, true) => "https://amoy.polygonscan.com",
            (Chain::Avalanche, false) => "https://snowtrace.io",
            (Chain::Avalanche, true) => "https://testnet.snowtrace.io",
            (Chain::Arbitrum, false) => "https://arbiscan.io",
            (Chain::Arbitrum, true) => "https://sepolia.arbiscan.io",
            (Chain::Optimism, false) => "https://optimistic.etherscan.io",
            (Chain::Optimism, true) => "https://sepolia-optimism.etherscan.io",
            (Chain::Base, false) => "https://basescan.org",
            (Chain::Base, true) => "https://sepolia.basescan.org",
            (Chain::Fantom, false) => "https://ftmscan.com",
            (Chain::Fantom, true) => "https://testnet.ftmscan.com",
            (Chain::Cronos, false) => "https://cronoscan.com",
            (Chain::Cronos, true) => "https://testnet.cronoscan.com",
            (Chain::Gnosis, false) => "https://gnosisscan.io",
            (Chain::Gnosis, true) => "https://gnosis-chiado.blockscout.com",
            (Chain::Solana, false) => "https://explorer.solana.com",
            (Chain::Solana, true) => "https://explorer.solana.com/?cluster=devnet",
            (Chain::Btc, false) => "https://blockstream.info",
            (Chain::Btc, true) => "https://blockstream.info/testnet",
            (Chain::Xrp, false) => "https://xrpscan.com",
            (Chain::Xrp, true) => "https://testnet.xrpl.org",
            (Chain::Cardano, false) => "https://cardanoscan.io",
            (Chain::Cardano, true) => "https://preprod.cardanoscan.io",
        }
    }

    /// Alchemy network name for API calls. None if Alchemy doesn't support this chain.
    pub fn alchemy_network_name(&self, testnet: bool) -> Option<&'static str> {
        match (self, testnet) {
            (Chain::Eth, false) => Some("eth-mainnet"),
            (Chain::Eth, true) => Some("eth-sepolia"),
            (Chain::Polygon, false) => Some("polygon-mainnet"),
            (Chain::Polygon, true) => Some("polygon-amoy"),
            (Chain::Arbitrum, false) => Some("arb-mainnet"),
            (Chain::Arbitrum, true) => Some("arb-sepolia"),
            (Chain::Optimism, false) => Some("opt-mainnet"),
            (Chain::Optimism, true) => Some("opt-sepolia"),
            (Chain::Base, false) => Some("base-mainnet"),
            (Chain::Base, true) => Some("base-sepolia"),
            (Chain::Solana, false) => Some("solana-mainnet"),
            (Chain::Solana, true) => Some("solana-devnet"),
            _ => None,
        }
    }

    /// Emoji for Telegram UI display.
    pub fn emoji(&self) -> &'static str {
        match self {
            Chain::Eth => "\u{1f537}",      // 🔷
            Chain::Bsc => "\u{1f7e1}",      // 🟡
            Chain::Solana => "\u{1f7e3}",   // 🟣
            Chain::Polygon => "\u{1f7e3}",  // 🟣
            Chain::Avalanche => "\u{1f534}",// 🔴
            Chain::Arbitrum => "\u{1f535}",  // 🔵
            Chain::Optimism => "\u{1f534}", // 🔴
            Chain::Base => "\u{1f535}",     // 🔵
            Chain::Fantom => "\u{1f47b}",   // 👻
            Chain::Cronos => "\u{1f48e}",   // 💎
            Chain::Gnosis => "\u{1f7e2}",   // 🟢
            Chain::Btc => "\u{20bf}",       // ₿
            Chain::Xrp => "\u{2715}",       // ✕
            Chain::Cardano => "\u{20b3}",  // ₳
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Chain::Eth => "Ethereum",
            Chain::Bsc => "BSC",
            Chain::Solana => "Solana",
            Chain::Polygon => "Polygon",
            Chain::Avalanche => "Avalanche",
            Chain::Arbitrum => "Arbitrum",
            Chain::Optimism => "Optimism",
            Chain::Base => "Base",
            Chain::Fantom => "Fantom",
            Chain::Cronos => "Cronos",
            Chain::Gnosis => "Gnosis",
            Chain::Btc => "Bitcoin",
            Chain::Xrp => "XRP",
            Chain::Cardano => "Cardano",
        }
    }

    pub fn all() -> &'static [Chain] {
        &[
            Chain::Btc,
            Chain::Eth,
            Chain::Bsc,
            Chain::Polygon,
            Chain::Avalanche,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Base,
            Chain::Fantom,
            Chain::Cronos,
            Chain::Gnosis,
            Chain::Solana,
            Chain::Xrp,
            Chain::Cardano,
        ]
    }

    /// Only EVM chains.
    pub fn all_evm() -> &'static [Chain] {
        &[
            Chain::Eth,
            Chain::Bsc,
            Chain::Polygon,
            Chain::Avalanche,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Base,
            Chain::Fantom,
            Chain::Cronos,
            Chain::Gnosis,
        ]
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Chain {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ETH" | "ETHEREUM" => Ok(Chain::Eth),
            "BSC" | "BNB" => Ok(Chain::Bsc),
            "SOLANA" | "SOL" => Ok(Chain::Solana),
            "POLYGON" | "MATIC" | "POL" => Ok(Chain::Polygon),
            "AVAX" | "AVALANCHE" => Ok(Chain::Avalanche),
            "ARBITRUM" | "ARB" => Ok(Chain::Arbitrum),
            "OPTIMISM" | "OP" => Ok(Chain::Optimism),
            "BASE" => Ok(Chain::Base),
            "FANTOM" | "FTM" => Ok(Chain::Fantom),
            "CRONOS" | "CRO" => Ok(Chain::Cronos),
            "GNOSIS" | "XDAI" => Ok(Chain::Gnosis),
            "BTC" | "BITCOIN" => Ok(Chain::Btc),
            "XRP" | "RIPPLE" => Ok(Chain::Xrp),
            "ADA" | "CARDANO" => Ok(Chain::Cardano),
            _ => Err(AppError::InvalidInput(format!(
                "Unsupported chain: {}. Supported: BTC, ETH, BSC, SOLANA, POLYGON, AVAX, ARBITRUM, OPTIMISM, BASE, FANTOM, CRONOS, GNOSIS, XRP, ADA",
                s
            ))),
        }
    }
}

// ─── AlertType ───────────────────────────────────────────────────────

/// Price alert trigger condition.
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    Above { target_price: f64 },
    Below { target_price: f64 },
    PercentChange { percent: f64, base_price: f64 },
}

/// The discriminant stored in the database (no payload).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertKind {
    Above,
    Below,
    PercentChange,
}

impl AlertKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertKind::Above => "above",
            AlertKind::Below => "below",
            AlertKind::PercentChange => "percent_change",
        }
    }
}

impl fmt::Display for AlertKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AlertKind {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "above" => Ok(AlertKind::Above),
            "below" => Ok(AlertKind::Below),
            "percent_change" | "percent" => Ok(AlertKind::PercentChange),
            _ => Err(AppError::InvalidInput(format!(
                "Invalid alert type: {}. Supported: above, below, percent_change",
                s
            ))),
        }
    }
}

// ─── TransactionStatus ──────────────────────────────────────────────

/// Status of a blockchain transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
}

impl TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Pending => "pending",
            TxStatus::Confirmed => "confirmed",
            TxStatus::Failed => "failed",
        }
    }
}

impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TxStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TxStatus::Pending),
            "confirmed" => Ok(TxStatus::Confirmed),
            "failed" => Ok(TxStatus::Failed),
            _ => Err(AppError::InvalidInput(format!("Invalid tx status: {}", s))),
        }
    }
}

// ─── ScheduleStatus ─────────────────────────────────────────────────

/// Status of a scheduled transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleStatus {
    Pending,
    Executed,
    Failed,
    Cancelled,
}

impl ScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScheduleStatus::Pending => "pending",
            ScheduleStatus::Executed => "executed",
            ScheduleStatus::Failed => "failed",
            ScheduleStatus::Cancelled => "cancelled",
        }
    }
}

impl fmt::Display for ScheduleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ScheduleStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ScheduleStatus::Pending),
            "executed" => Ok(ScheduleStatus::Executed),
            "failed" => Ok(ScheduleStatus::Failed),
            "cancelled" => Ok(ScheduleStatus::Cancelled),
            _ => Err(AppError::InvalidInput(format!(
                "Invalid schedule status: {}",
                s
            ))),
        }
    }
}

// ─── RecurringType ──────────────────────────────────────────────────

/// Recurrence pattern for scheduled transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurringType {
    Daily,
    Weekly,
    Monthly,
}

impl RecurringType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecurringType::Daily => "daily",
            RecurringType::Weekly => "weekly",
            RecurringType::Monthly => "monthly",
        }
    }
}

impl fmt::Display for RecurringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RecurringType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(RecurringType::Daily),
            "weekly" => Ok(RecurringType::Weekly),
            "monthly" => Ok(RecurringType::Monthly),
            _ => Err(AppError::InvalidInput(format!(
                "Invalid recurring type: {}. Supported: daily, weekly, monthly",
                s
            ))),
        }
    }
}

// ─── SwapStatus ─────────────────────────────────────────────────────

/// Status of a token swap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapStatus {
    Pending,
    Success,
    Failed,
}

impl SwapStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SwapStatus::Pending => "pending",
            SwapStatus::Success => "success",
            SwapStatus::Failed => "failed",
        }
    }
}

impl fmt::Display for SwapStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SwapStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(SwapStatus::Pending),
            "success" => Ok(SwapStatus::Success),
            "failed" => Ok(SwapStatus::Failed),
            _ => Err(AppError::InvalidInput(format!("Invalid swap status: {}", s))),
        }
    }
}
