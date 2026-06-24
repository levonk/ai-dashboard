use analytics_rs::Timer;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// AI Performance Metrics Collector
/// Collects AI-specific performance metrics including TTFT, token speeds, and token counts
pub struct AIMetricsCollector {
    ttft_timer: Timer,
    total_timer: Timer,
    prefill_timer: Timer,
    decode_timer: Timer,
    request_id: String,
    prompt_token_count: Option<u32>,
    output_token_count: Option<u32>,
    first_token_received: bool,
}

impl AIMetricsCollector {
    /// Create a new AI metrics collector
    pub fn new(request_id: String) -> Self {
        Self {
            ttft_timer: Timer::new(),
            total_timer: Timer::new(),
            prefill_timer: Timer::new(),
            decode_timer: Timer::new(),
            request_id,
            prompt_token_count: None,
            output_token_count: None,
            first_token_received: false,
        }
    }

    /// Start timing the total request processing
    pub fn start_total_timing(&mut self) {
        self.total_timer.start_timer();
    }

    /// Start timing for TTFT (Time to First Token)
    pub fn start_ttft(&mut self) {
        self.ttft_timer.start_timer();
    }

    /// Mark when the first token is received (stops TTFT timer)
    pub fn mark_first_token(&mut self) {
        if !self.first_token_received {
            self.ttft_timer.stop();
            self.first_token_received = true;
        }
    }

    /// Start timing for prefill phase
    pub fn start_prefill(&mut self) {
        self.prefill_timer.start_timer();
    }

    /// Stop timing for prefill phase
    pub fn stop_prefill(&mut self) {
        self.prefill_timer.stop();
    }

    /// Start timing for decode phase
    pub fn start_decode(&mut self) {
        self.decode_timer.start_timer();
    }

    /// Stop timing for decode phase
    pub fn stop_decode(&mut self) {
        self.decode_timer.stop();
    }

    /// Stop total timing
    pub fn stop_total_timing(&mut self) {
        self.total_timer.stop();
    }

    /// Set prompt token count
    pub fn set_prompt_token_count(&mut self, count: u32) {
        self.prompt_token_count = Some(count);
    }

    /// Set output token count
    pub fn set_output_token_count(&mut self, count: u32) {
        self.output_token_count = Some(count);
    }

    /// Get TTFT in milliseconds
    pub fn ttft_ms(&self) -> Option<f64> {
        if self.first_token_received {
            Some(self.ttft_timer.elapsed_millis())
        } else {
            None
        }
    }

    /// Get total processing time in milliseconds
    pub fn total_processing_time_ms(&self) -> Option<f64> {
        if !self.total_timer.is_running() {
            Some(self.total_timer.elapsed_millis())
        } else {
            None
        }
    }

    /// Calculate prefill token speed (tokens/second)
    pub fn prefill_token_speed(&self) -> Option<f64> {
        let prompt_tokens = self.prompt_token_count? as f64;
        let prefill_duration_sec = self.prefill_timer.elapsed_secs();
        
        if prefill_duration_sec > 0.0 {
            Some(prompt_tokens / prefill_duration_sec)
        } else {
            None
        }
    }

    /// Calculate decode token speed (tokens/second)
    pub fn decode_token_speed(&self) -> Option<f64> {
        let output_tokens = self.output_token_count? as f64;
        let decode_duration_sec = self.decode_timer.elapsed_secs();
        
        if decode_duration_sec > 0.0 {
            Some(output_tokens / decode_duration_sec)
        } else {
            None
        }
    }

    /// Get prompt token count
    pub fn prompt_token_count(&self) -> Option<u32> {
        self.prompt_token_count
    }

    /// Get output token count
    pub fn output_token_count(&self) -> Option<u32> {
        self.output_token_count
    }

    /// Calculate context length (total tokens in prompt)
    pub fn context_length(&self) -> Option<u32> {
        self.prompt_token_count
    }

    /// Calculate token efficiency (output tokens / total tokens)
    pub fn token_efficiency(&self) -> Option<f64> {
        let prompt_tokens = self.prompt_token_count? as f64;
        let output_tokens = self.output_token_count? as f64;
        let total_tokens = prompt_tokens + output_tokens;
        
        if total_tokens > 0.0 {
            Some(output_tokens / total_tokens)
        } else {
            None
        }
    }

