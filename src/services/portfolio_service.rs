use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;

use crate::db::WalletRepository;
use crate::enums::Chain;
use crate::error::Result;
use crate::rpc::RpcManager;
use crate::services::price_service::PriceService;
use crate::services::token_discovery_service::TokenDiscoveryService;

pub struct PortfolioService {
    wallet_repo: Arc<WalletRepository>,
    rpc_manager: Arc<RpcManager>,
    price_service: Arc<PriceService>,
    token_discovery: Option<Arc<TokenDiscoveryService>>,
    is_testnet: bool,
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
    pub name: Option<String>,
    pub total_balance: f64,
    pub usd_value: f64,
    pub usd_price: f64,
    pub price_change_24h: Option<f64>,
    pub logo_url: Option<String>,
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
        price_service: Arc<PriceService>,
        token_discovery: Option<Arc<TokenDiscoveryService>>,
        is_testnet: bool,
    ) -> Self {
        Self {
            wallet_repo,
            rpc_manager,
            price_service,
            token_discovery,
            is_testnet,
        }
    }

    /// Get complete portfolio for a user across all chains and wallets.
    pub async fn get_portfolio(&self, user_id: &str) -> Result<Portfolio> {
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

        for wallet in &wallets {
            chains_set.insert(wallet.chain.clone());

            let provider = match self.rpc_manager.get_provider_by_chain(&wallet.chain).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("No provider for chain {}: {}", wallet.chain, e);
                    continue;
                }
            };

            // Native token balance
            let chain_parsed = wallet.chain.parse::<Chain>().ok();
            let native_symbol = chain_parsed
                .map(|c| c.native_symbol())
                .unwrap_or(&wallet.chain);

            match provider.get_balance(&wallet.address).await {
                Ok(balance) => {
                    let balance_float: f64 = balance.balance.parse().unwrap_or(0.0);
                    let entry = holdings_map
                        .entry(native_symbol.to_string())
                        .or_insert_with(|| TokenHolding {
                            symbol: native_symbol.to_string(),
                            name: chain_parsed.map(|c| c.display_name().to_string()),
                            total_balance: 0.0,
                            usd_value: 0.0,
                            usd_price: 0.0,
                            price_change_24h: None,
                            logo_url: None,
                            wallets: vec![],
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

            // Token discovery (ERC-20 / SPL tokens)
            if let Some(ref discovery) = self.token_discovery {
                if let Some(chain) = chain_parsed {
                    if discovery.is_supported(&chain) {
                        match discovery
                            .get_all_token_balances(chain, &wallet.address, self.is_testnet)
                            .await
                        {
                            Ok(tokens) => {
                                for token in tokens {
                                    let balance_float: f64 =
                                        token.balance.parse().unwrap_or(0.0);
                                    if balance_float == 0.0 {
                                        continue;
                                    }

                                    let entry = holdings_map
                                        .entry(token.symbol.clone())
                                        .or_insert_with(|| TokenHolding {
                                            symbol: token.symbol.clone(),
                                            name: Some(token.name.clone()),
                                            total_balance: 0.0,
                                            usd_value: 0.0,
                                            usd_price: 0.0,
                                            price_change_24h: None,
                                            logo_url: token.logo_url.clone(),
                                            wallets: vec![],
                                        });

                                    entry.total_balance += balance_float;
                                    if entry.logo_url.is_none() && token.logo_url.is_some() {
                                        entry.logo_url = token.logo_url;
                                    }
                                    entry.wallets.push(WalletHolding {
                                        wallet_id: wallet.id.to_string(),
                                        chain: wallet.chain.clone(),
                                        address: wallet.address.clone(),
                                        balance: token.balance,
                                    });
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Token discovery failed for wallet {} on {}: {}",
                                    wallet.id,
                                    wallet.chain,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        // Fetch prices
        let symbols: Vec<String> = holdings_map.keys().cloned().collect();
        let prices = self.price_service.get_prices(&symbols).await.unwrap_or_default();

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
        holdings.sort_by(|a, b| {
            b.usd_value
                .partial_cmp(&a.usd_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(Portfolio {
            user_id: user_id.to_string(),
            holdings,
            total_usd_value,
            chains: chains_set.into_iter().collect(),
            wallet_count: wallets.len(),
        })
    }

    /// Get portfolio for a specific chain.
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
        let chain_parsed = chain.parse::<Chain>().ok();
        let symbol = chain_parsed
            .map(|c| c.native_symbol())
            .unwrap_or(chain);

        let mut total_balance = 0.0;
        let mut wallet_holdings = vec![];

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

        let price = self.price_service.get_price(symbol).await.ok();
        let (usd_price, usd_value, price_change_24h) = if let Some(p) = price {
            (p.usd_price, total_balance * p.usd_price, p.price_change_24h)
        } else {
            (0.0, 0.0, None)
        };

        let holding = TokenHolding {
            symbol: symbol.to_string(),
            name: chain_parsed.map(|c| c.display_name().to_string()),
            total_balance,
            usd_value,
            usd_price,
            price_change_24h,
            logo_url: None,
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
