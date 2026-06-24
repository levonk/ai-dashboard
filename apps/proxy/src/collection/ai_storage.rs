use crate::collection::{AIMetrics, AIMetricsDbValues};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// AI Metrics Storage Integration
/// Handles storage of AI performance metrics to database
pub struct AIMetricsStorage {
    // In a real implementation, this would hold database connection
    // For now, we provide the interface structure
}

impl AIMetricsStorage {
    /// Create new AI metrics storage
    pub fn new() -> Self {
        Self {}
    }

    /// Store AI metrics for a request
    pub fn store_ai_metrics(
        &self,
        request_event_id: Uuid,
        metrics: &AIMetricsDbValues,
    ) -> Result<()> {
        // Validate inputs
        self.validate_storage_input(request_event_id, metrics)?;

        // In a real implementation, this would execute:
        // INSERT INTO performance_metrics (
        //     request_event_id, ttft_ms, total_processing_time_ms,
        //     prefill_token_speed, decode_token_speed,
        //     prompt_token_count, output_token_count,
        //     context_length, token_efficiency, recorded_at
        // ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)

        Ok(())
    }

    /// Store AI metrics in batch
    pub fn store_batch_ai_metrics(
        &self,
        metrics_batch: Vec<(Uuid, AIMetricsDbValues)>,
    ) -> Result<usize> {
        for (request_event_id, metrics) in &metrics_batch {
            self.validate_storage_input(*request_event_id, metrics)?;
        }

        // In a real implementation, this would execute batch INSERT
        Ok(metrics_batch.len())
    }

    /// Query AI metrics for a specific request
    pub fn query_ai_metrics(&self, request_event_id: Uuid) -> Result<Option<AIMetricsDbValues>> {
        // In a real implementation, this would execute:
        // SELECT * FROM performance_metrics WHERE request_event_id = ?
        
        Ok(None)
    }

    /// Query AI metrics for a time range
    pub fn query_ai_metrics_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AIMetricsDbValues>> {
        // In a real implementation, this would execute:
        // SELECT * FROM performance_metrics 
        // WHERE recorded_at >= ? AND recorded_at <= ?
        // ORDER BY recorded_at DESC
        
        Ok(vec![])
    }

    /// Query aggregated AI metrics
    pub fn query_aggregated_ai_metrics(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        period_type: &str,
    ) -> Result<Value> {
        // In a real implementation, this would query the aggregates table:
        // SELECT * FROM performance_metrics_aggregates
        // WHERE period_start >= ? AND period_end <= ?
        // AND period_type = ?
        
        Ok(serde_json::json!({
            "period_type": period_type,
            "start_time": start_time,
            "end_time": end_time,
            "aggregates": []
        }))
    }

    /// Calculate and store aggregated metrics
    pub fn calculate_and_store_aggregates(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        period_type: &str,
    ) -> Result<()> {
        // In a real implementation, this would:
        // 1. Query raw metrics for the time period
        // 2. Calculate aggregates (avg, p50, p95, p99, etc.)
        // 3. Store in performance_metrics_aggregates table
        
        Ok(())
    }

    /// Delete old metrics based on retention policy
    pub fn apply_retention_policy(&self, retention_days: i64) -> Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::days(retention_days);
        
        // In a real implementation, this would execute:
        // DELETE FROM performance_metrics WHERE recorded_at < ?
        
