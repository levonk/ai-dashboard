use super::{AlertRule, AlertEvent, AlertCondition};
use super::storage::{AlertStorage, StorageError};
use super::evaluator::AlertEvaluator;
use super::anomaly::AnomalyDetector;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{Utc, Duration};

pub struct AlertTrigger {
    storage: Arc<AlertStorage>,
    evaluator: AlertEvaluator,
    anomaly_detector: AnomalyDetector,
}

impl AlertTrigger {
    pub fn new(storage: Arc<AlertStorage>) -> Self {
        Self {
            storage,
            evaluator: AlertEvaluator::new(),
            anomaly_detector: AnomalyDetector::new(),
        }
    }

    pub fn trigger_alert(
        &self,
        rule: &AlertRule,
        metric_value: f64,
        threshold_value: f64,
        context: HashMap<String, String>,
    ) -> Result<AlertEvent, StorageError> {
        let event = AlertEvent::new(
            rule.id.clone(),
            rule.name.clone(),
            rule.severity.clone(),
            metric_value,
            threshold_value,
            format!("Alert triggered for rule: {}", rule.name),
            context,
        );

        self.storage.add_event(event.clone())?;
        Ok(event)
    }

    pub fn evaluate_and_trigger(
        &self,
        rule: &AlertRule,
        metrics: &HashMap<String, f64>,
        _historical_data: Option<&HashMap<String, Vec<f64>>>,
    ) -> Result<Option<AlertEvent>, StorageError> {
        if !rule.enabled {
            return Ok(None);
        }

        // Check cooldown period
        if self.is_in_cooldown(rule)? {
            return Ok(None);
        }

        let evaluation_result = self.evaluator.evaluate(&rule.conditions, metrics)?;

        if evaluation_result.triggered {
            let mut context = evaluation_result.context;
            context.insert("evaluation_message".to_string(), evaluation_result.message.clone());

            let event = self.trigger_alert(
                rule,
                evaluation_result.metric_value,
                evaluation_result.threshold_value,
                context,
            )?;

            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    pub fn evaluate_anomaly_and_trigger(
        &self,
        rule: &AlertRule,
        metric: &str,
        value: f64,
        historical_data: &[f64],
    ) -> Result<Option<AlertEvent>, StorageError> {
        if !rule.enabled {
            return Ok(None);
        }

        // Check cooldown period
        if self.is_in_cooldown(rule)? {
            return Ok(None);
        }

        if let AlertCondition::Anomaly { sensitivity, .. } = &rule.conditions {
            let is_anomaly = self.anomaly_detector.detect_anomaly(metric, value, sensitivity.clone(), historical_data)?;

            if is_anomaly {
                let z_score = self.anomaly_detector.get_z_score(value, historical_data);

                let mut context = HashMap::new();
                context.insert("metric".to_string(), metric.to_string());
                context.insert("sensitivity".to_string(), format!("{:?}", sensitivity));
                if let Some(zs) = z_score {
                    context.insert("z_score".to_string(), zs.to_string());
                }

                let event = self.trigger_alert(
                    rule,
                    value,
                    0.0, // No threshold for anomaly detection
                    context,
                )?;

                Ok(Some(event))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn is_in_cooldown(&self, rule: &AlertRule) -> Result<bool, StorageError> {
        if rule.cooldown_period <= 0 {
            return Ok(false);
        }

        let recent_events = self.storage.get_events_in_time_range(
            Utc::now() - Duration::seconds(rule.cooldown_period),
            Utc::now(),
        )?;

        let has_recent_alert = recent_events
            .iter()
            .any(|event| event.rule_id == rule.id && !event.resolved);

        Ok(has_recent_alert)
    }

    pub fn batch_evaluate_and_trigger(
        &self,
        rules: &[AlertRule],
        metrics: &HashMap<String, f64>,
        historical_data: Option<&HashMap<String, Vec<f64>>>,
    ) -> Result<Vec<AlertEvent>, StorageError> {
        let mut triggered_events = Vec::new();

        for rule in rules {
            if let Some(event) = self.evaluate_and_trigger(rule, metrics, historical_data)? {
                triggered_events.push(event);
            }
        }

        Ok(triggered_events)
    }
}

impl Default for AlertTrigger {
    fn default() -> Self {
        Self::new(Arc::new(AlertStorage::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertCondition, ComparisonOperator, TimeWindow, AggregationType, AlertSeverity};

    #[test]
    fn test_trigger_alert() {
        let storage = Arc::new(AlertStorage::new());
        let trigger = AlertTrigger::new(storage.clone());
        
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

        let event = trigger.trigger_alert(&rule, 150.0, 100.0, HashMap::new()).unwrap();
        
        assert_eq!(event.rule_id, "test-rule");
        assert_eq!(event.metric_value, 150.0);
        
        let retrieved = storage.get_event(&event.id).unwrap();
        assert_eq!(retrieved.id, event.id);
    }

    #[test]
    fn test_evaluate_and_trigger() {
        let storage = Arc::new(AlertStorage::new());
        let trigger = AlertTrigger::new(storage.clone());
        
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
            0, // No cooldown for testing
            "test-user".to_string(),
        );

        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 150.0);

        let result = trigger.evaluate_and_trigger(&rule, &metrics, None).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_evaluate_and_trigger_not_triggered() {
        let storage = Arc::new(AlertStorage::new());
        let trigger = AlertTrigger::new(storage.clone());
        
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
            0,
            "test-user".to_string(),
        );

        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 50.0);

        let result = trigger.evaluate_and_trigger(&rule, &metrics, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cooldown_period() {
        let storage = Arc::new(AlertStorage::new());
        let trigger = AlertTrigger::new(storage.clone());
        
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
            300, // 5 minute cooldown
            "test-user".to_string(),
        );

        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 150.0);

        // First trigger should succeed
        let result1 = trigger.evaluate_and_trigger(&rule, &metrics, None).unwrap();
        assert!(result1.is_some());

        // Second trigger should be blocked by cooldown
        let result2 = trigger.evaluate_and_trigger(&rule, &metrics, None).unwrap();
        assert!(result2.is_none());
    }

    #[test]
    fn test_disabled_rule() {
        let storage = Arc::new(AlertStorage::new());
        let trigger = AlertTrigger::new(storage.clone());
        
        let mut rule = AlertRule::new(
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
            0,
            "test-user".to_string(),
        );

        rule.enabled = false;

        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 150.0);

        let result = trigger.evaluate_and_trigger(&rule, &metrics, None).unwrap();
        assert!(result.is_none());
    }
}