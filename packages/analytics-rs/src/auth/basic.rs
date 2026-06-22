use crate::auth::models::{User, Credentials, AuthResult, AuthError, PasswordPolicy, PasswordStrength};
use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use std::collections::VecDeque;

/// Password hasher using Argon2id
pub struct BasicPasswordHasher {
    argon2: Argon2<'static>,
}

impl BasicPasswordHasher {
    /// Create a new password hasher with secure defaults
    pub fn new() -> Self {
        let argon2 = Argon2::default();
        Self { argon2 }
    }
    
    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();
        Ok(password_hash)
    }
    
    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
        
        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map(|_| true)
            .map_err(|_| anyhow::anyhow!("Password verification failed"))
    }
}

impl Default for BasicPasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic authentication system
pub struct BasicAuth {
    hasher: BasicPasswordHasher,
    password_policy: PasswordPolicy,
    password_history: VecDeque<String>, // Store last N password hashes
    max_failed_attempts: u32,
    lockout_duration_minutes: u32,
}

impl BasicAuth {
    /// Create a new basic authentication system
    pub fn new() -> Self {
        Self {
            hasher: BasicPasswordHasher::new(),
            password_policy: PasswordPolicy::default(),
            password_history: VecDeque::with_capacity(5),
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
        }
    }
    
    /// Create with custom password policy
    pub fn with_policy(policy: PasswordPolicy) -> Self {
        let mut auth = Self::new();
        auth.password_policy = policy;
        auth
    }
    
    /// Authenticate a user with credentials
    pub fn authenticate(&self, user: &User, credentials: &Credentials) -> AuthResult {
        // Check if user is locked
        if let Some(locked_until) = user.locked_until {
            if locked_until > chrono::Utc::now() {
                return AuthResult::Failure {
                    reason: AuthError::UserLocked,
                };
            }
        }
        
        // Verify password
        match self.hasher.verify_password(&credentials.password, &user.password_hash) {
            Ok(true) => {
                // Successful authentication
                AuthResult::Success {
                    user_id: user.id,
                    session_id: crate::auth::models::SessionId::new_v4(),
                }
            }
            Ok(false) => {
                // Invalid password
                AuthResult::Failure {
                    reason: AuthError::InvalidCredentials,
                }
            }
            Err(e) => {
                // Hash verification error
                AuthResult::Failure {
                    reason: AuthError::InternalError(e.to_string()),
                }
            }
        }
    }
    
    /// Create a password hash for a new user
    pub fn create_password_hash(&self, password: &str) -> Result<String> {
        self.validate_password_strength(password)?;
        self.hasher.hash_password(password)
    }
    
    /// Validate password strength against policy
    pub fn validate_password_strength(&self, password: &str) -> Result<PasswordStrength> {
        let policy = &self.password_policy;
        
        // Check minimum length
        if password.len() < policy.min_length {
            anyhow::bail!("Password must be at least {} characters", policy.min_length);
        }
        
        // Check character requirements
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_number = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());
        
        if policy.require_uppercase && !has_upper {
            anyhow::bail!("Password must contain uppercase letters");
        }
        if policy.require_lowercase && !has_lower {
            anyhow::bail!("Password must contain lowercase letters");
        }
        if policy.require_numbers && !has_number {
            anyhow::bail!("Password must contain numbers");
        }
        if policy.require_special_chars && !has_special {
            anyhow::bail!("Password must contain special characters");
        }
        
        // Calculate strength
        let strength = self.calculate_strength(password, has_upper, has_lower, has_number, has_special);
        Ok(strength)
    }
    
    /// Calculate password strength
    fn calculate_strength(&self, password: &str, has_upper: bool, has_lower: bool, has_number: bool, has_special: bool) -> PasswordStrength {
        let mut score = 0;
        
        // Length score
        score += (password.len() as u32 / 4).min(3);
        
        // Character variety score
        if has_upper { score += 1; }
        if has_lower { score += 1; }
        if has_number { score += 1; }
        if has_special { score += 1; }
        
        match score {
            0..=3 => PasswordStrength::Weak,
            4..=5 => PasswordStrength::Fair,
            6..=7 => PasswordStrength::Good,
            _ => PasswordStrength::Strong,
        }
    }
    
    /// Check if password is in history (for reuse prevention)
    pub fn is_password_reused(&self, password: &str) -> bool {
        if !self.password_policy.prevent_reuse {
            return false;
        }
        
        self.password_history.iter()
            .any(|hash| {
                self.hasher.verify_password(password, hash).unwrap_or(false)
            })
    }
    
    /// Add password to history
    pub fn add_to_history(&mut self, password_hash: String) {
        if self.password_history.len() >= self.password_policy.history_count {
            self.password_history.pop_front();
        }
        self.password_history.push_back(password_hash);
    }
    
    /// Check if user should be locked due to failed attempts
    pub fn should_lock_user(&self, failed_attempts: u32) -> bool {
        failed_attempts >= self.max_failed_attempts
    }
    
    /// Calculate lockout expiration
    pub fn calculate_lockout_expiration(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now() + chrono::Duration::minutes(self.lockout_duration_minutes as i64)
    }
}

impl Default for BasicAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_hashing() {
        let hasher = BasicPasswordHasher::new();
        let password = "TestPassword123!";
        
        let hash = hasher.hash_password(password).unwrap();
        assert!(hasher.verify_password(password, &hash).unwrap());
        assert!(!hasher.verify_password("wrongpassword", &hash).unwrap());
    }
    
    #[test]
    fn test_password_validation() {
        let auth = BasicAuth::new();
        
        // Strong password
        let result = auth.validate_password_strength("StrongP@ssw0rd123");
        assert!(result.is_ok());
        
        // Weak password (too short)
        let result = auth.validate_password_strength("Weak1!");
        assert!(result.is_err());
        
        // Missing uppercase
        let result = auth.validate_password_strength("lowercase123!");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_password_strength() {
        let auth = BasicAuth::new();
        
        let weak = auth.validate_password_strength("weak").unwrap();
        assert_eq!(weak, PasswordStrength::Weak);
        
        let strong = auth.validate_password_strength("Str0ng!P@ssw0rd").unwrap();
        assert_eq!(strong, PasswordStrength::Strong);
    }
}