use crate::collection::tokens::{
    TokenThroughput, TokenGenerationRate, TokenCachingMetrics,
    TokenStreamingMetrics, TokenProviderCorrelation
};
use std::collections::HashMap;

/// Token metrics aggregation functions
pub struct TokenMetricsAggregator;

impl TokenMetricsAggregator {
    /// Aggregate throughput metrics across multiple requests
    pub fn aggregate_throughput(throughput_data: &[TokenThroughput]) -> ThroughputAggregation {
        if throughput_data.is_empty() {
            return ThroughputAggregation::default();
        }

        let total_tokens: u32 = throughput_data.iter().map(|t| t.total_tokens).sum();
        let total_duration_ms: u64 = throughput_data.iter().map(|t| t.duration_ms).sum();
        let average_tokens_per_second = if total_duration_ms > 0 {
            (total_tokens as f64 / total_duration_ms as f64) * 1000.0
        } else {
            0.0
        };

        let tokens_per_second_values: Vec<f64> = throughput_data.iter()
            .map(|t| t.tokens_per_second)
            .collect();

        let median_tokens_per_second = Self::calculate_median(&tokens_per_second_values);
        let p95_tokens_per_second = Self::calculate_percentile(&tokens_per_second_values, 95.0);
        let p99_tokens_per_second = Self::calculate_percentile(&tokens_per_second_values, 99.0);

        let min_tokens_per_second = tokens_per_second_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_tokens_per_second = tokens_per_second_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        ThroughputAggregation {
            total_requests: throughput_data.len() as u32,
            total_tokens,
            total_duration_ms,
            average_tokens_per_second,
            median_tokens_per_second,
            p95_tokens_per_second,
            p99_tokens_per_second,
            min_tokens_per_second,
            max_tokens_per_second,
        }
    }

    /// Aggregate generation rate metrics
    pub fn aggregate_generation_rates(generation_rates: &[TokenGenerationRate]) -> GenerationRateAggregation {
        if generation_rates.is_empty() {
            return GenerationRateAggregation::default();
        }

        let average_rates: Vec<f64> = generation_rates.iter()
            .map(|r| r.average_generation_rate)
            .collect();

        let peak_rates: Vec<f64> = generation_rates.iter()
            .map(|r| r.peak_generation_rate)
            .collect();

        let total_generation_time_ms: u64 = generation_rates.iter()
            .map(|r| r.total_generation_time_ms)
            .sum();

        GenerationRateAggregation {
            total_requests: generation_rates.len() as u32,
            average_generation_rate: Self::calculate_mean(&average_rates),
            median_generation_rate: Self::calculate_median(&average_rates),
            p95_generation_rate: Self::calculate_percentile(&average_rates, 95.0),
            average_peak_rate: Self::calculate_mean(&peak_rates),
            median_peak_rate: Self::calculate_median(&peak_rates),
            total_generation_time_ms,
        }
    }

    /// Aggregate caching metrics
    pub fn aggregate_caching(caching_metrics: &[TokenCachingMetrics]) -> CachingAggregation {
        if caching_metrics.is_empty() {
            return CachingAggregation::default();
        }

        let total_cache_hits: u32 = caching_metrics.iter().map(|m| m.cache_hits).sum();
        let total_cache_misses: u32 = caching_metrics.iter().map(|m| m.cache_misses).sum();
        let total_requests: u32 = caching_metrics.iter().map(|m| m.total_requests).sum();
        let total_tokens_saved: u32 = caching_metrics.iter().map(|m| m.tokens_saved).sum();
        let total_cost_savings_usd: f64 = caching_metrics.iter().map(|m| m.estimated_cost_savings_usd).sum();

        let overall_hit_rate = if total_requests > 0 {
            total_cache_hits as f64 / total_requests as f64
        } else {
            0.0
        };

        let hit_rates: Vec<f64> = caching_metrics.iter().map(|m| m.hit_rate).collect();

        CachingAggregation {
            total_requests,
            total_cache_hits,
            total_cache_misses,
            overall_hit_rate,
            median_hit_rate: Self::calculate_median(&hit_rates),
            total_tokens_saved,
            total_cost_savings_usd,
        }
    }

