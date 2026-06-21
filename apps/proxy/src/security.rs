use anyhow::{Context, Result};
use secrecy::{Secret, ExposeSecret};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tracing::{warn, info, debug};
use uuid::Uuid;
use zeroize::Zeroize;

/// Security utilities for handling sensitive data
pub struct SecurityUtils;

impl SecurityUtils {
    /// Patterns that indicate sensitive data (passwords, tokens, keys)
    const SECRET_PATTERNS: &'static [&'static str] = &[
        "password",
        "passwd",
        "pwd",
        "token",
        "api_key",
        "apikey",
        "secret",
        "private_key",
        "privatekey",
        "auth",
        "credential",
        "cred",
        "bearer",
        "session",
        "cookie",
    ];

    /// Redact potential secrets from a string
    pub fn redact_secrets(input: &str) -> String {
        let mut result = input.to_string();
        
        for pattern in Self::SECRET_PATTERNS {
            // Look for patterns like "password=value" or "password: value"
            let patterns_to_check = vec![
                format!("{}=", pattern),
                format!("{}:", pattern),
                format!("{} = ", pattern),
                format!("{} : ", pattern),
            ];
            
            for search_pattern in patterns_to_check {
                if let Some(pos) = result.find(&search_pattern) {
                    let start = pos + search_pattern.len();
                    // Find the end of the value (until next space, comma, or end of string)
                    let end = result[start..]
                        .find(|c: char| c.is_whitespace() || c == ',')
                        .map(|p| start + p)
                        .unwrap_or(result.len());
                    
                    // Replace the value with asterisks
                    let redacted = "*****".repeat((end - start).min(10));
                    result.replace_range(start..end, &redacted);
                }
            }
        }
        
        result
    }

    /// Check if a string contains potential secret patterns
    pub fn contains_secret(input: &str) -> bool {
        let lower = input.to_lowercase();
        Self::SECRET_PATTERNS.iter().any(|pattern| lower.contains(pattern))
    }

    /// Securely zero a string's memory
    pub fn zero_string(s: &mut String) {
        s.zeroize();
    }

    /// Securely zero a byte slice
    pub fn zero_bytes(bytes: &mut Vec<u8>) {
        bytes.zeroize();
    }

    /// Validate config file permissions (should be user-readable only)
    pub fn validate_config_permissions(path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false); // File doesn't exist, can't validate
        }

        #[cfg(unix)]
        {
            let metadata = fs::metadata(path)
                .context("Failed to read file metadata")?;
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Check if file is readable by others (world-readable)
            let world_readable = mode & 0o004 != 0;
            // Check if file is readable by group
            let group_readable = mode & 0o040 != 0;

            if world_readable || group_readable {
                warn!(
                    "Config file {} has insecure permissions: {:o}. Should be 0600 (user-readable only).",
                    path.display(),
                    mode & 0o777
                );
                return Ok(false);
            }

            debug!("Config file {} has secure permissions: {:o}", path.display(), mode & 0o777);
            Ok(true)
        }

        #[cfg(windows)]
        {
            // Windows uses ACLs, not Unix permissions
            // For now, we'll just log a warning if the file is in a shared location
            if let Some(parent) = path.parent() {
                if parent.to_string_lossy().contains("ProgramData") 
                    || parent.to_string_lossy().contains("Public") {
                    warn!("Config file in shared location: {}", path.display());
                    return Ok(false);
                }
            }
            Ok(true)
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Unknown platform, assume secure
            Ok(true)
        }
    }

    /// Set secure permissions on a config file (user-readable only)
    pub fn set_secure_permissions(path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(()); // File doesn't exist, nothing to set
        }

        #[cfg(unix)]
        {
            let metadata = fs::metadata(path)
                .context("Failed to read file metadata")?;
            let mut permissions = metadata.permissions();
            
            // Set to user-readable and writable only (0600)
            permissions.set_mode(0o600);
            fs::set_permissions(path, permissions)
                .context("Failed to set file permissions")?;
            
            info!("Set secure permissions (0600) on {}", path.display());
        }

        #[cfg(windows)]
        {
            // On Windows, we can't easily set Unix-style permissions
            // We'll just log a warning if the file is in a shared location
            debug!("Permission setting not fully supported on Windows for {}", path.display());
        }

        #[cfg(not(any(unix, windows)))]
        {
            debug!("Permission setting not supported on this platform");
        }

        Ok(())
    }

    /// Validate input to prevent injection attacks
    pub fn validate_input(input: &str) -> Result<()> {
        // Check for shell metacharacters that could be used for injection
        const DANGEROUS_CHARS: &[char] = &[';', '&', '|', '`', '$', '(', ')', '<', '>', '\n', '\r'];
        
        if input.chars().any(|c| DANGEROUS_CHARS.contains(&c)) {
            anyhow::bail!("Input contains potentially dangerous characters: {}", input);
        }

        // Check for command substitution patterns
        if input.contains("$(") || input.contains("`") {
            anyhow::bail!("Input contains command substitution patterns: {}", input);
        }

        // Check for path traversal attempts
        if input.contains("..") {
            anyhow::bail!("Input contains path traversal attempt: {}", input);
        }

        Ok(())
    }

    /// Sanitize input for safe use in commands
    pub fn sanitize_input(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\'', "\\'")
            .replace('$', "\\$")
            .replace('`', "\\`")
    }

    /// Check if running in a secure environment
    pub fn is_secure_environment() -> bool {
        // Check if running as root (insecure for CLI tools)
        #[cfg(unix)]
        {
            if unsafe { libc::geteuid() } == 0 {
                warn!("Running as root - this is insecure for CLI tools");
                return false;
            }
        }

        // Check if terminal is accessible (security risk in some contexts)
        if std::env::var("SSH_TTY").is_ok() {
            debug!("Running over SSH - additional security considerations apply");
        }

        true
    }

    /// Generate a warning message for insecure configuration
    pub fn insecure_config_warning(method: &str) -> String {
        format!(
            "⚠️  WARNING: Using insecure configuration method '{}'. \
             Consider using secure storage (OS keyring) or environment variables.",
            method
        )
    }
}

