use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub ca_path: Option<PathBuf>,
    pub client_auth: ClientAuthMode,
    pub min_tls_version: TlsVersion,
    pub max_tls_version: TlsVersion,
    pub cipher_suites: Vec<String>,
    pub prefer_server_cipher_order: bool,
}

/// Client authentication mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientAuthMode {
    None,
    RequestClientCert,
    RequireClientCert,
}

/// TLS version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
#[repr(u8)]
pub enum TlsVersion {
    Tls1_2 = 1,
    Tls1_3 = 2,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert_path: None,
            key_path: None,
            ca_path: None,
            client_auth: ClientAuthMode::None,
            min_tls_version: TlsVersion::Tls1_2,
            max_tls_version: TlsVersion::Tls1_3,
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ],
            prefer_server_cipher_order: true,
        }
    }
}

/// TLS manager for handling encryption in transit
pub struct TlsManager {
    config: TlsConfig,
}

impl TlsManager {
    /// Create a new TLS manager
    pub fn new(config: TlsConfig) -> Self {
        Self { config }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(TlsConfig::default())
    }
    
    /// Validate TLS configuration
    pub fn validate_config(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        
        if self.config.enabled {
            if self.config.cert_path.is_none() {
                errors.push("TLS is enabled but certificate path is not specified".to_string());
            }
            
            if self.config.key_path.is_none() {
                errors.push("TLS is enabled but key path is not specified".to_string());
            }
            
            if let Some(cert_path) = &self.config.cert_path {
                if !cert_path.exists() {
                    errors.push(format!("Certificate file does not exist: {}", cert_path.display()));
                }
            }
            
            if let Some(key_path) = &self.config.key_path {
                if !key_path.exists() {
                    errors.push(format!("Key file does not exist: {}", key_path.display()));
                }
            }
            
            if let Some(ca_path) = &self.config.ca_path {
                if !ca_path.exists() {
                    errors.push(format!("CA file does not exist: {}", ca_path.display()));
                }
            }
        }
        
        // Validate TLS version ordering
        if self.config.min_tls_version > self.config.max_tls_version {
            errors.push("Minimum TLS version cannot be greater than maximum TLS version".to_string());
        }
        
        Ok(errors)
    }
    
    /// Check if TLS is properly configured
    pub fn is_configured(&self) -> bool {
        if !self.config.enabled {
            return false;
        }
        
        self.config.cert_path.is_some() && self.config.key_path.is_some()
    }
    
    /// Get the configuration
    pub fn config(&self) -> &TlsConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: TlsConfig) {
        self.config = config;
    }
    
    /// Generate self-signed certificate for development
    pub fn generate_self_signed_cert(&self, output_dir: &Path) -> Result<(PathBuf, PathBuf)> {
        use std::process::Command;
        
        let cert_path = output_dir.join("server.crt");
        let key_path = output_dir.join("server.key");
        
        // Use OpenSSL to generate self-signed certificate
        let output = Command::new("openssl")
            .args([
                "req", "-x509", "-newkey", "rsa:4096",
                "-keyout", &key_path.to_string_lossy(),
                "-out", &cert_path.to_string_lossy(),
                "-days", "365",
                "-nodes",
                "-subj", "/CN=localhost/O=Development/C=US",
            ])
            .output()
            .context("Failed to generate self-signed certificate")?;
        
        if !output.status.success() {
            anyhow::bail!("OpenSSL command failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok((cert_path, key_path))
    }
    
    /// Get TLS configuration for Axum server
    pub fn get_axum_tls_config(&self) -> Result<Option<(PathBuf, PathBuf)>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        let cert_path = self.config.cert_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Certificate path not specified"))?
            .clone();
        
        let key_path = self.config.key_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Key path not specified"))?
            .clone();
        
        Ok(Some((cert_path, key_path)))
    }
    
    /// Get recommended security headers
    pub fn get_security_headers(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Strict-Transport-Security", "max-age=31536000; includeSubDomains"),
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("X-XSS-Protection", "1; mode=block"),
            ("Referrer-Policy", "strict-origin-when-cross-origin"),
            ("Permissions-Policy", "geolocation=(), microphone=(), camera=()"),
        ]
    }
}

impl Default for TlsManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tls_validation() {
        let config = TlsConfig::default();
        let manager = TlsManager::new(config);
        
        // Should have errors since cert/key paths are not set
        let errors = manager.validate_config().unwrap();
        assert!(!errors.is_empty());
    }
    
    #[test]
    fn test_tls_disabled() {
        let mut config = TlsConfig::default();
        config.enabled = false;
        let manager = TlsManager::new(config);
        
        // Should not have errors when TLS is disabled
        let errors = manager.validate_config().unwrap();
        assert!(errors.is_empty());
    }
    
    #[test]
    fn test_security_headers() {
        let manager = TlsManager::with_defaults();
        let headers = manager.get_security_headers();
        
        assert!(!headers.is_empty());
        assert!(headers.iter().any(|(k, _)| *k == "Strict-Transport-Security"));
    }
}