use super::{ DexProvider, SwapQuote, SwapResult };
use crate::error::{ AppError, Result };
use async_trait::async_trait;
use serde::{ Deserialize, Serialize };

// Jupiter API response structures
#[derive(Debug, Deserialize)]
struct JupiterQuoteResponse {
    #[serde(rename = "inAmount")]
    in_amount: String,
    #[serde(rename = "outAmount")]
    out_amount: String,
    #[serde(rename = "priceImpactPct")]
    price_impact_pct: f64,
    #[serde(rename = "marketInfos")]
    market_infos: Vec<MarketInfo>,
}

#[derive(Debug, Deserialize)]
struct MarketInfo {
    id: String,
    label: String,
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
}

#[derive(Debug, Serialize)]
struct JupiterSwapRequest {
    #[serde(rename = "userPublicKey")]
    user_public_key: String,
    #[serde(rename = "quoteResponse")]
    quote_response: serde_json::Value,
}

pub struct JupiterProvider {
    api_url: String,
    client: reqwest::Client,
}

impl JupiterProvider {
    pub fn new() -> Self {
        Self {
            api_url: "https://quote-api.jup.ag/v6".to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn get_token_decimals(&self, _mint: &str) -> Result<u8> {
        // In production, would query token metadata
        // For now, assume 9 decimals (SOL standard)
        Ok(9)
    }
}

#[async_trait]
impl DexProvider for JupiterProvider {
    async fn get_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: f64,
        slippage: f64
    ) -> Result<SwapQuote> {
        // Resolve token addresses (simplified - would need token list)
        let from_mint = self.resolve_token_mint(from_token)?;
        let to_mint = self.resolve_token_mint(to_token)?;

        // Convert amount to lamports/smallest unit
        let decimals = self.get_token_decimals(&from_mint).await?;
        let amount_lamports = (amount * (10_f64).powi(decimals as i32)) as u64;

        // Get quote from Jupiter API
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.api_url,
            from_mint,
            to_mint,
            amount_lamports,
            (slippage * 100.0) as u32
        );

        let response = self.client
            .get(&url)
            .send().await
            .map_err(|e| AppError::External(format!("Jupiter API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(
                AppError::External(format!("Jupiter API returned error: {}", response.status()))
            );
        }

        let quote: JupiterQuoteResponse = response
            .json().await
            .map_err(|e| AppError::External(format!("Failed to parse Jupiter response: {}", e)))?;

        let out_decimals = self.get_token_decimals(&to_mint).await?;
        let expected_to_amount =
            (quote.out_amount.parse::<u64>().unwrap_or(0) as f64) /
            (10_f64).powi(out_decimals as i32);
        let minimum_to_amount = expected_to_amount * (1.0 - slippage / 100.0);

        let route = quote.market_infos
            .iter()
            .map(|m| m.label.clone())
            .collect();

        Ok(SwapQuote {
            from_token: from_token.to_string(),
            from_token_address: Some(from_mint.clone()),
            to_token: to_token.to_string(),
            to_token_address: Some(to_mint.clone()),
            from_amount: amount,
            expected_to_amount,
            minimum_to_amount,
            price_impact: quote.price_impact_pct,
            route,
            estimated_gas: Some("5000".to_string()), // SOL lamports for transaction
            dex: self.name().to_string(),
        })
    }

    async fn execute_swap(
        &self,
        wallet_address: &str,
        private_key: &str,
        from_token: &str,
        to_token: &str,
        amount: f64,
        slippage: f64,
        min_output: f64
    ) -> Result<SwapResult> {
        // First get quote
        let quote = self.get_quote(from_token, to_token, amount, slippage).await?;

        // Get swap transaction from Jupiter
        let url = format!("{}/swap", self.api_url);

        let swap_request = JupiterSwapRequest {
            user_public_key: wallet_address.to_string(),
            quote_response: serde_json::json!({
                "inputMint": quote.from_token_address,
                "outputMint": quote.to_token_address,
                "inAmount": (amount * 1e9) as u64,
                "outAmount": (quote.expected_to_amount * 1e9) as u64,
            }),
        };

        let response = self.client
            .post(&url)
            .json(&swap_request)
            .send().await
            .map_err(|e| AppError::External(format!("Jupiter swap API error: {}", e)))?;

        let swap_response: serde_json::Value = response
            .json().await
            .map_err(|e| AppError::External(format!("Failed to parse swap response: {}", e)))?;

        // In production, would:
        // 1. Parse the transaction from response
        // 2. Sign with Solana keypair
        // 3. Send to Solana network
        // 4. Wait for confirmation

        // For now, return placeholder
        Ok(SwapResult {
            tx_hash: "SOLANA_TX_HASH_PLACEHOLDER".to_string(),
            from_amount: amount,
            to_amount: quote.expected_to_amount,
            gas_used: Some("5000".to_string()),
        })
    }

    fn name(&self) -> &str {
        "Jupiter"
    }

    fn supported_chains(&self) -> Vec<&str> {
        vec!["SOLANA"]
    }
}

impl JupiterProvider {
    fn resolve_token_mint(&self, token: &str) -> Result<String> {
        // Common Solana token mints
        let mint = match token.to_uppercase().as_str() {
            "SOL" => "So11111111111111111111111111111111111111112",
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "USDT" => "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            _ => token, // Assume it's already a mint address
        };
        Ok(mint.to_string())
    }
}
