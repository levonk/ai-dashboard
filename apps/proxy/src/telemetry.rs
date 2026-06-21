use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

use analytics_rs::TelemetryEvent;

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub sample_rate: f64,
    pub max_payload_size: usize,
    pub sanitize_headers: bool,
    pub redact_sensitive_data: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 1.0,
            max_payload_size: 1024 * 1024, // 1MB
            sanitize_headers: true,
            redact_sensitive_data: true,
        }
    }
}

/// Request metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub ai_client: String,
    pub ai_provider: String,
    pub model: String,
    pub input_type: String,
    pub tags: HashMap<String, String>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub status_code: u16,
    pub duration_ms: u64,
    pub output_tokens: u32,
    pub cache_hit: bool,
    pub error_message: Option<String>,
}

/// Telemetry collector
pub struct TelemetryCollector {
    config: TelemetryConfig,
    events: Vec<TelemetryEvent>,
}

impl TelemetryCollector {
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
        }
    }

    /// Collect request telemetry
    pub fn collect_request(&mut self, metadata: RequestMetadata) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Apply sampling
        if rand::random::<f64>() > self.config.sample_rate {
            debug!("Request sampled out");
            return Ok(());
        }

        let event = TelemetryEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: metadata.timestamp,
            ai_client: metadata.ai_client,
            ai_provider: metadata.ai_provider,
            model: metadata.model,
            input_type: metadata.input_type,
            input_tokens: 0, // Will be estimated separately
            output_tokens: 0,
            duration_ms: 0,
            cost_usd: 0.0,
            metadata: serde_json::to_value(metadata.tags).unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        };

        self.events.push(event);
        debug!("Collected request telemetry: {}", metadata.request_id);
        Ok(())
    }

    /// Collect response telemetry
    pub fn collect_response(&mut self, metadata: ResponseMetadata) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Find corresponding request event and update it
        if let Some(event) = self.events.iter_mut().find(|e| e.event_id == metadata.request_id) {
            event.duration_ms = metadata.duration_ms;
            event.output_tokens = metadata.output_tokens;
            if metadata.status_code >= 400 {
                event.metadata = serde_json::json!({
                    "error": metadata.error_message,
                    "status_code": metadata.status_code
                });
            }
            debug!("Updated response telemetry: {}", metadata.request_id);
        } else {
            warn!("No matching request found for response: {}", metadata.request_id);
        }

        Ok(())
    }

    /// Get all collected events
    pub fn get_events(&self) -> &[TelemetryEvent] {
        &self.events
    }

    /// Clear collected events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Middleware for collecting telemetry
pub struct TelemetryMiddleware {
    collector: Arc<std::sync::Mutex<TelemetryCollector>>,
}

impl TelemetryMiddleware {
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            collector: Arc::new(std::sync::Mutex::new(TelemetryCollector::new(config))),
        }
    }

    /// Get collector reference
    pub fn collector(&self) -> Arc<std::sync::Mutex<TelemetryCollector>> {
        self.collector.clone()
    }

    /// Sanitize headers by removing sensitive information
    fn sanitize_headers(headers: &HashMap<String, String>) -> HashMap<String, String> {
        let sensitive_keys = vec![
            "authorization",
            "x-api-key",
            "api-key",
            "x-goog-api-key",
            "x-amz-security-token",
            "cookie",
            "set-cookie",
        ];

        headers
            .iter()
            .map(|(k, v)| {
                let key = k.to_lowercase();
                if sensitive_keys.contains(&key.as_str()) {
                    (k.clone(), "[REDACTED]".to_string())
                } else {
                    (k.clone(), v.clone())
                }
            })
            .collect()
    }

    /// Redact sensitive data from payload
    fn redact_payload(payload: &str, max_size: usize) -> String {
        if payload.len() > max_size {
            format!("{}...[TRUNCATED]", &payload[..max_size])
        } else {
            payload.to_string()
        }
    }
}

/// Extract model from request
pub fn extract_model_from_request(headers: &HashMap<String, String>, body: &str) -> Option<String> {
    // Try to extract from headers first
    if let Some(model) = headers.get("x-model").or_else(|| headers.get("model")) {
        return Some(model.clone());
    }

    // Try to extract from body (common patterns)
    if body.contains("\"model\"") {
        if let Some(start) = body.find("\"model\"") {
            let after_key = &body[start + 7..];
            if let Some(colon) = after_key.find(':') {
                let after_colon = &after_key[colon + 1..].trim();
                if let Some(end) = after_colon.find(',') {
                    let model_value = after_colon[..end].trim().trim_matches('"');
                    return Some(model_value.to_string());
                }
            }
        }
    }

    None
}

