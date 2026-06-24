use crate::collection::{
    TokenThroughput, TokenGenerationRate, TokenCachingMetrics,
    TokenStreamingMetrics, TokenProviderCorrelation,
    TokenMetricsAggregator, ThroughputAggregation, CachingAggregation, ProviderAggregation
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Token metrics storage integration
pub struct TokenMetricsStorage {
    // In-memory storage for demonstration
    // In production, this would use a time-series database
    throughput_data: Vec<TokenThroughputRecord>,
    generation_rate_data: Vec<TokenGenerationRateRecord>,
    caching_data: Vec<TokenCachingRecord>,
    streaming_data: Vec<TokenStreamingRecord>,
    correlation_data: Vec<TokenProviderCorrelationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenThroughputRecord {
    id: String,
    request_id: String,
    total_tokens: u32,
    duration_ms: u64,
    tokens_per_second: f64,
    input_tokens_per_second: f64,
    output_tokens_per_second: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenGenerationRateRecord {
    id: String,
    request_id: String,
    generation_start_time: DateTime<Utc>,
    generation_end_time: DateTime<Utc>,
    total_generation_time_ms: u64,
    average_generation_rate: f64,
    peak_generation_rate: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenCachingRecord {
    id: String,
    request_id: String,
    cache_hits: u32,
    cache_misses: u32,
    total_requests: u32,
    hit_rate: f64,
    tokens_saved: u32,
    estimated_cost_savings_usd: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenStreamingRecord {
    id: String,
    request_id: String,
    total_streaming_chunks: u32,
    total_streaming_time_ms: u64,
    average_chunk_size: f64,
    time_to_first_token_ms: u64,
    streaming_stability: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenProviderCorrelationRecord {
    id: String,
    request_id: String,
    provider: String,
    model: String,
    token_count_json: String, // Serialized TokenCount
    cost_info_json: String,   // Serialized CostInfo
    response_time_ms: u64,
    throughput_json: String,  // Serialized TokenThroughput
    success: bool,
    error_message: Option<String>,
    timestamp: DateTime<Utc>,
}

impl TokenMetricsStorage {
    /// Create new token metrics storage
    pub fn new() -> Self {
        Self {
            throughput_data: Vec::new(),
            generation_rate_data: Vec::new(),
            caching_data: Vec::new(),
            streaming_data: Vec::new(),
            correlation_data: Vec::new(),
        }
    }

    /// Store throughput metrics
    pub fn store_throughput(&mut self, throughput: &TokenThroughput) -> Result<String> {
        let record = TokenThroughputRecord {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: throughput.request_id.clone(),
            total_tokens: throughput.total_tokens,
            duration_ms: throughput.duration_ms,
            tokens_per_second: throughput.tokens_per_second,
            input_tokens_per_second: throughput.input_tokens_per_second,
            output_tokens_per_second: throughput.output_tokens_per_second,
            timestamp: Utc::now(),
        };

        let id = record.id.clone();
        self.throughput_data.push(record);
        Ok(id)
    }

    /// Store generation rate metrics
    pub fn store_generation_rate(&mut self, generation_rate: &TokenGenerationRate) -> Result<String> {
        let start_time = DateTime::parse_from_rfc3339(&generation_rate.generation_start_time)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let end_time = DateTime::parse_from_rfc3339(&generation_rate.generation_end_time)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let record = TokenGenerationRateRecord {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: generation_rate.request_id.clone(),
            generation_start_time: start_time,
            generation_end_time: end_time,
            total_generation_time_ms: generation_rate.total_generation_time_ms,
            average_generation_rate: generation_rate.average_generation_rate,
            peak_generation_rate: generation_rate.peak_generation_rate,
            timestamp: Utc::now(),
        };

        let id = record.id.clone();
        self.generation_rate_data.push(record);
        Ok(id)
    }

    /// Store caching metrics
    pub fn store_caching_metrics(&mut self, caching: &TokenCachingMetrics) -> Result<String> {
        let record = TokenCachingRecord {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: caching.request_id.clone(),
            cache_hits: caching.cache_hits,
            cache_misses: caching.cache_misses,
            total_requests: caching.total_requests,
            hit_rate: caching.hit_rate,
            tokens_saved: caching.tokens_saved,
            estimated_cost_savings_usd: caching.estimated_cost_savings_usd,
            timestamp: Utc::now(),
        };

        let id = record.id.clone();
        self.caching_data.push(record);
        Ok(id)
    }

    /// Store streaming metrics
    pub fn store_streaming_metrics(&mut self, streaming: &TokenStreamingMetrics) -> Result<String> {
        let record = TokenStreamingRecord {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: streaming.request_id.clone(),
            total_streaming_chunks: streaming.total_streaming_chunks,
            total_streaming_time_ms: streaming.total_streaming_time_ms,
            average_chunk_size: streaming.average_chunk_size,
            time_to_first_token_ms: streaming.time_to_first_token_ms,
            streaming_stability: streaming.streaming_stability,
            timestamp: Utc::now(),
        };

        let id = record.id.clone();
        self.streaming_data.push(record);
        Ok(id)
    }

    /// Store provider correlation
    pub fn store_correlation(&mut self, correlation: &TokenProviderCorrelation) -> Result<String> {
        let token_count_json = serde_json::to_string(&correlation.token_count)?;
        let cost_info_json = serde_json::to_string(&correlation.cost_info)?;
        let throughput_json = serde_json::to_string(&correlation.throughput)?;

        let record = TokenProviderCorrelationRecord {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: correlation.request_id.clone(),
            provider: correlation.provider.clone(),
            model: correlation.model.clone(),
            token_count_json,
            cost_info_json,
            response_time_ms: correlation.response_time_ms,
            throughput_json,
            success: correlation.success,
            error_message: correlation.error_message.clone(),
            timestamp: Utc::now(),
        };

        let id = record.id.clone();
        self.correlation_data.push(record);
        Ok(id)
    }

    /// Query throughput metrics by time range
    pub fn query_throughput(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TokenThroughput>> {
        Ok(self
            .throughput_data
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .map(|r| TokenThroughput {
                request_id: r.request_id.clone(),
                total_tokens: r.total_tokens,
                duration_ms: r.duration_ms,
                tokens_per_second: r.tokens_per_second,
                input_tokens_per_second: r.input_tokens_per_second,
                output_tokens_per_second: r.output_tokens_per_second,
                timestamp: r.timestamp.to_rfc3339(),
            })
            .collect())
    }

    /// Query caching metrics by time range
    pub fn query_caching_metrics(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TokenCachingMetrics>> {
        Ok(self
            .caching_data
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .map(|r| TokenCachingMetrics {
                request_id: r.request_id.clone(),
                cache_hits: r.cache_hits,
                cache_misses: r.cache_misses,
                total_requests: r.total_requests,
                hit_rate: r.hit_rate,
                tokens_saved: r.tokens_saved,
                estimated_cost_savings_usd: r.estimated_cost_savings_usd,
            })
            .collect())
    }

    /// Query provider correlations by provider
    pub fn query_by_provider(&self, provider: &str) -> Result<Vec<TokenProviderCorrelation>> {
        Ok(self
            .correlation_data
            .iter()
            .filter(|r| r.provider == provider)
            .map(|r| {
                let token_count: crate::collection::TokenCount =
                    serde_json::from_str(&r.token_count_json).unwrap_or_default();
                let cost_info: crate::collection::CostInfo =
                    serde_json::from_str(&r.cost_info_json).unwrap_or_else(|_| {
                        crate::collection::CostInfo::new(0.0, 0.0, 0.0, 0.0)
                    });
                let throughput: TokenThroughput =
                    serde_json::from_str(&r.throughput_json).unwrap_or_else(|_| {
                        TokenThroughput {
                            request_id: r.request_id.clone(),
                            total_tokens: 0,
                            duration_ms: 0,
                            tokens_per_second: 0.0,
                            input_tokens_per_second: 0.0,
                            output_tokens_per_second: 0.0,
                            timestamp: r.timestamp.to_rfc3339(),
                        }
                    });

                TokenProviderCorrelation {
                    request_id: r.request_id.clone(),
                    provider: r.provider.clone(),
                    model: r.model.clone(),
                    token_count,
                    cost_info,
                    response_time_ms: r.response_time_ms,
                    throughput,
                    generation_rate: None,
                    streaming_metrics: None,
                    success: r.success,
                    error_message: r.error_message.clone(),
                }
            })
            .collect())
    }

    /// Get aggregated throughput metrics
    pub fn get_aggregated_throughput(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<ThroughputAggregation> {
        let throughput_data = self.query_throughput(start, end)?;
        Ok(TokenMetricsAggregator::aggregate_throughput(&throughput_data))
    }

    /// Get aggregated caching metrics
    pub fn get_aggregated_caching(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<CachingAggregation> {
        let caching_data = self.query_caching_metrics(start, end)?;
        Ok(TokenMetricsAggregator::aggregate_caching(&caching_data))
    }

    /// Get provider-specific aggregation
    pub fn get_provider_aggregation(&self, provider: &str) -> Result<ProviderAggregation> {
        let correlations = self.query_by_provider(provider)?;
        let aggregations = TokenMetricsAggregator::aggregate_by_provider(&correlations);
        Ok(aggregations
            .get(provider)
            .cloned()
            .unwrap_or_else(|| ProviderAggregation {
                total_requests: 0,
                successful_requests: 0,
                success_rate: 0.0,
                total_tokens: 0,
                total_cost_usd: 0.0,
                average_response_time_ms: 0.0,
                throughput: ThroughputAggregation::default(),
            }))
    }

    /// Batch store multiple correlations
    pub fn batch_store_correlations(
        &mut self,
        correlations: &[TokenProviderCorrelation],
    ) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        for correlation in correlations {
            let id = self.store_correlation(correlation)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Get statistics for all stored data
    pub fn get_storage_statistics(&self) -> TokenStorageStatistics {
        TokenStorageStatistics {
            total_throughput_records: self.throughput_data.len(),
            total_generation_rate_records: self.generation_rate_data.len(),
            total_caching_records: self.caching_data.len(),
            total_streaming_records: self.streaming_data.len(),
            total_correlation_records: self.correlation_data.len(),
            total_records: self.throughput_data.len()
                + self.generation_rate_data.len()
                + self.caching_data.len()
                + self.streaming_data.len()
                + self.correlation_data.len(),
        }
    }

    /// Clear old records based on retention policy
    pub fn clear_old_records(&mut self, retention_days: i64) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days);
        let initial_count = self.total_record_count();

        self.throughput_data.retain(|r| r.timestamp > cutoff);
        self.generation_rate_data.retain(|r| r.timestamp > cutoff);
        self.caching_data.retain(|r| r.timestamp > cutoff);
        self.streaming_data.retain(|r| r.timestamp > cutoff);
        self.correlation_data.retain(|r| r.timestamp > cutoff);

        let final_count = self.total_record_count();
        Ok(initial_count - final_count)
    }

    fn total_record_count(&self) -> u64 {
        (self.throughput_data.len()
            + self.generation_rate_data.len()
            + self.caching_data.len()
            + self.streaming_data.len()
            + self.correlation_data.len()) as u64
    }
}

impl Default for TokenMetricsStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct TokenStorageStatistics {
    pub total_throughput_records: usize,
    pub total_generation_rate_records: usize,
    pub total_caching_records: usize,
    pub total_streaming_records: usize,
    pub total_correlation_records: usize,
    pub total_records: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_store_throughput() {
        let mut storage = TokenMetricsStorage::new();
        let throughput = TokenThroughput {
            request_id: "req1".to_string(),
            total_tokens: 100,
            duration_ms: 1000,
            tokens_per_second: 100.0,
            input_tokens_per_second: 80.0,
            output_tokens_per_second: 20.0,
            timestamp: Utc::now().to_rfc3339(),
        };

        let id = storage.store_throughput(&throughput).unwrap();
        assert!(!id.is_empty());
        assert_eq!(storage.throughput_data.len(), 1);
    }

    #[test]
    fn test_store_caching_metrics() {
        let mut storage = TokenMetricsStorage::new();
        let caching = TokenCachingMetrics {
            request_id: "req1".to_string(),
            cache_hits: 8,
            cache_misses: 2,
            total_requests: 10,
            hit_rate: 0.8,
            tokens_saved: 1000,
            estimated_cost_savings_usd: 0.01,
        };

        let id = storage.store_caching_metrics(&caching).unwrap();
        assert!(!id.is_empty());
        assert_eq!(storage.caching_data.len(), 1);
    }

    #[test]
    fn test_query_throughput() {
        let mut storage = TokenMetricsStorage::new();
        let throughput = TokenThroughput {
            request_id: "req1".to_string(),
            total_tokens: 100,
            duration_ms: 1000,
            tokens_per_second: 100.0,
            input_tokens_per_second: 80.0,
            output_tokens_per_second: 20.0,
            timestamp: Utc::now().to_rfc3339(),
        };

        storage.store_throughput(&throughput).unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let results = storage.query_throughput(start, end).unwrap();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_aggregated_throughput() {
        let mut storage = TokenMetricsStorage::new();
        
        for i in 0..3 {
            let throughput = TokenThroughput {
                request_id: format!("req{}", i),
                total_tokens: 100,
                duration_ms: 1000,
                tokens_per_second: 100.0,
                input_tokens_per_second: 80.0,
                output_tokens_per_second: 20.0,
                timestamp: Utc::now().to_rfc3339(),
            };
            storage.store_throughput(&throughput).unwrap();
        }

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let agg = storage.get_aggregated_throughput(start, end).unwrap();

        assert_eq!(agg.total_requests, 3);
        assert_eq!(agg.total_tokens, 300);
    }

    #[test]
    fn test_clear_old_records() {
        let mut storage = TokenMetricsStorage::new();
        
        let old_throughput = TokenThroughput {
            request_id: "old_req".to_string(),
            total_tokens: 100,
            duration_ms: 1000,
            tokens_per_second: 100.0,
            input_tokens_per_second: 80.0,
            output_tokens_per_second: 20.0,
            timestamp: (Utc::now() - chrono::Duration::days(10)).to_rfc3339(),
        };

        let new_throughput = TokenThroughput {
            request_id: "new_req".to_string(),
            total_tokens: 100,
            duration_ms: 1000,
            tokens_per_second: 100.0,
            input_tokens_per_second: 80.0,
            output_tokens_per_second: 20.0,
            timestamp: Utc::now().to_rfc3339(),
        };

        storage.store_throughput(&old_throughput).unwrap();
        storage.store_throughput(&new_throughput).unwrap();

        let cleared = storage.clear_old_records(7).unwrap();
        assert_eq!(cleared, 1);
        assert_eq!(storage.throughput_data.len(), 1);
    }

    #[test]
    fn test_storage_statistics() {
        let mut storage = TokenMetricsStorage::new();
        
        let throughput = TokenThroughput {
            request_id: "req1".to_string(),
            total_tokens: 100,
            duration_ms: 1000,
            tokens_per_second: 100.0,
            input_tokens_per_second: 80.0,
            output_tokens_per_second: 20.0,
            timestamp: Utc::now().to_rfc3339(),
        };

        storage.store_throughput(&throughput).unwrap();

        let stats = storage.get_storage_statistics();
        assert_eq!(stats.total_throughput_records, 1);
        assert_eq!(stats.total_records, 1);
    }
}