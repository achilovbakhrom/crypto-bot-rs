use crate::enums::TxStatus;
use async_trait::async_trait;
use ethers::{
    prelude::*,
    providers::{ Http, Provider },
    types::{ TransactionRequest as EthTxRequest, U256 },
    utils::parse_units,
};
use std::sync::Arc;

use crate::chains::evm::{ tokens, wallet };
use crate::error::{ AppError, Result };
use crate::providers::{
    Balance,
    ChainProvider,
    TransactionRequest,
    TransactionResponse,
    WalletInfo,
};

#[derive(Clone)]
pub struct EvmProvider {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
    native_symbol: String,
}

impl EvmProvider {
    pub fn new(rpc_url: &str, chain_id: u64, native_symbol: &str) -> Result<Self> {
        let provider = Provider::<Http>
            ::try_from(rpc_url)
            .map_err(|e| AppError::Rpc(format!("Failed to create provider: {}", e)))?;

        Ok(Self {
            provider: Arc::new(provider),
            chain_id,
            native_symbol: native_symbol.to_string(),
        })
    }

    async fn get_erc20_balance(
        &self,
        wallet_address: &str,
        token_address: &str
    ) -> Result<Balance> {
        let wallet_addr: Address = wallet_address.parse().map_err(|_| AppError::InvalidAddress)?;
        let token_addr: Address = token_address.parse().map_err(|_| AppError::InvalidAddress)?;

        // ERC20 balanceOf ABI
        let abi = ethers::abi
            ::parse_abi(
                &[
                    "function balanceOf(address) external view returns (uint256)",
                    "function decimals() external view returns (uint8)",
                    "function symbol() external view returns (string)",
                ]
            )
            .map_err(|e| AppError::Chain(format!("Failed to parse ABI: {}", e)))?;

        let contract = Contract::new(token_addr, abi.clone(), self.provider.clone());

        let balance: U256 = contract
            .method::<_, U256>("balanceOf", wallet_addr)
            .map_err(|e| AppError::Chain(format!("Failed to call balanceOf: {}", e)))?
            .call().await
            .map_err(|e| AppError::Chain(format!("balanceOf call failed: {}", e)))?;

        // Try to get decimals and symbol
        let (decimals, symbol) = if
            let Some(token_info) = tokens::get_token_by_address(token_address)
        {
            (token_info.decimals, token_info.symbol.clone())
        } else {
            // Try to fetch from contract
            let decimals = match contract.method::<_, u8>("decimals", ()) {
                Ok(method) => method.call().await.ok().unwrap_or(18),
                Err(_) => 18,
            };

            let symbol = match contract.method::<_, String>("symbol", ()) {
                Ok(method) =>
                    method
                        .call().await
                        .ok()
                        .unwrap_or_else(|| "UNKNOWN".to_string()),
                Err(_) => "UNKNOWN".to_string(),
            };

            (decimals, symbol)
        };

        let balance_str = ethers::utils
            ::format_units(balance, decimals as u32)
            .map_err(|e| AppError::Chain(format!("Failed to format balance: {}", e)))?;

        Ok(Balance {
            balance: balance_str,
            symbol,
            decimals,
        })
    }

    async fn send_erc20_transaction(
        &self,
        wallet: LocalWallet,
        to: Address,
        amount: U256,
        token_address: Address,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        gas_limit: Option<U256>
    ) -> Result<TransactionResponse> {
        // ERC20 transfer ABI
        let abi = ethers::abi
            ::parse_abi(&["function transfer(address to, uint256 amount) external returns (bool)"])
            .map_err(|e| AppError::Chain(format!("Failed to parse ABI: {}", e)))?;

        let client = SignerMiddleware::new(
            self.provider.clone(),
            wallet.with_chain_id(self.chain_id)
        );
        let contract = Contract::new(token_address, abi, Arc::new(client));

        let mut call = contract
            .method::<_, bool>("transfer", (to, amount))
            .map_err(|e| AppError::Chain(format!("Failed to prepare transfer: {}", e)))?;

        // Set gas parameters
        if let Some(limit) = gas_limit {
            call.tx.set_gas(limit);
        }
        // Note: max_fee_per_gas and max_priority_fee_per_gas are set differently in ethers 2.0
        // They're automatically handled by the provider

        let pending_tx = call
            .send().await
            .map_err(|e| AppError::Chain(format!("Transaction failed: {}", e)))?;

        let tx_hash = format!("{:?}", pending_tx.tx_hash());

        Ok(TransactionResponse {
            tx_hash,
            status: TxStatus::Pending.to_string(),
        })
    }
}

#[async_trait]
impl ChainProvider for EvmProvider {
    async fn generate_wallet(&self, derivation_index: u32) -> Result<WalletInfo> {
        wallet::generate_wallet(derivation_index)
    }

    async fn restore_wallet(&self, secret: &str, derivation_index: u32) -> Result<WalletInfo> {
        wallet::detect_and_restore(secret, derivation_index)
    }

    async fn get_balance(&self, address: &str) -> Result<Balance> {
        let addr: Address = address.parse().map_err(|_| AppError::InvalidAddress)?;

        let balance = self.provider
            .get_balance(addr, None).await
            .map_err(|e| AppError::Rpc(format!("Failed to get balance: {}", e)))?;

        let balance_str = ethers::utils::format_ether(balance);

        Ok(Balance {
            balance: balance_str,
            symbol: self.native_symbol.clone(),
            decimals: 18,
        })
    }

    async fn get_token_balance(&self, address: &str, token_address: &str) -> Result<Balance> {
        self.get_erc20_balance(address, token_address).await
    }

