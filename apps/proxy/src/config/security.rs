use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Comprehensive security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub authentication: AuthenticationConfig,
    pub authorization: AuthorizationConfig,
    pub audit: AuditSecurityConfig,
    pub retention: RetentionSecurityConfig,
    pub encryption: EncryptionSecurityConfig,
    pub tls: TlsSecurityConfig,
    pub password_policy: PasswordSecurityPolicy,
    pub session_config: SessionConfig,
    pub rate_limiting: RateLimitingConfig,
    pub security_headers: SecurityHeadersConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    pub enabled: bool,
    pub require_mfa: bool,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub session_timeout_hours: u32,
    pub password_reset_token_expiry_hours: u32,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_mfa: false,
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
            session_timeout_hours: 24,
            password_reset_token_expiry_hours: 1,
        }
    }
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    pub enabled: bool,
    pub default_role: String,
    pub role_hierarchy_enabled: bool,
    pub custom_permissions_enabled: bool,
}

/// Audit security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSecurityConfig {
    pub enabled: bool,
    pub log_to_file: bool,
    pub log_to_database: bool,
    pub retention_days: u32,
}

impl Default for AuditSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_to_file: false,
            log_to_database: true,
            retention_days: 90,
        }
    }
}

/// Retention security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionSecurityConfig {
    pub default_retention_days: u32,
    pub enforcement_enabled: bool,
}

impl Default for RetentionSecurityConfig {
    fn default() -> Self {
        Self {
            default_retention_days: 90,
            enforcement_enabled: true,
        }
    }
}

/// Encryption security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSecurityConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub key_rotation_days: u32,
}

impl Default for EncryptionSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "aes-256-gcm".to_string(),
            key_rotation_days: 90,
        }
    }
}

/// Password security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordSecurityPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
}

impl Default for PasswordSecurityPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: Some(90),
        }
    }
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_role: "viewer".to_string(),
            role_hierarchy_enabled: true,
            custom_permissions_enabled: true,
        }
    }
}

/// TLS security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSecurityConfig {
    pub enabled: bool,
    pub min_tls_version: String,
    pub max_tls_version: String,
    pub cipher_suites: Vec<String>,
    pub require_client_cert: bool,
}

impl Default for TlsSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_tls_version: "1.2".to_string(),
            max_tls_version: "1.3".to_string(),
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ],
            require_client_cert: false,
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub max_sessions_per_user: usize,
    pub session_cleanup_interval_hours: u32,
    pub remember_me_duration_days: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions_per_user: 5,
            session_cleanup_interval_hours: 1,
            remember_me_duration_days: 30,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
    pub login_attempts_per_minute: u32,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_size: 10,
            login_attempts_per_minute: 5,
        }
    }
}

/// Security headers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeadersConfig {
    pub strict_transport_security: bool,
    pub content_type_options: bool,
    pub frame_options: bool,
    pub xss_protection: bool,
    pub referrer_policy: String,
    pub permissions_policy: String,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            strict_transport_security: true,
            content_type_options: true,
            frame_options: true,
            xss_protection: true,
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: "geolocation=(), microphone=(), camera=()".to_string(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication: AuthenticationConfig::default(),
            authorization: AuthorizationConfig::default(),
            audit: AuditSecurityConfig::default(),
            retention: RetentionSecurityConfig::default(),
            encryption: EncryptionSecurityConfig::default(),
            tls: TlsSecurityConfig::default(),
            password_policy: PasswordSecurityPolicy::default(),
            session_config: SessionConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            security_headers: SecurityHeadersConfig::default(),
        }
    }
}

/// Security configuration validator
pub struct SecurityConfigValidator;