        Ok(0)
    }

    /// Validate storage input
    fn validate_storage_input(&self, request_event_id: Uuid, metrics: &AIMetricsDbValues) -> Result<()> {
        if request_event_id.is_nil() {
            anyhow::bail!("Request event ID cannot be nil");
        }

        if metrics.request_id.is_empty() {
            anyhow::bail!("Request ID cannot be empty");
        }

        // Validate numeric ranges
        if let Some(ttft) = metrics.ttft_ms {
            if ttft < 0.0 || ttft > 1_000_000.0 {
                anyhow::bail!("TTFT must be between 0 and 1,000,000 ms");
            }
        }

        if let Some(total_time) = metrics.total_processing_time_ms {
            if total_time < 0.0 || total_time > 1_000_000.0 {
                anyhow::bail!("Total processing time must be between 0 and 1,000,000 ms");
            }
        }

        if let Some(efficiency) = metrics.token_efficiency {
            if efficiency < 0.0 || efficiency > 1.0 {
                anyhow::bail!("Token efficiency must be between 0 and 1");
            }
        }

        Ok(())
    }

    /// Convert AIMetrics to database-compatible format
    pub fn prepare_for_storage(
        request_event_id: Uuid,
        metrics: AIMetrics,
    ) -> Result<AIMetricsDbValues> {
        let db_values = metrics.to_db_values()?;
        
        // Additional validation for storage
        if request_event_id.is_nil() {
            anyhow::bail!("Request event ID cannot be nil");
        }

        Ok(db_values)
    }
}

impl Default for AIMetricsStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// AI Metrics Query Builder
pub struct AIMetricsQueryBuilder {
    request_event_id: Option<Uuid>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    min_ttft: Option<f64>,
    max_ttft: Option<f64>,
    min_efficiency: Option<f64>,
    limit: Option<usize>,
}

impl AIMetricsQueryBuilder {
    /// Create new query builder
    pub fn new() -> Self {
        Self {
            request_event_id: None,
            start_time: None,
            end_time: None,
            min_ttft: None,
            max_ttft: None,
            min_efficiency: None,
            limit: None,
        }
    }

    /// Filter by request event ID
    pub fn with_request_event_id(mut self, request_event_id: Uuid) -> Self {
        self.request_event_id = Some(request_event_id);
        self
    }

    /// Filter by time range start
    pub fn with_start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Filter by time range end
    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = Some(end_time);
        self
    }

    /// Filter by minimum TTFT
    pub fn with_min_ttft(mut self, min_ttft: f64) -> Self {
        self.min_ttft = Some(min_ttft);
        self
    }

    /// Filter by maximum TTFT
    pub fn with_max_ttft(mut self, max_ttft: f64) -> Self {
        self.max_ttft = Some(max_ttft);
        self
    }

    /// Filter by minimum token efficiency
    pub fn with_min_efficiency(mut self, min_efficiency: f64) -> Self {
        self.min_efficiency = Some(min_efficiency);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the query
    pub fn execute(&self, storage: &AIMetricsStorage) -> Result<Vec<AIMetricsDbValues>> {
        // In a real implementation, this would build and execute the SQL query
        // based on the filters set in the builder
        
        if let Some(request_event_id) = self.request_event_id {
            if let Some(metrics) = storage.query_ai_metrics(request_event_id)? {
                return Ok(vec![metrics]);
            }
        }

        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            return storage.query_ai_metrics_by_time_range(start, end);
        }

        Ok(vec![])
    }
}

