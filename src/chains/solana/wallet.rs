use bip39::Mnemonic;
use solana_keypair::Keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::{SeedDerivable, Signer};

use crate::error::{ AppError, Result };
use crate::providers::WalletInfo;

pub fn generate_wallet(_derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::generate(24).map_err(|e|
        AppError::Internal(format!("Failed to generate mnemonic: {}", e))
    )?;

    let mnemonic_phrase = mnemonic.to_string();

    restore_from_mnemonic(&mnemonic_phrase, 0)
}

pub fn restore_from_mnemonic(mnemonic_phrase: &str, _derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::parse(mnemonic_phrase).map_err(|_| AppError::InvalidMnemonic)?;

    let seed = mnemonic.to_seed("");

    // Create keypair from first 32 bytes of seed
    let keypair = Keypair::from_seed(&seed[..32]).map_err(|e|
        AppError::Internal(format!("Failed to create keypair: {}", e))
    )?;

    let address = keypair.pubkey().to_string();
    let private_key = bs58::encode(keypair.to_bytes()).into_string();

    Ok(WalletInfo {
        address,
        private_key,
        mnemonic: Some(mnemonic_phrase.to_string()),
    })
}

pub fn restore_from_private_key(private_key: &str) -> Result<WalletInfo> {
    let keypair_bytes = bs58
        ::decode(private_key)
        .into_vec()
        .map_err(|_| AppError::InvalidPrivateKey)?;

    let keypair = Keypair::try_from(keypair_bytes.as_slice()).map_err(|_| AppError::InvalidPrivateKey)?;

    let address = keypair.pubkey().to_string();
    let private_key = bs58::encode(keypair.to_bytes()).into_string();

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
    } else {
        // Try as base58 private key
        restore_from_private_key(secret)
    }
}

pub fn validate_address(address: &str) -> bool {
    address.parse::<Pubkey>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wallet() {
        let wallet = generate_wallet(0).unwrap();
        assert!(!wallet.address.is_empty());
        assert!(!wallet.private_key.is_empty());
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
    }

    #[test]
    fn test_validate_address() {
        let wallet = generate_wallet(0).unwrap();
        assert!(validate_address(&wallet.address));
        assert!(!validate_address("invalid"));
    }
}