/// API key manager for secure storage and validation
pub struct ApiKeyManager {
    keys: HashMap<String, SecureString>,
    key_rotation_enabled: bool,
}

impl ApiKeyManager {
    /// Create a new API key manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            key_rotation_enabled: true,
        }
    }

    /// Add an API key for a provider
    pub fn add_key(&mut self, provider: String, key: String) -> Result<()> {
        if key.is_empty() {
            anyhow::bail!("API key cannot be empty");
        }

        // Validate key format (basic check)
        if key.len() < 16 {
            warn!("API key for provider {} appears to be unusually short", provider);
        }

        self.keys.insert(provider.clone(), SecureString::new(key));
        info!("Added API key for provider: {}", provider);
        Ok(())
    }

    /// Get an API key for a provider
    pub fn get_key(&self, provider: &str) -> Option<&str> {
        self.keys.get(provider).map(|k| k.expose())
    }

    /// Remove an API key
    pub fn remove_key(&mut self, provider: &str) -> bool {
        self.keys.remove(provider).is_some()
    }

    /// Check if a provider has a key configured
    pub fn has_key(&self, provider: &str) -> bool {
        self.keys.contains_key(provider)
    }

    /// Get all provider names with configured keys
    pub fn list_providers(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    /// Enable or disable key rotation
    pub fn set_key_rotation(&mut self, enabled: bool) {
        self.key_rotation_enabled = enabled;
    }

    /// Validate API key format for specific providers
    pub fn validate_key_format(provider: &str, key: &str) -> Result<()> {
        match provider.to_lowercase().as_str() {
            "anthropic" => {
                if !key.starts_with("sk-ant-") {
                    anyhow::bail!("Anthropic API key should start with 'sk-ant-'");
                }
            }
            "openai" => {
                if !key.starts_with("sk-") {
                    anyhow::bail!("OpenAI API key should start with 'sk-'");
                }
            }
            "google" => {
                // Google API keys are typically alphanumeric strings
                if key.len() < 20 || key.len() > 100 {
                    anyhow::bail!("Google API key length appears invalid");
                }
            }
            _ => {
                // No specific validation for other providers
                debug!("No specific validation for provider: {}", provider);
            }
        }
        Ok(())
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Request validator for security checks
pub struct RequestValidator {
    max_request_size: usize,
    max_response_size: usize,
    allowed_origins: Vec<String>,
    rate_limit_enabled: bool,
}

impl RequestValidator {
    /// Create a new request validator
    pub fn new() -> Self {
        Self {
            max_request_size: 10 * 1024 * 1024, // 10MB
            max_response_size: 100 * 1024 * 1024, // 100MB
            allowed_origins: vec!["*".to_string()], // Allow all by default
            rate_limit_enabled: true,
        }
    }

    /// Validate request size
    pub fn validate_request_size(&self, size: usize) -> Result<()> {
        if size > self.max_request_size {
            anyhow::bail!("Request size {} exceeds maximum allowed size {}", size, self.max_request_size);
        }
        Ok(())
    }

    /// Validate response size
    pub fn validate_response_size(&self, size: usize) -> Result<()> {
        if size > self.max_response_size {
            anyhow::bail!("Response size {} exceeds maximum allowed size {}", size, self.max_response_size);
        }
        Ok(())
    }

    /// Validate request origin
    pub fn validate_origin(&self, origin: Option<&str>) -> Result<()> {
        if let Some(origin) = origin {
            if self.allowed_origins.contains(&"*".to_string()) {
                return Ok(());
            }
            
            if !self.allowed_origins.iter().any(|allowed| {
                allowed == "*" || origin == allowed || origin.ends_with(allowed)
            }) {
                anyhow::bail!("Origin '{}' is not allowed", origin);
            }
        }
        Ok(())
    }

    /// Validate request headers for security
    pub fn validate_headers(&self, headers: &HashMap<String, String>) -> Result<()> {
        // Check for suspicious headers
        const SUSPICIOUS_HEADERS: &[&str] = &[
            "x-forwarded-for",
            "x-real-ip",
            "x-originating-ip",
        ];

        for (key, value) in headers {
            let key_lower = key.to_lowercase();
            if SUSPICIOUS_HEADERS.contains(&key_lower.as_str()) {
                // Basic validation - check for IP spoofing patterns
                if value.contains("127.0.0.1") || value.contains("localhost") {
                    warn!("Suspicious header value detected: {} = {}", key, value);
                }
            }
        }

        Ok(())
    }

    /// Sanitize request path
    pub fn sanitize_path(&self, path: &str) -> Result<String> {
        // Remove path traversal attempts
        if path.contains("..") {
            anyhow::bail!("Path traversal attempt detected in path: {}", path);
        }

        // Remove null bytes
        if path.contains('\0') {
            anyhow::bail!("Null byte detected in path: {}", path);
        }

        // Normalize path
        let normalized = path
            .replace("//", "/")
            .replace("/./", "/");

        Ok(normalized)
    }

    /// Generate a unique request ID
    pub fn generate_request_id() -> String {
        format!("req_{}", Uuid::new_v4())
    }

    /// Set maximum request size
    pub fn set_max_request_size(&mut self, size: usize) {
        self.max_request_size = size;
    }

    /// Set maximum response size
    pub fn set_max_response_size(&mut self, size: usize) {
        self.max_response_size = size;
    }

    /// Set allowed origins
    pub fn set_allowed_origins(&mut self, origins: Vec<String>) {
        self.allowed_origins = origins;
    }

    /// Enable or disable rate limiting
    pub fn set_rate_limit(&mut self, enabled: bool) {
        self.rate_limit_enabled = enabled;
    }
}

impl Default for RequestValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter for API requests
pub struct RateLimiter {
    requests_per_minute: u32,
    requests_per_hour: u32,
    // In production, this would use a proper rate limiting data structure
    // For now, we'll use a simple in-memory counter
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
        }
    }

    /// Check if a request should be rate limited
    pub fn check_rate_limit(&self, client_id: &str) -> Result<()> {
        // TODO: Implement actual rate limiting logic
        // This would typically use Redis or an in-memory data structure
        debug!("Checking rate limit for client: {}", client_id);
        Ok(())
    }

    /// Set requests per minute limit
    pub fn set_requests_per_minute(&mut self, limit: u32) {
        self.requests_per_minute = limit;
    }

    /// Set requests per hour limit
    pub fn set_requests_per_hour(&mut self, limit: u32) {
        self.requests_per_hour = limit;
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Secure string wrapper that automatically zeros on drop
#[derive(Clone)]
pub struct SecureString {
    inner: Secret<String>,
}

impl SecureString {
    /// Create a new secure string
    pub fn new(s: String) -> Self {
        Self {
            inner: Secret::new(s),
        }
    }

    /// Expose the inner string (use with caution)
    pub fn expose(&self) -> &str {
        self.inner.expose_secret()
    }

    /// Get the length of the string without exposing it
    pub fn len(&self) -> usize {
        self.inner.expose_secret().len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.inner.expose_secret().is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // The Secret type handles zeroization automatically
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secrets() {
        let input = "password=secret123 token=abc123";
        let redacted = SecurityUtils::redact_secrets(input);
        assert!(!redacted.contains("secret123"));
        assert!(!redacted.contains("abc123"));
        assert!(redacted.contains("*****"));
    }

    #[test]
    fn test_contains_secret() {
        assert!(SecurityUtils::contains_secret("password=123"));
        assert!(SecurityUtils::contains_secret("api_key=abc"));
        assert!(!SecurityUtils::contains_secret("name=value"));
    }

    #[test]
    fn test_zero_string() {
        let mut s = String::from("secret");
        SecurityUtils::zero_string(&mut s);
        // After zeroing, the string should be empty or zeros
        assert!(s.is_empty() || s.chars().all(|c| c == '\0'));
    }

    #[test]
    fn test_validate_input_safe() {
        assert!(SecurityUtils::validate_input("safe_input").is_ok());
        assert!(SecurityUtils::validate_input("another-safe").is_ok());
    }

    #[test]
    fn test_validate_input_unsafe() {
        assert!(SecurityUtils::validate_input("rm -rf /").is_err());
        assert!(SecurityUtils::validate_input("echo $(whoami)").is_err());
        assert!(SecurityUtils::validate_input("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        let input = "test\"value";
        let sanitized = SecurityUtils::sanitize_input(input);
        assert!(sanitized.contains("\\\""));
        assert!(!sanitized.contains("\""));
    }

    #[test]
    fn test_secure_string() {
        let secure = SecureString::new("secret".to_string());
        assert_eq!(secure.len(), 6);
        assert!(!secure.is_empty());
        assert_eq!(secure.expose(), "secret");
    }

    #[test]
    fn test_secure_string_from_str() {
        let secure: SecureString = "secret".into();
        assert_eq!(secure.len(), 6);
    }

    #[test]
    fn test_insecure_config_warning() {
        let warning = SecurityUtils::insecure_config_warning("plaintext");
        assert!(warning.contains("WARNING"));
        assert!(warning.contains("plaintext"));
    }
}
