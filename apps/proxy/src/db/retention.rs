//! Data retention policy enforcement
//!
//! This module provides data retention policy management and enforcement
//! for time-series metrics data.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// Retention policy for a specific metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Metric name pattern (supports wildcards)
    pub metric_pattern: String,
    /// Retention duration
    pub retention_duration: Duration,
    /// Maximum data points to retain (optional)
    pub max_data_points: Option<usize>,
    /// Aggregation policy for old data
    pub aggregation_policy: AggregationPolicy,
    /// Whether the policy is enabled
    pub enabled: bool,
}

/// Aggregation policy for old data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggregationPolicy {
    /// No aggregation, delete old data
    None,
    /// Aggregate to hourly averages
    Hourly,
    /// Aggregate to daily averages
    Daily,
    /// Aggregate to weekly averages
    Weekly,
    /// Aggregate to monthly averages
    Monthly,
}

impl Default for AggregationPolicy {
    fn default() -> Self {
        Self::None
    }
}

/// Retention policy manager
pub struct RetentionPolicyManager {
    pool: PgPool,
    policies: HashMap<String, RetentionPolicy>,
}

impl RetentionPolicyManager {
    /// Create a new retention policy manager
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            policies: HashMap::new(),
        }
    }

    /// Add a retention policy
    pub fn add_policy(&mut self, policy: RetentionPolicy) -> Result<()> {
        if !policy.enabled {
            return Ok(());
        }

        self.policies.insert(policy.metric_pattern.clone(), policy);
        Ok(())
    }

    /// Remove a retention policy
    pub fn remove_policy(&mut self, metric_pattern: &str) {
        self.policies.remove(metric_pattern);
    }

    /// Get retention policy for a specific metric
    pub fn get_policy_for_metric(&self, metric_name: &str) -> Option<&RetentionPolicy> {
        // Find the first matching policy
        for (pattern, policy) in &self.policies {
            if self.pattern_matches(pattern, metric_name) {
                return Some(policy);
            }
        }
        None
    }

    /// Check if a pattern matches a metric name
    fn pattern_matches(&self, pattern: &str, metric_name: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return metric_name.starts_with(prefix) && metric_name.ends_with(suffix);
            }
        }

        pattern == metric_name
    }

    /// Enforce retention policies on time-series data
    pub async fn enforce_policies(&self) -> Result<RetentionEnforcementResult> {
        let mut result = RetentionEnforcementResult::default();

        for (pattern, policy) in &self.policies {
            if !policy.enabled {
                continue;
            }

            let cutoff_time = Utc::now() - policy.retention_duration;

            // Delete old data based on pattern
            let deleted_count = if pattern.contains('*') {
                self.delete_by_pattern(pattern, cutoff_time).await?
            } else {
                self.delete_by_metric(pattern, cutoff_time).await?
            };

            result.metrics_processed += 1;
            result.data_points_deleted += deleted_count;

            // Apply aggregation policy if configured
            if policy.aggregation_policy != AggregationPolicy::None {
                let aggregated_count = self.apply_aggregation(pattern, policy.aggregation_policy).await?;
                result.data_points_aggregated += aggregated_count;
            }
        }

        Ok(result)
    }

    /// Delete data for a specific metric older than cutoff time
    async fn delete_by_metric(&self, metric_name: &str, cutoff_time: DateTime<Utc>) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM time_series_metrics
            WHERE metric_name = $1
              AND timestamp < $2
            "#,
        )
        .bind(metric_name)
        .bind(cutoff_time)
        .execute(&self.pool)
        .await
        .context("Failed to delete old time-series data")?;

        Ok(result.rows_affected())
    }

    /// Delete data matching a pattern older than cutoff time
    async fn delete_by_pattern(&self, pattern: &str, cutoff_time: DateTime<Utc>) -> Result<i64> {
        let like_pattern = pattern.replace('*', '%');

        let result = sqlx::query(
            r#"
            DELETE FROM time_series_metrics
            WHERE metric_name LIKE $1
              AND timestamp < $2
            "#,
        )
        .bind(&like_pattern)
        .bind(cutoff_time)
        .execute(&self.pool)
        .await
        .context("Failed to delete old time-series data by pattern")?;

        Ok(result.rows_affected())
    }

    /// Apply aggregation policy to old data
    async fn apply_aggregation(&self, pattern: &str, policy: AggregationPolicy) -> Result<i64> {
        let window_size = match policy {
            AggregationPolicy::Hourly => "1 hour",
            AggregationPolicy::Daily => "1 day",
            AggregationPolicy::Weekly => "1 week",
            AggregationPolicy::Monthly => "1 month",
            AggregationPolicy::None => return Ok(0),
        };

        let like_pattern = pattern.replace('*', '%');

        // Aggregate old data and insert into aggregated table
        let result = sqlx::query(
            r#"
            INSERT INTO time_series_aggregates (metric_name, window_start, window_end, avg_value, min_value, max_value, count)
            SELECT 
                metric_name,
                date_trunc($1::text, timestamp) as window_start,
                date_trunc($1::text, timestamp) + ($1::interval) as window_end,
                AVG(value) as avg_value,
                MIN(value) as min_value,
                MAX(value) as max_value,
                COUNT(*) as count
            FROM time_series_metrics
            WHERE metric_name LIKE $2
              AND timestamp < NOW() - INTERVAL '7 days'
            GROUP BY metric_name, window_start
            ON CONFLICT (metric_name, window_start) DO UPDATE SET
                avg_value = EXCLUDED.avg_value,
                min_value = EXCLUDED.min_value,
                max_value = EXCLUDED.max_value,
                count = EXCLUDED.count
            "#,
        )
        .bind(window_size)
        .bind(&like_pattern)
        .execute(&self.pool)
        .await
        .context("Failed to aggregate time-series data")?;

        Ok(result.rows_affected())
    }

    /// Get retention statistics for a metric
    pub async fn get_retention_stats(&self, metric_name: &str) -> Result<RetentionStats> {
        let policy = self.get_policy_for_metric(metric_name);

        let total_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM time_series_metrics
            WHERE metric_name = $1
            "#,
        )
        .bind(metric_name)
        .fetch_one(&self.pool)
        .await
        .context("Failed to get retention stats")?;

        let oldest_timestamp: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            SELECT MIN(timestamp)
            FROM time_series_metrics
            WHERE metric_name = $1
            "#,
        )
        .bind(metric_name)
        .fetch_one(&self.pool)
        .await
        .context("Failed to get oldest timestamp")?;

        let newest_timestamp: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            SELECT MAX(timestamp)
            FROM time_series_metrics
            WHERE metric_name = $1
            "#,
        )
        .bind(metric_name)
        .fetch_one(&self.pool)
        .await
        .context("Failed to get newest timestamp")?;

        Ok(RetentionStats {
            metric_name: metric_name.to_string(),
            total_data_points: total_count as usize,
            oldest_timestamp,
            newest_timestamp,
            retention_policy: policy.cloned(),
        })
    }

    /// Get all retention policies
    pub fn policies(&self) -> &HashMap<String, RetentionPolicy> {
        &self.policies
    }
}

