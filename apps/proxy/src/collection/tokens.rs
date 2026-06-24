use analytics_rs::{TextTokenEstimator, TokenEstimate, CostCalculator};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::time::Duration;

/// Token counting and cost estimation
/// Collects token usage and calculates costs for AI requests
pub struct TokenCollector {
    text_estimator: TextTokenEstimator,
}

impl TokenCollector {
    /// Create a new token collector
    pub fn new() -> Self {
        Self {
            text_estimator: TextTokenEstimator::new(),
        }
    }

    /// Estimate tokens for text input
    pub fn estimate_text_tokens(&self, text: &str, model: &str) -> Result<TokenEstimate> {
        Ok(self.text_estimator.estimate_by_chars(text, model))
    }

    /// Estimate tokens for a chat message
    pub fn estimate_chat_tokens(&self, messages: &[Value], model: &str) -> Result<TokenEstimate> {
        let chat_messages: Vec<analytics_rs::ChatMessage> = messages
            .iter()
            .filter_map(|m| {
                Some(analytics_rs::ChatMessage {
                    role: m.get("role")?.as_str()?.to_string(),
                    content: m.get("content")?.as_str()?.to_string(),
                })
            })
            .collect();

        Ok(self.text_estimator.estimate_chat_tokens(&chat_messages, model))
    }

    /// Calculate cost for token usage
    pub fn calculate_cost(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64> {
        let cost_calculator = CostCalculator::new();
        cost_calculator.estimate_cost(provider, model, input_tokens, output_tokens, "text")
    }

    /// Extract and count tokens from request
    pub fn extract_request_tokens(&self, request: &Value, model: &str) -> Result<TokenCount> {
        let input_text = self.extract_input_text(request)?;
        let input_estimate = self.estimate_text_tokens(&input_text, model)?;

        Ok(TokenCount {
            input_tokens: input_estimate.estimated_tokens,
            output_tokens: 0, // Will be updated from response
            total_tokens: input_estimate.estimated_tokens,
            input_characters: input_text.len() as u32,
        })
    }

    /// Update token count with response data
    pub fn update_with_response(&self, token_count: &mut TokenCount, response: &Value, model: &str) -> Result<()> {
        let output_text = self.extract_output_text(response)?;
        let output_estimate = self.estimate_text_tokens(&output_text, model)?;

        token_count.output_tokens = output_estimate.estimated_tokens;
        token_count.total_tokens = token_count.input_tokens + token_count.output_tokens;

        Ok(())
    }

    /// Extract input text from request
    fn extract_input_text(&self, request: &Value) -> Result<String> {
        if let Some(prompt) = request.get("prompt").and_then(|p| p.as_str()) {
            return Ok(prompt.to_string());
        }

        if let Some(messages) = request.get("messages").and_then(|m| m.as_array()) {
            return serde_json::to_string(messages)
                .map_err(|e| anyhow::anyhow!("Failed to serialize messages: {}", e));
        }

        if let Some(content) = request.get("content").and_then(|c| c.as_str()) {
            return Ok(content.to_string());
        }

        Ok(String::new())
    }

    /// Extract output text from response
    fn extract_output_text(&self, response: &Value) -> Result<String> {
        if let Some(content) = response.get("content").and_then(|c| c.as_str()) {
            return Ok(content.to_string());
        }

        if let Some(text) = response.get("text").and_then(|t| t.as_str()) {
            return Ok(text.to_string());
        }

        if let Some(completion) = response.get("completion").and_then(|c| c.as_str()) {
            return Ok(completion.to_string());
        }

        Ok(String::new())
    }

    /// Calculate cost for a complete request/response cycle
    pub fn calculate_total_cost(
        &self,
        provider: &str,
        model: &str,
        token_count: &TokenCount,
    ) -> Result<f64> {
        let cost_calculator = CostCalculator::new();
        cost_calculator.estimate_cost(
            provider,
            model,
            token_count.input_tokens,
            token_count.output_tokens,
            "text",
        )
    }
}

impl Default for TokenCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Token count for a request/response cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCount {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub input_characters: u32,
}

impl TokenCount {
    /// Create a new token count
    pub fn new() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            input_characters: 0,
        }
    }

    /// Calculate token efficiency (tokens per character)
    pub fn token_efficiency(&self) -> f64 {
        if self.input_characters > 0 {
            self.input_tokens as f64 / self.input_characters as f64
        } else {
            0.0
        }
    }

    /// Calculate output ratio (output / input)
    pub fn output_ratio(&self) -> f64 {
        if self.input_tokens > 0 {
            self.output_tokens as f64 / self.input_tokens as f64
        } else {
            0.0
        }
    }
}