impl SecurityConfigValidator {
    /// Validate security configuration
    pub fn validate(config: &SecurityConfig) -> Result<Vec<SecurityValidationIssue>> {
        let mut issues = Vec::new();
        
        // Validate authentication config
        if config.authentication.enabled {
            if config.authentication.max_failed_attempts == 0 {
                issues.push(SecurityValidationIssue {
                    severity: ValidationSeverity::Warning,
                    category: "authentication".to_string(),
                    message: "Max failed attempts set to 0 may allow unlimited login attempts".to_string(),
                    recommendation: "Set max_failed_attempts to at least 3".to_string(),
                });
            }
            
            if config.authentication.lockout_duration_minutes == 0 {
                issues.push(SecurityValidationIssue {
                    severity: ValidationSeverity::Warning,
                    category: "authentication".to_string(),
                    message: "Lockout duration set to 0 may not provide effective protection".to_string(),
                    recommendation: "Set lockout_duration_minutes to at least 15".to_string(),
                });
            }
        }
        
        // Validate password policy
        if config.password_policy.min_length < 8 {
            issues.push(SecurityValidationIssue {
                severity: ValidationSeverity::Error,
                category: "password_policy".to_string(),
                message: "Password minimum length is less than 8 characters".to_string(),
                recommendation: "Set min_length to at least 8 (recommended: 12)".to_string(),
            });
        }
        
        if !config.password_policy.require_uppercase || !config.password_policy.require_lowercase {
            issues.push(SecurityValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "password_policy".to_string(),
                message: "Password policy does not require both uppercase and lowercase letters".to_string(),
                recommendation: "Enable both require_uppercase and require_lowercase".to_string(),
            });
        }
        
        // Validate TLS config
        if config.tls.enabled && config.tls.min_tls_version.as_str() < "1.2" {
            issues.push(SecurityValidationIssue {
                severity: ValidationSeverity::Error,
                category: "tls".to_string(),
                message: "Minimum TLS version is less than 1.2".to_string(),
                recommendation: "Set min_tls_version to at least 1.2 (recommended: 1.3)".to_string(),
            });
        }
        
        // Validate encryption config
        if !config.encryption.enabled {
            issues.push(SecurityValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "encryption".to_string(),
                message: "Encryption at rest is disabled".to_string(),
                recommendation: "Enable encryption for production environments".to_string(),
            });
        }
        
        // Validate rate limiting
        if config.rate_limiting.enabled {
            if config.rate_limiting.requests_per_minute == 0 {
                issues.push(SecurityValidationIssue {
                    severity: ValidationSeverity::Warning,
                    category: "rate_limiting".to_string(),
                    message: "Rate limiting is enabled but requests_per_minute is 0".to_string(),
                    recommendation: "Set requests_per_minute to a reasonable value (e.g., 60)".to_string(),
                });
            }
        } else {
            issues.push(SecurityValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "rate_limiting".to_string(),
                message: "Rate limiting is disabled".to_string(),
                recommendation: "Enable rate limiting for production environments".to_string(),
            });
        }
        
        // Validate audit config
        if !config.audit.enabled {
            issues.push(SecurityValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "audit".to_string(),
                message: "Audit logging is disabled".to_string(),
                recommendation: "Enable audit logging for security compliance".to_string(),
            });
        }
        
        Ok(issues)
    }
    
    /// Check if configuration is OWASP compliant
    pub fn check_owasp_compliance(config: &SecurityConfig) -> OwaspComplianceReport {
        let mut report = OwaspComplianceReport::default();
        
        // Check authentication
        if config.authentication.enabled && config.authentication.max_failed_attempts > 0 {
            report.brute_force_protection = true;
        }
        
        if config.password_policy.min_length >= 12 {
            report.strong_password_policy = true;
        }
        
        if config.tls.enabled && config.tls.min_tls_version.as_str() >= "1.2" {
            report.tls_encryption = true;
        }
        
        if config.audit.enabled {
            report.security_logging = true;
        }
        
        if config.encryption.enabled {
            report.data_encryption = true;
        }
        
        if config.security_headers.strict_transport_security {
            report.security_headers = true;
        }
        
        if config.rate_limiting.enabled {
            report.rate_limiting = true;
        }
        
        report.overall_compliance = report.calculate_overall_compliance();
        
        report
    }
}

/// Security validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityValidationIssue {
    pub severity: ValidationSeverity,
    pub category: String,
    pub message: String,
    pub recommendation: String,
}