    async fn send_transaction(
        &self,
        private_key: &str,
        request: TransactionRequest
    ) -> Result<TransactionResponse> {
        let wallet: LocalWallet = private_key
            .trim_start_matches("0x")
            .parse()
            .map_err(|_| AppError::InvalidPrivateKey)?;

        let to: Address = request.to.parse().map_err(|_| AppError::InvalidAddress)?;

        // Parse gas parameters
        let max_fee_per_gas = request.max_fee_per_gas
            .as_ref()
            .map(|s| s.parse::<U256>())
            .transpose()
            .map_err(|_| AppError::InvalidInput("Invalid max_fee_per_gas".to_string()))?;

        let max_priority_fee_per_gas = request.max_priority_fee_per_gas
            .as_ref()
            .map(|s| s.parse::<U256>())
            .transpose()
            .map_err(|_| AppError::InvalidInput("Invalid max_priority_fee_per_gas".to_string()))?;

        let gas_limit = request.gas_limit.map(U256::from);

        if let Some(token_address) = request.token_address {
            // ERC20 token transfer
            let token_addr: Address = token_address.parse().map_err(|_| AppError::InvalidAddress)?;

            // Get token decimals
            let decimals = if let Some(token_info) = tokens::get_token_by_address(&token_address) {
                token_info.decimals
            } else {
                18 // Default to 18 if unknown
            };

            let amount = parse_units(&request.amount, decimals as u32)
                .map_err(|e| AppError::InvalidInput(format!("Invalid amount: {}", e)))?
                .into();

            self.send_erc20_transaction(
                wallet,
                to,
                amount,
                token_addr,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                gas_limit
            ).await
        } else {
            // Native ETH transfer
            let amount: U256 = parse_units(&request.amount, 18)
                .map_err(|e| AppError::InvalidInput(format!("Invalid amount: {}", e)))?
                .into();

            let client = SignerMiddleware::new(
                self.provider.clone(),
                wallet.with_chain_id(self.chain_id)
            );

            let mut tx = EthTxRequest::new().to(to).value(amount);

            // Set gas parameters - Note: EIP-1559 params handled automatically by provider
            if let Some(limit) = gas_limit {
                tx = tx.gas(limit);
            }

            let pending_tx = client
                .send_transaction(tx, None).await
                .map_err(|e| AppError::Chain(format!("Transaction failed: {}", e)))?;

            let tx_hash = format!("{:?}", pending_tx.tx_hash());

            Ok(TransactionResponse {
                tx_hash,
                status: TxStatus::Pending.to_string(),
            })
        }
    }

    async fn estimate_gas(
        &self,
        from: &str,
        to: &str,
        amount: &str,
        token_address: Option<&str>
    ) -> Result<crate::providers::GasEstimate> {
        let from_addr: Address = from.parse().map_err(|_| AppError::InvalidAddress)?;
        let to_addr: Address = to.parse().map_err(|_| AppError::InvalidAddress)?;

        // Get current gas price and EIP-1559 fees
        let gas_price = self.provider
            .get_gas_price().await
            .map_err(|e| AppError::Rpc(format!("Failed to get gas price: {}", e)))?;

        let (max_fee, max_priority_fee) = match self.provider.estimate_eip1559_fees(None).await {
            Ok((max_fee, max_priority_fee)) => (max_fee, max_priority_fee),
            Err(_) => {
                // Fallback to legacy gas price if EIP-1559 not supported
                (gas_price, U256::from(0))
            }
        };

        // Estimate gas limit
        let estimated_gas = if let Some(token_addr) = token_address {
            // ERC20 transfer estimation
            let token_address: Address = token_addr.parse().map_err(|_| AppError::InvalidAddress)?;
            let amount_u256: U256 = parse_units(amount, 18)
                .map_err(|_| AppError::InvalidInput("Invalid amount".to_string()))?
                .into();

            let contract = tokens::get_erc20_contract(token_address, self.provider.clone());
            let call = contract
                .method::<_, ()>("transfer", (to_addr, amount_u256))
                .map_err(|e| AppError::Chain(format!("Failed to create call: {}", e)))?;

            call
                .estimate_gas().await
                .map_err(|e| AppError::Chain(format!("Gas estimation failed: {}", e)))?
        } else {
            // Native token transfer estimation
            let amount_u256: U256 = parse_units(amount, 18)
                .map_err(|_| AppError::InvalidInput("Invalid amount".to_string()))?
                .into();

            let tx = EthTxRequest::new().from(from_addr).to(to_addr).value(amount_u256);

            self.provider
                .estimate_gas(&tx.into(), None).await
                .map_err(|e| AppError::Chain(format!("Gas estimation failed: {}", e)))?
        };

        // Calculate total cost in wei
        let total_cost_wei = estimated_gas * max_fee;
        let total_cost_eth = ethers::utils
            ::format_units(total_cost_wei, 18)
            .map_err(|_| AppError::Internal("Failed to format units".to_string()))?;

        Ok(crate::providers::GasEstimate {
            estimated_gas: estimated_gas.as_u64(),
            gas_price: Some(
                format!("{}", ethers::utils::format_units(gas_price, "gwei").unwrap_or_default())
            ),
            max_fee_per_gas: Some(
                format!("{}", ethers::utils::format_units(max_fee, "gwei").unwrap_or_default())
            ),
            max_priority_fee_per_gas: Some(
                format!(
                    "{}",
                    ethers::utils::format_units(max_priority_fee, "gwei").unwrap_or_default()
                )
            ),
            total_cost_native: total_cost_eth,
            total_cost_usd: None, // Will be calculated by service layer
        })
    }

    fn validate_address(&self, address: &str) -> bool {
        wallet::validate_address(address)
    }
}
