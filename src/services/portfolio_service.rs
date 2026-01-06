use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use crate::db::WalletRepository;
use crate::error::Result;
use crate::rpc::RpcManager;
use crate::services::price_service::{ PriceService, TokenPrice };

pub struct PortfolioService {
    wallet_repo: Arc<WalletRepository>,
    rpc_manager: Arc<RpcManager>,
    price_service: Arc<PriceService>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletHolding {
    pub wallet_id: String,
    pub chain: String,
    pub address: String,
    pub balance: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenHolding {
    pub symbol: String,
    pub total_balance: f64,
    pub usd_value: f64,
    pub usd_price: f64,
    pub price_change_24h: Option<f64>,
    pub wallets: Vec<WalletHolding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Portfolio {
    pub user_id: String,
    pub holdings: Vec<TokenHolding>,
    pub total_usd_value: f64,
    pub chains: Vec<String>,
    pub wallet_count: usize,
}

impl PortfolioService {
    pub fn new(
        wallet_repo: Arc<WalletRepository>,
        rpc_manager: Arc<RpcManager>,
        price_service: Arc<PriceService>
    ) -> Self {
        Self {
            wallet_repo,
            rpc_manager,
            price_service,
        }
    }

    /// Get complete portfolio for a user across all chains and wallets
    pub async fn get_portfolio(&self, user_id: &str) -> Result<Portfolio> {
        // Get all user's wallets
        let wallets = self.wallet_repo.find_by_user(user_id).await?;

        if wallets.is_empty() {
            return Ok(Portfolio {
                user_id: user_id.to_string(),
                holdings: vec![],
                total_usd_value: 0.0,
                chains: vec![],
                wallet_count: 0,
            });
        }

        let mut holdings_map: HashMap<String, TokenHolding> = HashMap::new();
        let mut chains_set = std::collections::HashSet::new();

        // Fetch balances for each wallet
        for wallet in &wallets {
            chains_set.insert(wallet.chain.clone());

            let provider = self.rpc_manager.get_provider_by_chain(&wallet.chain).await?;

            // Get native token balance
            match provider.get_balance(&wallet.address).await {
                Ok(balance) => {
                    let balance_float: f64 = balance.balance.parse().unwrap_or(0.0);

                    let symbol = match wallet.chain.as_str() {
                        "ETH" => "ETH",
                        "BSC" => "BNB",
                        "SOLANA" => "SOL",
                        _ => &wallet.chain,
                    };

                    let entry = holdings_map.entry(symbol.to_string()).or_insert_with(|| {
                        TokenHolding {
                            symbol: symbol.to_string(),
                            total_balance: 0.0,
                            usd_value: 0.0,
                            usd_price: 0.0,
                            price_change_24h: None,
                            wallets: vec![],
                        }
                    });

                    entry.total_balance += balance_float;
                    entry.wallets.push(WalletHolding {
                        wallet_id: wallet.id.to_string(),
                        chain: wallet.chain.clone(),
                        address: wallet.address.clone(),
                        balance: balance.balance.clone(),
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to get balance for wallet {}: {}", wallet.id, e);
                }
            }
        }

        // Fetch prices for all tokens
        let symbols: Vec<String> = holdings_map.keys().cloned().collect();
        let prices = self.price_service.get_prices(&symbols).await.unwrap_or_default();

        // Calculate USD values
        let mut total_usd_value = 0.0;
        for (symbol, holding) in holdings_map.iter_mut() {
            if let Some(price) = prices.get(symbol) {
                holding.usd_price = price.usd_price;
                holding.usd_value = holding.total_balance * price.usd_price;
                holding.price_change_24h = price.price_change_24h;
                total_usd_value += holding.usd_value;
            }
        }

        let mut holdings: Vec<TokenHolding> = holdings_map.into_values().collect();
        holdings.sort_by(|a, b| b.usd_value.partial_cmp(&a.usd_value).unwrap());

        Ok(Portfolio {
            user_id: user_id.to_string(),
            holdings,
            total_usd_value,
            chains: chains_set.into_iter().collect(),
            wallet_count: wallets.len(),
        })
    }

    /// Get portfolio for a specific chain
    pub async fn get_chain_portfolio(&self, user_id: &str, chain: &str) -> Result<Portfolio> {
        let wallets = self.wallet_repo.find_by_user_and_chain(user_id, chain).await?;

        if wallets.is_empty() {
            return Ok(Portfolio {
                user_id: user_id.to_string(),
                holdings: vec![],
                total_usd_value: 0.0,
                chains: vec![],
                wallet_count: 0,
            });
        }

        let provider = self.rpc_manager.get_provider_by_chain(chain).await?;
        let symbol = match chain {
            "ETH" => "ETH",
            "BSC" => "BNB",
            "SOLANA" => "SOL",
            _ => chain,
        };

        let mut total_balance = 0.0;
        let mut wallet_holdings = vec![];

        // Fetch balances for each wallet
        for wallet in &wallets {
            match provider.get_balance(&wallet.address).await {
                Ok(balance) => {
                    let balance_float: f64 = balance.balance.parse().unwrap_or(0.0);
                    total_balance += balance_float;

                    wallet_holdings.push(WalletHolding {
                        wallet_id: wallet.id.to_string(),
                        chain: wallet.chain.clone(),
                        address: wallet.address.clone(),
                        balance: balance.balance.clone(),
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to get balance for wallet {}: {}", wallet.id, e);
                }
            }
        }

        // Get price
        let price = self.price_service.get_price(symbol).await.ok();
        let (usd_price, usd_value, price_change_24h) = if let Some(p) = price {
            (p.usd_price, total_balance * p.usd_price, p.price_change_24h)
        } else {
            (0.0, 0.0, None)
        };

        let holding = TokenHolding {
            symbol: symbol.to_string(),
            total_balance,
            usd_value,
            usd_price,
            price_change_24h,
            wallets: wallet_holdings,
        };

        Ok(Portfolio {
            user_id: user_id.to_string(),
            holdings: vec![holding],
            total_usd_value: usd_value,
            chains: vec![chain.to_string()],
            wallet_count: wallets.len(),
        })
    }
}
