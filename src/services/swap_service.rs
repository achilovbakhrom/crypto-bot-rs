use crate::db::entity::swap;
use crate::dex::{ DexProvider, SwapQuote };
use crate::dex::uniswap::UniswapV2Provider;
use crate::dex::jupiter::JupiterProvider;
use crate::enums::{ Chain, SwapStatus };
use crate::error::{ AppError, Result };
use crate::services::WalletService;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue,
    ColumnTrait,
    DatabaseConnection,
    EntityTrait,
    QueryFilter,
    QueryOrder,
    prelude::Decimal,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct SwapService {
    db: DatabaseConnection,
    wallet_service: Arc<WalletService>,
}

#[derive(Debug, Clone)]
pub struct SwapRequest {
    pub user_id: String,
    pub wallet_id: Uuid,
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
    pub slippage: f64, // Percentage (e.g., 1.0 for 1%)
}

#[derive(Debug, Clone)]
pub struct SwapQuoteRequest {
    pub chain: String,
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
    pub slippage: f64,
}

impl SwapService {
    pub fn new(db: DatabaseConnection, wallet_service: Arc<WalletService>) -> Self {
        Self { db, wallet_service }
    }

    /// Get swap quote from appropriate DEX
    pub async fn get_swap_quote(&self, request: SwapQuoteRequest) -> Result<SwapQuote> {
        let provider = self.get_dex_provider(&request.chain)?;

        provider.get_quote(
            &request.from_token,
            &request.to_token,
            request.amount,
            request.slippage
        ).await
    }

    /// Execute a token swap
    pub async fn execute_swap(&self, request: SwapRequest) -> Result<swap::Model> {
        // Get wallet details
        let wallet = self.wallet_service.get_wallet(request.wallet_id).await?;

        // Verify wallet belongs to user
        if wallet.user_id != request.user_id {
            return Err(AppError::Validation("Wallet does not belong to this user".to_string()));
        }

        // Get DEX provider for chain
        let provider = self.get_dex_provider(&wallet.chain)?;

        // Get quote first
        let quote = provider.get_quote(
            &request.from_token,
            &request.to_token,
            request.amount,
            request.slippage
        ).await?;

        // Validate price impact
        if quote.price_impact > 5.0 {
            return Err(
                AppError::Validation(format!("Price impact too high: {:.2}%", quote.price_impact))
            );
        }

        // Create pending swap record
        let swap_id = Uuid::new_v4();
        let swap_entity = swap::ActiveModel {
            id: ActiveValue::Set(swap_id),
            user_id: ActiveValue::Set(request.user_id.clone()),
            wallet_id: ActiveValue::Set(request.wallet_id),
            chain: ActiveValue::Set(wallet.chain.clone()),
            dex: ActiveValue::Set(provider.name().to_string()),
            from_token: ActiveValue::Set(request.from_token.clone()),
            from_token_address: ActiveValue::Set(quote.from_token_address.clone()),
            to_token: ActiveValue::Set(request.to_token.clone()),
            to_token_address: ActiveValue::Set(quote.to_token_address.clone()),
            from_amount: ActiveValue::Set(Decimal::from_f64_retain(request.amount).unwrap()),
            to_amount: ActiveValue::Set(Decimal::from_f64_retain(0.0).unwrap()),
            expected_to_amount: ActiveValue::Set(
                Some(Decimal::from_f64_retain(quote.expected_to_amount).unwrap())
            ),
            price_impact: ActiveValue::Set(
                Some(Decimal::from_f64_retain(quote.price_impact).unwrap())
            ),
            slippage: ActiveValue::Set(Decimal::from_f64_retain(request.slippage).unwrap()),
            tx_hash: ActiveValue::Set(None),
            status: ActiveValue::Set(SwapStatus::Pending.to_string()),
            error_message: ActiveValue::Set(None),
            gas_fee: ActiveValue::Set(None),
            route: ActiveValue::Set(Some(serde_json::json!(quote.route))),
            created_at: ActiveValue::Set(chrono::Utc::now()),
            updated_at: ActiveValue::Set(chrono::Utc::now()),
        };

        let swap_model = swap_entity.insert(&self.db).await.map_err(|e| AppError::Database(e))?;

        // Execute swap
        // Note: In production, this should decrypt the private key properly
        match
            provider.execute_swap(
                &wallet.address,
                "ENCRYPTED_KEY_PLACEHOLDER", // Would decrypt wallet.encrypted_private_key
                &request.from_token,
                &request.to_token,
                request.amount,
                request.slippage,
                quote.minimum_to_amount
            ).await
        {
            Ok(result) => {
                // Update swap record with success
                let mut swap_active: swap::ActiveModel = swap_model.clone().into();
                swap_active.status = ActiveValue::Set(SwapStatus::Success.to_string());
                swap_active.tx_hash = ActiveValue::Set(Some(result.tx_hash.clone()));
                swap_active.to_amount = ActiveValue::Set(
                    Decimal::from_f64_retain(result.to_amount).unwrap()
                );
                swap_active.gas_fee = result.gas_used
                    .map(|g| {
                        ActiveValue::Set(
                            Some(Decimal::from_f64_retain(g.parse::<f64>().unwrap_or(0.0)).unwrap())
                        )
                    })
                    .unwrap_or(ActiveValue::NotSet);
                swap_active.updated_at = ActiveValue::Set(chrono::Utc::now());

                swap_active.update(&self.db).await.map_err(|e| AppError::Database(e))
            }
            Err(e) => {
                // Update swap record with failure
                let mut swap_active: swap::ActiveModel = swap_model.clone().into();
                swap_active.status = ActiveValue::Set(SwapStatus::Failed.to_string());
                swap_active.error_message = ActiveValue::Set(Some(e.to_string()));
                swap_active.updated_at = ActiveValue::Set(chrono::Utc::now());

                let failed_swap = swap_active
                    .update(&self.db).await
                    .map_err(|e| AppError::Database(e))?;

                Err(e)
            }
        }
    }

