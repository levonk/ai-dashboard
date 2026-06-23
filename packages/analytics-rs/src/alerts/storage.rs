use super::{AlertRule, AlertEvent, AlertStatistics};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    
    #[error("Event not found: {0}")]
    EventNotFound(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// In-memory storage for alert rules and events
/// In production, this would be replaced with a database implementation
#[derive(Clone)]
pub struct AlertStorage {
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    events: Arc<RwLock<HashMap<String, AlertEvent>>>,
    events_by_rule: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl AlertStorage {
    /// Create a new alert storage instance
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
            events_by_rule: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new alert rule
    pub fn add_rule(&self, rule: AlertRule) -> Result<(), StorageError> {
        let validation = rule.validate();
        if !validation.valid {
            return Err(StorageError::ValidationError(
                validation.errors.join(", ")
            ));
        }

        let mut rules = self.rules.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if rules.contains_key(&rule.id) {
            return Err(StorageError::StorageError(
                format!("Rule with id {} already exists", rule.id)
            ));
        }

        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Update an existing alert rule
    pub fn update_rule(&self, rule: AlertRule) -> Result<(), StorageError> {
        let validation = rule.validate();
        if !validation.valid {
            return Err(StorageError::ValidationError(
                validation.errors.join(", ")
            ));
        }

        let mut rules = self.rules.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if !rules.contains_key(&rule.id) {
            return Err(StorageError::RuleNotFound(rule.id.clone()));
        }

        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Delete an alert rule
    pub fn delete_rule(&self, rule_id: &str) -> Result<(), StorageError> {
        let mut rules = self.rules.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if !rules.remove(rule_id).is_some() {
            return Err(StorageError::RuleNotFound(rule_id.to_string()));
        }

        Ok(())
    }

    /// Get an alert rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Result<AlertRule, StorageError> {
        let rules = self.rules.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        rules.get(rule_id)
            .cloned()
            .ok_or_else(|| StorageError::RuleNotFound(rule_id.to_string()))
    }

    /// List all alert rules
    pub fn list_rules(&self) -> Result<Vec<AlertRule>, StorageError> {
        let rules = self.rules.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        Ok(rules.values().cloned().collect())
    }

    /// List enabled alert rules
    pub fn list_enabled_rules(&self) -> Result<Vec<AlertRule>, StorageError> {
        let rules = self.rules.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        Ok(rules.values()
            .filter(|rule| rule.enabled)
            .cloned()
            .collect())
    }

    /// Add an alert event
    pub fn add_event(&self, event: AlertEvent) -> Result<(), StorageError> {
        let mut events = self.events.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        let mut events_by_rule = self.events_by_rule.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;

        events.insert(event.id.clone(), event.clone());
        
        events_by_rule
            .entry(event.rule_id.clone())
            .or_insert_with(Vec::new)
            .push(event.id.clone());

        Ok(())
    }

    /// Get an alert event by ID
    pub fn get_event(&self, event_id: &str) -> Result<AlertEvent, StorageError> {
        let events = self.events.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        events.get(event_id)
            .cloned()
            .ok_or_else(|| StorageError::EventNotFound(event_id.to_string()))
    }

    /// List events for a specific rule
    pub fn list_events_by_rule(&self, rule_id: &str) -> Result<Vec<AlertEvent>, StorageError> {
        let events = self.events.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        let events_by_rule = self.events_by_rule.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;

        if let Some(event_ids) = events_by_rule.get(rule_id) {
            let mut result = Vec::new();
            for event_id in event_ids {
                if let Some(event) = events.get(event_id) {
                    result.push(event.clone());
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// List all events
    pub fn list_all_events(&self) -> Result<Vec<AlertEvent>, StorageError> {
        let events = self.events.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        Ok(events.values().cloned().collect())
    }

    /// Update an alert event (e.g., mark as resolved)
    pub fn update_event(&self, event: AlertEvent) -> Result<(), StorageError> {
        let mut events = self.events.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if !events.contains_key(&event.id) {
            return Err(StorageError::EventNotFound(event.id.clone()));
        }

        events.insert(event.id.clone(), event);
        Ok(())
    }

    /// Get alert statistics
    pub fn get_statistics(&self) -> Result<AlertStatistics, StorageError> {
        let events = self.events.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;

        let _rules = self.rules.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;

        let total_alerts = events.len() as i64;
        let active_alerts = events.values()
            .filter(|e| !e.resolved)
            .count() as i64;
        let resolved_alerts = events.values()
            .filter(|e| e.resolved)
            .count() as i64;

        let mut alerts_by_severity = HashMap::new();
        let mut alerts_by_rule = HashMap::new();

        for event in events.values() {
            *alerts_by_severity.entry(event.severity.clone()).or_insert(0) += 1;
            *alerts_by_rule.entry(event.rule_id.clone()).or_insert(0) += 1;
        }

        // Calculate average resolution time
        let resolved_events: Vec<_> = events.values()
            .filter(|e| e.resolved && e.resolved_at.is_some())
            .collect();

        let average_resolution_time = if !resolved_events.is_empty() {
            let total: i64 = resolved_events.iter()
                .filter_map(|e| {
                    e.resolved_at.map(|resolved| {
                        (resolved - e.triggered_at).num_seconds()
                    })
                })
                .sum();
            Some(total / resolved_events.len() as i64)
        } else {
            None
        };

        Ok(AlertStatistics {
            total_alerts,
            active_alerts,
            resolved_alerts,
            alerts_by_severity,
            alerts_by_rule,
            average_resolution_time,
        })
    }

    /// Get events within a time range
    pub fn get_events_in_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AlertEvent>, StorageError> {
        let events = self.events.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        Ok(events.values()
            .filter(|e| e.triggered_at >= start && e.triggered_at <= end)
            .cloned()
            .collect())
    }

    /// Get active (unresolved) events
    pub fn get_active_events(&self) -> Result<Vec<AlertEvent>, StorageError> {
        let events = self.events.read()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        Ok(events.values()
            .filter(|e| !e.resolved)
            .cloned()
            .collect())
    }

    /// Resolve an event by ID
    pub fn resolve_event(&self, event_id: &str, notes: Option<String>) -> Result<(), StorageError> {
        let mut events = self.events.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if let Some(event) = events.get_mut(event_id) {
            event.resolve(notes);
            Ok(())
        } else {
            Err(StorageError::EventNotFound(event_id.to_string()))
        }
    }

    /// Clear all data (useful for testing)
    pub fn clear(&self) -> Result<(), StorageError> {
        let mut rules = self.rules.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        let mut events = self.events.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;
        let mut events_by_rule = self.events_by_rule.write()
            .map_err(|e| StorageError::StorageError(e.to_string()))?;

        rules.clear();
        events.clear();
        events_by_rule.clear();

        Ok(())
    }
}

impl Default for AlertStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertCondition, ComparisonOperator, TimeWindow, AggregationType, AlertSeverity};

    #[test]
    fn test_add_and_get_rule() {
        let storage = AlertStorage::new();
        
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

        storage.add_rule(rule.clone()).unwrap();
        let retrieved = storage.get_rule("test-rule").unwrap();
        
        assert_eq!(retrieved.id, rule.id);
        assert_eq!(retrieved.name, rule.name);
    }

    #[test]
    fn test_add_duplicate_rule() {
        let storage = AlertStorage::new();
        
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

        storage.add_rule(rule.clone()).unwrap();
        let result = storage.add_rule(rule);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_list_rules() {
        let storage = AlertStorage::new();
        
        let rule1 = AlertRule::new(
            "rule-1".to_string(),
            "Rule 1".to_string(),
            "First rule".to_string(),
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

        let rule2 = AlertRule::new(
            "rule-2".to_string(),
            "Rule 2".to_string(),
            "Second rule".to_string(),
            AlertCondition::Threshold {
                metric: "requests".to_string(),
                operator: ComparisonOperator::GreaterThan,
                value: 1000.0,
                time_window: TimeWindow {
                    duration: 3600,
                    aggregation: AggregationType::Count,
                },
            },
            vec![],
            AlertSeverity::Error,
            600,
            "test-user".to_string(),
        );

        storage.add_rule(rule1).unwrap();
        storage.add_rule(rule2).unwrap();
        
        let rules = storage.list_rules().unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_add_and_get_event() {
        let storage = AlertStorage::new();
        
        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Critical,
            150.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        storage.add_event(event.clone()).unwrap();
        let retrieved = storage.get_event(&event.id).unwrap();
        
        assert_eq!(retrieved.id, event.id);
        assert_eq!(retrieved.rule_id, event.rule_id);
    }

    #[test]
    fn test_list_events_by_rule() {
        let storage = AlertStorage::new();
        
        let event1 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Critical,
            150.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        let event2 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            120.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        let event3 = AlertEvent::new(
            "rule-2".to_string(),
            "Another Rule".to_string(),
            AlertSeverity::Error,
            2000.0,
            1000.0,
            "Requests exceeded threshold".to_string(),
            HashMap::new(),
        );

        storage.add_event(event1).unwrap();
        storage.add_event(event2).unwrap();
        storage.add_event(event3).unwrap();
        
        let rule1_events = storage.list_events_by_rule("rule-1").unwrap();
        assert_eq!(rule1_events.len(), 2);
        
        let rule2_events = storage.list_events_by_rule("rule-2").unwrap();
        assert_eq!(rule2_events.len(), 1);
    }

    #[test]
    fn test_resolve_event() {
        let storage = AlertStorage::new();
        
        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Critical,
            150.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        let event_id = event.id.clone();
        storage.add_event(event).unwrap();
        storage.resolve_event(&event_id, Some("Fixed".to_string())).unwrap();
        
        let retrieved = storage.get_event(&event_id).unwrap();
        assert!(retrieved.resolved);
        assert_eq!(retrieved.resolution_notes, Some("Fixed".to_string()));
    }

    #[test]
    fn test_get_statistics() {
        let storage = AlertStorage::new();
        
        let event1 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Critical,
            150.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        let event2 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            120.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        storage.add_event(event1).unwrap();
        storage.add_event(event2).unwrap();
        
        let stats = storage.get_statistics().unwrap();
        assert_eq!(stats.total_alerts, 2);
        assert_eq!(stats.active_alerts, 2);
        assert_eq!(stats.resolved_alerts, 0);
    }

    #[test]
    fn test_get_active_events() {
        let storage = AlertStorage::new();
        
        let mut event1 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Critical,
            150.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        let event2 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            120.0,
            100.0,
            "Cost exceeded threshold".to_string(),
            HashMap::new(),
        );

        event1.resolve(Some("Fixed".to_string()));
        
        storage.add_event(event1).unwrap();
        storage.add_event(event2).unwrap();
        
        let active = storage.get_active_events().unwrap();
        assert_eq!(active.len(), 1);
    }
}