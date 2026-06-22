use super::{AlertCondition, EvaluationResult, ComparisonOperator, LogicalOperator, TimeWindow};
use super::storage::StorageError;
use std::collections::HashMap;

pub struct AlertEvaluator;

impl AlertEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        condition: &AlertCondition,
        metrics: &HashMap<String, f64>,
    ) -> Result<EvaluationResult, StorageError> {
        match condition {
            AlertCondition::Threshold { metric, operator, value, time_window } => {
                self.evaluate_threshold(metric, operator, *value, time_window, metrics)
            }
            AlertCondition::Composite { operator, conditions } => {
                self.evaluate_composite(operator, conditions, metrics)
            }
            AlertCondition::Anomaly { .. } => {
                // Anomaly detection is handled separately
                Ok(EvaluationResult {
                    triggered: false,
                    metric_value: 0.0,
                    threshold_value: 0.0,
                    message: "Anomaly detection not implemented in evaluator".to_string(),
                    context: HashMap::new(),
                })
            }
        }
    }

    fn evaluate_threshold(
        &self,
        metric: &str,
        operator: &ComparisonOperator,
        threshold: f64,
        time_window: &TimeWindow,
        metrics: &HashMap<String, f64>,
    ) -> Result<EvaluationResult, StorageError> {
        let metric_value = metrics.get(metric)
            .copied()
            .unwrap_or(0.0);

        let triggered = match operator {
            ComparisonOperator::GreaterThan => metric_value > threshold,
            ComparisonOperator::GreaterThanOrEqual => metric_value >= threshold,
            ComparisonOperator::LessThan => metric_value < threshold,
            ComparisonOperator::LessThanOrEqual => metric_value <= threshold,
            ComparisonOperator::Equal => (metric_value - threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (metric_value - threshold).abs() >= f64::EPSILON,
        };

        let message = if triggered {
            format!(
                "Metric '{}' {} threshold {} (value: {})",
                metric,
                self.operator_to_string(operator),
                threshold,
                metric_value
            )
        } else {
            format!(
                "Metric '{}' within normal range (value: {}, threshold: {})",
                metric,
                metric_value,
                threshold
            )
        };

        let mut context = HashMap::new();
        context.insert("metric".to_string(), metric.to_string());
        context.insert("operator".to_string(), self.operator_to_string(operator));
        context.insert("threshold".to_string(), threshold.to_string());
        context.insert("time_window_duration".to_string(), time_window.duration.to_string());
        context.insert("aggregation".to_string(), format!("{:?}", time_window.aggregation));

        Ok(EvaluationResult {
            triggered,
            metric_value,
            threshold_value: threshold,
            message,
            context,
        })
    }

    fn evaluate_composite(
        &self,
        operator: &LogicalOperator,
        conditions: &[AlertCondition],
        metrics: &HashMap<String, f64>,
    ) -> Result<EvaluationResult, StorageError> {
        if conditions.is_empty() {
            return Ok(EvaluationResult {
                triggered: false,
                metric_value: 0.0,
                threshold_value: 0.0,
                message: "Composite condition has no sub-conditions".to_string(),
                context: HashMap::new(),
            });
        }

        let results: Vec<EvaluationResult> = conditions
            .iter()
            .map(|cond| self.evaluate(cond, metrics))
            .collect::<Result<Vec<_>, _>>()?;

        let triggered = match operator {
            LogicalOperator::And => results.iter().all(|r| r.triggered),
            LogicalOperator::Or => results.iter().any(|r| r.triggered),
        };

        let triggered_conditions: Vec<_> = results.iter()
            .filter(|r| r.triggered)
            .map(|r| r.message.clone())
            .collect();

        let message = if triggered {
            format!(
                "Composite condition triggered: {}",
                triggered_conditions.join("; ")
            )
        } else {
            "Composite condition not triggered".to_string()
        };

        let mut context = HashMap::new();
        context.insert("operator".to_string(), format!("{:?}", operator));
        context.insert("condition_count".to_string(), conditions.len().to_string());
        context.insert("triggered_count".to_string(), triggered_conditions.len().to_string());

        Ok(EvaluationResult {
            triggered,
            metric_value: results.iter().map(|r| r.metric_value).sum::<f64>() / results.len() as f64,
            threshold_value: results.iter().map(|r| r.threshold_value).sum::<f64>() / results.len() as f64,
            message,
            context,
        })
    }

    fn operator_to_string(&self, operator: &ComparisonOperator) -> String {
        match operator {
            ComparisonOperator::GreaterThan => ">".to_string(),
            ComparisonOperator::GreaterThanOrEqual => ">=".to_string(),
            ComparisonOperator::LessThan => "<".to_string(),
            ComparisonOperator::LessThanOrEqual => "<=".to_string(),
            ComparisonOperator::Equal => "==".to_string(),
            ComparisonOperator::NotEqual => "!=".to_string(),
        }
    }
}

