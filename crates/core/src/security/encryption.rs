use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use crate::{config::SecurityConfig, error::Result, Error};

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn new(config: &SecurityConfig) -> Result<Self> {
        let key_bytes = config.aes_encryption_key.as_bytes();
        
        if key_bytes.len() != 32 {
            return Err(Error::new(
                crate::error::ErrorCode::EncryptionError,
                "AES key must be exactly 32 bytes"
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| Error::new(crate::error::ErrorCode::EncryptionError, format!("Encryption failed: {}", e)))?;

        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(Error::new(crate::error::ErrorCode::DecryptionError, "Invalid ciphertext length"));
        }

        let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| Error::new(crate::error::ErrorCode::DecryptionError, format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<String> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let encrypted = self.encrypt(plaintext.as_bytes())?;
        Ok(STANDARD.encode(&encrypted))
    }

    pub fn decrypt_string(&self, ciphertext: &str) -> Result<String> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let decoded = STANDARD.decode(ciphertext)
            .map_err(|e| Error::new(crate::error::ErrorCode::DecryptionError, format!("Invalid base64: {}", e)))?;
        
        let decrypted = self.decrypt(&decoded)?;
        
        String::from_utf8(decrypted)
            .map_err(|e| Error::new(crate::error::ErrorCode::DecryptionError, format!("Invalid UTF-8: {}", e)))
    }
}

