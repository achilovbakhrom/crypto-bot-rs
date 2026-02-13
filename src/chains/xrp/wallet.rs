use bip39::Mnemonic;
use ed25519_dalek::SigningKey;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256, Sha512};

use crate::error::{AppError, Result};
use crate::providers::WalletInfo;

/// XRP uses the first 16 bytes of SHA-512 hash of the seed as the Ed25519 seed.
fn derive_xrp_keypair(seed_bytes: &[u8]) -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let mut hasher = Sha512::new();
    hasher.update(seed_bytes);
    let hash = hasher.finalize();

    // XRP Ed25519 derivation: use first 32 bytes of SHA-512(seed)
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&hash[..32]);

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();

    (signing_key, verifying_key)
}

/// Convert an Ed25519 public key to an XRP classic address.
/// Process: SHA-256 → RIPEMD-160 → prepend 0x00 → append 4-byte checksum → Base58
pub fn pub_key_to_classic_address(pub_key_bytes: &[u8]) -> String {
    // For XRP Ed25519, prefix the pubkey with 0xED
    let mut prefixed = Vec::with_capacity(33);
    prefixed.push(0xED);
    prefixed.extend_from_slice(pub_key_bytes);

    // SHA-256 hash
    let sha256_hash = Sha256::digest(&prefixed);

    // RIPEMD-160 hash
    let ripemd_hash = Ripemd160::digest(&sha256_hash);

    // Prepend account type byte (0x00 for classic address)
    let mut payload = Vec::with_capacity(21);
    payload.push(0x00);
    payload.extend_from_slice(&ripemd_hash);

    // Double SHA-256 for checksum
    let checksum_hash1 = Sha256::digest(&payload);
    let checksum_hash2 = Sha256::digest(&checksum_hash1);
    let checksum = &checksum_hash2[..4];

    // Append checksum
    payload.extend_from_slice(checksum);

    // Base58 encode (XRP uses the same alphabet as Bitcoin)
    bs58::encode(&payload)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .into_string()
}

pub fn generate_wallet(_derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::generate(24)
        .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;
    restore_from_mnemonic(&mnemonic.to_string(), 0)
}

pub fn restore_from_mnemonic(phrase: &str, _derivation_index: u32) -> Result<WalletInfo> {
    let mnemonic = Mnemonic::parse(phrase)
        .map_err(|e| AppError::InvalidInput(format!("Invalid mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed("");

    let (signing_key, verifying_key) = derive_xrp_keypair(&seed[..32]);

    let address = pub_key_to_classic_address(verifying_key.as_bytes());
    let private_key_hex = hex::encode(signing_key.to_bytes());

    Ok(WalletInfo {
        address,
        private_key: private_key_hex,
        mnemonic: Some(phrase.to_string()),
    })
}

pub fn restore_from_private_key(hex_key: &str) -> Result<WalletInfo> {
    let key_bytes = hex::decode(hex_key.trim_start_matches("0x"))
        .map_err(|e| AppError::InvalidInput(format!("Invalid hex private key: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(AppError::InvalidInput(
            "XRP Ed25519 private key must be 32 bytes".to_string(),
        ));
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    let signing_key = SigningKey::from_bytes(&key_array);
    let verifying_key = signing_key.verifying_key();

    let address = pub_key_to_classic_address(verifying_key.as_bytes());

    Ok(WalletInfo {
        address,
        private_key: hex::encode(signing_key.to_bytes()),
        mnemonic: None,
    })
}

pub fn detect_and_restore(secret: &str, derivation_index: u32) -> Result<WalletInfo> {
    let word_count = secret.split_whitespace().count();
    if word_count == 12 || word_count == 24 {
        restore_from_mnemonic(secret, derivation_index)
    } else {
        restore_from_private_key(secret)
    }
}
