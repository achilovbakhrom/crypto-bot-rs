use std::collections::HashMap;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u8,
    pub mint_address: String,
}

lazy_static! {
    pub static ref SPL_TOKENS: HashMap<String, TokenInfo> = {
        let mut m = HashMap::new();
        
        // Solana Top Tokens
        m.insert("USDC".to_string(), TokenInfo {
            symbol: "USDC".to_string(),
            decimals: 6,
            mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        });
        m.insert("USDT".to_string(), TokenInfo {
            symbol: "USDT".to_string(),
            decimals: 6,
            mint_address: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
        });
        m.insert("SOL".to_string(), TokenInfo {
            symbol: "SOL".to_string(),
            decimals: 9,
            mint_address: "So11111111111111111111111111111111111111112".to_string(), // Wrapped SOL
        });
        m.insert("RAY".to_string(), TokenInfo {
            symbol: "RAY".to_string(),
            decimals: 6,
            mint_address: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
        });
        m.insert("SRM".to_string(), TokenInfo {
            symbol: "SRM".to_string(),
            decimals: 6,
            mint_address: "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt".to_string(),
        });
        m.insert("FTT".to_string(), TokenInfo {
            symbol: "FTT".to_string(),
            decimals: 6,
            mint_address: "AGFEad2et2ZJif9jaGpdMixQqvW5i81aBdvKe7PHNfz3".to_string(),
        });
        m.insert("MNGO".to_string(), TokenInfo {
            symbol: "MNGO".to_string(),
            decimals: 6,
            mint_address: "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac".to_string(),
        });
        m.insert("ORCA".to_string(), TokenInfo {
            symbol: "ORCA".to_string(),
            decimals: 6,
            mint_address: "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE".to_string(),
        });
        m.insert("COPE".to_string(), TokenInfo {
            symbol: "COPE".to_string(),
            decimals: 6,
            mint_address: "8HGyAAB1yoM1ttS7pXjHMa3dukTFGQggnFFH3hJZgzQh".to_string(),
        });
        m.insert("STEP".to_string(), TokenInfo {
            symbol: "STEP".to_string(),
            decimals: 9,
            mint_address: "StepAscQoEioFxxWGnh2sLBDFp9d8rvKz2Yp39iDpyT".to_string(),
        });
        m.insert("MEDIA".to_string(), TokenInfo {
            symbol: "MEDIA".to_string(),
            decimals: 6,
            mint_address: "ETAtLmCmsoiEEKfNrHKJ2kYy3MoABhU6NQvpSfij5tDs".to_string(),
        });
        m.insert("ROPE".to_string(), TokenInfo {
            symbol: "ROPE".to_string(),
            decimals: 9,
            mint_address: "8PMHT4swUMtBzgHnh5U564N5sjPSiUz2cjEQzFnnP1Fo".to_string(),
        });
        m.insert("MER".to_string(), TokenInfo {
            symbol: "MER".to_string(),
            decimals: 6,
            mint_address: "MERt85fc5boKw3BW1eYdxonEuJNvXbiMbs6hvheau5K".to_string(),
        });
        m.insert("TULIP".to_string(), TokenInfo {
            symbol: "TULIP".to_string(),
            decimals: 6,
            mint_address: "TuLipcqtGVXP9XR62wM8WWCm6a9vhLs7T1uoWBk6FDs".to_string(),
        });
        m.insert("SNY".to_string(), TokenInfo {
            symbol: "SNY".to_string(),
            decimals: 6,
            mint_address: "4dmKkXNHdgYsXqBHCuMikNQWwVomZURhYvkkX5c4pQ7y".to_string(),
        });
        m.insert("BOP".to_string(), TokenInfo {
            symbol: "BOP".to_string(),
            decimals: 8,
            mint_address: "BLwTnYKqf7u4qjgZrrsKeNs2EzWkMLqVCu6j8iHyrNA3".to_string(),
        });
        m.insert("SAMO".to_string(), TokenInfo {
            symbol: "SAMO".to_string(),
            decimals: 9,
            mint_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
        });
        m.insert("SLRS".to_string(), TokenInfo {
            symbol: "SLRS".to_string(),
            decimals: 6,
            mint_address: "SLRSSpSLUTP7okbCUBYStWCo1vUgyt775faPqz8HUMr".to_string(),
        });
        m.insert("ATLAS".to_string(), TokenInfo {
            symbol: "ATLAS".to_string(),
            decimals: 8,
            mint_address: "ATLASXmbPQxBUYbxPsV97usA3fPQYEqzQBUHgiFCUsXx".to_string(),
        });
        m.insert("POLIS".to_string(), TokenInfo {
            symbol: "POLIS".to_string(),
            decimals: 8,
            mint_address: "poLisWXnNRwC6oBu1vHiuKQzFjGL4XDSu4g9qjz9qVk".to_string(),
        });
        
        m
    };
}

pub fn get_token_by_symbol(symbol: &str) -> Option<&TokenInfo> {
    SPL_TOKENS.get(&symbol.to_uppercase())
}

pub fn get_token_by_mint(mint_address: &str) -> Option<&TokenInfo> {
    SPL_TOKENS.values().find(|t| t.mint_address == mint_address)
}