    /// Collect all AI metrics into a structured record
    pub fn collect_metrics(&self) -> Result<AIMetrics> {
        Ok(AIMetrics {
            request_id: self.request_id.clone(),
            ttft_ms: self.ttft_ms(),
            total_processing_time_ms: self.total_processing_time_ms(),
            prefill_token_speed: self.prefill_token_speed(),
            decode_token_speed: self.decode_token_speed(),
            prompt_token_count: self.prompt_token_count(),
            output_token_count: self.output_token_count(),
            context_length: self.context_length(),
            token_efficiency: self.token_efficiency(),
            recorded_at: Utc::now(),
        })
    }

    /// Reset all timers and counters
    pub fn reset(&mut self) {
        self.ttft_timer.reset();
        self.total_timer.reset();
        self.prefill_timer.reset();
        self.decode_timer.reset();
        self.prompt_token_count = None;
        self.output_token_count = None;
        self.first_token_received = false;
    }
}

impl Default for AIMetricsCollector {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

/// AI Performance Metrics Record
#[derive(Debug, Clone)]
pub struct AIMetrics {
    pub request_id: String,
    pub ttft_ms: Option<f64>,
    pub total_processing_time_ms: Option<f64>,
    pub prefill_token_speed: Option<f64>,
    pub decode_token_speed: Option<f64>,
    pub prompt_token_count: Option<u32>,
    pub output_token_count: Option<u32>,
    pub context_length: Option<u32>,
    pub token_efficiency: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

impl AIMetrics {
    /// Create a new AI metrics record
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            ttft_ms: None,
            total_processing_time_ms: None,
            prefill_token_speed: None,
            decode_token_speed: None,
            prompt_token_count: None,
            output_token_count: None,
            context_length: None,
            token_efficiency: None,
            recorded_at: Utc::now(),
        }
    }

    /// Convert to database-compatible format
    pub fn to_db_values(&self) -> Result<AIMetricsDbValues> {
        Ok(AIMetricsDbValues {
            request_id: self.request_id.clone(),
            ttft_ms: self.ttft_ms,
            total_processing_time_ms: self.total_processing_time_ms,
            prefill_token_speed: self.prefill_token_speed,
            decode_token_speed: self.decode_token_speed,
            prompt_token_count: self.prompt_token_count.map(|v| v as i32),
            output_token_count: self.output_token_count.map(|v| v as i32),
            context_length: self.context_length.map(|v| v as i32),
            token_efficiency: self.token_efficiency,
            recorded_at: self.recorded_at,
        })
    }
}

/// Database-compatible AI metrics values
#[derive(Debug, Clone)]
pub struct AIMetricsDbValues {
    pub request_id: String,
    pub ttft_ms: Option<f64>,
    pub total_processing_time_ms: Option<f64>,
    pub prefill_token_speed: Option<f64>,
    pub decode_token_speed: Option<f64>,
    pub prompt_token_count: Option<i32>,
    pub output_token_count: Option<i32>,
    pub context_length: Option<i32>,
    pub token_efficiency: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

/// Extract AI metrics from provider response
pub fn extract_ai_metrics_from_response(
    response: &Value,
    prompt_tokens: Option<u32>,
) -> Result<AIMetrics> {
    let mut metrics = AIMetrics::new("extracted".to_string());
    
    // Extract token counts from response if available
    if let Some(usage) = response.get("usage") {
        if let Some(prompt_tokens_response) = usage.get("prompt_tokens").and_then(|v| v.as_u64()) {
            metrics.prompt_token_count = Some(prompt_tokens_response as u32);
        }
        if let Some(completion_tokens) = usage.get("completion_tokens").and_then(|v| v.as_u64()) {
            metrics.output_token_count = Some(completion_tokens as u32);
        }
    }
    
    // Use provided prompt tokens if not in response
    if metrics.prompt_token_count.is_none() && prompt_tokens.is_some() {
        metrics.prompt_token_count = prompt_tokens;
    }
    
    // Calculate derived metrics
    if let (Some(prompt), Some(output)) = (metrics.prompt_token_count, metrics.output_token_count) {
        metrics.context_length = Some(prompt);
        let total = prompt + output;
        if total > 0 {
            metrics.token_efficiency = Some(output as f64 / total as f64);
        }
    }
    
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_metrics_collector_creation() {
        let collector = AIMetricsCollector::new("test-request".to_string());
        assert_eq!(collector.request_id, "test-request");
        assert!(!collector.first_token_received);
    }

    #[test]
    fn test_ttft_measurement() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.start_ttft();
        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.mark_first_token();
        
        let ttft = collector.ttft_ms();
        assert!(ttft.is_some());
        assert!(ttft.unwrap() >= 10.0);
    }

