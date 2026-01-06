use std::collections::HashMap;
use lazy_static::lazy_static;
use ethers::{ prelude::*, providers::Provider, types::Address };
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u8,
    pub address: String,
}

lazy_static! {
    pub static ref ERC20_TOKENS: HashMap<String, TokenInfo> = {
        let mut m = HashMap::new();
        
        // Ethereum Mainnet Top Tokens
        m.insert("USDT".to_string(), TokenInfo {
            symbol: "USDT".to_string(),
            decimals: 6,
            address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
        });
        m.insert("USDC".to_string(), TokenInfo {
            symbol: "USDC".to_string(),
            decimals: 6,
            address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        });
        m.insert("WETH".to_string(), TokenInfo {
            symbol: "WETH".to_string(),
            decimals: 18,
            address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        });
        m.insert("DAI".to_string(), TokenInfo {
            symbol: "DAI".to_string(),
            decimals: 18,
            address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
        });
        m.insert("WBTC".to_string(), TokenInfo {
            symbol: "WBTC".to_string(),
            decimals: 8,
            address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),
        });
        m.insert("LINK".to_string(), TokenInfo {
            symbol: "LINK".to_string(),
            decimals: 18,
            address: "0x514910771AF9Ca656af840dff83E8264EcF986CA".to_string(),
        });
        m.insert("UNI".to_string(), TokenInfo {
            symbol: "UNI".to_string(),
            decimals: 18,
            address: "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984".to_string(),
        });
        m.insert("MATIC".to_string(), TokenInfo {
            symbol: "MATIC".to_string(),
            decimals: 18,
            address: "0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0".to_string(),
        });
        m.insert("SHIB".to_string(), TokenInfo {
            symbol: "SHIB".to_string(),
            decimals: 18,
            address: "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE".to_string(),
        });
        m.insert("APE".to_string(), TokenInfo {
            symbol: "APE".to_string(),
            decimals: 18,
            address: "0x4d224452801ACEd8B2F0aebE155379bb5D594381".to_string(),
        });
        m.insert("CRO".to_string(), TokenInfo {
            symbol: "CRO".to_string(),
            decimals: 8,
            address: "0xA0b73E1Ff0B80914AB6fe0444E65848C4C34450b".to_string(),
        });
        m.insert("LDO".to_string(), TokenInfo {
            symbol: "LDO".to_string(),
            decimals: 18,
            address: "0x5A98FcBEA516Cf06857215779Fd812CA3beF1B32".to_string(),
        });
        m.insert("AAVE".to_string(), TokenInfo {
            symbol: "AAVE".to_string(),
            decimals: 18,
            address: "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9".to_string(),
        });
        m.insert("SNX".to_string(), TokenInfo {
            symbol: "SNX".to_string(),
            decimals: 18,
            address: "0xC011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F".to_string(),
        });
        m.insert("MKR".to_string(), TokenInfo {
            symbol: "MKR".to_string(),
            decimals: 18,
            address: "0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2".to_string(),
        });
        m.insert("COMP".to_string(), TokenInfo {
            symbol: "COMP".to_string(),
            decimals: 18,
            address: "0xc00e94Cb662C3520282E6f5717214004A7f26888".to_string(),
        });
        m.insert("CRV".to_string(), TokenInfo {
            symbol: "CRV".to_string(),
            decimals: 18,
            address: "0xD533a949740bb3306d119CC777fa900bA034cd52".to_string(),
        });
        m.insert("GRT".to_string(), TokenInfo {
            symbol: "GRT".to_string(),
            decimals: 18,
            address: "0xc944E90C64B2c07662A292be6244BDf05Cda44a7".to_string(),
        });
        m.insert("FXS".to_string(), TokenInfo {
            symbol: "FXS".to_string(),
            decimals: 18,
            address: "0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0".to_string(),
        });
        m.insert("1INCH".to_string(), TokenInfo {
            symbol: "1INCH".to_string(),
            decimals: 18,
            address: "0x111111111117dC0aa78b770fA6A738034120C302".to_string(),
        });
        
        m
    };
}

pub fn get_token_by_symbol(symbol: &str) -> Option<&TokenInfo> {
    ERC20_TOKENS.get(&symbol.to_uppercase())
}

pub fn get_token_by_address(address: &str) -> Option<&TokenInfo> {
    let address_lower = address.to_lowercase();
    ERC20_TOKENS.values().find(|t| t.address.to_lowercase() == address_lower)
}

pub fn get_erc20_contract<M: Middleware + 'static>(
    token_address: Address,
    provider: Arc<M>
) -> Contract<M> {
    let abi = ethers::abi
        ::parse_abi(
            &[
                "function balanceOf(address) view returns (uint256)",
                "function transfer(address to, uint256 amount) returns (bool)",
                "function decimals() view returns (uint8)",
                "function symbol() view returns (string)",
            ]
        )
        .expect("Failed to parse ERC20 ABI");

    Contract::new(token_address, abi, provider)
}
