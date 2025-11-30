use aes_gcm::{ aead::{ Aead, KeyInit, OsRng }, Aes256Gcm, Nonce };
use rand::RngCore;

use crate::error::{ AppError, Result };

pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(AppError::Encryption("Encryption key must be 32 bytes".to_string()));
        }

        let cipher = Aes256Gcm::new_from_slice(key).map_err(|e|
            AppError::Encryption(e.to_string())
        )?;

        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::Encryption(e.to_string()))?;

        // Combine nonce + ciphertext and encode as hex
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(hex::encode(combined))
    }

    pub fn decrypt(&self, encrypted_hex: &str) -> Result<String> {
        let combined = hex
            ::decode(encrypted_hex)
            .map_err(|e| AppError::Encryption(format!("Invalid hex: {}", e)))?;

        if combined.len() < 12 {
            return Err(AppError::Encryption("Encrypted data too short".to_string()));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Encryption(e.to_string()))?;

        String::from_utf8(plaintext).map_err(|e|
            AppError::Encryption(format!("Invalid UTF-8: {}", e))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32];
        let encryptor = Encryptor::new(&key).unwrap();

        let plaintext = "test private key 0x1234567890abcdef";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let key = [0u8; 32];
        let encryptor = Encryptor::new(&key).unwrap();

        let plaintext = "same plaintext";
        let encrypted1 = encryptor.encrypt(plaintext).unwrap();
        let encrypted2 = encryptor.encrypt(plaintext).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(encryptor.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(encryptor.decrypt(&encrypted2).unwrap(), plaintext);
    }
}
