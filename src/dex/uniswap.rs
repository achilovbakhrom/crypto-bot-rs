use super::{ DexProvider, SwapQuote, SwapResult };
use crate::error::{ AppError, Result };
use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;

// Uniswap V2 Router ABI (simplified for swaps)
abigen!(
    IUniswapV2Router,
    r#"[
        function swapExactTokensForTokens(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
        function getAmountsOut(uint amountIn, address[] memory path) external view returns (uint[] memory amounts)
        function WETH() external pure returns (address)
    ]"#
);

// ERC20 ABI for approvals
abigen!(
    IERC20,
    r#"[
        function approve(address spender, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
    ]"#
);

pub struct UniswapV2Provider {
    router_address: Address,
    chain: String,
    provider: Arc<Provider<Http>>,
}

impl UniswapV2Provider {
    pub fn new(chain: &str, rpc_url: &str) -> Result<Self> {
        let router_address = match chain {
            "ETH" =>
                "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
                    .parse()
                    .map_err(|e| AppError::Internal(format!("Invalid router address: {}", e)))?,
            "BSC" =>
                "0x10ED43C718714eb63d5aA57B78B54704E256024E" // PancakeSwap V2
                    .parse()
                    .map_err(|e| AppError::Internal(format!("Invalid router address: {}", e)))?,
            _ => {
                return Err(
                    AppError::Validation(format!("Unsupported chain for Uniswap: {}", chain))
                );
            }
        };

        let provider = Provider::<Http>
            ::try_from(rpc_url)
            .map_err(|e| AppError::Internal(format!("Failed to create provider: {}", e)))?;

        Ok(Self {
            router_address,
            chain: chain.to_string(),
            provider: Arc::new(provider),
        })
    }

    fn get_weth_address(&self) -> Address {
        match self.chain.as_str() {
            "ETH" => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().unwrap(),
            "BSC" =>
                "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c" // WBNB
                    .parse()
                    .unwrap(),
            _ => panic!("Unsupported chain"),
        }
    }

    fn resolve_token_address(&self, token: &str) -> Result<Address> {
        // Handle native tokens
        if token == "ETH" || token == "BNB" {
            return Ok(self.get_weth_address());
        }

        // Parse as address
        token.parse().map_err(|e| AppError::Validation(format!("Invalid token address: {}", e)))
    }
}

#[async_trait]
impl DexProvider for UniswapV2Provider {
    async fn get_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: f64,
        slippage: f64
    ) -> Result<SwapQuote> {
        let from_address = self.resolve_token_address(from_token)?;
        let to_address = self.resolve_token_address(to_token)?;

        let router = IUniswapV2Router::new(self.router_address, self.provider.clone());

        // Convert amount to Wei (assuming 18 decimals)
        let amount_in = U256::from((amount * 1e18) as u128);

        // Build swap path
        let path = vec![from_address, to_address];

        // Get amounts out
        let amounts_out = router
            .get_amounts_out(amount_in, path.clone())
            .call().await
            .map_err(|e| AppError::Blockchain(format!("Failed to get quote: {}", e)))?;

        let expected_out = amounts_out
            .last()
            .ok_or_else(|| AppError::Internal("No output amount".to_string()))?;

        let expected_to_amount = (expected_out.as_u128() as f64) / 1e18;
        let minimum_to_amount = expected_to_amount * (1.0 - slippage / 100.0);

        // Calculate price impact (simplified)
        let price = expected_to_amount / amount;
        let price_impact = 0.0; // Would need pool reserves for accurate calculation

        Ok(SwapQuote {
            from_token: from_token.to_string(),
            from_token_address: Some(format!("{:?}", from_address)),
            to_token: to_token.to_string(),
            to_token_address: Some(format!("{:?}", to_address)),
            from_amount: amount,
            expected_to_amount,
            minimum_to_amount,
            price_impact,
            route: path
                .iter()
                .map(|addr| format!("{:?}", addr))
                .collect(),
            estimated_gas: Some("150000".to_string()),
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
        let from_address = self.resolve_token_address(from_token)?;
        let to_address = self.resolve_token_address(to_token)?;

        // Create wallet
        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid private key: {}", e)))?;
        let client = SignerMiddleware::new(self.provider.clone(), wallet);
        let client_arc = Arc::new(client);

        // Approve tokens if needed
        let token_contract = IERC20::new(from_address, client_arc.clone());
        let amount_in = U256::from((amount * 1e18) as u128);

        let allowance = token_contract
            .allowance(
                wallet_address
                    .parse()
                    .map_err(|e| AppError::Validation(format!("Invalid address: {}", e)))?,
                self.router_address
            )
            .call().await
            .map_err(|e| AppError::Blockchain(format!("Failed to check allowance: {}", e)))?;

        if allowance < amount_in {
            let approve_tx = token_contract
                .approve(self.router_address, U256::MAX)
                .send().await
                .map_err(|e| AppError::Blockchain(format!("Failed to approve: {}", e)))?.await
                .map_err(|e| AppError::Blockchain(format!("Approval failed: {}", e)))?;
        }

        // Execute swap
        let router = IUniswapV2Router::new(self.router_address, client_arc.clone());
        let path = vec![from_address, to_address];
        let amount_out_min = U256::from((min_output * 1e18) as u128);
        let to = wallet_address
            .parse()
            .map_err(|e| AppError::Validation(format!("Invalid address: {}", e)))?;
        let deadline = U256::from(
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() +
                300
        ); // 5 minutes

        let receipt = router
            .swap_exact_tokens_for_tokens(amount_in, amount_out_min, path, to, deadline)
            .send().await
            .map_err(|e| AppError::Blockchain(format!("Swap failed: {}", e)))?.await
            .map_err(|e| AppError::Blockchain(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| AppError::Internal("No receipt".to_string()))?;

        Ok(SwapResult {
            tx_hash: format!("{:?}", receipt.transaction_hash),
            from_amount: amount,
            to_amount: min_output, // Actual amount would need event parsing
            gas_used: receipt.gas_used.map(|g| g.to_string()),
        })
    }

    fn name(&self) -> &str {
        match self.chain.as_str() {
            "ETH" => "Uniswap V2",
            "BSC" => "PancakeSwap V2",
            _ => "Unknown",
        }
    }

    fn supported_chains(&self) -> Vec<&str> {
        vec!["ETH", "BSC"]
    }
}
