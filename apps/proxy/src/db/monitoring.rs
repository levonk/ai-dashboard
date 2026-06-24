//! Storage performance monitoring
//!
//! This module provides monitoring and metrics for time-series storage
//! performance to identify bottlenecks and track system health.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Storage performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    /// Write latency in milliseconds
    pub write_latency_ms: f64,
    /// Read latency in milliseconds
    pub read_latency_ms: f64,
    /// Write throughput (operations per second)
    pub write_throughput: f64,
    /// Read throughput (operations per second)
    pub read_throughput: f64,
    /// Current buffer size
    pub buffer_size: usize,
    /// Cache hit rate (percentage)
    pub cache_hit_rate: f64,
    /// Storage size in bytes
    pub storage_size_bytes: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Timestamp of metrics collection
    pub timestamp: DateTime<Utc>,
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self {
            write_latency_ms: 0.0,
            read_latency_ms: 0.0,
            write_throughput: 0.0,
            read_throughput: 0.0,
            buffer_size: 0,
            cache_hit_rate: 0.0,
            storage_size_bytes: 0,
            active_connections: 0,
            timestamp: Utc::now(),
        }
    }
}

/// Performance operation tracker
#[derive(Debug, Clone)]
pub struct OperationTracker {
    operation_type: String,
    start_time: Instant,
    end_time: Option<Instant>,
    success: bool,
    error_message: Option<String>,
}

impl OperationTracker {
    /// Start tracking a new operation
    pub fn start(operation_type: String) -> Self {
        Self {
            operation_type,
            start_time: Instant::now(),
            end_time: None,
            success: false,
            error_message: None,
        }
    }

    /// Mark the operation as successful
    pub fn success(mut self) -> Self {
        self.end_time = Some(Instant::now());
        self.success = true;
        self
    }

    /// Mark the operation as failed
    pub fn failure(mut self, error: String) -> Self {
        self.end_time = Some(Instant::now());
        self.success = false;
        self.error_message = Some(error);
        self
    }

    /// Get the operation duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        let end = self.end_time.unwrap_or_else(Instant::now);
        end.duration_since(self.start_time).as_secs_f64() * 1000.0
    }

    /// Get the operation type
    pub fn operation_type(&self) -> &str {
        &self.operation_type
    }

    /// Check if the operation was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// Storage performance monitor
pub struct StorageMonitor {
    pool: PgPool,
    metrics_history: Arc<RwLock<Vec<StorageMetrics>>>,
    operation_stats: Arc<RwLock<HashMap<String, OperationStats>>>,
    max_history_size: usize,
}

/// Operation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationStats {
    /// Total number of operations
    pub total_count: u64,
    /// Number of successful operations
    pub success_count: u64,
    /// Number of failed operations
    pub failure_count: u64,
    /// Total duration in milliseconds
    pub total_duration_ms: f64,
    /// Minimum duration in milliseconds
    pub min_duration_ms: f64,
    /// Maximum duration in milliseconds
    pub max_duration_ms: f64,
}

impl OperationStats {
    /// Record an operation
    pub fn record(&mut self, duration_ms: f64, success: bool) {
        self.total_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        self.total_duration_ms += duration_ms;
        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);
    }

    /// Get average duration in milliseconds
    pub fn avg_duration_ms(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.total_duration_ms / self.total_count as f64
        }
    }

    /// Get success rate (percentage)
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.total_count as f64) * 100.0
        }
    }
}

