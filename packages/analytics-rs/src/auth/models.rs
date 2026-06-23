use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashSet;

/// Unique identifier for a user
pub type UserId = Uuid;

/// Unique identifier for a session
pub type SessionId = Uuid;

/// User account with authentication and authorization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
}

/// User roles with hierarchical permissions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    /// Full system access
    Admin,
    /// Read-only access to all data
    Viewer,
    /// Read access with limited write permissions
    Analyst,
}

/// Individual permission for fine-grained access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // User management
    CreateUser,
    ReadUser,
    UpdateUser,
    DeleteUser,
    
    // Analytics data
    ReadAnalytics,
    ExportAnalytics,
    DeleteAnalytics,
    
    // Configuration
    ReadConfig,
    UpdateConfig,
    
    // Audit logs
    ReadAuditLogs,
    
    // Alerts
    ManageAlerts,
    
    // System administration
    SystemAdmin,
}

/// User session for authenticated access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// Authentication token for API access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
}

/// Types of authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Bearer,
    ApiKey,
    Session,
}

/// Result of authentication attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthResult {
    Success { user_id: UserId, session_id: SessionId },
    Failure { reason: AuthError },
}

/// Authentication error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    UserLocked,
    SessionExpired,
    SessionNotFound,
    InvalidToken,
    PermissionDenied,
    RateLimited,
    MfaRequired,
    InternalError(String),
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub prevent_reuse: bool,
    pub history_count: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: Some(90),
            prevent_reuse: true,
            history_count: 5,
        }
    }
}

/// Password strength assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Fair,
    Good,
    Strong,
}

impl Role {
    /// Get all permissions for this role
    pub fn permissions(&self) -> HashSet<Permission> {
        match self {
            Role::Admin => {
                let mut perms = HashSet::new();
                perms.insert(Permission::CreateUser);
                perms.insert(Permission::ReadUser);
                perms.insert(Permission::UpdateUser);
                perms.insert(Permission::DeleteUser);
                perms.insert(Permission::ReadAnalytics);
                perms.insert(Permission::ExportAnalytics);
                perms.insert(Permission::DeleteAnalytics);
                perms.insert(Permission::ReadConfig);
                perms.insert(Permission::UpdateConfig);
                perms.insert(Permission::ReadAuditLogs);
                perms.insert(Permission::ManageAlerts);
                perms.insert(Permission::SystemAdmin);
                perms
            }
            Role::Viewer => {
                let mut perms = HashSet::new();
                perms.insert(Permission::ReadUser);
                perms.insert(Permission::ReadAnalytics);
                perms.insert(Permission::ReadConfig);
                perms.insert(Permission::ReadAuditLogs);
                perms
            }
            Role::Analyst => {
                let mut perms = HashSet::new();
                perms.insert(Permission::ReadUser);
                perms.insert(Permission::ReadAnalytics);
                perms.insert(Permission::ExportAnalytics);
                perms.insert(Permission::ReadConfig);
                perms.insert(Permission::ReadAuditLogs);
                perms.insert(Permission::ManageAlerts);
                perms
            }
        }
    }
    
    /// Check if this role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::UserLocked => write!(f, "User account is locked"),
            AuthError::SessionExpired => write!(f, "Session has expired"),
            AuthError::SessionNotFound => write!(f, "Session not found"),
            AuthError::InvalidToken => write!(f, "Invalid authentication token"),
            AuthError::PermissionDenied => write!(f, "Permission denied"),
            AuthError::RateLimited => write!(f, "Rate limit exceeded"),
            AuthError::MfaRequired => write!(f, "Multi-factor authentication required"),
            AuthError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}