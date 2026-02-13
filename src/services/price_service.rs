use std::collections::HashMap;
use std::sync::Arc;
use std::time::{ Duration, SystemTime };
use tokio::sync::RwLock;
use serde::{ Deserialize, Serialize };
use crate::error::{ AppError, Result };

const BINANCE_API_BASE: &str = "https://api.binance.com/api/v3";
const CACHE_DURATION_SECS: u64 = 60; // Cache prices for 1 minute
const MAX_RETRIES: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub symbol: String,
    pub usd_price: f64,
    pub price_change_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub volume_24h: Option<f64>,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone)]
struct CachedPrice {
    price: TokenPrice,
    fetched_at: SystemTime,
}

pub struct PriceService {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CachedPrice>>>,
}

#[derive(Deserialize)]
struct BinanceTicker24hr {
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "priceChangePercent")]
    price_change_percent: String,
    #[serde(rename = "quoteVolume")]
    quote_volume: String,
}

impl PriceService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder().timeout(Duration::from_secs(10)).build().unwrap(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get price for a single token by symbol (ETH, BNB, SOL, etc.)
    pub async fn get_price(&self, symbol: &str) -> Result<TokenPrice> {
        let symbol_upper = symbol.to_uppercase();

        // Check cache first
        if let Some(cached) = self.get_from_cache(&symbol_upper).await {
            return Ok(cached);
        }

        // Map to Binance trading pair
        let binance_symbol = self
            .symbol_to_binance_pair(&symbol_upper)
            .ok_or_else(|| AppError::InvalidInput(format!("Unknown token symbol: {}", symbol)))?;

        let price = self.fetch_ticker_24hr(&binance_symbol, &symbol_upper).await?;

        // Update cache
        self.update_cache(symbol_upper, price.clone()).await;

        Ok(price)
    }

    /// Get prices for multiple tokens at once
    pub async fn get_prices(&self, symbols: &[String]) -> Result<HashMap<String, TokenPrice>> {
        let mut results = HashMap::new();
        let mut symbols_to_fetch = Vec::new();

        // Check cache for each symbol
        for symbol in symbols {
            let symbol_upper = symbol.to_uppercase();
            if let Some(cached) = self.get_from_cache(&symbol_upper).await {
                results.insert(symbol_upper, cached);
            } else {
                symbols_to_fetch.push(symbol_upper);
            }
        }

        // Fetch missing prices from API
        if !symbols_to_fetch.is_empty() {
            let fetched = self.fetch_multiple_prices(&symbols_to_fetch).await?;
            for (symbol, price) in fetched {
                self.update_cache(symbol.clone(), price.clone()).await;
                results.insert(symbol, price);
            }
        }

        Ok(results)
    }

    /// Get price for a token by contract address.
    /// Binance doesn't support contract address lookups directly,
    /// so we try to resolve known token addresses to symbols.
    pub async fn get_token_price_by_address(
        &self,
        _chain: &str,
        _address: &str
    ) -> Result<TokenPrice> {
        Err(AppError::External(
            "Token price by contract address is not supported with Binance API. Use token symbol instead.".to_string()
        ))
    }

    async fn get_from_cache(&self, symbol: &str) -> Option<TokenPrice> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(symbol) {
            let age = SystemTime::now()
                .duration_since(cached.fetched_at)
                .unwrap_or(Duration::from_secs(999));

            if age.as_secs() < CACHE_DURATION_SECS {
                return Some(cached.price.clone());
            }
        }
        None
    }

    async fn update_cache(&self, symbol: String, price: TokenPrice) {
        let mut cache = self.cache.write().await;
        cache.insert(symbol, CachedPrice {
            price,
            fetched_at: SystemTime::now(),
        });
    }

    /// Fetch a URL with retry on 429 rate-limit responses
    async fn fetch_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        let mut last_err = None;
        for attempt in 0..MAX_RETRIES {
            let response = self.client.get(url)
                .send().await
                .map_err(|e| AppError::External(format!("Binance API error: {}", e)))?;

            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let wait_secs = 2u64.pow(attempt + 1);
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                last_err = Some(AppError::External("Binance rate limited".to_string()));
                continue;
            }

            if !response.status().is_success() {
                return Err(
                    AppError::External(format!("Binance API returned status: {}", response.status()))
                );
            }

            return Ok(response);
        }
        Err(last_err.unwrap_or_else(|| AppError::External("Binance API request failed after retries".to_string())))
    }

    async fn fetch_ticker_24hr(&self, binance_symbol: &str, symbol: &str) -> Result<TokenPrice> {
        // Stablecoins pegged to USD
        if matches!(symbol, "USDT" | "USDC" | "DAI" | "BUSD") {
            return Ok(TokenPrice {
                symbol: symbol.to_string(),
                usd_price: 1.0,
                price_change_24h: Some(0.0),
                market_cap: None,
                volume_24h: None,
                last_updated: SystemTime::now(),
            });
        }

        let url = format!(
            "{}/ticker/24hr?symbol={}",
            BINANCE_API_BASE,
            binance_symbol
        );

        let response = self.fetch_with_retry(&url).await?;

        let ticker: BinanceTicker24hr = response
            .json().await
            .map_err(|e| AppError::External(format!("Failed to parse Binance response: {}", e)))?;

        let usd_price: f64 = ticker.last_price.parse().unwrap_or(0.0);
        let price_change_pct: f64 = ticker.price_change_percent.parse().unwrap_or(0.0);
        let volume: f64 = ticker.quote_volume.parse().unwrap_or(0.0);

        Ok(TokenPrice {
            symbol: symbol.to_string(),
            usd_price,
            price_change_24h: Some(price_change_pct),
            market_cap: None, // Binance doesn't provide market cap
            volume_24h: Some(volume),
            last_updated: SystemTime::now(),
        })
    }

    async fn fetch_multiple_prices(
        &self,
        symbols: &[String]
    ) -> Result<HashMap<String, TokenPrice>> {
        // Build list of Binance pairs we need
        let pairs: Vec<(&String, String)> = symbols
            .iter()
            .filter_map(|s| self.symbol_to_binance_pair(s).map(|pair| (s, pair)))
            .collect();

        if pairs.is_empty() {
            return Ok(HashMap::new());
        }

        // Build the symbols JSON array for batch request
        let binance_symbols: Vec<String> = pairs.iter().map(|(_, p)| format!("\"{}\"", p)).collect();
        let symbols_param = format!("[{}]", binance_symbols.join(","));

        let url = format!(
            "{}/ticker/24hr?symbols={}",
            BINANCE_API_BASE,
            urlencoding::encode(&symbols_param)
        );

        let response = self.fetch_with_retry(&url).await?;

        let tickers: Vec<BinanceTicker24hr> = response
            .json().await
            .map_err(|e| AppError::External(format!("Failed to parse Binance response: {}", e)))?;

        // Build a map from Binance symbol -> ticker
        let ticker_map: HashMap<String, &BinanceTicker24hr> = tickers
            .iter()
            .map(|t| (t.symbol.clone(), t))
            .collect();

        let mut results = HashMap::new();
        for (symbol, binance_pair) in &pairs {
            // Handle stablecoins
            if matches!(symbol.as_str(), "USDT" | "USDC" | "DAI" | "BUSD") {
                results.insert((*symbol).clone(), TokenPrice {
                    symbol: (*symbol).clone(),
                    usd_price: 1.0,
                    price_change_24h: Some(0.0),
                    market_cap: None,
                    volume_24h: None,
                    last_updated: SystemTime::now(),
                });
                continue;
            }

            if let Some(ticker) = ticker_map.get(binance_pair) {
                let usd_price: f64 = ticker.last_price.parse().unwrap_or(0.0);
                let price_change_pct: f64 = ticker.price_change_percent.parse().unwrap_or(0.0);
                let volume: f64 = ticker.quote_volume.parse().unwrap_or(0.0);

                results.insert((*symbol).clone(), TokenPrice {
                    symbol: (*symbol).clone(),
                    usd_price,
                    price_change_24h: Some(price_change_pct),
                    market_cap: None,
                    volume_24h: Some(volume),
                    last_updated: SystemTime::now(),
                });
            }
        }

        Ok(results)
    }

    /// Map a token symbol to a Binance USDT trading pair.
    fn symbol_to_binance_pair(&self, symbol: &str) -> Option<String> {
        // Stablecoins — return a dummy pair; handled specially in fetch methods
        if matches!(symbol, "USDT" | "USDC" | "DAI" | "BUSD") {
            return Some(format!("{}USDT", symbol));
        }

        let base = match symbol {
            "ETH" | "ETHEREUM" => "ETH",
            "BNB" | "BSC" => "BNB",
            "SOL" | "SOLANA" => "SOL",
            "BTC" | "BITCOIN" => "BTC",
            "WETH" => "ETH",
            "WBNB" => "BNB",
            "MATIC" | "POL" => "MATIC",
            "AVAX" => "AVAX",
            "LINK" => "LINK",
            "UNI" => "UNI",
            "AAVE" => "AAVE",
            "SHIB" => "SHIB",
            "DOGE" => "DOGE",
            "DOT" => "DOT",
            "ADA" => "ADA",
            "XRP" => "XRP",
            "ARB" => "ARB",
            "OP" => "OP",
            "APT" => "APT",
            "SUI" => "SUI",
            "ATOM" => "ATOM",
            "NEAR" => "NEAR",
            "FTM" => "FTM",
            "CRO" | "CRONOS" => "CRO",
            "XDAI" | "GNOSIS" => return None, // Not on Binance
            "CRV" => "CRV",
            "MKR" => "MKR",
            "LDO" => "LDO",
            "PEPE" => "PEPE",
            "WIF" => "WIF",
            "JUP" => "JUP",
            "BONK" => "BONK",
            "RAY" => "RAY",
            "CAKE" => "CAKE",
            "INJ" => "INJ",
            "TIA" => "TIA",
            "RENDER" | "RNDR" => "RENDER",
            "FET" => "FET",
            "GRT" => "GRT",
            "SNX" => "SNX",
            "COMP" => "COMP",
            "SUSHI" => "SUSHI",
            "1INCH" => "1INCH",
            _ => return None,
        };
        Some(format!("{}USDT", base))
    }
}

impl Default for PriceService {
    fn default() -> Self {
        Self::new()
    }
}
