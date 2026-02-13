use std::sync::Arc;
use uuid::Uuid;

use crate::db::WalletRepository;
use crate::enums::Chain;
use crate::error::Result;
use crate::providers::GasEstimate;
use crate::rpc::RpcManager;
use crate::services::PriceService;

pub struct GasEstimationService {
    repository: Arc<WalletRepository>,
    rpc_manager: Arc<RpcManager>,
    price_service: Arc<PriceService>,
}

impl GasEstimationService {
    pub fn new(
        repository: Arc<WalletRepository>,
        rpc_manager: Arc<RpcManager>,
        price_service: Arc<PriceService>
    ) -> Self {
        Self {
            repository,
            rpc_manager,
            price_service,
        }
    }

    pub async fn estimate_transaction_fee(
        &self,
        wallet_id: Uuid,
        to: &str,
        amount: &str,
        token_address: Option<&str>
    ) -> Result<GasEstimateWithUsd> {
        // Get wallet to determine chain
        let wallet = self.repository.find_by_id(wallet_id).await?;

        // Get provider for the chain
        let provider = self.rpc_manager.get_provider_by_chain(&wallet.chain).await?;

        // Estimate gas
        let mut gas_estimate = provider.estimate_gas(
            &wallet.address,
            to,
            amount,
            token_address
        ).await?;

        // Get native token price for USD calculation
        let chain: Chain = wallet.chain.parse()?;
        let native_symbol = chain.native_symbol();

        if let Ok(price) = self.price_service.get_price(native_symbol).await {
            let fee_native: f64 = gas_estimate.total_cost_native.parse().unwrap_or(0.0);
            gas_estimate.total_cost_usd = Some(fee_native * price.usd_price);
        }

        Ok(GasEstimateWithUsd {
            chain: wallet.chain,
            gas_estimate,
        })
    }

    pub async fn get_gas_price_recommendations(
        &self,
        chain: &str
    ) -> Result<GasPriceRecommendation> {
        let provider = self.rpc_manager.get_provider_by_chain(chain).await?;

        // For demonstration, estimate with a dummy transaction
        let parsed: Chain = chain.parse()?;
        let dummy_addr = parsed.dummy_address();
        let estimate = provider.estimate_gas(dummy_addr, dummy_addr, "0.001", None).await?;

        // Parse gas prices
        let base_price = estimate.gas_price.clone().unwrap_or_default();
        let max_fee = estimate.max_fee_per_gas.clone().unwrap_or_default();
        let priority_fee = estimate.max_priority_fee_per_gas.clone().unwrap_or_default();

        Ok(GasPriceRecommendation {
            chain: chain.to_string(),
            slow: GasOption {
                gas_price: base_price.clone(),
                max_fee_per_gas: max_fee.clone(),
                max_priority_fee_per_gas: priority_fee.clone(),
                estimated_time: "~5 min".to_string(),
            },
            normal: GasOption {
                gas_price: base_price.clone(),
                max_fee_per_gas: max_fee.clone(),
                max_priority_fee_per_gas: priority_fee.clone(),
                estimated_time: "~1 min".to_string(),
            },
            fast: GasOption {
                gas_price: base_price,
                max_fee_per_gas: max_fee,
                max_priority_fee_per_gas: priority_fee,
                estimated_time: "~15 sec".to_string(),
            },
        })
    }
}

#[derive(serde::Serialize)]
pub struct GasEstimateWithUsd {
    pub chain: String,
    #[serde(flatten)]
    pub gas_estimate: GasEstimate,
}

#[derive(serde::Serialize)]
pub struct GasPriceRecommendation {
    pub chain: String,
    pub slow: GasOption,
    pub normal: GasOption,
    pub fast: GasOption,
}

#[derive(serde::Serialize)]
pub struct GasOption {
    pub gas_price: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub estimated_time: String,
}