impl Default for AlertEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertCondition, ComparisonOperator, TimeWindow, AggregationType, LogicalOperator};

    #[test]
    fn test_evaluate_threshold_greater_than() {
        let evaluator = AlertEvaluator::new();
        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 150.0);

        let condition = AlertCondition::Threshold {
            metric: "cost".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: 100.0,
            time_window: TimeWindow {
                duration: 3600,
                aggregation: AggregationType::Sum,
            },
        };

        let result = evaluator.evaluate(&condition, &metrics).unwrap();
        assert!(result.triggered);
        assert_eq!(result.metric_value, 150.0);
        assert_eq!(result.threshold_value, 100.0);
    }

    #[test]
    fn test_evaluate_threshold_not_triggered() {
        let evaluator = AlertEvaluator::new();
        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 50.0);

        let condition = AlertCondition::Threshold {
            metric: "cost".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: 100.0,
            time_window: TimeWindow {
                duration: 3600,
                aggregation: AggregationType::Sum,
            },
        };

        let result = evaluator.evaluate(&condition, &metrics).unwrap();
        assert!(!result.triggered);
        assert_eq!(result.metric_value, 50.0);
    }

    #[test]
    fn test_evaluate_composite_and() {
        let evaluator = AlertEvaluator::new();
        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 150.0);
        metrics.insert("requests".to_string(), 2000.0);

        let condition = AlertCondition::Composite {
            operator: LogicalOperator::And,
            conditions: vec![
                AlertCondition::Threshold {
                    metric: "cost".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: 100.0,
                    time_window: TimeWindow {
                        duration: 3600,
                        aggregation: AggregationType::Sum,
                    },
                },
                AlertCondition::Threshold {
                    metric: "requests".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: 1000.0,
                    time_window: TimeWindow {
                        duration: 3600,
                        aggregation: AggregationType::Count,
                    },
                },
            ],
        };

        let result = evaluator.evaluate(&condition, &metrics).unwrap();
        assert!(result.triggered);
    }

    #[test]
    fn test_evaluate_composite_and_not_triggered() {
        let evaluator = AlertEvaluator::new();
        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 150.0);
        metrics.insert("requests".to_string(), 500.0);

        let condition = AlertCondition::Composite {
            operator: LogicalOperator::And,
            conditions: vec![
                AlertCondition::Threshold {
                    metric: "cost".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: 100.0,
                    time_window: TimeWindow {
                        duration: 3600,
                        aggregation: AggregationType::Sum,
                    },
                },
                AlertCondition::Threshold {
                    metric: "requests".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: 1000.0,
                    time_window: TimeWindow {
                        duration: 3600,
                        aggregation: AggregationType::Count,
                    },
                },
            ],
        };

        let result = evaluator.evaluate(&condition, &metrics).unwrap();
        assert!(!result.triggered);
    }

    #[test]
    fn test_evaluate_composite_or() {
        let evaluator = AlertEvaluator::new();
        let mut metrics = HashMap::new();
        metrics.insert("cost".to_string(), 150.0);
        metrics.insert("requests".to_string(), 500.0);

        let condition = AlertCondition::Composite {
            operator: LogicalOperator::Or,
            conditions: vec![
                AlertCondition::Threshold {
                    metric: "cost".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: 100.0,
                    time_window: TimeWindow {
                        duration: 3600,
                        aggregation: AggregationType::Sum,
                    },
                },
                AlertCondition::Threshold {
                    metric: "requests".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: 1000.0,
                    time_window: TimeWindow {
                        duration: 3600,
                        aggregation: AggregationType::Count,
                    },
                },
            ],
        };

        let result = evaluator.evaluate(&condition, &metrics).unwrap();
        assert!(result.triggered);
    }

    #[test]
    fn test_evaluate_missing_metric() {
        let evaluator = AlertEvaluator::new();
        let metrics = HashMap::new();

        let condition = AlertCondition::Threshold {
            metric: "cost".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: 100.0,
            time_window: TimeWindow {
                duration: 3600,
                aggregation: AggregationType::Sum,
            },
        };

        let result = evaluator.evaluate(&condition, &metrics).unwrap();
        assert!(!result.triggered);
        assert_eq!(result.metric_value, 0.0);
    }
}