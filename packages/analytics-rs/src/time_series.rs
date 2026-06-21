use crate::models::{TelemetryEvent, TimeRange};
use anyhow::Result;
use std::collections::HashMap;

pub struct TimeSeriesAnalyzer;

impl TimeSeriesAnalyzer {
    pub fn analyze_time_series(events: &[TelemetryEvent], time_range: &TimeRange) -> Result<serde_json::Value> {
        let filtered_events: Vec<TelemetryEvent> = events
            .iter()
            .filter(|e| e.timestamp >= time_range.start && e.timestamp <= time_range.end)
            .cloned()
            .collect();
        
        if filtered_events.is_empty() {
            return Ok(serde_json::json!({
                "time_series": [],
                "summary": {
                    "total_events": 0,
                    "time_range": {
                        "start": time_range.start.to_rfc3339(),
                        "end": time_range.end.to_rfc3339()
                    }
                }
            }));
        }
        
        // Group events by hour for time series
        let mut time_series: HashMap<String, serde_json::Value> = HashMap::new();
        let mut unique_models_per_hour: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
        
        for event in &filtered_events {
            let hour_key = event.timestamp.format("%Y-%m-%d %H:00").to_string();
            
            unique_models_per_hour.entry(hour_key.clone())
                .or_insert_with(std::collections::HashSet::new)
                .insert(event.model.clone());
            
            let entry = time_series.entry(hour_key.clone()).or_insert_with(|| {
                serde_json::json!({
                    "timestamp": hour_key,
                    "count": 0,
                    "total_cost_usd": 0.0,
                    "total_input_tokens": 0,
                    "total_output_tokens": 0,
                    "avg_duration_ms": 0.0,
                    "unique_models_count": 0
                })
            });
            
            entry["count"] = serde_json::json!(entry["count"].as_u64().unwrap_or(0) + 1);
            entry["total_cost_usd"] = serde_json::json!(entry["total_cost_usd"].as_f64().unwrap_or(0.0) + event.cost_usd);
            entry["total_input_tokens"] = serde_json::json!(entry["total_input_tokens"].as_u64().unwrap_or(0) + event.input_tokens as u64);
            entry["total_output_tokens"] = serde_json::json!(entry["total_output_tokens"].as_u64().unwrap_or(0) + event.output_tokens as u64);
            
            // Calculate average duration
            let current_avg = entry["avg_duration_ms"].as_f64().unwrap_or(0.0);
            let count = entry["count"].as_u64().unwrap_or(1) as f64;
            let new_avg = (current_avg * (count - 1.0) + event.duration_ms as f64) / count;
            entry["avg_duration_ms"] = serde_json::json!(new_avg);
        }
        
        // Update unique models count
        for (hour_key, unique_models) in &unique_models_per_hour {
            if let Some(entry) = time_series.get_mut(hour_key) {
                entry["unique_models_count"] = serde_json::json!(unique_models.len());
            }
        }
        
        // Convert to sorted array
        let mut series: Vec<serde_json::Value> = time_series.values().cloned().collect();
        series.sort_by(|a, b| {
            let a_time = a["timestamp"].as_str().unwrap_or("");
            let b_time = b["timestamp"].as_str().unwrap_or("");
            a_time.cmp(b_time)
        });
        
        Ok(serde_json::json!({
            "time_series": series,
            "summary": {
                "total_events": filtered_events.len(),
                "time_range": {
                    "start": time_range.start.to_rfc3339(),
                    "end": time_range.end.to_rfc3339()
                },
                "total_cost_usd": filtered_events.iter().map(|e| e.cost_usd).sum::<f64>(),
                "total_input_tokens": filtered_events.iter().map(|e| e.input_tokens).sum::<u32>(),
                "total_output_tokens": filtered_events.iter().map(|e| e.output_tokens).sum::<u32>()
            }
        }))
    }

    pub fn calculate_trends(events: &[TelemetryEvent]) -> Result<serde_json::Value> {
        if events.len() < 2 {
            return Ok(serde_json::json!({
                "trend": "insufficient_data",
                "message": "Need at least 2 events to calculate trends"
            }));
        }
        
        // Sort events by timestamp
        let mut sorted_events = events.to_vec();
        sorted_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Calculate trends for different metrics
        let costs: Vec<f64> = sorted_events.iter().map(|e| e.cost_usd).collect();
        let input_tokens: Vec<u32> = sorted_events.iter().map(|e| e.input_tokens).collect();
        let output_tokens: Vec<u32> = sorted_events.iter().map(|e| e.output_tokens).collect();
        let durations: Vec<u64> = sorted_events.iter().map(|e| e.duration_ms).collect();
        
        let cost_trend = Self::calculate_linear_trend(&costs);
        let input_trend = Self::calculate_linear_trend_u32(&input_tokens);
        let output_trend = Self::calculate_linear_trend_u32(&output_tokens);
        let duration_trend = Self::calculate_linear_trend_u64(&durations);
        
        Ok(serde_json::json!({
            "cost_trend": {
                "direction": cost_trend.direction,
                "slope": cost_trend.slope,
                "change_percent": cost_trend.change_percent
            },
            "input_tokens_trend": {
                "direction": input_trend.direction,
                "slope": input_trend.slope,
                "change_percent": input_trend.change_percent
            },
            "output_tokens_trend": {
                "direction": output_trend.direction,
                "slope": output_trend.slope,
                "change_percent": output_trend.change_percent
            },
            "duration_trend": {
                "direction": duration_trend.direction,
                "slope": duration_trend.slope,
                "change_percent": duration_trend.change_percent
            }
        }))
    }
    
