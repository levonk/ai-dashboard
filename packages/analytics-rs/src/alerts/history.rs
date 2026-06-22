use super::AlertEvent;
use super::storage::{AlertStorage, StorageError};
use chrono::{DateTime, Utc};
use std::sync::Arc;

pub struct AlertHistory {
    storage: Arc<AlertStorage>,
}

impl AlertHistory {
    pub fn new(storage: Arc<AlertStorage>) -> Self {
        Self { storage }
    }

    pub fn track_event(&self, event: &AlertEvent) -> Result<(), StorageError> {
        self.storage.add_event(event.clone())
    }

    pub fn get_history(
        &self,
        rule_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AlertEvent>, StorageError> {
        let all_events = self.storage.list_events_by_rule(rule_id)?;
        
        Ok(all_events
            .into_iter()
            .filter(|event| event.triggered_at >= start && event.triggered_at <= end)
            .collect())
    }

    pub fn get_active_history(
        &self,
        rule_id: &str,
    ) -> Result<Vec<AlertEvent>, StorageError> {
        let all_events = self.storage.list_events_by_rule(rule_id)?;
        
        Ok(all_events
            .into_iter()
            .filter(|event| !event.resolved)
            .collect())
    }

    pub fn get_resolved_history(
        &self,
        rule_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AlertEvent>, StorageError> {
        let all_events = self.storage.list_events_by_rule(rule_id)?;
        
        Ok(all_events
            .into_iter()
            .filter(|event| event.resolved && event.triggered_at >= start && event.triggered_at <= end)
            .collect())
    }

    pub fn get_history_by_severity(
        &self,
        rule_id: &str,
        severity: &crate::alerts::AlertSeverity,
    ) -> Result<Vec<AlertEvent>, StorageError> {
        let all_events = self.storage.list_events_by_rule(rule_id)?;
        
        Ok(all_events
            .into_iter()
            .filter(|event| &event.severity == severity)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::AlertSeverity;
    use crate::AlertStorage;
    use std::collections::HashMap;

    #[test]
    fn test_track_and_retrieve_history() {
        let storage = Arc::new(AlertStorage::new());
        let history = AlertHistory::new(storage.clone());
        
        let event1 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let event2 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Error,
            200.0,
            100.0,
            "Test alert 2".to_string(),
            HashMap::new(),
        );

        history.track_event(&event1).unwrap();
        history.track_event(&event2).unwrap();
        
        let retrieved = history.get_history("rule-1", Utc::now() - chrono::Duration::hours(1), Utc::now() + chrono::Duration::hours(1)).unwrap();
        assert_eq!(retrieved.len(), 2);
    }

    #[test]
    fn test_get_active_history() {
        let storage = Arc::new(AlertStorage::new());
        let history = AlertHistory::new(storage.clone());
        
        let mut event1 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let event2 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Error,
            200.0,
            100.0,
            "Test alert 2".to_string(),
            HashMap::new(),
        );

        event1.resolve(Some("Fixed".to_string()));
        
        history.track_event(&event1).unwrap();
        history.track_event(&event2).unwrap();
        
        let active = history.get_active_history("rule-1").unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].severity, AlertSeverity::Error);
    }

    #[test]
    fn test_get_history_by_severity() {
        let storage = Arc::new(AlertStorage::new());
        let history = AlertHistory::new(storage.clone());
        
        let event1 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let event2 = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Error,
            200.0,
            100.0,
            "Test alert 2".to_string(),
            HashMap::new(),
        );

        history.track_event(&event1).unwrap();
        history.track_event(&event2).unwrap();
        
        let warnings = history.get_history_by_severity("rule-1", &AlertSeverity::Warning).unwrap();
        assert_eq!(warnings.len(), 1);
        
        let errors = history.get_history_by_severity("rule-1", &AlertSeverity::Error).unwrap();
        assert_eq!(errors.len(), 1);
    }
}