    /// Aggregate streaming metrics
    pub fn aggregate_streaming(streaming_metrics: &[TokenStreamingMetrics]) -> StreamingAggregation {
        if streaming_metrics.is_empty() {
            return StreamingAggregation::default();
        }

        let total_chunks: u32 = streaming_metrics.iter().map(|m| m.total_streaming_chunks).sum();
        let total_streaming_time_ms: u64 = streaming_metrics.iter().map(|m| m.total_streaming_time_ms).sum();
        let average_chunk_sizes: Vec<f64> = streaming_metrics.iter().map(|m| m.average_chunk_size).collect();
        let time_to_first_tokens: Vec<u64> = streaming_metrics.iter().map(|m| m.time_to_first_token_ms).collect();
        let streaming_stabilities: Vec<f64> = streaming_metrics.iter().map(|m| m.streaming_stability).collect();

        StreamingAggregation {
            total_requests: streaming_metrics.len() as u32,
            total_chunks,
            total_streaming_time_ms,
            average_chunk_size: Self::calculate_mean(&average_chunk_sizes),
            median_chunk_size: Self::calculate_median(&average_chunk_sizes),
            average_time_to_first_token_ms: Self::calculate_mean_u64(&time_to_first_tokens),
            median_time_to_first_token_ms: Self::calculate_median_u64(&time_to_first_tokens),
            average_streaming_stability: Self::calculate_mean(&streaming_stabilities),
            median_streaming_stability: Self::calculate_median(&streaming_stabilities),
        }
    }

    /// Aggregate provider correlations
    pub fn aggregate_by_provider(correlations: &[TokenProviderCorrelation]) -> HashMap<String, ProviderAggregation> {
        let mut by_provider: HashMap<String, Vec<&TokenProviderCorrelation>> = HashMap::new();

        for correlation in correlations {
            by_provider
                .entry(correlation.provider.clone())
                .or_insert_with(Vec::new)
                .push(correlation);
        }

        let mut aggregations = HashMap::new();

        for (provider, provider_correlations) in by_provider {
            let total_requests = provider_correlations.len() as u32;
            let successful_requests = provider_correlations.iter().filter(|c| c.success).count() as u32;
            let success_rate = if total_requests > 0 {
                successful_requests as f64 / total_requests as f64
            } else {
                0.0
            };

            let total_tokens: u32 = provider_correlations.iter()
                .map(|c| c.token_count.total_tokens)
                .sum();

            let total_cost_usd: f64 = provider_correlations.iter()
                .map(|c| c.cost_info.total_cost_usd)
                .sum();

            let average_response_time_ms: f64 = provider_correlations.iter()
                .map(|c| c.response_time_ms as f64)
                .sum::<f64>() / total_requests as f64;

            let throughput_data: Vec<TokenThroughput> = provider_correlations.iter()
                .map(|c| c.throughput.clone())
                .collect();

            let throughput_agg = Self::aggregate_throughput(&throughput_data);

            aggregations.insert(
                provider,
                ProviderAggregation {
                    total_requests,
                    successful_requests,
                    success_rate,
                    total_tokens,
                    total_cost_usd,
                    average_response_time_ms,
                    throughput: throughput_agg,
                },
            );
        }

        aggregations
    }

    /// Aggregate by model
    pub fn aggregate_by_model(correlations: &[TokenProviderCorrelation]) -> HashMap<String, ModelAggregation> {
        let mut by_model: HashMap<String, Vec<&TokenProviderCorrelation>> = HashMap::new();

        for correlation in correlations {
            let model_key = format!("{}::{}", correlation.provider, correlation.model);
            by_model
                .entry(model_key)
                .or_insert_with(Vec::new)
                .push(correlation);
        }

        let mut aggregations = HashMap::new();

        for (model_key, model_correlations) in by_model {
            let parts: Vec<&str> = model_key.split("::").collect();
            let provider = parts[0].to_string();
            let model = parts[1].to_string();

            let total_requests = model_correlations.len() as u32;
            let total_tokens: u32 = model_correlations.iter()
                .map(|c| c.token_count.total_tokens)
                .sum();

            let total_cost_usd: f64 = model_correlations.iter()
                .map(|c| c.cost_info.total_cost_usd)
                .sum();

            let average_cost_per_1k_tokens = if total_tokens > 0 {
                (total_cost_usd / total_tokens as f64) * 1000.0
            } else {
                0.0
            };

            let throughput_data: Vec<TokenThroughput> = model_correlations.iter()
                .map(|c| c.throughput.clone())
                .collect();

            let throughput_agg = Self::aggregate_throughput(&throughput_data);

            aggregations.insert(
                model_key,
                ModelAggregation {
                    provider,
                    model,
                    total_requests,
                    total_tokens,
                    total_cost_usd,
                    average_cost_per_1k_tokens,
                    throughput: throughput_agg,
                },
            );
        }

        aggregations
    }

