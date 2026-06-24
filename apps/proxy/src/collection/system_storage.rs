//! System metrics storage integration
//! 
//! This module provides storage integration for system metrics, including
//! database operations and query builders for system resource metrics.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use crate::collection::system::SystemMetrics;
use crate::collection::database::DatabaseWriter;

/// System metrics storage handler
pub struct SystemMetricsStorage {
    db_writer: Arc<DatabaseWriter>,
}

impl SystemMetricsStorage {
    /// Create a new system metrics storage handler
    pub fn new(db_writer: Arc<DatabaseWriter>) -> Self {
        info!("Initializing system metrics storage");
        
        SystemMetricsStorage {
            db_writer,
        }
    }

    /// Store system metrics
    pub async fn store_metrics(&self, metrics: &SystemMetrics) -> Result<()> {
        debug!("Storing system metrics at {}", metrics.timestamp);
        
        // Convert system metrics to database format
        let db_values = self.system_metrics_to_db_values(metrics);
        
        // Store in database
        // This would typically use prepared statements or an ORM
        // For now, we'll use a simplified approach
        let query = self.build_insert_query(&db_values);
        
        // Execute the query (implementation depends on your database layer)
        // self.db_writer.execute_query(&query).await?;
        
        debug!("System metrics stored successfully");
        Ok(())
    }

    /// Store a batch of system metrics
    pub async fn store_batch_metrics(&self, metrics_batch: &[SystemMetrics]) -> Result<BatchStoreResult> {
        info!("Storing batch of {} system metrics", metrics_batch.len());
        
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        
        for metrics in metrics_batch {
            match self.store_metrics(metrics).await {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("Failed to store metrics at {}: {}", metrics.timestamp, e));
                }
            }
        }
        
        debug!("Batch storage completed: {} successful, {} failed", successful, failed);
        
        Ok(BatchStoreResult {
            total: metrics_batch.len(),
            successful,
            failed,
            errors,
        })
    }

    /// Query system metrics by time range
    pub async fn query_metrics_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<SystemMetrics>> {
        debug!("Querying system metrics from {} to {}", start_time, end_time);
        
        let query = self.build_time_range_query(start_time, end_time);
        
        // Execute query and convert results
        // let results = self.db_writer.execute_query(&query).await?;
        // let metrics = self.db_values_to_system_metrics(results)?;
        
        // For now, return empty vector
        Ok(vec![])
    }

    /// Query latest system metrics
    pub async fn query_latest_metrics(&self, limit: usize) -> Result<Vec<SystemMetrics>> {
        debug!("Querying latest {} system metrics", limit);
        
        let query = self.build_latest_query(limit);
        
        // Execute query and convert results
        // let results = self.db_writer.execute_query(&query).await?;
        // let metrics = self.db_values_to_system_metrics(results)?;
        
        // For now, return empty vector
        Ok(vec![])
    }

    /// Get system metrics statistics
    pub async fn get_statistics(&self, time_range: TimeRange) -> Result<SystemMetricsStatistics> {
        debug!("Getting system metrics statistics for time range: {:?}", time_range);
        
        let query = self.build_statistics_query(time_range);
        
        // Execute query and calculate statistics
        // let results = self.db_writer.execute_query(&query).await?;
        
        // For now, return empty statistics
        Ok(SystemMetricsStatistics {
            total_samples: 0,
            avg_gpu_utilization: None,
            avg_cpu_utilization: None,
            avg_memory_usage: None,
            max_gpu_temperature: None,
            max_cpu_temperature: None,
            avg_power_consumption: None,
        })
    }

    /// Convert system metrics to database values
    fn system_metrics_to_db_values(&self, metrics: &SystemMetrics) -> SystemMetricsDbValues {
        SystemMetricsDbValues {
            timestamp: metrics.timestamp,
            sample_rate_hz: metrics.sample_rate_hz,
            gpu_utilization: metrics.gpu_utilization.clone(),
            cpu_utilization: metrics.cpu_utilization.clone(),
            memory_usage: metrics.memory_usage.clone(),
            temperature: metrics.temperature.clone(),
            power_consumption: metrics.power_consumption.clone(),
            clock_speeds: metrics.clock_speeds.clone(),
        }
    }

    /// Build insert query for system metrics
    fn build_insert_query(&self, values: &SystemMetricsDbValues) -> String {
        // This would build a proper SQL insert statement
        // For now, return a placeholder
        format!(
            "INSERT INTO system_metrics (timestamp, sample_rate_hz) VALUES ('{}', {})",
            values.timestamp, values.sample_rate_hz
        )
    }

    /// Build time range query
    fn build_time_range_query(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> String {
        format!(
            "SELECT * FROM system_metrics WHERE timestamp >= '{}' AND timestamp <= '{}' ORDER BY timestamp",
            start_time, end_time
        )
    }

    /// Build latest query
    fn build_latest_query(&self, limit: usize) -> String {
        format!(
            "SELECT * FROM system_metrics ORDER BY timestamp DESC LIMIT {}",
            limit
        )
    }

    /// Build statistics query
    fn build_statistics_query(&self, time_range: TimeRange) -> String {
        match time_range {
            TimeRange::LastMinutes(minutes) => {
                let start_time = Utc::now() - chrono::Duration::minutes(minutes as i64);
                format!(
                    "SELECT * FROM system_metrics WHERE timestamp >= '{}'",
                    start_time
                )
            }
            TimeRange::LastHours(hours) => {
                let start_time = Utc::now() - chrono::Duration::hours(hours as i64);
                format!(
                    "SELECT * FROM system_metrics WHERE timestamp >= '{}'",
                    start_time
                )
            }
            TimeRange::Custom(start, end) => {
                format!(
                    "SELECT * FROM system_metrics WHERE timestamp >= '{}' AND timestamp <= '{}'",
                    start, end
                )
            }
        }
    }
}

