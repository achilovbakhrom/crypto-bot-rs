use bip39::Mnemonic;
use blake2::digest::{consts::U28, Digest};
use blake2::Blake2b;
use ed25519_dalek::SigningKey;
use hmac::{Hmac, Mac};
use sha2::Sha512;

use crate::error::{AppError, Result};
use crate::providers::WalletInfo;

type Blake2b224 = Blake2b<U28>;
type HmacSha512 = Hmac<Sha512>;

/// Derive an Ed25519 keypair from seed bytes using HMAC-SHA512 (simplified CIP-1852).
/// Real Cardano wallets use an extended key scheme, but for basic wallet generation
/// this produces a valid Ed25519 keypair that can receive ADA.
fn derive_cardano_keypair(seed: &[u8], derivation_index: u32) -> (SigningKey, ed25519_dalek::VerifyingKey) {
    // Use HMAC-SHA512 with "ed25519 cardano seed" as key
    let mut mac = HmacSha512::new_from_slice(b"ed25519 cardano seed")
        .expect("HMAC can take key of any size");
    mac.update(seed);
    // Include derivation index
    mac.update(&derivation_index.to_be_bytes());
    let result = mac.finalize().into_bytes();

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&result[..32]);

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();

    (signing_key, verifying_key)
}

/// Build a Cardano enterprise address (type 0x60/0x70) from an Ed25519 public key.
/// Enterprise addresses have only a payment credential (no staking).
/// Format: header_byte || Blake2b-224(pubkey)
fn pub_key_to_address(pub_key_bytes: &[u8], testnet: bool) -> String {
    // Blake2b-224 hash of the public key
    let mut hasher = Blake2b224::new();
    hasher.update(pub_key_bytes);
    let key_hash = hasher.finalize();

    // Header byte: 0x61 = mainnet enterprise, 0x60 = testnet enterprise
    // Actually: type 6 (enterprise) + network tag (0=testnet, 1=mainnet)
    let header = if testnet { 0x60u8 } else { 0x61u8 };

    let mut payload = Vec::with_capacity(29);
    payload.push(header);
    payload.extend_from_slice(&key_hash);

    // Bech32 encode
    let hrp = if testnet {
        bech32::Hrp::parse("addr_test").expect("valid hrp")
    } else {
        bech32::Hrp::parse("addr").expect("valid hrp")
    };

    bech32::encode::<bech32::Bech32>(hrp, &payload)
        .expect("valid bech32 encoding")
}

pub fn generate_wallet(testnet: bool, derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::generate(24)
        .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;
    restore_from_mnemonic(&mnemonic.to_string(), testnet, derivation_index)
}

pub fn restore_from_mnemonic(phrase: &str, testnet: bool, derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::parse(phrase)
        .map_err(|e| AppError::InvalidInput(format!("Invalid mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed("");

    let (signing_key, verifying_key) = derive_cardano_keypair(&seed, derivation_index);

    let address = pub_key_to_address(verifying_key.as_bytes(), testnet);
    let private_key_hex = hex::encode(signing_key.to_bytes());

    Ok(WalletInfo {
        address,
        private_key: private_key_hex,
        mnemonic: Some(phrase.to_string()),
    })
}

pub fn restore_from_private_key(hex_key: &str, testnet: bool) -> Result<WalletInfo> {
    let key_bytes = hex::decode(hex_key.trim_start_matches("0x"))
        .map_err(|e| AppError::InvalidInput(format!("Invalid hex private key: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(AppError::InvalidInput(
            "Cardano Ed25519 private key must be 32 bytes".to_string(),
        ));
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    let signing_key = SigningKey::from_bytes(&key_array);
    let verifying_key = signing_key.verifying_key();

    let address = pub_key_to_address(verifying_key.as_bytes(), testnet);

    Ok(WalletInfo {
        address,
        private_key: hex::encode(signing_key.to_bytes()),
        mnemonic: None,
    })
}

pub fn detect_and_restore(secret: &str, testnet: bool, derivation_index: u32) -> Result<WalletInfo> {
    let word_count = secret.split_whitespace().count();
    if word_count == 12 || word_count == 24 {
        restore_from_mnemonic(secret, testnet, derivation_index)
    } else {
        restore_from_private_key(secret, testnet)
    }
}