    /// Calculate time-series aggregation for token metrics
    pub fn aggregate_time_series(
        correlations: &[TokenProviderCorrelation],
        window_minutes: u64,
    ) -> Vec<TimeSeriesTokenMetrics> {
        if correlations.is_empty() {
            return Vec::new();
        }

        let mut time_windows: HashMap<String, Vec<&TokenProviderCorrelation>> = HashMap::new();

        for correlation in correlations {
            let timestamp = correlation.throughput.timestamp.clone();
            let window_key = Self::get_time_window(&timestamp, window_minutes);
            time_windows
                .entry(window_key)
                .or_insert_with(Vec::new)
                .push(correlation);
        }

        let mut time_series = Vec::new();

        for (window_key, window_correlations) in time_windows {
            let total_tokens: u32 = window_correlations.iter()
                .map(|c| c.token_count.total_tokens)
                .sum();

            let total_cost_usd: f64 = window_correlations.iter()
                .map(|c| c.cost_info.total_cost_usd)
                .sum();

            let throughput_data: Vec<TokenThroughput> = window_correlations.iter()
                .map(|c| c.throughput.clone())
                .collect();

            let throughput_agg = Self::aggregate_throughput(&throughput_data);

            time_series.push(TimeSeriesTokenMetrics {
                window_start: window_key,
                window_minutes,
                total_requests: window_correlations.len() as u32,
                total_tokens,
                total_cost_usd,
                throughput: throughput_agg,
            });
        }

        time_series.sort_by(|a, b| a.window_start.cmp(&b.window_start));
        time_series
    }

    /// Get time window key for aggregation
    fn get_time_window(timestamp: &str, window_minutes: u64) -> String {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
            let timestamp_secs = dt.timestamp();
            let window_secs = window_minutes * 60;
            let window_start = (timestamp_secs / window_secs as i64) * window_secs as i64;
            
            if let Some(window_dt) = chrono::DateTime::from_timestamp(window_start, 0) {
                return window_dt.to_rfc3339();
            }
        }
        timestamp.to_string()
    }

    /// Calculate mean of values
    fn calculate_mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// Calculate mean of u64 values
    fn calculate_mean_u64(values: &[u64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<u64>() as f64 / values.len() as f64
    }

    /// Calculate median of values
    fn calculate_median(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
        } else {
            sorted[len / 2]
        }
    }

    /// Calculate median of u64 values
    fn calculate_median_u64(values: &[u64]) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let mut sorted = values.to_vec();
        sorted.sort();
        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) / 2
        } else {
            sorted[len / 2]
        }
    }

    /// Calculate percentile of values
    fn calculate_percentile(values: &[f64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = sorted.len();
        if len == 0 {
            return 0.0;
        }
        let index = ((percentile / 100.0) * (len - 1) as f64) as usize;
        sorted[index.min(len - 1)]
    }
}

/// Aggregated throughput metrics
#[derive(Debug, Clone, Default)]
pub struct ThroughputAggregation {
    pub total_requests: u32,
    pub total_tokens: u32,
    pub total_duration_ms: u64,
    pub average_tokens_per_second: f64,
    pub median_tokens_per_second: f64,
    pub p95_tokens_per_second: f64,
    pub p99_tokens_per_second: f64,
    pub min_tokens_per_second: f64,
    pub max_tokens_per_second: f64,
}

/// Aggregated generation rate metrics
#[derive(Debug, Clone, Default)]
pub struct GenerationRateAggregation {
    pub total_requests: u32,
    pub average_generation_rate: f64,
    pub median_generation_rate: f64,
    pub p95_generation_rate: f64,
    pub average_peak_rate: f64,
    pub median_peak_rate: f64,
    pub total_generation_time_ms: u64,
}

/// Aggregated caching metrics
#[derive(Debug, Clone, Default)]
pub struct CachingAggregation {
    pub total_requests: u32,
    pub total_cache_hits: u32,
    pub total_cache_misses: u32,
    pub overall_hit_rate: f64,
    pub median_hit_rate: f64,
    pub total_tokens_saved: u32,
    pub total_cost_savings_usd: f64,
}

/// Aggregated streaming metrics
#[derive(Debug, Clone, Default)]
pub struct StreamingAggregation {
    pub total_requests: u32,
    pub total_chunks: u32,
    pub total_streaming_time_ms: u64,
    pub average_chunk_size: f64,
    pub median_chunk_size: f64,
    pub average_time_to_first_token_ms: f64,
    pub median_time_to_first_token_ms: u64,
    pub average_streaming_stability: f64,
    pub median_streaming_stability: f64,
}

