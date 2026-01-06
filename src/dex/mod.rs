use async_trait::async_trait;
use crate::error::Result;
use serde::{ Deserialize, Serialize };

pub mod uniswap;
pub mod jupiter;

/// Swap quote information returned by DEX providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub from_token: String,
    pub from_token_address: Option<String>,
    pub to_token: String,
    pub to_token_address: Option<String>,
    pub from_amount: f64,
    pub expected_to_amount: f64,
    pub minimum_to_amount: f64, // After slippage
    pub price_impact: f64, // Percentage
    pub route: Vec<String>, // Token addresses in the swap route
    pub estimated_gas: Option<String>,
    pub dex: String,
}

/// Swap execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub tx_hash: String,
    pub from_amount: f64,
    pub to_amount: f64,
    pub gas_used: Option<String>,
}

/// Trait for DEX providers (Uniswap, PancakeSwap, Jupiter, etc.)
#[async_trait]
pub trait DexProvider: Send + Sync {
    /// Get a quote for swapping tokens
    async fn get_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: f64,
        slippage: f64
    ) -> Result<SwapQuote>;

    /// Execute a token swap
    async fn execute_swap(
        &self,
        wallet_address: &str,
        private_key: &str,
        from_token: &str,
        to_token: &str,
        amount: f64,
        slippage: f64,
        min_output: f64
    ) -> Result<SwapResult>;

    /// Get the DEX name
    fn name(&self) -> &str;

    /// Get supported chains
    fn supported_chains(&self) -> Vec<&str>;
}
