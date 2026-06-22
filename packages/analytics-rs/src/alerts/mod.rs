pub mod storage;
pub mod evaluator;
pub mod anomaly;
pub mod history;
pub mod trigger;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub conditions: AlertCondition,
    pub notification_channels: Vec<NotificationChannel>,
    pub severity: AlertSeverity,
    pub cooldown_period: i64, // seconds
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

/// Alert condition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Single threshold condition
    Threshold {
        metric: String,
        operator: ComparisonOperator,
        value: f64,
        time_window: TimeWindow,
    },
    /// Multiple conditions with AND/OR logic
    Composite {
        operator: LogicalOperator,
        conditions: Vec<AlertCondition>,
    },
    /// Anomaly detection condition
    Anomaly {
        metric: String,
        sensitivity: AnomalySensitivity,
        time_window: TimeWindow,
        baseline_period: i64, // seconds
    },
}

/// Comparison operators for threshold conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Logical operators for composite conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
}

/// Time window for alert evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub duration: i64, // seconds
    pub aggregation: AggregationType,
}

/// Aggregation type for time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Max,
    Min,
    Count,
    P95,
    P99,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Anomaly detection sensitivity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySensitivity {
    Low,
    Medium,
    High,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub channel_type: ChannelType,
    pub config: ChannelConfig,
    pub enabled: bool,
}

/// Channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Email,
    Slack,
    Webhook,
    Sms,
}

/// Channel-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelConfig {
    Email {
        recipients: Vec<String>,
        subject_template: String,
        body_template: String,
    },
    Slack {
        webhook_url: String,
        channel: String,
        username: String,
        icon_emoji: String,
    },
    Webhook {
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body_template: String,
    },
    Sms {
        phone_numbers: Vec<String>,
        message_template: String,
    },
}

/// Alert event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub triggered_at: DateTime<Utc>,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub context: HashMap<String, String>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
}

/// Alert evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub triggered: bool,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub context: HashMap<String, String>,
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub resolved_alerts: i64,
    pub alerts_by_severity: HashMap<AlertSeverity, i64>,
    pub alerts_by_rule: HashMap<String, i64>,
    pub average_resolution_time: Option<i64>, // seconds
}

/// Alert rule validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl AlertRule {
    /// Create a new alert rule
    pub fn new(
        id: String,
        name: String,
        description: String,
        conditions: AlertCondition,
        notification_channels: Vec<NotificationChannel>,
        severity: AlertSeverity,
        cooldown_period: i64,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            enabled: true,
            conditions,
            notification_channels,
            severity,
            cooldown_period,
            created_at: now,
            updated_at: now,
            created_by,
        }
    }

    /// Validate the alert rule
    pub fn validate(&self) -> RuleValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate cooldown period
        if self.cooldown_period < 0 {
            errors.push("Cooldown period cannot be negative".to_string());
        }

        // Validate notification channels
        if self.notification_channels.is_empty() {
            warnings.push("No notification channels configured".to_string());
        }

        // Validate conditions
        self.validate_conditions(&self.conditions, &mut errors, &mut warnings);

        RuleValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    fn validate_conditions(
        &self,
        condition: &AlertCondition,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        match condition {
            AlertCondition::Threshold { metric, value, time_window, .. } => {
                if metric.is_empty() {
                    errors.push("Threshold condition requires a metric".to_string());
                }
                if value.is_nan() || value.is_infinite() {
                    errors.push("Threshold value must be a valid number".to_string());
                }
                if time_window.duration <= 0 {
                    errors.push("Time window duration must be positive".to_string());
                }
            }
            AlertCondition::Composite { operator: _, conditions } => {
                if conditions.is_empty() {
                    errors.push("Composite condition must have at least one sub-condition".to_string());
                }
                for cond in conditions {
                    self.validate_conditions(cond, errors, warnings);
                }
                if conditions.len() == 1 {
                    warnings.push("Composite condition with single sub-condition - consider using direct condition".to_string());
                }
            }
            AlertCondition::Anomaly { metric, time_window, baseline_period, .. } => {
                if metric.is_empty() {
                    errors.push("Anomaly condition requires a metric".to_string());
                }
                if time_window.duration <= 0 {
                    errors.push("Time window duration must be positive".to_string());
                }
                if *baseline_period <= 0 {
                    errors.push("Baseline period must be positive".to_string());
                }
            }
        }
    }
}

