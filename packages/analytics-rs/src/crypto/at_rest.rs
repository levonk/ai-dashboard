use serde::{Deserialize, Serialize};
use anyhow::Result;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, AeadCore},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use std::sync::{Arc, RwLock};
use zeroize::Zeroize;

/// Encryption algorithms supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl Default for EncryptionAlgorithm {
    fn default() -> Self {
        Self::Aes256Gcm
    }
}

/// Configuration for encryption at rest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_rotation_days: u32,
    pub enabled: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::default(),
            key_rotation_days: 90,
            enabled: true,
        }
    }
}

/// Key manager for encryption keys
pub struct KeyManager {
    current_key: Arc<RwLock<Vec<u8>>>,
    key_version: Arc<RwLock<u32>>,
    key_created_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl KeyManager {
    /// Create a new key manager with a generated key
    pub fn new() -> Result<Self> {
        let key = Self::generate_key()?;
        Ok(Self {
            current_key: Arc::new(RwLock::new(key)),
            key_version: Arc::new(RwLock::new(1)),
            key_created_at: Arc::new(RwLock::new(chrono::Utc::now())),
        })
    }
    
    /// Create with a specific key
    pub fn with_key(key: Vec<u8>) -> Self {
        Self {
            current_key: Arc::new(RwLock::new(key)),
            key_version: Arc::new(RwLock::new(1)),
            key_created_at: Arc::new(RwLock::new(chrono::Utc::now())),
        }
    }
    
    /// Generate a new encryption key
    fn generate_key() -> Result<Vec<u8>> {
        let mut key = vec![0u8; 32]; // 256 bits for AES-256
        rand::thread_rng().fill(&mut key[..]);
        Ok(key)
    }
    
    /// Get the current encryption key
    pub fn get_key(&self) -> Result<Vec<u8>> {
        let key = self.current_key.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on key: {}", e))?;
        Ok(key.clone())
    }
    
    /// Get the current key version
    pub fn get_key_version(&self) -> Result<u32> {
        let version = self.key_version.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on key version: {}", e))?;
        Ok(*version)
    }
    
    /// Get the key creation time
    pub fn get_key_created_at(&self) -> Result<chrono::DateTime<chrono::Utc>> {
        let created_at = self.key_created_at.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on key created at: {}", e))?;
        Ok(*created_at)
    }
    
    /// Rotate the encryption key
    pub fn rotate_key(&self) -> Result<()> {
        let new_key = Self::generate_key()?;
        
        {
            let mut key = self.current_key.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on key: {}", e))?;
            key.zeroize();
            *key = new_key;
        }
        
        {
            let mut version = self.key_version.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on key version: {}", e))?;
            *version += 1;
        }
        
        {
            let mut created_at = self.key_created_at.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on key created at: {}", e))?;
            *created_at = chrono::Utc::now();
        }
        
        Ok(())
    }
    
    /// Check if key needs rotation
    pub fn needs_rotation(&self, max_days: u32) -> Result<bool> {
        let created_at = self.get_key_created_at()?;
        let age_days = (chrono::Utc::now() - created_at).num_days() as u32;
        Ok(age_days >= max_days)
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new().expect("Failed to generate encryption key")
    }
}

/// Encryption manager for data at rest
pub struct EncryptionManager {
    key_manager: Arc<KeyManager>,
    config: EncryptionConfig,
}

impl EncryptionManager {
    /// Create a new encryption manager
    pub fn new(key_manager: Arc<KeyManager>, config: EncryptionConfig) -> Self {
        Self { key_manager, config }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Result<Self> {
        let key_manager = Arc::new(KeyManager::new()?);
        Ok(Self::new(key_manager, EncryptionConfig::default()))
    }
    
    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String> {
        if !self.config.enabled {
            // Return base64 encoded plaintext if encryption is disabled
            return Ok(general_purpose::STANDARD.encode(plaintext));
        }
        
        let key = self.key_manager.get_key()?;
        
        match self.config.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
                
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext)
                    .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
                
                // Combine nonce and ciphertext
                let mut combined = nonce.to_vec();
                combined.extend_from_slice(&ciphertext);
                
                Ok(general_purpose::STANDARD.encode(&combined))
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // For now, use AES-256-GCM as fallback
                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
                
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext)
                    .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
                
                let mut combined = nonce.to_vec();
                combined.extend_from_slice(&ciphertext);
                
                Ok(general_purpose::STANDARD.encode(&combined))
            }
        }
    }
    
    /// Decrypt data
    pub fn decrypt(&self, ciphertext: &str) -> Result<Vec<u8>> {
        if !self.config.enabled {
            // Return base64 decoded plaintext if encryption is disabled
            return Ok(general_purpose::STANDARD.decode(ciphertext)?);
        }
        
        let key = self.key_manager.get_key()?;
        let combined = general_purpose::STANDARD.decode(ciphertext)?;
        
        if combined.len() < 12 {
            anyhow::bail!("Invalid ciphertext: too short");
        }
        
        let (nonce, ciphertext_bytes) = combined.split_at(12);
        
        match self.config.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
                
                let nonce = Nonce::from_slice(nonce);
                let plaintext = cipher.decrypt(nonce, ciphertext_bytes)
                    .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
                
                Ok(plaintext)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // For now, use AES-256-GCM as fallback
                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
                
                let nonce = Nonce::from_slice(nonce);
                let plaintext = cipher.decrypt(nonce, ciphertext_bytes)
                    .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
                
                Ok(plaintext)
            }
        }
    }
    
    /// Encrypt a string
    pub fn encrypt_string(&self, plaintext: &str) -> Result<String> {
        self.encrypt(plaintext.as_bytes())
    }
    
    /// Decrypt to a string
    pub fn decrypt_string(&self, ciphertext: &str) -> Result<String> {
        let bytes = self.decrypt(ciphertext)?;
        String::from_utf8(bytes).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
    }
    
    /// Get the key manager
    pub fn key_manager(&self) -> &Arc<KeyManager> {
        &self.key_manager
    }
    
    /// Get the configuration
    pub fn config(&self) -> &EncryptionConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: EncryptionConfig) {
        self.config = config;
    }
    
    /// Check if key rotation is needed
    pub fn needs_key_rotation(&self) -> Result<bool> {
        self.key_manager.needs_rotation(self.config.key_rotation_days)
    }
    
    /// Rotate the encryption key
    pub fn rotate_key(&self) -> Result<()> {
        self.key_manager.rotate_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let manager = EncryptionManager::with_defaults().unwrap();
        
        let plaintext = "Hello, World!";
        let ciphertext = manager.encrypt_string(plaintext).unwrap();
        let decrypted = manager.decrypt_string(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_key_rotation() {
        let manager = EncryptionManager::with_defaults().unwrap();
        
        let version_before = manager.key_manager.get_key_version().unwrap();
        manager.rotate_key().unwrap();
        let version_after = manager.key_manager.get_key_version().unwrap();
        
        assert_eq!(version_after, version_before + 1);
    }
    
    #[test]
    fn test_encryption_disabled() {
        let mut config = EncryptionConfig::default();
        config.enabled = false;
        let key_manager = Arc::new(KeyManager::new().unwrap());
        let manager = EncryptionManager::new(key_manager, config);
        
        let plaintext = "Hello, World!";
        let ciphertext = manager.encrypt_string(plaintext).unwrap();
        let decrypted = manager.decrypt_string(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}