    /// Get swap history for a user
    pub async fn get_swap_history(
        &self,
        user_id: &str,
        wallet_id: Option<Uuid>,
        limit: Option<u64>
    ) -> Result<Vec<swap::Model>> {
        let mut query = swap::Entity::find().filter(swap::Column::UserId.eq(user_id));

        if let Some(wid) = wallet_id {
            query = query.filter(swap::Column::WalletId.eq(wid));
        }

        let mut swaps = query
            .order_by_desc(swap::Column::CreatedAt)
            .all(&self.db).await
            .map_err(|e| AppError::Database(e))?;

        // Apply limit if specified
        if let Some(lim) = limit {
            swaps.truncate(lim as usize);
        }

        Ok(swaps)
    }

    /// Get DEX provider based on chain
    fn get_dex_provider(&self, chain: &str) -> Result<Box<dyn DexProvider>> {
        let parsed: Chain = chain.parse()?;
        match parsed {
            Chain::Solana => Ok(Box::new(JupiterProvider::new())),
            chain if chain.is_evm() => {
                // For EVM chains, use Uniswap V2 compatible DEX
                let rpc_url = match chain {
                    Chain::Eth => "https://eth.llamarpc.com",
                    Chain::Bsc => "https://bsc-dataseed.binance.org",
                    Chain::Polygon => "https://polygon-rpc.com",
                    Chain::Avalanche => "https://api.avax.network/ext/bc/C/rpc",
                    Chain::Arbitrum => "https://arb1.arbitrum.io/rpc",
                    Chain::Optimism => "https://mainnet.optimism.io",
                    Chain::Base => "https://mainnet.base.org",
                    Chain::Fantom => "https://rpc.ftm.tools",
                    Chain::Cronos => "https://evm.cronos.org",
                    Chain::Gnosis => "https://rpc.gnosischain.com",
                    _ => unreachable!(),
                };
                Ok(Box::new(UniswapV2Provider::new(chain.as_str(), rpc_url)?))
            }
            _ => Err(AppError::InvalidInput(format!("Swap not supported for chain: {}", chain))),
        }
    }
}