    fn calculate_linear_trend(values: &[f64]) -> TrendResult {
        if values.is_empty() {
            return TrendResult {
                direction: "none".to_string(),
                slope: 0.0,
                change_percent: 0.0,
            };
        }
        
        let n = values.len() as f64;
        let first = values.first().unwrap_or(&0.0);
        let last = values.last().unwrap_or(&0.0);
        
        if *first == 0.0 {
            return TrendResult {
                direction: "none".to_string(),
                slope: 0.0,
                change_percent: 0.0,
            };
        }
        
        let slope = (last - first) / (n - 1.0).max(1.0);
        let change_percent = ((last - first) / first.abs()) * 100.0;
        
        let direction = if change_percent > 5.0 {
            "increasing"
        } else if change_percent < -5.0 {
            "decreasing"
        } else {
            "stable"
        };
        
        TrendResult {
            direction: direction.to_string(),
            slope,
            change_percent,
        }
    }
    
    fn calculate_linear_trend_u32(values: &[u32]) -> TrendResult {
        let float_values: Vec<f64> = values.iter().map(|&v| v as f64).collect();
        Self::calculate_linear_trend(&float_values)
    }
    
    fn calculate_linear_trend_u64(values: &[u64]) -> TrendResult {
        let float_values: Vec<f64> = values.iter().map(|&v| v as f64).collect();
        Self::calculate_linear_trend(&float_values)
    }

    pub fn detect_anomalies(events: &[TelemetryEvent]) -> Result<Vec<String>> {
        if events.len() < 10 {
            return Ok(vec!["Insufficient data for anomaly detection".to_string()]);
        }
        
        let mut anomalies = Vec::new();
        
        // Detect cost anomalies using z-score
        let costs: Vec<f64> = events.iter().map(|e| e.cost_usd).collect();
        let cost_anomalies = Self::detect_statistical_anomalies(&costs, "cost");
        anomalies.extend(cost_anomalies);
        
        // Detect duration anomalies
        let durations: Vec<f64> = events.iter().map(|e| e.duration_ms as f64).collect();
        let duration_anomalies = Self::detect_statistical_anomalies(&durations, "duration");
        anomalies.extend(duration_anomalies);
        
        // Detect token count anomalies
        let input_tokens: Vec<f64> = events.iter().map(|e| e.input_tokens as f64).collect();
        let token_anomalies = Self::detect_statistical_anomalies(&input_tokens, "input_tokens");
        anomalies.extend(token_anomalies);
        
        if anomalies.is_empty() {
            anomalies.push("No significant anomalies detected".to_string());
        }
        
        Ok(anomalies)
    }
    
    fn detect_statistical_anomalies(values: &[f64], metric_name: &str) -> Vec<String> {
        let mut anomalies = Vec::new();
        
        if values.is_empty() {
            return anomalies;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return anomalies;
        }
        
        // Detect outliers using z-score > 3
        for (i, &value) in values.iter().enumerate() {
            let z_score = (value - mean) / std_dev;
            if z_score.abs() > 3.0 {
                anomalies.push(format!(
                    "Anomaly detected in {} at index {}: value {:.2} (z-score: {:.2})",
                    metric_name, i, value, z_score
                ));
            }
        }
        
        anomalies
    }
    
    pub fn calculate_moving_average(events: &[TelemetryEvent], window_size: usize) -> Result<Vec<serde_json::Value>> {
        if events.len() < window_size {
            return Ok(vec![]);
        }
        
        let mut sorted_events = events.to_vec();
        sorted_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        let mut moving_averages = Vec::new();
        
        for i in window_size..=sorted_events.len() {
            let window = &sorted_events[i - window_size..i];
            let avg_cost = window.iter().map(|e| e.cost_usd).sum::<f64>() / window.len() as f64;
            let avg_input = window.iter().map(|e| e.input_tokens).sum::<u32>() as f64 / window.len() as f64;
            let avg_output = window.iter().map(|e| e.output_tokens).sum::<u32>() as f64 / window.len() as f64;
            
            moving_averages.push(serde_json::json!({
                "timestamp": window.last().unwrap().timestamp.to_rfc3339(),
                "avg_cost_usd": avg_cost,
                "avg_input_tokens": avg_input,
                "avg_output_tokens": avg_output
            }));
        }
        
        Ok(moving_averages)
    }
}

#[derive(Debug)]
struct TrendResult {
    direction: String,
    slope: f64,
    change_percent: f64,
}