    #[test]
    fn test_total_processing_time() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.start_total_timing();
        std::thread::sleep(std::time::Duration::from_millis(20));
        collector.stop_total_timing();
        
        let total_time = collector.total_processing_time_ms();
        assert!(total_time.is_some());
        assert!(total_time.unwrap() >= 20.0);
    }

    #[test]
    fn test_prefill_token_speed() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.set_prompt_token_count(100);
        collector.start_prefill();
        std::thread::sleep(std::time::Duration::from_millis(50));
        collector.stop_prefill();
        
        let speed = collector.prefill_token_speed();
        assert!(speed.is_some());
        // 100 tokens in ~50ms = ~2000 tokens/second
        assert!(speed.unwrap() > 1000.0);
    }

    #[test]
    fn test_decode_token_speed() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.set_output_token_count(50);
        collector.start_decode();
        std::thread::sleep(std::time::Duration::from_millis(25));
        collector.stop_decode();
        
        let speed = collector.decode_token_speed();
        assert!(speed.is_some());
        // 50 tokens in ~25ms = ~2000 tokens/second
        assert!(speed.unwrap() > 1000.0);
    }

    #[test]
    fn test_token_counts() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.set_prompt_token_count(150);
        collector.set_output_token_count(75);
        
        assert_eq!(collector.prompt_token_count(), Some(150));
        assert_eq!(collector.output_token_count(), Some(75));
        assert_eq!(collector.context_length(), Some(150));
    }

    #[test]
    fn test_token_efficiency() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.set_prompt_token_count(100);
        collector.set_output_token_count(50);
        
        let efficiency = collector.token_efficiency();
        assert!(efficiency.is_some());
        // 50 / (100 + 50) = 0.333...
        assert!((efficiency.unwrap() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_collect_metrics() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.start_total_timing();
        collector.start_ttft();
        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.mark_first_token();
        collector.set_prompt_token_count(100);
        collector.set_output_token_count(50);
        collector.stop_total_timing();
        
        let metrics = collector.collect_metrics().unwrap();
        assert_eq!(metrics.request_id, "test-request");
        assert!(metrics.ttft_ms.is_some());
        assert!(metrics.total_processing_time_ms.is_some());
        assert_eq!(metrics.prompt_token_count, Some(100));
        assert_eq!(metrics.output_token_count, Some(50));
    }

    #[test]
    fn test_reset() {
        let mut collector = AIMetricsCollector::new("test-request".to_string());
        collector.start_total_timing();
        collector.set_prompt_token_count(100);
        collector.mark_first_token();
        
        collector.reset();
        
        assert!(!collector.first_token_received);
        assert!(collector.prompt_token_count.is_none());
        assert!(collector.ttft_ms().is_none());
    }

    #[test]
    fn test_extract_ai_metrics_from_response() {
        let response = serde_json::json!({
            "usage": {
                "prompt_tokens": 150,
                "completion_tokens": 75
            }
        });
        
        let metrics = extract_ai_metrics_from_response(&response, None).unwrap();
        assert_eq!(metrics.prompt_token_count, Some(150));
        assert_eq!(metrics.output_token_count, Some(75));
        assert_eq!(metrics.context_length, Some(150));
        assert!(metrics.token_efficiency.is_some());
    }

    #[test]
    fn test_extract_ai_metrics_with_fallback() {
        let response = serde_json::json!({
            "usage": {
                "completion_tokens": 50
            }
        });
        
        let metrics = extract_ai_metrics_from_response(&response, Some(100)).unwrap();
        assert_eq!(metrics.prompt_token_count, Some(100)); // Fallback
        assert_eq!(metrics.output_token_count, Some(50));
    }

    #[test]
    fn test_ai_metrics_to_db_values() {
        let mut metrics = AIMetrics::new("test-request".to_string());
        metrics.ttft_ms = Some(100.5);
        metrics.total_processing_time_ms = Some(500.0);
        metrics.prompt_token_count = Some(150);
        metrics.output_token_count = Some(75);
        metrics.token_efficiency = Some(0.333);
        
        let db_values = metrics.to_db_values().unwrap();
        assert_eq!(db_values.request_id, "test-request");
        assert_eq!(db_values.ttft_ms, Some(100.5));
        assert_eq!(db_values.prompt_token_count, Some(150));
    }
}
