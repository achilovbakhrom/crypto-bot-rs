use bip39::{ Mnemonic, Language, Seed };
use solana_sdk::{ signature::{ Keypair, Signer }, pubkey::Pubkey };
use ed25519_dalek_bip32::ExtendedSecretKey;

use crate::error::{ AppError, Result };
use crate::providers::WalletInfo;

const SOLANA_DERIVATION_PATH: &str = "m/44'/501'";

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

    let seed = Seed::new(&mnemonic, "");
    let seed_bytes = seed.as_bytes();

    // Derive keypair using BIP44 path for Solana
    let derivation_path = format!("{}/{}'/{}'", SOLANA_DERIVATION_PATH, derivation_index, 0);

    let extended_key = ExtendedSecretKey::from_seed(seed_bytes)
        .and_then(|extended| {
            let parts: Vec<&str> = derivation_path.trim_start_matches("m/").split('/').collect();
            let mut key = extended;
            for part in parts {
                let index = part
                    .trim_end_matches('\'')
                    .parse::<u32>()
                    .map_err(|_| ed25519_dalek_bip32::Error::InvalidChildNumber)?;
                let hardened = part.ends_with('\'');
                let child_index = if hardened {
                    ed25519_dalek_bip32::ChildIndex::Hardened(index)
                } else {
                    ed25519_dalek_bip32::ChildIndex::Normal(index)
                };
                key = key.derive(&child_index)?;
            }
            Ok(key)
        })
        .map_err(|e| AppError::Chain(format!("Failed to derive key: {}", e)))?;

    let secret_key = extended_key.secret_key;
    let keypair = Keypair::from_bytes(&secret_key.to_bytes()).map_err(|e|
        AppError::Chain(format!("Failed to create keypair: {}", e))
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
    // Try base58 decoding
    let keypair_bytes = bs58
        ::decode(private_key)
        .into_vec()
        .map_err(|_| AppError::InvalidPrivateKey)?;

    let keypair = Keypair::from_bytes(&keypair_bytes).map_err(|_| AppError::InvalidPrivateKey)?;

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
