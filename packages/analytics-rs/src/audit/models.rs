use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Audit event for tracking system operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource: String,
    pub action: String,
    pub details: serde_json::Value,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    // Authentication events
    UserLogin,
    UserLogout,
    LoginFailed,
    PasswordChanged,
    PasswordResetRequested,
    MfaEnabled,
    MfaDisabled,
    
    // User management events
    UserCreated,
    UserUpdated,
    UserDeleted,
    RoleAssigned,
    RoleRevoked,
    
    // Data access events
    DataRead,
    DataExported,
    DataDeleted,
    
    // Configuration events
    ConfigRead,
    ConfigUpdated,
    
    // System events
    SystemStarted,
    SystemStopped,
    SystemError,
    
    // API events
    ApiAccess,
    ApiError,
    
    // Custom events
    Custom(String),
}

/// Severity levels for audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFilter {
    pub event_types: Option<Vec<AuditEventType>>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub severity: Option<AuditSeverity>,
    pub success: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

/// Query for audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub filters: AuditFilter,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: AuditEventType,
        severity: AuditSeverity,
        resource: String,
        action: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            severity,
            user_id: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            resource,
            action,
            details: serde_json::Value::Object(serde_json::Map::new()),
            success: true,
            error_message: None,
        }
    }
    
    /// Set user context
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Set session context
    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    /// Set request context
    pub fn with_request_context(mut self, ip_address: String, user_agent: String) -> Self {
        self.ip_address = Some(ip_address);
        self.user_agent = Some(user_agent);
        self
    }
    
    /// Set details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
    
    /// Mark as failed
    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }
}

impl AuditFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self {
            event_types: None,
            user_id: None,
            session_id: None,
            severity: None,
            success: None,
            start_time: None,
            end_time: None,
            resource: None,
            action: None,
        }
    }
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditQuery {
    /// Create a new audit query
    pub fn new() -> Self {
        Self {
            filters: AuditFilter::new(),
            limit: Some(100),
            offset: Some(0),
            order_by: Some("timestamp".to_string()),
            order_direction: Some("desc".to_string()),
        }
    }
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self::new()
    }
}