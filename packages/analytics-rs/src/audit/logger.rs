use crate::audit::models::{AuditEvent, AuditEventType, AuditSeverity};
use crate::audit::storage::AuditStorage;
use anyhow::Result;
use std::sync::Arc;

/// Configuration for audit logging
#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_to_file: bool,
    pub log_to_database: bool,
    pub retention_days: u32,
    pub log_all_events: bool,
    pub min_severity: AuditSeverity,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_to_file: false,
            log_to_database: true,
            retention_days: 90,
            log_all_events: true,
            min_severity: AuditSeverity::Info,
        }
    }
}

/// Audit logger for recording system events
pub struct AuditLogger {
    storage: Arc<dyn AuditStorage>,
    config: AuditConfig,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(storage: Arc<dyn AuditStorage>, config: AuditConfig) -> Self {
        Self { storage, config }
    }
    
    /// Create with default configuration
    pub fn with_storage(storage: Arc<dyn AuditStorage>) -> Self {
        Self::new(storage, AuditConfig::default())
    }
    
    /// Log an audit event
    pub fn log(&self, event: AuditEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Check severity threshold
        if event.severity < self.config.min_severity {
            return Ok(());
        }
        
        // Store the event
        self.storage.store_event(event)
    }
    
    /// Log a user login event
    pub fn log_user_login(
        &self,
        user_id: uuid::Uuid,
        session_id: uuid::Uuid,
        ip_address: String,
        user_agent: String,
        success: bool,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::UserLogin,
            if success { AuditSeverity::Info } else { AuditSeverity::Warning },
            "authentication".to_string(),
            "login".to_string(),
        )
        .with_user(user_id)
        .with_session(session_id)
        .with_request_context(ip_address, user_agent);
        
        let event = if success {
            event
        } else {
            event.with_error("Authentication failed".to_string())
        };
        
        self.log(event)
    }
    
    /// Log a user logout event
    pub fn log_user_logout(
        &self,
        user_id: uuid::Uuid,
        session_id: uuid::Uuid,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::UserLogout,
            AuditSeverity::Info,
            "authentication".to_string(),
            "logout".to_string(),
        )
        .with_user(user_id)
        .with_session(session_id);
        
        self.log(event)
    }
    
    /// Log a data access event
    pub fn log_data_access(
        &self,
        user_id: uuid::Uuid,
        session_id: uuid::Uuid,
        resource: String,
        action: String,
        success: bool,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::DataRead,
            if success { AuditSeverity::Info } else { AuditSeverity::Warning },
            resource,
            action,
        )
        .with_user(user_id)
        .with_session(session_id);
        
        let event = if success {
            event
        } else {
            event.with_error("Data access failed".to_string())
        };
        
        self.log(event)
    }
    
    /// Log a configuration change event
    pub fn log_config_change(
        &self,
        user_id: uuid::Uuid,
        resource: String,
        action: String,
        details: serde_json::Value,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::ConfigUpdated,
            AuditSeverity::Warning,
            resource,
            action,
        )
        .with_user(user_id)
        .with_details(details);
        
        self.log(event)
    }
    
    /// Log a user management event
    pub fn log_user_management(
        &self,
        admin_user_id: uuid::Uuid,
        target_user_id: uuid::Uuid,
        action: String,
        details: serde_json::Value,
    ) -> Result<()> {
        let event = AuditEvent::new(
            match action.as_str() {
                "create" => AuditEventType::UserCreated,
                "update" => AuditEventType::UserUpdated,
                "delete" => AuditEventType::UserDeleted,
                _ => AuditEventType::Custom(action.clone()),
            },
            AuditSeverity::Warning,
            "user_management".to_string(),
            action,
        )
        .with_user(admin_user_id)
        .with_details({
            let mut details_map = serde_json::Map::new();
            details_map.insert("target_user_id".to_string(), serde_json::Value::String(target_user_id.to_string()));
            if let serde_json::Value::Object(map) = details {
                for (key, value) in map {
                    details_map.insert(key, value);
                }
            }
            serde_json::Value::Object(details_map)
        });
        
        self.log(event)
    }
    
    /// Log a system error event
    pub fn log_system_error(
        &self,
        error_message: String,
        resource: String,
        details: serde_json::Value,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::SystemError,
            AuditSeverity::Error,
            resource,
            "error".to_string(),
        )
        .with_error(error_message)
        .with_details(details);
        
        self.log(event)
    }
    
    /// Get the storage backend
    pub fn storage(&self) -> &Arc<dyn AuditStorage> {
        &self.storage
    }
    
    /// Get the configuration
    pub fn config(&self) -> &AuditConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AuditConfig) {
        self.config = config;
    }
    
    /// Clean up old audit events
    pub fn cleanup_old_events(&self) -> Result<usize> {
        let retention_duration = chrono::Duration::days(self.config.retention_days as i64);
        self.storage.cleanup_old_events(retention_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::storage::InMemoryAuditStorage;
    
    #[test]
    fn test_audit_logger() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let logger = AuditLogger::with_storage(storage.clone());
        
        let user_id = uuid::Uuid::new_v4();
        let session_id = uuid::Uuid::new_v4();
        
        // Log a user login
        logger.log_user_login(
            user_id,
            session_id,
            "127.0.0.1".to_string(),
            "TestAgent/1.0".to_string(),
            true,
        ).unwrap();
        
        // Query events
        let events = storage.get_events_by_user(&user_id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuditEventType::UserLogin);
    }
}