impl Default for TokenCount {
    fn default() -> Self {
        Self::new()
    }
}

/// Cost information for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    pub input_cost_usd: f64,
    pub output_cost_usd: f64,
    pub total_cost_usd: f64,
    pub cost_per_1k_input_tokens: f64,
    pub cost_per_1k_output_tokens: f64,
}

impl CostInfo {
    /// Create new cost info
    pub fn new(
        input_cost_usd: f64,
        output_cost_usd: f64,
        cost_per_1k_input_tokens: f64,
        cost_per_1k_output_tokens: f64,
    ) -> Self {
        Self {
            input_cost_usd,
            output_cost_usd,
            total_cost_usd: input_cost_usd + output_cost_usd,
            cost_per_1k_input_tokens,
            cost_per_1k_output_tokens,
        }
    }

    /// Calculate cost per token
    pub fn cost_per_token(&self, total_tokens: u32) -> f64 {
        if total_tokens > 0 {
            self.total_cost_usd / total_tokens as f64
        } else {
            0.0
        }
    }
}

/// Token throughput metrics (tokens per second)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenThroughput {
    pub request_id: String,
    pub total_tokens: u32,
    pub duration_ms: u64,
    pub tokens_per_second: f64,
    pub input_tokens_per_second: f64,
    pub output_tokens_per_second: f64,
    pub timestamp: String,
}