impl Default for AIMetricsQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_metrics_storage_creation() {
        let storage = AIMetricsStorage::new();
        // Storage created successfully
    }

    #[test]
    fn test_store_ai_metrics() {
        let storage = AIMetricsStorage::new();
        let request_event_id = Uuid::new_v4();
        
        let mut metrics = AIMetrics::new("test-request".to_string());
        metrics.ttft_ms = Some(100.0);
        metrics.total_processing_time_ms = Some(500.0);
        metrics.prompt_token_count = Some(150);
        metrics.output_token_count = Some(75);
        
        let db_values = metrics.to_db_values().unwrap();
        let result = storage.store_ai_metrics(request_event_id, &db_values);
        assert!(result.is_ok());
    }

    #[test]
    fn test_store_batch_ai_metrics() {
        let storage = AIMetricsStorage::new();
        
        let batch = vec![
            (Uuid::new_v4(), AIMetricsDbValues {
                request_id: "req1".to_string(),
                ttft_ms: Some(100.0),
                total_processing_time_ms: Some(500.0),
                prefill_token_speed: None,
                decode_token_speed: None,
                prompt_token_count: Some(150),
                output_token_count: Some(75),
                context_length: Some(150),
                token_efficiency: Some(0.333),
                recorded_at: Utc::now(),
            }),
            (Uuid::new_v4(), AIMetricsDbValues {
                request_id: "req2".to_string(),
                ttft_ms: Some(150.0),
                total_processing_time_ms: Some(600.0),
                prefill_token_speed: None,
                decode_token_speed: None,
                prompt_token_count: Some(200),
                output_token_count: Some(100),
                context_length: Some(200),
                token_efficiency: Some(0.333),
                recorded_at: Utc::now(),
            }),
        ];
        
        let result = storage.store_batch_ai_metrics(batch);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_validation_nil_request_id() {
        let storage = AIMetricsStorage::new();
        let metrics = AIMetricsDbValues {
            request_id: "test".to_string(),
            ttft_ms: Some(100.0),
            total_processing_time_ms: Some(500.0),
            prefill_token_speed: None,
            decode_token_speed: None,
            prompt_token_count: Some(150),
            output_token_count: Some(75),
            context_length: Some(150),
            token_efficiency: Some(0.333),
            recorded_at: Utc::now(),
        };
        
        let result = storage.store_ai_metrics(Uuid::nil(), &metrics);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_empty_request_id() {
        let storage = AIMetricsStorage::new();
        let metrics = AIMetricsDbValues {
            request_id: "".to_string(),
            ttft_ms: Some(100.0),
            total_processing_time_ms: Some(500.0),
            prefill_token_speed: None,
            decode_token_speed: None,
            prompt_token_count: Some(150),
            output_token_count: Some(75),
            context_length: Some(150),
            token_efficiency: Some(0.333),
            recorded_at: Utc::now(),
        };
        
        let result = storage.store_ai_metrics(Uuid::new_v4(), &metrics);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_ttft_range() {
        let storage = AIMetricsStorage::new();
        let mut metrics = AIMetricsDbValues {
            request_id: "test".to_string(),
            ttft_ms: Some(-10.0), // Invalid: negative
            total_processing_time_ms: Some(500.0),
            prefill_token_speed: None,
            decode_token_speed: None,
            prompt_token_count: Some(150),
            output_token_count: Some(75),
            context_length: Some(150),
            token_efficiency: Some(0.333),
            recorded_at: Utc::now(),
        };
        
        let result = storage.store_ai_metrics(Uuid::new_v4(), &metrics);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_efficiency_range() {
        let storage = AIMetricsStorage::new();
        let mut metrics = AIMetricsDbValues {
            request_id: "test".to_string(),
            ttft_ms: Some(100.0),
            total_processing_time_ms: Some(500.0),
            prefill_token_speed: None,
            decode_token_speed: None,
            prompt_token_count: Some(150),
            output_token_count: Some(75),
            context_length: Some(150),
            token_efficiency: Some(1.5), // Invalid: > 1.0
            recorded_at: Utc::now(),
        };
        
        let result = storage.store_ai_metrics(Uuid::new_v4(), &metrics);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_builder() {
        let builder = AIMetricsQueryBuilder::new()
            .with_request_event_id(Uuid::new_v4())
            .with_min_ttft(50.0)
            .with_max_ttft(200.0)
            .with_limit(100);
        
        assert!(builder.request_event_id.is_some());
        assert_eq!(builder.min_ttft, Some(50.0));
        assert_eq!(builder.max_ttft, Some(200.0));
        assert_eq!(builder.limit, Some(100));
    }

    #[test]
    fn test_prepare_for_storage() {
        let storage = AIMetricsStorage::new();
        let request_event_id = Uuid::new_v4();
        
        let mut metrics = AIMetrics::new("test-request".to_string());
        metrics.ttft_ms = Some(100.0);
        metrics.prompt_token_count = Some(150);
        
        let db_values = storage.prepare_for_storage(request_event_id, metrics);
        assert!(db_values.is_ok());
    }
}
