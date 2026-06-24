//! Data cleanup jobs for time-series storage
//!
//! This module provides automated cleanup jobs for maintaining time-series
//! data storage performance and managing data lifecycle.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};

use super::retention::RetentionPolicyManager;

/// Cleanup job configuration
#[derive(Debug, Clone)]
pub struct CleanupJobConfig {
    /// Interval between cleanup runs
    pub cleanup_interval: Duration,
    /// Batch size for deletion operations
    pub batch_size: usize,
    /// Maximum runtime per cleanup cycle
    pub max_runtime: Duration,
    /// Enable automatic cleanup
    pub enabled: bool,
}

impl Default for CleanupJobConfig {
    fn default() -> Self {
        Self {
            cleanup_interval: Duration::hours(6),
            batch_size: 10000,
            max_runtime: Duration::minutes(30),
            enabled: true,
        }
    }
}

/// Cleanup job result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupJobResult {
    /// Job start time
    pub start_time: DateTime<Utc>,
    /// Job end time
    pub end_time: DateTime<Utc>,
    /// Duration of the cleanup job
    pub duration_secs: f64,
    /// Number of data points deleted
    pub data_points_deleted: i64,
    /// Number of data points aggregated
    pub data_points_aggregated: i64,
    /// Number of tables cleaned
    pub tables_cleaned: usize,
    /// Whether the job completed successfully
    pub success: bool,
    /// Error message if job failed
    pub error: Option<String>,
}

impl Default for CleanupJobResult {
    fn default() -> Self {
        Self {
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration_secs: 0.0,
            data_points_deleted: 0,
            data_points_aggregated: 0,
            tables_cleaned: 0,
            success: false,
            error: None,
        }
    }
}

/// Cleanup job manager
pub struct CleanupJobManager {
    pool: PgPool,
    config: CleanupJobConfig,
    retention_manager: Arc<Mutex<RetentionPolicyManager>>,
    is_running: Arc<Mutex<bool>>,
}