impl TokenThroughput {
    /// Calculate throughput from token count and duration
    pub fn calculate(request_id: String, token_count: &TokenCount, duration: Duration) -> Self {
        let duration_ms = duration.as_millis() as u64;
        let duration_sec = duration.as_secs_f64();

        let tokens_per_second = if duration_sec > 0.0 {
            token_count.total_tokens as f64 / duration_sec
        } else {
            0.0
        };

        let input_tokens_per_second = if duration_sec > 0.0 {
            token_count.input_tokens as f64 / duration_sec
        } else {
            0.0
        };

        let output_tokens_per_second = if duration_sec > 0.0 {
            token_count.output_tokens as f64 / duration_sec
        } else {
            0.0
        };

        Self {
            request_id,
            total_tokens: token_count.total_tokens,
            duration_ms,
            tokens_per_second,
            input_tokens_per_second,
            output_tokens_per_second,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Check if throughput meets performance threshold
    pub fn meets_threshold(&self, threshold_tokens_per_sec: f64) -> bool {
        self.tokens_per_second >= threshold_tokens_per_sec
    }
}

/// Token generation rate analysis
#[derive(Debug, Clone)]
pub struct TokenGenerationRate {
    pub request_id: String,
    pub generation_start_time: String,
    pub generation_end_time: String,
    pub total_generation_time_ms: u64,
    pub average_generation_rate: f64, // tokens per second
    pub peak_generation_rate: f64,
    pub generation_intervals: Vec<GenerationInterval>,
}

#[derive(Debug, Clone)]
pub struct GenerationInterval {
    pub start_ms: u64,
    pub end_ms: u64,
    pub tokens_generated: u32,
    pub rate: f64, // tokens per second
}

impl TokenGenerationRate {
    /// Analyze generation rate from streaming token data
    pub fn analyze(request_id: String, streaming_data: &[StreamingTokenData]) -> Self {
        if streaming_data.is_empty() {
            return Self {
                request_id,
                generation_start_time: String::new(),
                generation_end_time: String::new(),
                total_generation_time_ms: 0,
                average_generation_rate: 0.0,
                peak_generation_rate: 0.0,
                generation_intervals: Vec::new(),
            };
        }

        let start_time = streaming_data.first().unwrap().timestamp.clone();
        let end_time = streaming_data.last().unwrap().timestamp.clone();
        let total_time_ms = streaming_data.last().unwrap().elapsed_ms;

        let total_tokens: u32 = streaming_data.iter().map(|d| d.token_count).sum();
        let average_rate = if total_time_ms > 0 {
            (total_tokens as f64 / total_time_ms as f64) * 1000.0
        } else {
            0.0
        };

        let mut intervals = Vec::new();
        let mut peak_rate: f64 = 0.0;

        for window in streaming_data.windows(2) {
            let interval = GenerationInterval {
                start_ms: window[0].elapsed_ms,
                end_ms: window[1].elapsed_ms,
                tokens_generated: window[1].token_count - window[0].token_count,
                rate: if window[1].elapsed_ms > window[0].elapsed_ms {
                    ((window[1].token_count - window[0].token_count) as f64 /
                     (window[1].elapsed_ms - window[0].elapsed_ms) as f64) * 1000.0
                } else {
                    0.0
                },
            };
            peak_rate = peak_rate.max(interval.rate);
            intervals.push(interval);
        }

        Self {
            request_id,
            generation_start_time: start_time,
            generation_end_time: end_time,
            total_generation_time_ms: total_time_ms,
            average_generation_rate: average_rate,
            peak_generation_rate: peak_rate,
            generation_intervals: intervals,
        }
    }
}

/// Streaming token data for real-time metrics
#[derive(Debug, Clone)]
pub struct StreamingTokenData {
    pub timestamp: String,
    pub elapsed_ms: u64,
    pub token_count: u32,
    pub is_final: bool,
}

/// Token caching efficiency metrics
#[derive(Debug, Clone)]
pub struct TokenCachingMetrics {
    pub request_id: String,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub total_requests: u32,
    pub hit_rate: f64,
    pub tokens_saved: u32,
    pub estimated_cost_savings_usd: f64,
}

impl TokenCachingMetrics {
    /// Calculate caching efficiency
    pub fn calculate(
        request_id: String,
        cache_hits: u32,
        cache_misses: u32,
        tokens_saved: u32,
        cost_per_token: f64,
    ) -> Self {
        let total_requests = cache_hits + cache_misses;
        let hit_rate = if total_requests > 0 {
            cache_hits as f64 / total_requests as f64
        } else {
            0.0
        };

        let estimated_cost_savings = tokens_saved as f64 * cost_per_token;

        Self {
            request_id,
            cache_hits,
            cache_misses,
            total_requests,
            hit_rate,
            tokens_saved,
            estimated_cost_savings_usd: estimated_cost_savings,
        }
    }

    /// Update metrics with new cache result
    pub fn update(&mut self, hit: bool, tokens_saved: u32, cost_per_token: f64) {
        if hit {
            self.cache_hits += 1;
            self.tokens_saved += tokens_saved;
            self.estimated_cost_savings_usd += tokens_saved as f64 * cost_per_token;
        } else {
            self.cache_misses += 1;
        }
        self.total_requests += 1;
        self.hit_rate = if self.total_requests > 0 {
            self.cache_hits as f64 / self.total_requests as f64
        } else {
            0.0
        };
    }
}

/// Token streaming metrics
#[derive(Debug, Clone)]
pub struct TokenStreamingMetrics {
    pub request_id: String,
    pub total_streaming_chunks: u32,
    pub total_streaming_time_ms: u64,
    pub average_chunk_size: f64,
    pub time_to_first_token_ms: u64,
    pub time_between_chunks_ms: Vec<u64>,
    pub streaming_stability: f64, // lower is more stable
}

impl TokenStreamingMetrics {
    /// Calculate streaming metrics from chunk data
    pub fn calculate(request_id: String, chunks: &[StreamingChunk]) -> Self {
        if chunks.is_empty() {
            return Self {
                request_id,
                total_streaming_chunks: 0,
                total_streaming_time_ms: 0,
                average_chunk_size: 0.0,
                time_to_first_token_ms: 0,
                time_between_chunks_ms: Vec::new(),
                streaming_stability: 0.0,
            };
        }

        let total_chunks = chunks.len() as u32;
        let total_time_ms = chunks.last().unwrap().elapsed_ms;
        let total_tokens: u32 = chunks.iter().map(|c| c.token_count).sum();
        let average_chunk_size = if total_chunks > 0 {
            total_tokens as f64 / total_chunks as f64
        } else {
            0.0
        };

        let time_to_first_token = chunks.first().unwrap().elapsed_ms;

        let mut time_between_chunks = Vec::new();
        for window in chunks.windows(2) {
            time_between_chunks.push(window[1].elapsed_ms - window[0].elapsed_ms);
        }

        let streaming_stability = if time_between_chunks.len() > 1 {
            let mean = time_between_chunks.iter().sum::<u64>() as f64 / time_between_chunks.len() as f64;
            let variance = time_between_chunks.iter()
                .map(|&x| {
                    let diff = x as f64 - mean;
                    diff * diff
                })
                .sum::<f64>() / time_between_chunks.len() as f64;
            variance.sqrt() / mean // coefficient of variation
        } else {
            0.0
        };

        Self {
            request_id,
            total_streaming_chunks: total_chunks,
            total_streaming_time_ms: total_time_ms,
            average_chunk_size,
            time_to_first_token_ms: time_to_first_token,
            time_between_chunks_ms: time_between_chunks,
            streaming_stability,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamingChunk {
    pub elapsed_ms: u64,
    pub token_count: u32,
}

/// Token metrics correlated with AI provider responses
#[derive(Debug, Clone)]
pub struct TokenProviderCorrelation {
    pub request_id: String,
    pub provider: String,
    pub model: String,
    pub token_count: TokenCount,
    pub cost_info: CostInfo,
    pub response_time_ms: u64,
    pub throughput: TokenThroughput,
    pub generation_rate: Option<TokenGenerationRate>,
    pub streaming_metrics: Option<TokenStreamingMetrics>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl TokenProviderCorrelation {
    /// Create correlation from request/response data
    pub fn create(
        request_id: String,
        provider: String,
        model: String,
        token_count: TokenCount,
        cost_info: CostInfo,
        response_time_ms: u64,
        throughput: TokenThroughput,
        success: bool,
        error_message: Option<String>,
    ) -> Self {
        Self {
            request_id,
            provider,
            model,
            token_count,
            cost_info,
            response_time_ms,
            throughput,
            generation_rate: None,
            streaming_metrics: None,
            success,
            error_message,
        }
    }

    /// Add generation rate data
    pub fn with_generation_rate(mut self, generation_rate: TokenGenerationRate) -> Self {
        self.generation_rate = Some(generation_rate);
        self
    }

    /// Add streaming metrics
    pub fn with_streaming_metrics(mut self, streaming_metrics: TokenStreamingMetrics) -> Self {
        self.streaming_metrics = Some(streaming_metrics);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_token_throughput_calculation() {
        let token_count = TokenCount {
            input_tokens: 80,
            output_tokens: 20,
            total_tokens: 100,
            input_characters: 400,
        };

        let duration = Duration::from_millis(1000);
        let throughput = TokenThroughput::calculate("req1".to_string(), &token_count, duration);

        assert_eq!(throughput.request_id, "req1");
        assert_eq!(throughput.total_tokens, 100);
        assert_eq!(throughput.duration_ms, 1000);
        assert!((throughput.tokens_per_second - 100.0).abs() < 0.01);
        assert!((throughput.input_tokens_per_second - 80.0).abs() < 0.01);
        assert!((throughput.output_tokens_per_second - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_token_throughput_threshold() {
        let token_count = TokenCount {
            input_tokens: 80,
            output_tokens: 20,
            total_tokens: 100,
            input_characters: 400,
        };

        let duration = Duration::from_millis(1000);
        let throughput = TokenThroughput::calculate("req1".to_string(), &token_count, duration);

        assert!(throughput.meets_threshold(50.0));
        assert!(!throughput.meets_threshold(150.0));
    }

    #[test]
    fn test_token_generation_rate_analysis() {
        let streaming_data = vec![
            StreamingTokenData {
                timestamp: chrono::Utc::now().to_rfc3339(),
                elapsed_ms: 0,
                token_count: 0,
                is_final: false,
            },
            StreamingTokenData {
                timestamp: chrono::Utc::now().to_rfc3339(),
                elapsed_ms: 100,
                token_count: 10,
                is_final: false,
            },
            StreamingTokenData {
                timestamp: chrono::Utc::now().to_rfc3339(),
                elapsed_ms: 200,
                token_count: 20,
                is_final: true,
            },
        ];

        let generation_rate = TokenGenerationRate::analyze("req1".to_string(), &streaming_data);

        assert_eq!(generation_rate.request_id, "req1");
        assert_eq!(generation_rate.total_generation_time_ms, 200);
        assert!((generation_rate.average_generation_rate - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_token_caching_metrics_calculation() {
        let caching = TokenCachingMetrics::calculate("req1".to_string(), 8, 2, 1000, 0.00001);

        assert_eq!(caching.request_id, "req1");
        assert_eq!(caching.cache_hits, 8);
        assert_eq!(caching.cache_misses, 2);
        assert_eq!(caching.total_requests, 10);
        assert!((caching.hit_rate - 0.8).abs() < 0.01);
        assert_eq!(caching.tokens_saved, 1000);
        assert!((caching.estimated_cost_savings_usd - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_token_caching_metrics_update() {
        let mut caching = TokenCachingMetrics::calculate("req1".to_string(), 8, 2, 1000, 0.00001);

        caching.update(true, 500, 0.00001);
        assert_eq!(caching.cache_hits, 9);
        assert_eq!(caching.tokens_saved, 1500);

        caching.update(false, 0, 0.00001);
        assert_eq!(caching.cache_misses, 3);
        assert_eq!(caching.total_requests, 12);
    }

    #[test]
    fn test_token_streaming_metrics_calculation() {
        let chunks = vec![
            StreamingChunk {
                elapsed_ms: 50,
                token_count: 5,
            },
            StreamingChunk {
                elapsed_ms: 100,
                token_count: 10,
            },
            StreamingChunk {
                elapsed_ms: 150,
                token_count: 15,
            },
        ];

        let streaming = TokenStreamingMetrics::calculate("req1".to_string(), &chunks);

        assert_eq!(streaming.request_id, "req1");
        assert_eq!(streaming.total_streaming_chunks, 3);
        assert_eq!(streaming.total_streaming_time_ms, 150);
        assert_eq!(streaming.time_to_first_token_ms, 50);
        assert!((streaming.average_chunk_size - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_token_provider_correlation_creation() {
        let token_count = TokenCount {
            input_tokens: 80,
            output_tokens: 20,
            total_tokens: 100,
            input_characters: 400,
        };

        let cost_info = CostInfo::new(0.01, 0.02, 15.0, 30.0);

        let throughput = TokenThroughput::calculate(
            "req1".to_string(),
            &token_count,
            Duration::from_millis(1000),
        );

        let correlation = TokenProviderCorrelation::create(
            "req1".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            token_count,
            cost_info,
            1000,
            throughput,
            true,
            None,
        );

        assert_eq!(correlation.request_id, "req1");
        assert_eq!(correlation.provider, "anthropic");
        assert_eq!(correlation.model, "claude-3-opus");
        assert!(correlation.success);
        assert!(correlation.error_message.is_none());
    }

    #[test]
    fn test_token_provider_correlation_with_generation_rate() {
        let token_count = TokenCount {
            input_tokens: 80,
            output_tokens: 20,
            total_tokens: 100,
            input_characters: 400,
        };

        let cost_info = CostInfo::new(0.01, 0.02, 15.0, 30.0);

        let throughput = TokenThroughput::calculate(
            "req1".to_string(),
            &token_count,
            Duration::from_millis(1000),
        );

        let generation_rate = TokenGenerationRate {
            request_id: "req1".to_string(),
            generation_start_time: chrono::Utc::now().to_rfc3339(),
            generation_end_time: chrono::Utc::now().to_rfc3339(),
            total_generation_time_ms: 500,
            average_generation_rate: 200.0,
            peak_generation_rate: 250.0,
            generation_intervals: vec![],
        };

        let correlation = TokenProviderCorrelation::create(
            "req1".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            token_count,
            cost_info,
            1000,
            throughput,
            true,
            None,
        )
        .with_generation_rate(generation_rate);

        assert!(correlation.generation_rate.is_some());
        assert_eq!(
            correlation.generation_rate.as_ref().unwrap().average_generation_rate,
            200.0
        );
    }

    #[test]
    fn test_empty_streaming_data() {
        let streaming_data: Vec<StreamingTokenData> = vec![];
        let generation_rate = TokenGenerationRate::analyze("req1".to_string(), &streaming_data);

        assert_eq!(generation_rate.total_tokens_generated, 0);
        assert_eq!(generation_rate.average_generation_rate, 0.0);
    }

    #[test]
    fn test_empty_chunks() {
        let chunks: Vec<StreamingChunk> = vec![];
        let streaming = TokenStreamingMetrics::calculate("req1".to_string(), &chunks);

        assert_eq!(streaming.total_streaming_chunks, 0);
        assert_eq!(streaming.average_chunk_size, 0.0);
    }
}