/// Retention enforcement result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetentionEnforcementResult {
    /// Number of metrics processed
    pub metrics_processed: usize,
    /// Number of data points deleted
    pub data_points_deleted: i64,
    /// Number of data points aggregated
    pub data_points_aggregated: i64,
}

/// Retention statistics for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionStats {
    /// Metric name
    pub metric_name: String,
    /// Total number of data points
    pub total_data_points: usize,
    /// Oldest timestamp in the dataset
    pub oldest_timestamp: Option<DateTime<Utc>>,
    /// Newest timestamp in the dataset
    pub newest_timestamp: Option<DateTime<Utc>>,
    /// Retention policy applied to this metric
    pub retention_policy: Option<RetentionPolicy>,
}

/// Default retention policies for common metric types
pub fn default_retention_policies() -> Vec<RetentionPolicy> {
    vec![
        RetentionPolicy {
            metric_pattern: "cpu_usage".to_string(),
            retention_duration: Duration::days(30),
            max_data_points: Some(100000),
            aggregation_policy: AggregationPolicy::Hourly,
            enabled: true,
        },
        RetentionPolicy {
            metric_pattern: "memory_usage".to_string(),
            retention_duration: Duration::days(30),
            max_data_points: Some(100000),
            aggregation_policy: AggregationPolicy::Hourly,
            enabled: true,
        },
        RetentionPolicy {
            metric_pattern: "token_throughput".to_string(),
            retention_duration: Duration::days(90),
            max_data_points: Some(500000),
            aggregation_policy: AggregationPolicy::Daily,
            enabled: true,
        },
        RetentionPolicy {
            metric_pattern: "*".to_string(),
            retention_duration: Duration::days(7),
            max_data_points: None,
            aggregation_policy: AggregationPolicy::None,
            enabled: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_policy_creation() {
        let policy = RetentionPolicy {
            metric_pattern: "cpu_usage".to_string(),
            retention_duration: Duration::days(30),
            max_data_points: Some(100000),
            aggregation_policy: AggregationPolicy::Hourly,
            enabled: true,
        };

        assert_eq!(policy.metric_pattern, "cpu_usage");
        assert!(policy.enabled);
    }

    #[test]
    fn test_pattern_matching() {
        let manager = RetentionPolicyManager::new(
            // Mock pool - not used in pattern matching test
            sqlx::postgres::PgPool::connect_lazy("postgresql://localhost:5432/test")
                .unwrap()
        );

        assert!(manager.pattern_matches("*", "any_metric"));
        assert!(manager.pattern_matches("cpu_*", "cpu_usage"));
        assert!(manager.pattern_matches("cpu_*", "cpu_temp"));
        assert!(!manager.pattern_matches("cpu_*", "memory_usage"));
        assert!(manager.pattern_matches("exact_match", "exact_match"));
    }

    #[test]
    fn test_default_retention_policies() {
        let policies = default_retention_policies();
        assert!(!policies.is_empty());
        assert!(policies.iter().any(|p| p.metric_pattern == "cpu_usage"));
    }

    #[test]
    fn test_retention_enforcement_result() {
        let result = RetentionEnforcementResult::default();
        assert_eq!(result.metrics_processed, 0);
        assert_eq!(result.data_points_deleted, 0);
    }

    #[test]
    fn test_retention_policy_serialization() {
        let policy = RetentionPolicy {
            metric_pattern: "test_*".to_string(),
            retention_duration: Duration::days(7),
            max_data_points: Some(50000),
            aggregation_policy: AggregationPolicy::Daily,
            enabled: true,
        };

        let serialized = serde_json::to_string(&policy).unwrap();
        let deserialized: RetentionPolicy = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.metric_pattern, policy.metric_pattern);
        assert_eq!(deserialized.retention_duration, policy.retention_duration);
    }

    #[test]
    fn test_aggregation_policy_variants() {
        assert_eq!(AggregationPolicy::None as i32, 0);
        assert_eq!(AggregationPolicy::Hourly as i32, 1);
        assert_eq!(AggregationPolicy::Daily as i32, 2);
    }

    #[test]
    fn test_retention_stats() {
        let stats = RetentionStats {
            metric_name: "cpu_usage".to_string(),
            total_data_points: 1000,
            oldest_timestamp: Some(Utc::now() - Duration::days(30)),
            newest_timestamp: Some(Utc::now()),
            retention_policy: None,
        };

        assert_eq!(stats.metric_name, "cpu_usage");
        assert_eq!(stats.total_data_points, 1000);
        assert!(stats.oldest_timestamp.is_some());
    }

    #[test]
    fn test_retention_policy_disabled() {
        let policy = RetentionPolicy {
            metric_pattern: "test_metric".to_string(),
            retention_duration: Duration::days(30),
            max_data_points: None,
            aggregation_policy: AggregationPolicy::None,
            enabled: false,
        };

        assert!(!policy.enabled);
    }

    #[test]
    fn test_retention_policy_manager_add_remove() {
        let manager = RetentionPolicyManager::new(
            sqlx::postgres::PgPool::connect_lazy("postgresql://localhost:5432/test")
                .unwrap()
        );

        let policy = RetentionPolicy {
            metric_pattern: "test_*".to_string(),
            retention_duration: Duration::days(7),
            max_data_points: None,
            aggregation_policy: AggregationPolicy::None,
            enabled: true,
        };

        let result = manager.add_policy(policy.clone());
        assert!(result.is_ok());

        assert!(manager.get_policy_for_metric("test_metric").is_some());

        manager.remove_policy("test_*");
        assert!(manager.get_policy_for_metric("test_metric").is_none());
    }
}