impl StorageMonitor {
    /// Create a new storage performance monitor
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            operation_stats: Arc::new(RwLock::new(HashMap::new())),
            max_history_size: 1000,
        }
    }

    /// Create a new storage performance monitor with custom history size
    pub fn with_history_size(pool: PgPool, max_history_size: usize) -> Self {
        Self {
            pool,
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            operation_stats: Arc::new(RwLock::new(HashMap::new())),
            max_history_size,
        }
    }

    /// Record an operation
    pub async fn record_operation(&self, tracker: OperationTracker) {
        let mut stats = self.operation_stats.write().await;
        let operation_stats = stats
            .entry(tracker.operation_type().to_string())
            .or_default();

        operation_stats.record(tracker.duration_ms(), tracker.is_success());
    }

    /// Collect current storage metrics
    pub async fn collect_metrics(&self) -> Result<StorageMetrics> {
        let storage_size = self.get_storage_size().await?;
        let active_connections = self.get_active_connections().await?;

        let operation_stats = self.operation_stats.read().await;
        let write_stats = operation_stats.get("write");
        let read_stats = operation_stats.get("read");

        let write_latency_ms = write_stats
            .map(|s| s.avg_duration_ms())
            .unwrap_or(0.0);
        let read_latency_ms = read_stats.map(|s| s.avg_duration_ms()).unwrap_or(0.0);

        let write_throughput = write_stats
            .map(|s| {
                if s.avg_duration_ms() > 0.0 {
                    1000.0 / s.avg_duration_ms()
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        let read_throughput = read_stats
            .map(|s| {
                if s.avg_duration_ms() > 0.0 {
                    1000.0 / s.avg_duration_ms()
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        let metrics = StorageMetrics {
            write_latency_ms,
            read_latency_ms,
            write_throughput,
            read_throughput,
            buffer_size: 0, // Would be populated from time-series storage
            cache_hit_rate: 0.0, // Would be populated from cache
            storage_size_bytes: storage_size,
            active_connections,
            timestamp: Utc::now(),
        };

        // Add to history
        let mut history = self.metrics_history.write().await;
        history.push(metrics.clone());
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        Ok(metrics)
    }

    /// Get storage size in bytes
    async fn get_storage_size(&self) -> Result<u64> {
        let metrics_size: i64 = sqlx::query_scalar(
            r#"
            SELECT pg_total_relation_size('time_series_metrics')
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get time_series_metrics table size")?;

        let aggregates_size: i64 = sqlx::query_scalar(
            r#"
            SELECT pg_total_relation_size('time_series_aggregates')
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get time_series_aggregates table size")?;

        Ok((metrics_size + aggregates_size) as u64)
    }

    /// Get number of active connections
    async fn get_active_connections(&self) -> Result<u32> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM pg_stat_activity
            WHERE datname = current_database()
              AND state = 'active'
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get active connection count")?;

        Ok(count as u32)
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self) -> Vec<StorageMetrics> {
        let history = self.metrics_history.read().await;
        history.clone()
    }

    /// Get operation statistics
    pub async fn get_operation_stats(&self) -> HashMap<String, OperationStats> {
        let stats = self.operation_stats.read().await;
        stats.clone()
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let current_metrics = self.collect_metrics().await.unwrap_or_default();
        let operation_stats = self.get_operation_stats().await;

        let write_stats = operation_stats.get("write");
        let read_stats = operation_stats.get("read");

        PerformanceSummary {
            current_metrics,
            write_success_rate: write_stats.map(|s| s.success_rate()).unwrap_or(0.0),
            read_success_rate: read_stats.map(|s| s.success_rate()).unwrap_or(0.0),
            total_operations: operation_stats.values().map(|s| s.total_count).sum(),
            total_failures: operation_stats.values().map(|s| s.failure_count).sum(),
        }
    }

    /// Identify performance bottlenecks
    pub async fn identify_bottlenecks(&self) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        let summary = self.get_performance_summary().await;

        // Check write latency
        if summary.current_metrics.write_latency_ms > 100.0 {
            bottlenecks.push(Bottleneck {
                severity: BottleneckSeverity::High,
                category: BottleneckCategory::WriteLatency,
                description: format!(
                    "Write latency is high: {:.2}ms",
                    summary.current_metrics.write_latency_ms
                ),
                recommendation: "Consider increasing batch size or optimizing write operations".to_string(),
            });
        }

        // Check read latency
        if summary.current_metrics.read_latency_ms > 2000.0 {
            bottlenecks.push(Bottleneck {
                severity: BottleneckSeverity::High,
                category: BottleneckCategory::ReadLatency,
                description: format!(
                    "Read latency is high: {:.2}ms",
                    summary.current_metrics.read_latency_ms
                ),
                recommendation: "Consider adding indexes or optimizing queries".to_string(),
            });
        }

        // Check write success rate
        if summary.write_success_rate < 95.0 {
            bottlenecks.push(Bottleneck {
                severity: BottleneckSeverity::Medium,
                category: BottleneckCategory::WriteFailures,
                description: format!(
                    "Write success rate is low: {:.2}%",
                    summary.write_success_rate
                ),
                recommendation: "Check database connectivity and retry logic".to_string(),
            });
        }

        // Check storage size
        if summary.current_metrics.storage_size_bytes > 10 * 1024 * 1024 * 1024 {
            bottlenecks.push(Bottleneck {
                severity: BottleneckSeverity::Medium,
                category: BottleneckCategory::StorageSize,
                description: format!(
                    "Storage size is large: {} GB",
                    summary.current_metrics.storage_size_bytes / (1024 * 1024 * 1024)
                ),
                recommendation: "Consider implementing data retention policies".to_string(),
            });
        }

        bottlenecks
    }

    /// Reset operation statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.operation_stats.write().await;
        stats.clear();
    }
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Current storage metrics
    pub current_metrics: StorageMetrics,
    /// Write operation success rate (percentage)
    pub write_success_rate: f64,
    /// Read operation success rate (percentage)
    pub read_success_rate: f64,
    /// Total number of operations
    pub total_operations: u64,
    /// Total number of failures
    pub total_failures: u64,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Severity level
    pub severity: BottleneckSeverity,
    /// Bottleneck category
    pub category: BottleneckCategory,
    /// Description of the bottleneck
    pub description: String,
    /// Recommendation to resolve the bottleneck
    pub recommendation: String,
}

/// Bottleneck severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Bottleneck category
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BottleneckCategory {
    WriteLatency,
    ReadLatency,
    WriteFailures,
    ReadFailures,
    StorageSize,
    CacheHitRate,
    ConnectionPool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_tracker() {
        let tracker = OperationTracker::start("test_operation".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let tracker = tracker.success();

        assert!(tracker.is_success());
        assert!(tracker.duration_ms() >= 10.0);
    }

    #[test]
    fn test_operation_stats() {
        let mut stats = OperationStats::default();
        stats.record(100.0, true);
        stats.record(200.0, true);
        stats.record(50.0, false);

        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.avg_duration_ms(), 350.0 / 3.0);
        assert_eq!(stats.success_rate(), 200.0 / 3.0 * 100.0);
    }

    #[test]
    fn test_storage_metrics_default() {
        let metrics = StorageMetrics::default();
        assert_eq!(metrics.write_latency_ms, 0.0);
        assert_eq!(metrics.read_latency_ms, 0.0);
    }

    #[test]
    fn test_operation_tracker_failure() {
        let tracker = OperationTracker::start("test_operation".to_string());
        let tracker = tracker.failure("Test error".to_string());

        assert!(!tracker.is_success());
        assert!(tracker.error_message.is_some());
        assert_eq!(tracker.error_message.unwrap(), "Test error");
    }

    #[test]
    fn test_operation_stats_serialization() {
        let stats = OperationStats {
            total_count: 100,
            success_count: 95,
            failure_count: 5,
            total_duration_ms: 10000.0,
            min_duration_ms: 50.0,
            max_duration_ms: 500.0,
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: OperationStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.total_count, stats.total_count);
        assert_eq!(deserialized.success_rate(), stats.success_rate());
    }

    #[test]
    fn test_storage_metrics_serialization() {
        let metrics = StorageMetrics {
            write_latency_ms: 100.0,
            read_latency_ms: 50.0,
            write_throughput: 1000.0,
            read_throughput: 2000.0,
            buffer_size: 100,
            cache_hit_rate: 85.0,
            storage_size_bytes: 1024000,
            active_connections: 5,
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: StorageMetrics = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.write_latency_ms, metrics.write_latency_ms);
        assert_eq!(deserialized.cache_hit_rate, metrics.cache_hit_rate);
    }

    #[test]
    fn test_performance_summary() {
        let summary = PerformanceSummary {
            current_metrics: StorageMetrics::default(),
            write_success_rate: 95.0,
            read_success_rate: 98.0,
            total_operations: 1000,
            total_failures: 20,
        };

        assert_eq!(summary.write_success_rate, 95.0);
        assert_eq!(summary.total_operations, 1000);
    }

    #[test]
    fn test_bottleneck_creation() {
        let bottleneck = Bottleneck {
            severity: BottleneckSeverity::High,
            category: BottleneckCategory::WriteLatency,
            description: "High write latency detected".to_string(),
            recommendation: "Optimize write operations".to_string(),
        };

        assert_eq!(bottleneck.severity, BottleneckSeverity::High);
        assert_eq!(bottleneck.category, BottleneckCategory::WriteLatency);
    }

    #[test]
    fn test_bottleneck_serialization() {
        let bottleneck = Bottleneck {
            severity: BottleneckSeverity::Medium,
            category: BottleneckCategory::StorageSize,
            description: "Storage size is large".to_string(),
            recommendation: "Implement retention policies".to_string(),
        };

        let serialized = serde_json::to_string(&bottleneck).unwrap();
        let deserialized: Bottleneck = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.severity, bottleneck.severity);
        assert_eq!(deserialized.category, bottleneck.category);
    }

    #[test]
    fn test_operation_stats_empty() {
        let stats = OperationStats::default();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.avg_duration_ms(), 0.0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_operation_tracker_duration() {
        let tracker = OperationTracker::start("quick_operation".to_string());
        let tracker = tracker.success();

        // Quick operation should have very short duration
        assert!(tracker.duration_ms() < 100.0);
    }

    #[test]
    fn test_bottleneck_severity_variants() {
        let severities = vec![
            BottleneckSeverity::Low,
            BottleneckSeverity::Medium,
            BottleneckSeverity::High,
            BottleneckSeverity::Critical,
        ];

        assert_eq!(severities.len(), 4);
    }

    #[test]
    fn test_bottleneck_category_variants() {
        let categories = vec![
            BottleneckCategory::WriteLatency,
            BottleneckCategory::ReadLatency,
            BottleneckCategory::WriteFailures,
            BottleneckCategory::ReadFailures,
            BottleneckCategory::StorageSize,
            BottleneckCategory::CacheHitRate,
            BottleneckCategory::ConnectionPool,
        ];

        assert_eq!(categories.len(), 7);
    }
}