impl CleanupJobManager {
    /// Create a new cleanup job manager
    pub fn new(
        pool: PgPool,
        config: CleanupJobConfig,
        retention_manager: RetentionPolicyManager,
    ) -> Self {
        Self {
            pool,
            config,
            retention_manager: Arc::new(Mutex::new(retention_manager)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a new cleanup job manager with default configuration
    pub fn new_with_defaults(pool: PgPool, retention_manager: RetentionPolicyManager) -> Self {
        Self::new(pool, CleanupJobConfig::default(), retention_manager)
    }

    /// Start the automated cleanup job
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(()); // Already running
        }
        *is_running = true;
        drop(is_running);

        let pool = self.pool.clone();
        let config = self.config.clone();
        let retention_manager = self.retention_manager.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut timer = interval(config.cleanup_interval);
            timer.tick().await; // Skip first immediate tick

            loop {
                timer.tick().await;

                let should_run = {
                    let running = is_running.lock().await;
                    *running
                };

                if !should_run {
                    break;
                }

                if config.enabled {
                    let result = Self::run_cleanup_job(&pool, &retention_manager, &config).await;
                    match result {
                        Ok(job_result) => {
                            // Cleanup job completed: deleted {} points, aggregated {} points in {:.2}s
                            let _ = (job_result.data_points_deleted, job_result.data_points_aggregated, job_result.duration_secs);
                        }
                        Err(e) => {
                            // Cleanup job failed: {}
                            let _ = e;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the automated cleanup job
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        Ok(())
    }

    /// Run a single cleanup job manually
    pub async fn run_manual_cleanup(&self) -> Result<CleanupJobResult> {
        Self::run_cleanup_job(&self.pool, &self.retention_manager, &self.config).await
    }

    /// Run the cleanup job
    async fn run_cleanup_job(
        pool: &PgPool,
        retention_manager: &Arc<Mutex<RetentionPolicyManager>>,
        config: &CleanupJobConfig,
    ) -> Result<CleanupJobResult> {
        let start_time = Utc::now();
        let mut result = CleanupJobResult {
            start_time,
            ..Default::default()
        };

        // Enforce retention policies
        let retention_manager_guard = retention_manager.lock().await;
        let enforcement_result = retention_manager_guard.enforce_policies().await;
        drop(retention_manager_guard);

        match enforcement_result {
            Ok(enforcement) => {
                result.data_points_deleted = enforcement.data_points_deleted;
                result.data_points_aggregated = enforcement.data_points_aggregated;
                result.tables_cleaned = enforcement.metrics_processed;
            }
            Err(e) => {
                result.error = Some(e.to_string());
                result.end_time = Utc::now();
                result.duration_secs = (result.end_time - result.start_time).num_seconds() as f64;
                return Ok(result);
            }
        }

        // Clean up orphaned data
        let orphaned_cleanup = Self::cleanup_orphaned_data(pool, config).await;
        if let Ok(count) = orphaned_cleanup {
            result.data_points_deleted += count;
        }

        // Vacuum tables to reclaim space
        let vacuum_result = Self::vacuum_tables(pool).await;
        if vacuum_result.is_ok() {
            result.tables_cleaned += 1;
        }

        result.end_time = Utc::now();
        result.duration_secs = (result.end_time - result.start_time).num_seconds() as f64;
        result.success = true;

        Ok(result)
    }

    /// Clean up orphaned data (data without corresponding parent records)
    async fn cleanup_orphaned_data(pool: &PgPool, config: &CleanupJobConfig) -> Result<i64> {
        // Clean up time-series metrics without corresponding request events
        let result = sqlx::query(
            r#"
            DELETE FROM time_series_metrics
            WHERE request_event_id IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1 FROM request_events
                  WHERE request_events.id = time_series_metrics.request_event_id
              )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to cleanup orphaned time-series metrics")?;

        Ok(result.rows_affected())
    }

    /// Vacuum tables to reclaim disk space
    async fn vacuum_tables(pool: &PgPool) -> Result<()> {
        // Vacuum the time-series metrics table
        sqlx::query("VACUUM ANALYZE time_series_metrics")
            .execute(pool)
            .await
            .context("Failed to vacuum time_series_metrics table")?;

        // Vacuum the aggregates table
        sqlx::query("VACUUM ANALYZE time_series_aggregates")
            .execute(pool)
            .await
            .context("Failed to vacuum time_series_aggregates table")?;

        Ok(())
    }

    /// Get cleanup job statistics
    pub async fn get_cleanup_stats(&self) -> Result<CleanupStats> {
        let total_metrics: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM time_series_metrics
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get total metrics count")?;

        let total_aggregates: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM time_series_aggregates
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get total aggregates count")?;

        let table_size_metrics: i64 = sqlx::query_scalar(
            r#"
            SELECT pg_total_relation_size('time_series_metrics')
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get table size for metrics")?;

        let table_size_aggregates: i64 = sqlx::query_scalar(
            r#"
            SELECT pg_total_relation_size('time_series_aggregates')
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get table size for aggregates")?;

        Ok(CleanupStats {
            total_metrics: total_metrics as usize,
            total_aggregates: total_aggregates as usize,
            storage_size_bytes: (table_size_metrics + table_size_aggregates) as usize,
            is_running: *self.is_running.lock().await,
        })
    }

    /// Check if the cleanup job is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }
}

/// Cleanup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    /// Total number of metrics in storage
    pub total_metrics: usize,
    /// Total number of aggregates in storage
    pub total_aggregates: usize,
    /// Total storage size in bytes
    pub storage_size_bytes: usize,
    /// Whether the cleanup job is currently running
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_job_config_default() {
        let config = CleanupJobConfig::default();
        assert_eq!(config.cleanup_interval, Duration::hours(6));
        assert_eq!(config.batch_size, 10000);
        assert!(config.enabled);
    }

    #[test]
    fn test_cleanup_job_result_default() {
        let result = CleanupJobResult::default();
        assert!(!result.success);
        assert_eq!(result.data_points_deleted, 0);
    }

    #[test]
    fn test_cleanup_stats() {
        let stats = CleanupStats {
            total_metrics: 1000,
            total_aggregates: 100,
            storage_size_bytes: 1024000,
            is_running: false,
        };

        assert_eq!(stats.total_metrics, 1000);
        assert_eq!(stats.total_aggregates, 100);
        assert!(!stats.is_running);
    }

    #[test]
    fn test_cleanup_job_result_serialization() {
        let result = CleanupJobResult {
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration_secs: 10.5,
            data_points_deleted: 1000,
            data_points_aggregated: 500,
            tables_cleaned: 2,
            success: true,
            error: None,
        };

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: CleanupJobResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.data_points_deleted, result.data_points_deleted);
        assert!(deserialized.success);
    }

    #[test]
    fn test_cleanup_job_config_custom() {
        let config = CleanupJobConfig {
            cleanup_interval: Duration::hours(12),
            batch_size: 5000,
            max_runtime: Duration::minutes(15),
            enabled: false,
        };

        assert_eq!(config.cleanup_interval, Duration::hours(12));
        assert_eq!(config.batch_size, 5000);
        assert!(!config.enabled);
    }

    #[test]
    fn test_cleanup_stats_serialization() {
        let stats = CleanupStats {
            total_metrics: 1000,
            total_aggregates: 100,
            storage_size_bytes: 1024000,
            is_running: true,
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: CleanupStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.total_metrics, stats.total_metrics);
        assert!(deserialized.is_running);
    }

    #[test]
    fn test_cleanup_job_result_with_error() {
        let result = CleanupJobResult {
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration_secs: 5.0,
            data_points_deleted: 0,
            data_points_aggregated: 0,
            tables_cleaned: 0,
            success: false,
            error: Some("Database connection failed".to_string()),
        };

        assert!(!result.success);
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "Database connection failed");
    }
}