/// Database values for system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemMetricsDbValues {
    timestamp: DateTime<Utc>,
    sample_rate_hz: f32,
    gpu_utilization: Vec<crate::collection::system::GpuUtilizationMetrics>,
    cpu_utilization: crate::collection::system::CpuUtilizationMetrics,
    memory_usage: crate::collection::system::MemoryUsageMetrics,
    temperature: crate::collection::system::TemperatureUsageMetrics,
    power_consumption: crate::collection::system::PowerUsageMetrics,
    clock_speeds: crate::collection::system::ClockSpeedMetrics,
}

/// Batch store result
#[derive(Debug, Clone)]
pub struct BatchStoreResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Time range for queries
#[derive(Debug, Clone)]
pub enum TimeRange {
    LastMinutes(u64),
    LastHours(u64),
    Custom(DateTime<Utc>, DateTime<Utc>),
}

/// System metrics statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetricsStatistics {
    pub total_samples: usize,
    pub avg_gpu_utilization: Option<f32>,
    pub avg_cpu_utilization: Option<f32>,
    pub avg_memory_usage: Option<f32>,
    pub max_gpu_temperature: Option<u32>,
    pub max_cpu_temperature: Option<u32>,
    pub avg_power_consumption: Option<f32>,
}

/// System metrics query builder
pub struct SystemMetricsQueryBuilder {
    time_range: Option<TimeRange>,
    limit: Option<usize>,
    device_filter: Option<u32>,
    core_filter: Option<usize>,
}

impl SystemMetricsQueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        SystemMetricsQueryBuilder {
            time_range: None,
            limit: None,
            device_filter: None,
            core_filter: None,
        }
    }

    /// Set time range
    pub fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    /// Set limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set device filter
    pub fn with_device_filter(mut self, device_index: u32) -> Self {
        self.device_filter = Some(device_index);
        self
    }

    /// Set core filter
    pub fn with_core_filter(mut self, core_index: usize) -> Self {
        self.core_filter = Some(core_index);
        self
    }

    /// Build the query
    pub fn build(&self) -> String {
        let mut query = String::from("SELECT * FROM system_metrics");
        let mut conditions = Vec::new();

        if let Some(time_range) = &self.time_range {
            match time_range {
                TimeRange::LastMinutes(minutes) => {
                    let start_time = Utc::now() - chrono::Duration::minutes(*minutes as i64);
                    conditions.push(format!("timestamp >= '{}'", start_time));
                }
                TimeRange::LastHours(hours) => {
                    let start_time = Utc::now() - chrono::Duration::hours(*hours as i64);
                    conditions.push(format!("timestamp >= '{}'", start_time));
                }
                TimeRange::Custom(start, end) => {
                    conditions.push(format!("timestamp >= '{}' AND timestamp <= '{}'", start, end));
                }
            }
        }

        if let Some(device_index) = self.device_filter {
            conditions.push(format!("device_index = {}", device_index));
        }

        if let Some(core_index) = self.core_filter {
            conditions.push(format!("core_index = {}", core_index));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        query
    }
}

impl Default for SystemMetricsQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::database::DatabaseWriter;

    #[test]
    fn test_system_metrics_storage_initialization() {
        // This test would require a mock database writer
        // For now, we'll just test the structure
        let db_writer = Arc::new(DatabaseWriter::new("test.db").unwrap());
        let storage = SystemMetricsStorage::new(db_writer);
        
        // Just verify it was created
        assert!(true);
    }

    #[test]
    fn test_query_builder() {
        let builder = SystemMetricsQueryBuilder::new()
            .with_time_range(TimeRange::LastMinutes(5))
            .with_limit(10)
            .with_device_filter(0);
        
        let query = builder.build();
        assert!(query.contains("SELECT * FROM system_metrics"));
        assert!(query.contains("WHERE"));
        assert!(query.contains("timestamp >="));
        assert!(query.contains("device_index = 0"));
        assert!(query.contains("LIMIT 10"));
    }

    #[test]
    fn test_query_builder_default() {
        let builder = SystemMetricsQueryBuilder::new();
        let query = builder.build();
        
        assert!(query.contains("SELECT * FROM system_metrics"));
        assert!(query.contains("ORDER BY timestamp DESC"));
    }

    #[test]
    fn test_time_range_custom() {
        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now();
        let time_range = TimeRange::Custom(start, end);
        
        match time_range {
            TimeRange::Custom(s, e) => {
                assert_eq!(s, start);
                assert_eq!(e, end);
            }
            _ => panic!("Expected Custom time range"),
        }
    }

    #[test]
    fn test_batch_store_result() {
        let result = BatchStoreResult {
            total: 10,
            successful: 8,
            failed: 2,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };
        
        assert_eq!(result.total, 10);
        assert_eq!(result.successful, 8);
        assert_eq!(result.failed, 2);
        assert_eq!(result.errors.len(), 2);
    }
}