/// Provider-specific aggregation
#[derive(Debug, Clone)]
pub struct ProviderAggregation {
    pub total_requests: u32,
    pub successful_requests: u32,
    pub success_rate: f64,
    pub total_tokens: u32,
    pub total_cost_usd: f64,
    pub average_response_time_ms: f64,
    pub throughput: ThroughputAggregation,
}

/// Model-specific aggregation
#[derive(Debug, Clone)]
pub struct ModelAggregation {
    pub provider: String,
    pub model: String,
    pub total_requests: u32,
    pub total_tokens: u32,
    pub total_cost_usd: f64,
    pub average_cost_per_1k_tokens: f64,
    pub throughput: ThroughputAggregation,
}

/// Time-series token metrics
#[derive(Debug, Clone)]
pub struct TimeSeriesTokenMetrics {
    pub window_start: String,
    pub window_minutes: u64,
    pub total_requests: u32,
    pub total_tokens: u32,
    pub total_cost_usd: f64,
    pub throughput: ThroughputAggregation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_aggregate_throughput() {
        let throughput_data = vec![
            TokenThroughput {
                request_id: "req1".to_string(),
                total_tokens: 100,
                duration_ms: 1000,
                tokens_per_second: 100.0,
                input_tokens_per_second: 80.0,
                output_tokens_per_second: 20.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            TokenThroughput {
                request_id: "req2".to_string(),
                total_tokens: 200,
                duration_ms: 2000,
                tokens_per_second: 100.0,
                input_tokens_per_second: 160.0,
                output_tokens_per_second: 40.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];

        let agg = TokenMetricsAggregator::aggregate_throughput(&throughput_data);

        assert_eq!(agg.total_requests, 2);
        assert_eq!(agg.total_tokens, 300);
        assert_eq!(agg.total_duration_ms, 3000);
        assert!((agg.average_tokens_per_second - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregate_caching() {
        let caching_metrics = vec![
            TokenCachingMetrics {
                request_id: "req1".to_string(),
                cache_hits: 8,
                cache_misses: 2,
                total_requests: 10,
                hit_rate: 0.8,
                tokens_saved: 1000,
                estimated_cost_savings_usd: 0.01,
            },
            TokenCachingMetrics {
                request_id: "req2".to_string(),
                cache_hits: 6,
                cache_misses: 4,
                total_requests: 10,
                hit_rate: 0.6,
                tokens_saved: 500,
                estimated_cost_savings_usd: 0.005,
            },
        ];

        let agg = TokenMetricsAggregator::aggregate_caching(&caching_metrics);

        assert_eq!(agg.total_requests, 20);
        assert_eq!(agg.total_cache_hits, 14);
        assert_eq!(agg.total_cache_misses, 6);
        assert!((agg.overall_hit_rate - 0.7).abs() < 0.01);
        assert_eq!(agg.total_tokens_saved, 1500);
    }

    #[test]
    fn test_aggregate_by_provider() {
        let correlations = vec![
            TokenProviderCorrelation {
                request_id: "req1".to_string(),
                provider: "anthropic".to_string(),
                model: "claude-3-opus".to_string(),
                token_count: TokenCount::new(),
                cost_info: CostInfo::new(0.01, 0.02, 15.0, 30.0),
                response_time_ms: 1000,
                throughput: TokenThroughput {
                    request_id: "req1".to_string(),
                    total_tokens: 100,
                    duration_ms: 1000,
                    tokens_per_second: 100.0,
                    input_tokens_per_second: 80.0,
                    output_tokens_per_second: 20.0,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                generation_rate: None,
                streaming_metrics: None,
                success: true,
                error_message: None,
            },
        ];

        let by_provider = TokenMetricsAggregator::aggregate_by_provider(&correlations);

        assert!(by_provider.contains_key("anthropic"));
        let anthropic_agg = by_provider.get("anthropic").unwrap();
        assert_eq!(anthropic_agg.total_requests, 1);
        assert_eq!(anthropic_agg.successful_requests, 1);
    }

    #[test]
    fn test_calculate_median() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let median = TokenMetricsAggregator::calculate_median(&values);
        assert!((median - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let p95 = TokenMetricsAggregator::calculate_percentile(&values, 95.0);
        assert!((p95 - 9.55).abs() < 0.1);
    }
}