/// Estimate input tokens from text (rough approximation)
pub fn estimate_tokens(text: &str) -> u32 {
    // Rough approximation: ~4 characters per token for English text
    // This is a simple heuristic - actual tokenization depends on the model
    ((text.len() as f32) / 4.0).ceil() as u32
}

/// Create request metadata from HTTP request
pub fn create_request_metadata(
    request_id: String,
    ai_client: String,
    ai_provider: String,
    headers: HashMap<String, String>,
    body: &str,
) -> RequestMetadata {
    let model = extract_model_from_request(&headers, body).unwrap_or_else(|| "unknown".to_string());
    let input_type = if body.contains("\"image\"") || body.contains("\"image_url\"") {
        "image".to_string()
    } else if body.contains("\"audio\"") {
        "audio".to_string()
    } else {
        "text".to_string()
    };

    let mut tags = HashMap::new();
    tags.insert("request_id".to_string(), request_id.clone());
    
    RequestMetadata {
        request_id,
        timestamp: Utc::now(),
        ai_client,
        ai_provider,
        model,
        input_type,
        tags,
    }
}

/// Create response metadata from HTTP response
pub fn create_response_metadata(
    request_id: String,
    status_code: u16,
    duration_ms: u64,
    headers: HashMap<String, String>,
    body: &str,
) -> ResponseMetadata {
    let cache_hit = headers.get("x-cache").map(|v| v == "HIT").unwrap_or(false);
    let output_tokens = estimate_tokens(body);
    let error_message = if status_code >= 400 {
        Some(format!("HTTP {}", status_code))
    } else {
        None
    };

    ResponseMetadata {
        request_id,
        timestamp: Utc::now(),
        status_code,
        duration_ms,
        output_tokens,
        cache_hit,
        error_message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.sample_rate, 1.0);
        assert!(config.sanitize_headers);
    }

    #[test]
    fn test_collect_request() {
        let config = TelemetryConfig::default();
        let mut collector = TelemetryCollector::new(config);

        let metadata = RequestMetadata {
            request_id: "test-123".to_string(),
            timestamp: Utc::now(),
            ai_client: "claude-code".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            tags: HashMap::new(),
        };

        assert!(collector.collect_request(metadata).is_ok());
        assert_eq!(collector.event_count(), 1);
    }

    #[test]
    fn test_collect_response() {
        let config = TelemetryConfig::default();
        let mut collector = TelemetryCollector::new(config);

        let request_metadata = RequestMetadata {
            request_id: "test-123".to_string(),
            timestamp: Utc::now(),
            ai_client: "claude-code".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            tags: HashMap::new(),
        };

        collector.collect_request(request_metadata).unwrap();

        let response_metadata = ResponseMetadata {
            request_id: "test-123".to_string(),
            timestamp: Utc::now(),
            status_code: 200,
            duration_ms: 1500,
            output_tokens: 100,
            cache_hit: false,
            error_message: None,
        };

        assert!(collector.collect_response(response_metadata).is_ok());
        
        let events = collector.get_events();
        assert_eq!(events[0].duration_ms, 1500);
        assert_eq!(events[0].output_tokens, 100);
    }

    #[test]
    fn test_sanitize_headers() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer secret".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());

        let sanitized = TelemetryMiddleware::sanitize_headers(&headers);
        assert_eq!(sanitized.get("authorization").unwrap(), "[REDACTED]");
        assert_eq!(sanitized.get("content-type").unwrap(), "application/json");
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens("hello world"), 3);
        assert_eq!(estimate_tokens("This is a longer sentence with more words"), 12);
    }

    #[test]
    fn test_extract_model_from_request() {
        let mut headers = HashMap::new();
        headers.insert("x-model".to_string(), "gpt-4".to_string());

        assert_eq!(
            extract_model_from_request(&headers, "{}"),
            Some("gpt-4".to_string())
        );
    }

    #[test]
    fn test_create_request_metadata() {
        let headers = HashMap::new();
        let metadata = create_request_metadata(
            "test-123".to_string(),
            "claude-code".to_string(),
            "openai".to_string(),
            headers,
            "{}"
        );

        assert_eq!(metadata.request_id, "test-123");
        assert_eq!(metadata.ai_client, "claude-code");
        assert_eq!(metadata.ai_provider, "openai");
    }
}
