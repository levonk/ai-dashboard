use crate::models::{TelemetryEvent, Filter, FilterOperator};
use anyhow::Result;

pub struct FilterEngine;

impl FilterEngine {
    pub fn apply(events: &[TelemetryEvent], filters: &[Filter]) -> Result<Vec<TelemetryEvent>> {
        let mut filtered = events.to_vec();
        
        for filter in filters {
            filtered = Self::apply_single_filter(&filtered, filter)?;
        }
        
        Ok(filtered)
    }

    fn apply_single_filter(events: &[TelemetryEvent], filter: &Filter) -> Result<Vec<TelemetryEvent>> {
        let field_value = Self::get_field_value_as_string(&filter.field);
        
        let filtered: Vec<TelemetryEvent> = events
            .iter()
            .filter(|event| Self::matches_filter(event, filter, &field_value))
            .cloned()
            .collect();
        
        Ok(filtered)
    }
    
    fn matches_filter(event: &TelemetryEvent, filter: &Filter, field_value: &str) -> bool {
        let event_value = Self::get_event_field_value(event, field_value);
        
        match &filter.operator {
            FilterOperator::Equals => {
                Self::values_equal(&event_value, &filter.value)
            }
            FilterOperator::NotEquals => {
                !Self::values_equal(&event_value, &filter.value)
            }
            FilterOperator::Contains => {
                Self::value_contains(&event_value, &filter.value)
            }
            FilterOperator::NotContains => {
                !Self::value_contains(&event_value, &filter.value)
            }
            FilterOperator::GreaterThan => {
                Self::value_compare(&event_value, &filter.value, |a, b| a > b)
            }
            FilterOperator::LessThan => {
                Self::value_compare(&event_value, &filter.value, |a, b| a < b)
            }
            FilterOperator::GreaterThanOrEqual => {
                Self::value_compare(&event_value, &filter.value, |a, b| a >= b)
            }
            FilterOperator::LessThanOrEqual => {
                Self::value_compare(&event_value, &filter.value, |a, b| a <= b)
            }
            FilterOperator::In => {
                Self::value_in(&event_value, &filter.value)
            }
            FilterOperator::NotIn => {
                !Self::value_in(&event_value, &filter.value)
            }
        }
    }
    
    fn get_event_field_value(event: &TelemetryEvent, field: &str) -> serde_json::Value {
        match field {
            "event_id" => serde_json::json!(event.event_id),
            "timestamp" => serde_json::json!(event.timestamp.to_rfc3339()),
            "ai_client" => serde_json::json!(event.ai_client),
            "ai_provider" => serde_json::json!(event.ai_provider),
            "model" => serde_json::json!(event.model),
            "input_type" => serde_json::json!(event.input_type),
            "input_tokens" => serde_json::json!(event.input_tokens),
            "output_tokens" => serde_json::json!(event.output_tokens),
            "duration_ms" => serde_json::json!(event.duration_ms),
            "cost_usd" => serde_json::json!(event.cost_usd),
            _ => serde_json::json!(null),
        }
    }
    
    fn get_field_value_as_string(field: &str) -> String {
        field.to_string()
    }
    
    fn values_equal(event_value: &serde_json::Value, filter_value: &serde_json::Value) -> bool {
        event_value == filter_value
    }
    
    fn value_contains(event_value: &serde_json::Value, filter_value: &serde_json::Value) -> bool {
        let event_str = event_value.as_str().unwrap_or("");
        let filter_str = filter_value.as_str().unwrap_or("");
        event_str.to_lowercase().contains(&filter_str.to_lowercase())
    }
    
    fn value_compare<F>(event_value: &serde_json::Value, filter_value: &serde_json::Value, compare: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let event_num = match event_value {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => return false,
        };
        
        let filter_num = match filter_value {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => return false,
        };
        
        compare(event_num, filter_num)
    }
    
    fn value_in(event_value: &serde_json::Value, filter_value: &serde_json::Value) -> bool {
        if let Some(filter_array) = filter_value.as_array() {
            filter_array.contains(event_value)
        } else {
            false
        }
    }
    
    pub fn apply_time_range_filter(events: &[TelemetryEvent], start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Result<Vec<TelemetryEvent>> {
        let filtered: Vec<TelemetryEvent> = events
            .iter()
            .filter(|event| event.timestamp >= start && event.timestamp <= end)
            .cloned()
            .collect();
        
        Ok(filtered)
    }
    
    pub fn apply_client_filter(events: &[TelemetryEvent], clients: &[String]) -> Result<Vec<TelemetryEvent>> {
        let filtered: Vec<TelemetryEvent> = events
            .iter()
            .filter(|event| clients.contains(&event.ai_client))
            .cloned()
            .collect();
        
        Ok(filtered)
    }
    
    pub fn apply_provider_filter(events: &[TelemetryEvent], providers: &[String]) -> Result<Vec<TelemetryEvent>> {
        let filtered: Vec<TelemetryEvent> = events
            .iter()
            .filter(|event| providers.contains(&event.ai_provider))
            .cloned()
            .collect();
        
        Ok(filtered)
    }
    
    pub fn apply_model_filter(events: &[TelemetryEvent], models: &[String]) -> Result<Vec<TelemetryEvent>> {
        let filtered: Vec<TelemetryEvent> = events
            .iter()
            .filter(|event| models.contains(&event.model))
            .cloned()
            .collect();
        
        Ok(filtered)
    }
}