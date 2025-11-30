use bip39::{ Mnemonic, Language };
use ethers::signers::{ LocalWallet, Signer };
use ethers::core::types::H160;
use ethers::utils::hex;

use crate::error::{ AppError, Result };
use crate::providers::WalletInfo;

const ETH_DERIVATION_PATH: &str = "m/44'/60'/0'/0";

pub fn generate_wallet(derivation_index: u32) -> Result<WalletInfo> {
    // Generate 24-word mnemonic
    let mnemonic = Mnemonic::generate_in(Language::English, 24).map_err(|e|
        AppError::Internal(format!("Failed to generate mnemonic: {}", e))
    )?;

    let mnemonic_phrase = mnemonic.to_string();

    restore_from_mnemonic(&mnemonic_phrase, derivation_index)
}

pub fn restore_from_mnemonic(mnemonic_phrase: &str, derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_phrase).map_err(
        |_| AppError::InvalidMnemonic
    )?;

    let derivation_path = format!("{}/{}", ETH_DERIVATION_PATH, derivation_index);

    let wallet = LocalWallet::new_mnemonic_builder()
        .phrase(mnemonic_phrase)
        .derivation_path(&derivation_path)
        .map_err(|e| AppError::Chain(format!("Invalid derivation path: {}", e)))?
        .build()
        .map_err(|e| AppError::Chain(format!("Failed to build wallet: {}", e)))?;

    let address = format!("{:?}", wallet.address());
    let private_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));

    Ok(WalletInfo {
        address,
        private_key,
        mnemonic: Some(mnemonic_phrase.to_string()),
    })
}

pub fn restore_from_private_key(private_key: &str) -> Result<WalletInfo> {
    let private_key = private_key.trim_start_matches("0x");

    let wallet: LocalWallet = private_key.parse().map_err(|_| AppError::InvalidPrivateKey)?;

    let address = format!("{:?}", wallet.address());
    let private_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));

    Ok(WalletInfo {
        address,
        private_key,
        mnemonic: None,
    })
}

pub fn detect_and_restore(secret: &str, derivation_index: u32) -> Result<WalletInfo> {
    let secret = secret.trim();

    // Check if it's a mnemonic (contains spaces, 12 or 24 words)
    let word_count = secret.split_whitespace().count();

    if word_count == 12 || word_count == 24 {
        restore_from_mnemonic(secret, derivation_index)
    } else if secret.starts_with("0x") || secret.len() == 64 {
        // Likely a private key
        restore_from_private_key(secret)
    } else {
        Err(
            AppError::InvalidInput(
                "Secret must be either a 12/24-word mnemonic or a hex private key".to_string()
            )
        )
    }
}

pub fn validate_address(address: &str) -> bool {
    address.parse::<H160>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wallet() {
        let wallet = generate_wallet(0).unwrap();
        assert!(wallet.address.starts_with("0x"));
        assert!(wallet.private_key.starts_with("0x"));
        assert!(wallet.mnemonic.is_some());

        let mnemonic = wallet.mnemonic.unwrap();
        assert_eq!(mnemonic.split_whitespace().count(), 24);
    }

    #[test]
    fn test_restore_from_mnemonic() {
        let wallet1 = generate_wallet(0).unwrap();
        let mnemonic = wallet1.mnemonic.unwrap();

        let wallet2 = restore_from_mnemonic(&mnemonic, 0).unwrap();
        assert_eq!(wallet1.address, wallet2.address);
        assert_eq!(wallet1.private_key, wallet2.private_key);
    }

    #[test]
    fn test_restore_from_private_key() {
        let wallet1 = generate_wallet(0).unwrap();
        let private_key = wallet1.private_key.clone();

        let wallet2 = restore_from_private_key(&private_key).unwrap();
        assert_eq!(wallet1.address, wallet2.address);
        assert!(wallet2.mnemonic.is_none());
    }

    #[test]
    fn test_validate_address() {
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"));
        assert!(!validate_address("invalid"));
        assert!(!validate_address("0x123"));
    }
}
