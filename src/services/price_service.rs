use std::collections::HashMap;
use std::sync::Arc;
use std::time::{ Duration, SystemTime };
use tokio::sync::RwLock;
use serde::{ Deserialize, Serialize };
use crate::error::{ AppError, Result };

const COINGECKO_API_BASE: &str = "https://api.coingecko.com/api/v3";
const CACHE_DURATION_SECS: u64 = 60; // Cache prices for 1 minute

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
struct CoinGeckoSimplePrice {
    usd: f64,
    usd_24h_change: Option<f64>,
    usd_market_cap: Option<f64>,
    usd_24h_vol: Option<f64>,
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

        // Fetch from API
        let coin_id = self.symbol_to_coingecko_id(&symbol_upper);
        let price = self.fetch_price_from_api(&coin_id, &symbol_upper).await?;

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

    /// Get price for a token by contract address (for ERC20/BEP20 tokens)
    pub async fn get_token_price_by_address(
        &self,
        chain: &str,
        address: &str
    ) -> Result<TokenPrice> {
        let platform = match chain {
            "ETH" | "ETHEREUM" => "ethereum",
            "BSC" | "BNB" => "binance-smart-chain",
            _ => {
                return Err(
                    AppError::InvalidInput(format!("Unsupported chain for token price: {}", chain))
                );
            }
        };

        let url = format!(
            "{}/simple/token_price/{}?contract_addresses={}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&include_24hr_vol=true",
            COINGECKO_API_BASE,
            platform,
            address.to_lowercase()
        );

        let response = self.client
            .get(&url)
            .send().await
            .map_err(|e| AppError::External(format!("CoinGecko API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(
                AppError::External(format!("CoinGecko API returned status: {}", response.status()))
            );
        }

        let data: HashMap<String, CoinGeckoSimplePrice> = response
            .json().await
            .map_err(|e| AppError::External(format!("Failed to parse CoinGecko response: {}", e)))?;

        let price_data = data
            .get(&address.to_lowercase())
            .ok_or_else(|| AppError::External("Token price not found".to_string()))?;

        Ok(TokenPrice {
            symbol: "TOKEN".to_string(),
            usd_price: price_data.usd,
            price_change_24h: price_data.usd_24h_change,
            market_cap: price_data.usd_market_cap,
            volume_24h: price_data.usd_24h_vol,
            last_updated: SystemTime::now(),
        })
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

    async fn fetch_price_from_api(&self, coin_id: &str, symbol: &str) -> Result<TokenPrice> {
        let url = format!(
            "{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&include_24hr_vol=true",
            COINGECKO_API_BASE,
            coin_id
        );

        let response = self.client
            .get(&url)
            .send().await
            .map_err(|e| AppError::External(format!("CoinGecko API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(
                AppError::External(format!("CoinGecko API returned status: {}", response.status()))
            );
        }

        let data: HashMap<String, CoinGeckoSimplePrice> = response
            .json().await
            .map_err(|e| AppError::External(format!("Failed to parse CoinGecko response: {}", e)))?;

        let price_data = data
            .get(coin_id)
            .ok_or_else(|| AppError::External("Price not found".to_string()))?;

        Ok(TokenPrice {
            symbol: symbol.to_string(),
            usd_price: price_data.usd,
            price_change_24h: price_data.usd_24h_change,
            market_cap: price_data.usd_market_cap,
            volume_24h: price_data.usd_24h_vol,
            last_updated: SystemTime::now(),
        })
    }

    async fn fetch_multiple_prices(
        &self,
        symbols: &[String]
    ) -> Result<HashMap<String, TokenPrice>> {
        let coin_ids: Vec<String> = symbols
            .iter()
            .map(|s| self.symbol_to_coingecko_id(s))
            .collect();

        let ids_str = coin_ids.join(",");
        let url = format!(
            "{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&include_24hr_vol=true",
            COINGECKO_API_BASE,
            ids_str
        );

        let response = self.client
            .get(&url)
            .send().await
            .map_err(|e| AppError::External(format!("CoinGecko API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(
                AppError::External(format!("CoinGecko API returned status: {}", response.status()))
            );
        }

        let data: HashMap<String, CoinGeckoSimplePrice> = response
            .json().await
            .map_err(|e| AppError::External(format!("Failed to parse CoinGecko response: {}", e)))?;

        let mut results = HashMap::new();
        for (i, symbol) in symbols.iter().enumerate() {
            if let Some(price_data) = data.get(&coin_ids[i]) {
                results.insert(symbol.clone(), TokenPrice {
                    symbol: symbol.clone(),
                    usd_price: price_data.usd,
                    price_change_24h: price_data.usd_24h_change,
                    market_cap: price_data.usd_market_cap,
                    volume_24h: price_data.usd_24h_vol,
                    last_updated: SystemTime::now(),
                });
            }
        }

        Ok(results)
    }

    fn symbol_to_coingecko_id(&self, symbol: &str) -> String {
        let id = match symbol {
            "ETH" | "ETHEREUM" => "ethereum",
            "BNB" | "BSC" => "binancecoin",
            "SOL" | "SOLANA" => "solana",
            "BTC" | "BITCOIN" => "bitcoin",
            "USDT" => "tether",
            "USDC" => "usd-coin",
            "DAI" => "dai",
            "WETH" => "weth",
            "WBNB" => "wbnb",
            _ => {
                return symbol.to_lowercase();
            }
        };
        id.to_string()
    }
}

impl Default for PriceService {
    fn default() -> Self {
        Self::new()
    }
}
