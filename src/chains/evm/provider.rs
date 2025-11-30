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
        let balance_of = "function balanceOf(address) external view returns (uint256)";
        let decimals_fn = "function decimals() external view returns (uint8)";
        let symbol_fn = "function symbol() external view returns (string)";

        let contract = Contract::new(
            token_addr,
            vec![balance_of.parse().unwrap()],
            self.provider.clone()
        );

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
            let decimals_contract = Contract::new(
                token_addr,
                vec![decimals_fn.parse().unwrap()],
                self.provider.clone()
            );
            let symbol_contract = Contract::new(
                token_addr,
                vec![symbol_fn.parse().unwrap()],
                self.provider.clone()
            );

            let decimals = match decimals_contract.method::<_, u8>("decimals", ()) {
                Ok(method) => method.call().await.ok().unwrap_or(18),
                Err(_) => 18,
            };

            let symbol = match symbol_contract.method::<_, String>("symbol", ()) {
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
        let transfer_fn = "function transfer(address to, uint256 amount) external returns (bool)";

        let client = SignerMiddleware::new(
            self.provider.clone(),
            wallet.with_chain_id(self.chain_id)
        );
        let contract = Contract::new(
            token_address,
            vec![transfer_fn.parse().unwrap()],
            Arc::new(client)
        );

        let mut call = contract
            .method::<_, bool>("transfer", (to, amount))
            .map_err(|e| AppError::Chain(format!("Failed to prepare transfer: {}", e)))?;

        // Set gas parameters (EIP-1559)
        if let Some(max_fee) = max_fee_per_gas {
            call.tx.set_max_fee_per_gas(max_fee);
        }
        if let Some(priority_fee) = max_priority_fee_per_gas {
            call.tx.set_max_priority_fee_per_gas(priority_fee);
        }
        if let Some(limit) = gas_limit {
            call.tx.set_gas(limit);
        }

        let pending_tx = call
            .send().await
            .map_err(|e| AppError::Chain(format!("Transaction failed: {}", e)))?;

        let tx_hash = format!("{:?}", pending_tx.tx_hash());

        Ok(TransactionResponse {
            tx_hash,
            status: "pending".to_string(),
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
            let amount = parse_units(&request.amount, 18)
                .map_err(|e| AppError::InvalidInput(format!("Invalid amount: {}", e)))?
                .into();

            let client = SignerMiddleware::new(
                self.provider.clone(),
                wallet.with_chain_id(self.chain_id)
            );

            let mut tx = EthTxRequest::new().to(to).value(amount);

            // Set EIP-1559 gas parameters
            if let Some(max_fee) = max_fee_per_gas {
                tx = tx.max_fee_per_gas(max_fee);
            }
            if let Some(priority_fee) = max_priority_fee_per_gas {
                tx = tx.max_priority_fee_per_gas(priority_fee);
            }
            if let Some(limit) = gas_limit {
                tx = tx.gas(limit);
            }

            let pending_tx = client
                .send_transaction(tx, None).await
                .map_err(|e| AppError::Chain(format!("Transaction failed: {}", e)))?;

            let tx_hash = format!("{:?}", pending_tx.tx_hash());

            Ok(TransactionResponse {
                tx_hash,
                status: "pending".to_string(),
            })
        }
    }

    fn validate_address(&self, address: &str) -> bool {
        wallet::validate_address(address)
    }
}