impl NotificationChannel {
    /// Create a new notification channel
    pub fn new(id: String, channel_type: ChannelType, config: ChannelConfig) -> Self {
        Self {
            id,
            channel_type,
            config,
            enabled: true,
        }
    }

    /// Validate the notification channel
    pub fn validate(&self) -> Result<(), String> {
        match &self.config {
            ChannelConfig::Email { recipients, subject_template, body_template } => {
                if recipients.is_empty() {
                    return Err("Email channel requires at least one recipient".to_string());
                }
                if subject_template.is_empty() {
                    return Err("Email channel requires a subject template".to_string());
                }
                if body_template.is_empty() {
                    return Err("Email channel requires a body template".to_string());
                }
            }
            ChannelConfig::Slack { webhook_url, channel, .. } => {
                if webhook_url.is_empty() {
                    return Err("Slack channel requires a webhook URL".to_string());
                }
                if channel.is_empty() {
                    return Err("Slack channel requires a channel name".to_string());
                }
            }
            ChannelConfig::Webhook { url, method, .. } => {
                if url.is_empty() {
                    return Err("Webhook channel requires a URL".to_string());
                }
                if method.is_empty() {
                    return Err("Webhook channel requires an HTTP method".to_string());
                }
            }
            ChannelConfig::Sms { phone_numbers, message_template } => {
                if phone_numbers.is_empty() {
                    return Err("SMS channel requires at least one phone number".to_string());
                }
                if message_template.is_empty() {
                    return Err("SMS channel requires a message template".to_string());
                }
            }
        }
        Ok(())
    }
}

impl AlertEvent {
    /// Create a new alert event
    pub fn new(
        rule_id: String,
        rule_name: String,
        severity: AlertSeverity,
        metric_value: f64,
        threshold_value: f64,
        message: String,
        context: HashMap<String, String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id,
            rule_name,
            severity,
            triggered_at: Utc::now(),
            metric_value,
            threshold_value,
            message,
            context,
            resolved: false,
            resolved_at: None,
            resolution_notes: None,
        }
    }

    /// Resolve the alert
    pub fn resolve(&mut self, notes: Option<String>) {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
        self.resolution_notes = notes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_validation() {
        let rule = AlertRule::new(
            "test-rule".to_string(),
            "Test Rule".to_string(),
            "A test rule".to_string(),
            AlertCondition::Threshold {
                metric: "cost".to_string(),
                operator: ComparisonOperator::GreaterThan,
                value: 100.0,
                time_window: TimeWindow {
                    duration: 3600,
                    aggregation: AggregationType::Sum,
                },
            },
            vec![],
            AlertSeverity::Warning,
            300,
            "test-user".to_string(),
        );

        let result = rule.validate();
        assert!(result.valid);
        assert!(result.warnings.contains(&"No notification channels configured".to_string()));
    }

    #[test]
    fn test_alert_rule_invalid_threshold() {
        let rule = AlertRule::new(
            "test-rule".to_string(),
            "Test Rule".to_string(),
            "A test rule".to_string(),
            AlertCondition::Threshold {
                metric: "".to_string(),
                operator: ComparisonOperator::GreaterThan,
                value: f64::NAN,
                time_window: TimeWindow {
                    duration: -1,
                    aggregation: AggregationType::Sum,
                },
            },
            vec![],
            AlertSeverity::Warning,
            300,
            "test-user".to_string(),
        );

        let result = rule.validate();
        assert!(!result.valid);
        assert!(result.errors.len() >= 2);
    }

    #[test]
    fn test_notification_channel_validation() {
        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Email,
            ChannelConfig::Email {
                recipients: vec!["test@example.com".to_string()],
                subject_template: "Alert: {{rule_name}}".to_string(),
                body_template: "Alert triggered".to_string(),
            },
        );

        assert!(channel.validate().is_ok());
    }

    #[test]
    fn test_notification_channel_invalid() {
        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Email,
            ChannelConfig::Email {
                recipients: vec![],
                subject_template: "".to_string(),
                body_template: "".to_string(),
            },
        );

        assert!(channel.validate().is_err());
    }

    #[test]
    fn test_alert_event_creation() {
        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Critical,
            150.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        assert!(!event.resolved);
        assert_eq!(event.metric_value, 150.0);
        assert_eq!(event.threshold_value, 100.0);
    }

    #[test]
    fn test_alert_event_resolution() {
        let mut event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Critical,
            150.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        event.resolve(Some("Issue fixed".to_string()));
        assert!(event.resolved);
        assert!(event.resolved_at.is_some());
        assert_eq!(event.resolution_notes, Some("Issue fixed".to_string()));
    }
}