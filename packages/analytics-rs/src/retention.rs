use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use anyhow::Result;
use std::collections::HashMap;

/// Data retention policy for different data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub data_type: DataType,
    pub retention_duration: Duration,
    pub retention_action: RetentionAction,
    pub enabled: bool,
}

/// Types of data that can have retention policies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataType {
    AuditLogs,
    UserSessions,
    TelemetryEvents,
    AnalyticsData,
    ErrorLogs,
    SystemMetrics,
    Custom(String),
}

/// Actions to take when retention period expires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionAction {
    Delete,
    Archive,
    Anonymize,
    Compress,
}

/// Data retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub policies: HashMap<String, RetentionPolicy>,
    pub default_retention_days: u32,
    pub enforcement_enabled: bool,
    pub cleanup_schedule: CleanupSchedule,
}

/// Schedule for automatic cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupSchedule {
    pub enabled: bool,
    pub cron_expression: String,
    pub timezone: String,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        let mut policies = HashMap::new();
        
        // Default policies
        policies.insert(
            "audit_logs".to_string(),
            RetentionPolicy {
                data_type: DataType::AuditLogs,
                retention_duration: Duration::days(90),
                retention_action: RetentionAction::Archive,
                enabled: true,
            }
        );
        
        policies.insert(
            "user_sessions".to_string(),
            RetentionPolicy {
                data_type: DataType::UserSessions,
                retention_duration: Duration::days(30),
                retention_action: RetentionAction::Delete,
                enabled: true,
            }
        );
        
        policies.insert(
            "telemetry_events".to_string(),
            RetentionPolicy {
                data_type: DataType::TelemetryEvents,
                retention_duration: Duration::days(365),
                retention_action: RetentionAction::Compress,
                enabled: true,
            }
        );
        
        policies.insert(
            "error_logs".to_string(),
            RetentionPolicy {
                data_type: DataType::ErrorLogs,
                retention_duration: Duration::days(180),
                retention_action: RetentionAction::Archive,
                enabled: true,
            }
        );
        
        Self {
            policies,
            default_retention_days: 90,
            enforcement_enabled: true,
            cleanup_schedule: CleanupSchedule {
                enabled: true,
                cron_expression: "0 2 * * *".to_string(), // Daily at 2 AM
                timezone: "UTC".to_string(),
            },
        }
    }
}

/// Data retention manager
pub struct RetentionManager {
    config: RetentionConfig,
}

impl RetentionManager {
    /// Create a new retention manager
    pub fn new(config: RetentionConfig) -> Self {
        Self { config }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(RetentionConfig::default())
    }
    
    /// Add or update a retention policy
    pub fn set_policy(&mut self, name: String, policy: RetentionPolicy) {
        self.config.policies.insert(name, policy);
    }
    
    /// Get a retention policy by name
    pub fn get_policy(&self, name: &str) -> Option<&RetentionPolicy> {
        self.config.policies.get(name)
    }
    
    /// Remove a retention policy
    pub fn remove_policy(&mut self, name: &str) -> Option<RetentionPolicy> {
        self.config.policies.remove(name)
    }
    
    /// Check if data should be retained based on policy
    pub fn should_retain(&self, data_type: &DataType, created_at: DateTime<Utc>) -> Result<bool> {
        if !self.config.enforcement_enabled {
            return Ok(true);
        }
        
        // Find applicable policy
        let policy = self.config.policies.values()
            .find(|p| &p.data_type == data_type && p.enabled);
        
        if let Some(policy) = policy {
            let cutoff = Utc::now() - policy.retention_duration;
            Ok(created_at > cutoff)
        } else {
            // Use default retention
            let cutoff = Utc::now() - Duration::days(self.config.default_retention_days as i64);
            Ok(created_at > cutoff)
        }
    }
    
    /// Get retention deadline for data
    pub fn get_retention_deadline(&self, data_type: &DataType, created_at: DateTime<Utc>) -> Result<DateTime<Utc>> {
        let policy = self.config.policies.values()
            .find(|p| &p.data_type == data_type && p.enabled);
        
        if let Some(policy) = policy {
            Ok(created_at + policy.retention_duration)
        } else {
            Ok(created_at + Duration::days(self.config.default_retention_days as i64))
        }
    }
    
    /// Get all data that should be cleaned up
    pub fn get_data_for_cleanup(&self, data_type: &DataType, data: &[(DateTime<Utc>, String)]) -> Result<Vec<String>> {
        let cutoff = Utc::now() - self.get_retention_duration(data_type)?;
        
        Ok(data.iter()
            .filter(|(created_at, _)| *created_at < cutoff)
            .map(|(_, id)| id.clone())
            .collect())
    }
    
    /// Get retention duration for a data type
    fn get_retention_duration(&self, data_type: &DataType) -> Result<Duration> {
        let policy = self.config.policies.values()
            .find(|p| &p.data_type == data_type && p.enabled);
        
        if let Some(policy) = policy {
            Ok(policy.retention_duration)
        } else {
            Ok(Duration::days(self.config.default_retention_days as i64))
        }
    }
    
    /// Enable or disable enforcement
    pub fn set_enforcement(&mut self, enabled: bool) {
        self.config.enforcement_enabled = enabled;
    }
    
    /// Get the configuration
    pub fn config(&self) -> &RetentionConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: RetentionConfig) {
        self.config = config;
    }
    
    /// Validate retention policies
    pub fn validate_policies(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        
        for (name, policy) in &self.config.policies {
            // Check retention duration is reasonable
            if policy.retention_duration.num_days() < 1 {
                errors.push(format!("Policy '{}': Retention duration too short (must be at least 1 day)", name));
            }
            
            if policy.retention_duration.num_days() > 3650 {
                errors.push(format!("Policy '{}': Retention duration too long (maximum 10 years)", name));
            }
            
            // Check if policy is enabled but enforcement is disabled
            if policy.enabled && !self.config.enforcement_enabled {
                errors.push(format!("Policy '{}': Policy enabled but enforcement is disabled", name));
            }
        }
        
        Ok(errors)
    }
}

impl Default for RetentionManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retention_manager() {
        let manager = RetentionManager::with_defaults();
        
        let now = Utc::now();
        let old_data = now - Duration::days(100);
        let new_data = now - Duration::days(10);
        
        // Test audit logs retention (90 days default)
        assert!(manager.should_retain(&DataType::AuditLogs, new_data).unwrap());
        assert!(!manager.should_retain(&DataType::AuditLogs, old_data).unwrap());
    }
    
    #[test]
    fn test_custom_policy() {
        let mut manager = RetentionManager::with_defaults();
        
        let custom_policy = RetentionPolicy {
            data_type: DataType::Custom("test_data".to_string()),
            retention_duration: Duration::days(30),
            retention_action: RetentionAction::Delete,
            enabled: true,
        };
        
        manager.set_policy("test".to_string(), custom_policy);
        
        let policy = manager.get_policy("test");
        assert!(policy.is_some());
        assert_eq!(policy.unwrap().retention_duration.num_days(), 30);
    }
    
    #[test]
    fn test_validation() {
        let manager = RetentionManager::with_defaults();
        let errors = manager.validate_policies().unwrap();
        assert!(errors.is_empty());
    }
}