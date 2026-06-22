use crate::models::{TelemetryEvent, AnalyticsQuery, AggregationType};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Aggregator {
    cache: Arc<Mutex<AggregationCache>>,
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(AggregationCache::new())),
        }
    }

    pub fn with_cache_ttl(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(AggregationCache::with_ttl(Duration::from_secs(ttl_seconds)))),
        }
    }
}

struct AggregationCache {
    data: HashMap<String, (serde_json::Value, Instant)>,
    ttl: Duration,
}

impl AggregationCache {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(300), // Default 5 minutes
        }
    }

    fn with_ttl(ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
        }
    }

    fn get(&self, key: &str) -> Option<serde_json::Value> {
        if let Some((value, timestamp)) = self.data.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            }
        }
        None
    }

    fn set(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, (value, Instant::now()));
    }

    fn clear_expired(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }
}

impl Aggregator {
    pub fn aggregate(&self, events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<serde_json::Value> {
        // Generate cache key
        let cache_key = format!("{:?}_{:?}", query.aggregation, query.group_by);
        
        // Check cache
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return Ok(cached);
        }
        
        // Perform aggregation
        let result = match &query.aggregation {
            AggregationType::Count => self.count(events, query),
            AggregationType::Sum => self.sum(events, query),
            AggregationType::Average => self.average(events, query),
            AggregationType::Min => self.min(events, query),
            AggregationType::Max => self.max(events, query),
            AggregationType::Percentile(p) => self.percentile(events, query, *p),
        };
        
        // Cache the result
        if let Ok(ref result) = result {
            self.cache.lock().unwrap().set(cache_key, result.clone());
        }
        
        result
    }

    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear_expired();
    }

    fn count(&self, events: &[TelemetryEvent], _query: &AnalyticsQuery) -> Result<serde_json::Value> {
        Ok(serde_json::json!(events.len()))
    }

    fn sum(&self, events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<serde_json::Value> {
        if query.group_by.is_empty() {
            let total_cost: f64 = events.iter().map(|e| e.cost_usd).sum();
            let total_input_tokens: u32 = events.iter().map(|e| e.input_tokens).sum();
            let total_output_tokens: u32 = events.iter().map(|e| e.output_tokens).sum();
            
            Ok(serde_json::json!({
                "total_cost_usd": total_cost,
                "total_input_tokens": total_input_tokens,
                "total_output_tokens": total_output_tokens
            }))
        } else {
            // Grouped sum implementation
            let mut grouped: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            
            for event in events {
                let key = query.group_by.iter()
                    .map(|field| Self::get_field_value(event, field))
                    .collect::<Vec<_>>()
                    .join("/");
                
                let entry = grouped.entry(key).or_insert_with(|| serde_json::json!({
                    "total_cost_usd": 0.0,
                    "total_input_tokens": 0,
                    "total_output_tokens": 0,
                    "count": 0
                }));
                
                entry["total_cost_usd"] = serde_json::json!(entry["total_cost_usd"].as_f64().unwrap_or(0.0) + event.cost_usd);
                entry["total_input_tokens"] = serde_json::json!(entry["total_input_tokens"].as_u64().unwrap_or(0) + event.input_tokens as u64);
                entry["total_output_tokens"] = serde_json::json!(entry["total_output_tokens"].as_u64().unwrap_or(0) + event.output_tokens as u64);
                entry["count"] = serde_json::json!(entry["count"].as_u64().unwrap_or(0) + 1);
            }
            
            Ok(serde_json::json!(grouped))
        }
    }

    fn average(&self, events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<serde_json::Value> {
        if events.is_empty() {
            return Ok(serde_json::json!({
                "avg_cost_usd": 0.0,
                "avg_input_tokens": 0.0,
                "avg_output_tokens": 0.0,
                "avg_duration_ms": 0.0
            }));
        }
        
        if query.group_by.is_empty() {
            let avg_cost = events.iter().map(|e| e.cost_usd).sum::<f64>() / events.len() as f64;
            let avg_input = events.iter().map(|e| e.input_tokens).sum::<u32>() as f64 / events.len() as f64;
            let avg_output = events.iter().map(|e| e.output_tokens).sum::<u32>() as f64 / events.len() as f64;
            let avg_duration = events.iter().map(|e| e.duration_ms).sum::<u64>() as f64 / events.len() as f64;
            
            Ok(serde_json::json!({
                "avg_cost_usd": avg_cost,
                "avg_input_tokens": avg_input,
                "avg_output_tokens": avg_output,
                "avg_duration_ms": avg_duration
            }))
        } else {
            // Grouped average implementation
            let mut grouped: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            
            for event in events {
                let key = query.group_by.iter()
                    .map(|field| Self::get_field_value(event, field))
                    .collect::<Vec<_>>()
                    .join("/");
                
                let entry = grouped.entry(key).or_insert_with(|| serde_json::json!({
                    "total_cost_usd": 0.0,
                    "total_input_tokens": 0,
                    "total_output_tokens": 0,
                    "total_duration_ms": 0,
                    "count": 0
                }));
                
                entry["total_cost_usd"] = serde_json::json!(entry["total_cost_usd"].as_f64().unwrap_or(0.0) + event.cost_usd);
                entry["total_input_tokens"] = serde_json::json!(entry["total_input_tokens"].as_u64().unwrap_or(0) + event.input_tokens as u64);
                entry["total_output_tokens"] = serde_json::json!(entry["total_output_tokens"].as_u64().unwrap_or(0) + event.output_tokens as u64);
                entry["total_duration_ms"] = serde_json::json!(entry["total_duration_ms"].as_u64().unwrap_or(0) + event.duration_ms);
                entry["count"] = serde_json::json!(entry["count"].as_u64().unwrap_or(0) + 1);
            }
            
            // Calculate averages
            for (_, value) in grouped.iter_mut() {
                let count = value["count"].as_u64().unwrap_or(1) as f64;
                value["avg_cost_usd"] = serde_json::json!(value["total_cost_usd"].as_f64().unwrap_or(0.0) / count);
                value["avg_input_tokens"] = serde_json::json!(value["total_input_tokens"].as_u64().unwrap_or(0) as f64 / count);
                value["avg_output_tokens"] = serde_json::json!(value["total_output_tokens"].as_u64().unwrap_or(0) as f64 / count);
                value["avg_duration_ms"] = serde_json::json!(value["total_duration_ms"].as_u64().unwrap_or(0) as f64 / count);
            }
            
            Ok(serde_json::json!(grouped))
        }
    }

    fn min(&self, events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<serde_json::Value> {
        if events.is_empty() {
            return Ok(serde_json::json!({
                "min_cost_usd": 0.0,
                "min_input_tokens": 0,
                "min_output_tokens": 0,
                "min_duration_ms": 0
            }));
        }
        
        if query.group_by.is_empty() {
            let min_cost = events.iter().map(|e| e.cost_usd).fold(f64::MAX, f64::min);
            let min_input = events.iter().map(|e| e.input_tokens).min().unwrap_or(0);
            let min_output = events.iter().map(|e| e.output_tokens).min().unwrap_or(0);
            let min_duration = events.iter().map(|e| e.duration_ms).min().unwrap_or(0);
            
            Ok(serde_json::json!({
                "min_cost_usd": min_cost,
                "min_input_tokens": min_input,
                "min_output_tokens": min_output,
                "min_duration_ms": min_duration
            }))
        } else {
            // Grouped min implementation
            let mut grouped: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            
            for event in events {
                let key = query.group_by.iter()
                    .map(|field| Self::get_field_value(event, field))
                    .collect::<Vec<_>>()
                    .join("/");
                
                let entry = grouped.entry(key).or_insert_with(|| serde_json::json!({
                    "min_cost_usd": f64::MAX,
                    "min_input_tokens": u32::MAX,
                    "min_output_tokens": u32::MAX,
                    "min_duration_ms": u64::MAX
                }));
                
                entry["min_cost_usd"] = serde_json::json!(entry["min_cost_usd"].as_f64().unwrap_or(f64::MAX).min(event.cost_usd));
                entry["min_input_tokens"] = serde_json::json!(entry["min_input_tokens"].as_u64().unwrap_or(u32::MAX as u64).min(event.input_tokens as u64));
                entry["min_output_tokens"] = serde_json::json!(entry["min_output_tokens"].as_u64().unwrap_or(u32::MAX as u64).min(event.output_tokens as u64));
                entry["min_duration_ms"] = serde_json::json!(entry["min_duration_ms"].as_u64().unwrap_or(u64::MAX).min(event.duration_ms));
            }
            
            Ok(serde_json::json!(grouped))
        }
    }

    fn max(&self, events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<serde_json::Value> {
        if events.is_empty() {
            return Ok(serde_json::json!({
                "max_cost_usd": 0.0,
                "max_input_tokens": 0,
                "max_output_tokens": 0,
                "max_duration_ms": 0
            }));
        }
        
        if query.group_by.is_empty() {
            let max_cost = events.iter().map(|e| e.cost_usd).fold(f64::MIN, f64::max);
            let max_input = events.iter().map(|e| e.input_tokens).max().unwrap_or(0);
            let max_output = events.iter().map(|e| e.output_tokens).max().unwrap_or(0);
            let max_duration = events.iter().map(|e| e.duration_ms).max().unwrap_or(0);
            
            Ok(serde_json::json!({
                "max_cost_usd": max_cost,
                "max_input_tokens": max_input,
                "max_output_tokens": max_output,
                "max_duration_ms": max_duration
            }))
        } else {
            // Grouped max implementation
            let mut grouped: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            
            for event in events {
                let key = query.group_by.iter()
                    .map(|field| Self::get_field_value(event, field))
                    .collect::<Vec<_>>()
                    .join("/");
                
                let entry = grouped.entry(key).or_insert_with(|| serde_json::json!({
                    "max_cost_usd": f64::MIN,
                    "max_input_tokens": 0,
                    "max_output_tokens": 0,
                    "max_duration_ms": 0
                }));
                
                entry["max_cost_usd"] = serde_json::json!(entry["max_cost_usd"].as_f64().unwrap_or(f64::MIN).max(event.cost_usd));
                entry["max_input_tokens"] = serde_json::json!(entry["max_input_tokens"].as_u64().unwrap_or(0).max(event.input_tokens as u64));
                entry["max_output_tokens"] = serde_json::json!(entry["max_output_tokens"].as_u64().unwrap_or(0).max(event.output_tokens as u64));
                entry["max_duration_ms"] = serde_json::json!(entry["max_duration_ms"].as_u64().unwrap_or(0).max(event.duration_ms));
            }
            
            Ok(serde_json::json!(grouped))
        }
    }

    fn percentile(&self, events: &[TelemetryEvent], query: &AnalyticsQuery, _percentile: f64) -> Result<serde_json::Value> {
        if events.is_empty() {
            return Ok(serde_json::json!({
                "p50_cost_usd": 0.0,
                "p90_cost_usd": 0.0,
                "p95_cost_usd": 0.0,
                "p99_cost_usd": 0.0
            }));
        }
        
        if query.group_by.is_empty() {
            let mut costs: Vec<f64> = events.iter().map(|e| e.cost_usd).collect();
            costs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            // Calculate multiple percentiles
            let percentiles = vec![0.5, 0.9, 0.95, 0.99];
            let mut result = serde_json::Map::new();
            
            for p in percentiles {
                let p_value = if p <= 0.0 || p >= 1.0 {
                    0.0
                } else {
                    let index = ((costs.len() - 1) as f64 * p) as usize;
                    costs[index]
                };
                result.insert(format!("p{}_cost_usd", (p * 100.0) as u32), serde_json::json!(p_value));
            }
            
            Ok(serde_json::json!(result))
        } else {
            // Grouped percentile implementation
            let mut grouped: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
            
            for event in events {
                let key = query.group_by.iter()
                    .map(|field| Self::get_field_value(event, field))
                    .collect::<Vec<_>>()
                    .join("/");
                
                grouped.entry(key).or_insert_with(Vec::new).push(event.cost_usd);
            }
            
            let mut result: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            let percentiles = vec![0.5, 0.9, 0.95, 0.99];
            
            for (key, mut costs) in grouped {
                costs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mut percentile_result = serde_json::Map::new();
                
                for p in &percentiles {
                    let index = ((costs.len() - 1) as f64 * p) as usize;
                    let p_value = if *p <= 0.0 || *p >= 1.0 { 0.0 } else { costs[index] };
                    percentile_result.insert(format!("p{}_cost_usd", (*p * 100.0) as u32), serde_json::json!(p_value));
                }
                
                result.insert(key, serde_json::json!(percentile_result));
            }
            
            Ok(serde_json::json!(result))
        }
    }
    
    fn get_field_value(event: &TelemetryEvent, field: &str) -> String {
        match field {
            "ai_client" => event.ai_client.clone(),
            "ai_provider" => event.ai_provider.clone(),
            "model" => event.model.clone(),
            "input_type" => event.input_type.clone(),
            _ => "unknown".to_string(),
        }
    }

    pub fn histogram(&self, events: &[TelemetryEvent], field: &str, buckets: Vec<f64>) -> Result<serde_json::Value> {
        if events.is_empty() {
            return Ok(serde_json::json!({
                "field": field,
                "buckets": []
            }));
        }

        let mut histogram: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        
        for event in events {
            let value = match field {
                "cost_usd" => event.cost_usd,
                "input_tokens" => event.input_tokens as f64,
                "output_tokens" => event.output_tokens as f64,
                "duration_ms" => event.duration_ms as f64,
                _ => continue,
            };
            
            let bucket_label = Self::find_bucket_static(value, &buckets);
            *histogram.entry(bucket_label).or_insert(0) += 1;
        }
        
        Ok(serde_json::json!({
            "field": field,
            "buckets": histogram
        }))
    }

    fn find_bucket_static(value: f64, buckets: &[f64]) -> String {
        for (i, &bucket) in buckets.iter().enumerate() {
            if value <= bucket {
                if i == 0 {
                    return format!("<= {}", bucket);
                } else {
                    return format!("{}-{}", buckets[i - 1], bucket);
                }
            }
        }
        format!("> {}", buckets.last().unwrap_or(&0.0))
    }

    pub fn rate_calculations(&self, events: &[TelemetryEvent], time_window_seconds: u64) -> Result<serde_json::Value> {
        if events.is_empty() || time_window_seconds == 0 {
            return Ok(serde_json::json!({
                "requests_per_second": 0.0,
                "input_tokens_per_second": 0.0,
                "output_tokens_per_second": 0.0,
                "total_tokens_per_second": 0.0
            }));
        }

        let total_requests = events.len() as f64;
        let total_input_tokens: u32 = events.iter().map(|e| e.input_tokens).sum();
        let total_output_tokens: u32 = events.iter().map(|e| e.output_tokens).sum();
        let window_secs = time_window_seconds as f64;

        Ok(serde_json::json!({
            "requests_per_second": total_requests / window_secs,
            "input_tokens_per_second": total_input_tokens as f64 / window_secs,
            "output_tokens_per_second": total_output_tokens as f64 / window_secs,
            "total_tokens_per_second": (total_input_tokens + total_output_tokens) as f64 / window_secs
        }))
    }
}