/// Validation severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// OWASP compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwaspComplianceReport {
    pub brute_force_protection: bool,
    pub strong_password_policy: bool,
    pub tls_encryption: bool,
    pub security_logging: bool,
    pub data_encryption: bool,
    pub security_headers: bool,
    pub rate_limiting: bool,
    pub overall_compliance: f64,
}

impl Default for OwaspComplianceReport {
    fn default() -> Self {
        Self {
            brute_force_protection: false,
            strong_password_policy: false,
            tls_encryption: false,
            security_logging: false,
            data_encryption: false,
            security_headers: false,
            rate_limiting: false,
            overall_compliance: 0.0,
        }
    }
}

impl OwaspComplianceReport {
    fn calculate_overall_compliance(&self) -> f64 {
        let total = 7;
        let compliant = [
            self.brute_force_protection,
            self.strong_password_policy,
            self.tls_encryption,
            self.security_logging,
            self.data_encryption,
            self.security_headers,
            self.rate_limiting,
        ]
        .iter()
        .filter(|&&x| x)
        .count();
        
        (compliant as f64 / total as f64) * 100.0
    }
}

/// Security configuration manager
pub struct SecurityConfigManager {
    config: SecurityConfig,
}

impl SecurityConfigManager {
    /// Create a new security configuration manager
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SecurityConfig::default())
    }
    
    /// Get the configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: SecurityConfig) {
        self.config = config;
    }
    
    /// Validate current configuration
    pub fn validate(&self) -> Result<Vec<SecurityValidationIssue>> {
        SecurityConfigValidator::validate(&self.config)
    }
    
    /// Check OWASP compliance
    pub fn check_owasp_compliance(&self) -> OwaspComplianceReport {
        SecurityConfigValidator::check_owasp_compliance(&self.config)
    }
    
    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read security config file")?;
        
        let config: SecurityConfig = serde_yaml::from_str(&content)
            .context("Failed to parse security config")?;
        
        Ok(Self::new(config))
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_yaml::to_string(&self.config)
            .context("Failed to serialize security config")?;
        
        std::fs::write(path, content)
            .context("Failed to write security config file")?;
        
        Ok(())
    }
    
    /// Generate recommended configuration for production
    pub fn production_config() -> SecurityConfig {
        let mut config = SecurityConfig::default();
        
        // Strengthen password policy
        config.password_policy.min_length = 14;
        config.password_policy.require_special_chars = true;
        
        // Strengthen authentication
        config.authentication.max_failed_attempts = 3;
        config.authentication.lockout_duration_minutes = 15;
        
        // Strengthen TLS
        config.tls.min_tls_version = "1.3".to_string();
        config.tls.max_tls_version = "1.3".to_string();
        
        // Enable all security features
        config.audit.enabled = true;
        config.encryption.enabled = true;
        config.rate_limiting.enabled = true;
        
        config
    }
    
    /// Generate recommended configuration for development
    pub fn development_config() -> SecurityConfig {
        let mut config = SecurityConfig::default();
        
        // More lenient for development
        config.password_policy.min_length = 8;
        config.authentication.max_failed_attempts = 10;
        config.authentication.lockout_duration_minutes = 5;
        
        // Disable some strict checks for development
        config.tls.enabled = false;
        config.rate_limiting.requests_per_minute = 1000;
        
        config
    }
}

impl Default for SecurityConfigManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_validation() {
        let config = SecurityConfig::default();
        let issues = SecurityConfigValidator::validate(&config).unwrap();
        
        // Should have some warnings with default config
        assert!(!issues.is_empty());
    }
    
    #[test]
    fn test_owasp_compliance() {
        let config = SecurityConfig::default();
        let report = SecurityConfigValidator::check_owasp_compliance(&config);
        
        // Default config should have some compliance
        assert!(report.overall_compliance > 0.0);
    }
    
    #[test]
    fn test_production_config() {
        let config = SecurityConfigManager::production_config();
        let issues = SecurityConfigValidator::validate(&config).unwrap();
        
        // Production config should have no errors
        let errors: Vec<_> = issues.iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .collect();
        assert!(errors.is_empty());
    }
}
