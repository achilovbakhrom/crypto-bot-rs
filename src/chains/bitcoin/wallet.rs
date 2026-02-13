use bip32::XPrv;
use bip39::Mnemonic;
use bitcoin::key::PrivateKey;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network};

use crate::error::{AppError, Result};
use crate::providers::WalletInfo;

/// BIP84 derivation paths for native SegWit (bech32).
const BIP84_MAINNET_PATH: &str = "m/84'/0'/0'/0";
const BIP84_TESTNET_PATH: &str = "m/84'/1'/0'/0";

pub fn generate_wallet(testnet: bool, derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::generate(24)
        .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;
    restore_from_mnemonic(&mnemonic.to_string(), testnet, derivation_index)
}

pub fn restore_from_mnemonic(
    phrase: &str,
    testnet: bool,
    derivation_index: u32,
) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::parse(phrase)
        .map_err(|e| AppError::InvalidInput(format!("Invalid mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed("");

    let base_path = if testnet {
        BIP84_TESTNET_PATH
    } else {
        BIP84_MAINNET_PATH
    };
    let path = format!("{}/{}", base_path, derivation_index);
    let derivation_path: bip32::DerivationPath = path
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid derivation path: {}", e)))?;

    let child_xprv = XPrv::derive_from_path(&seed, &derivation_path)
        .map_err(|e| AppError::Internal(format!("Key derivation failed: {}", e)))?;

    let network = if testnet {
        Network::Testnet
    } else {
        Network::Bitcoin
    };

    let secret_key = bitcoin::secp256k1::SecretKey::from_slice(&child_xprv.to_bytes())
        .map_err(|e| AppError::Internal(format!("Invalid secret key: {}", e)))?;
    let private_key = PrivateKey::new(secret_key, network);

    let secp = Secp256k1::new();
    let public_key = CompressedPublicKey::from_private_key(&secp, &private_key)
        .map_err(|e| AppError::Internal(format!("Failed to derive public key: {}", e)))?;
    let address = Address::p2wpkh(&public_key, network);

    Ok(WalletInfo {
        address: address.to_string(),
        private_key: private_key.to_wif(),
        mnemonic: Some(phrase.to_string()),
    })
}

pub fn restore_from_private_key(wif: &str, testnet: bool) -> Result<WalletInfo> {
    let private_key: PrivateKey = wif
        .parse()
        .map_err(|e| AppError::InvalidInput(format!("Invalid WIF private key: {}", e)))?;

    let network = if testnet {
        Network::Testnet
    } else {
        Network::Bitcoin
    };

    let secp = Secp256k1::new();
    let public_key = CompressedPublicKey::from_private_key(&secp, &private_key)
        .map_err(|e| AppError::Internal(format!("Failed to derive public key: {}", e)))?;
    let address = Address::p2wpkh(&public_key, network);

    Ok(WalletInfo {
        address: address.to_string(),
        private_key: private_key.to_wif(),
        mnemonic: None,
    })
}

pub fn detect_and_restore(
    secret: &str,
    testnet: bool,
    derivation_index: u32,
) -> Result<WalletInfo> {
    let word_count = secret.split_whitespace().count();
    if word_count == 12 || word_count == 24 {
        restore_from_mnemonic(secret, testnet, derivation_index)
    } else {
        restore_from_private_key(secret, testnet)
    }
}
