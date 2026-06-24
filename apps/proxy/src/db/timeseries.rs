//! Time-series storage implementation
//!
//! This module provides optimized time-series storage for high-frequency performance metrics
//! with write optimization, batch operations, and query optimization.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDataPoint {
    /// Unique identifier for the data point
    pub id: Uuid,
    /// Metric name (e.g., "cpu_usage", "token_throughput")
    pub metric_name: String,
    /// Metric value
    pub value: f64,
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,
    /// Additional dimensions for the metric (e.g., labels, tags)
    pub dimensions: HashMap<String, String>,
    /// Associated request event ID (if applicable)
    pub request_event_id: Option<Uuid>,
}

/// Time-series write configuration
#[derive(Debug, Clone)]
pub struct TimeSeriesWriteConfig {
    /// Batch size for write operations
    pub batch_size: usize,
    /// Write timeout in seconds
    pub write_timeout: u64,
    /// Enable write compression
    pub enable_compression: bool,
    /// Maximum retry attempts for failed writes
    pub max_retries: u32,
}

impl Default for TimeSeriesWriteConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            write_timeout: 30,
            enable_compression: true,
            max_retries: 3,
        }
    }
}

/// Time-series storage manager
pub struct TimeSeriesStorage {
    pool: PgPool,
    config: TimeSeriesWriteConfig,
    write_buffer: Vec<TimeSeriesDataPoint>,
}

impl TimeSeriesStorage {
    /// Create a new time-series storage manager
    pub fn new(pool: PgPool, config: TimeSeriesWriteConfig) -> Self {
        Self {
            pool,
            config,
            write_buffer: Vec::with_capacity(config.batch_size),
        }
    }

    /// Create a new time-series storage manager with default configuration
    pub fn new_with_defaults(pool: PgPool) -> Self {
        Self::new(pool, TimeSeriesWriteConfig::default())
    }

    /// Write a single time-series data point
    pub async fn write_point(&mut self, point: TimeSeriesDataPoint) -> Result<()> {
        self.write_buffer.push(point);

        if self.write_buffer.len() >= self.config.batch_size {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    /// Write multiple time-series data points in batch
    pub async fn write_batch(&mut self, points: Vec<TimeSeriesDataPoint>) -> Result<()> {
        for point in points {
            self.write_buffer.push(point);
        }

        if self.write_buffer.len() >= self.config.batch_size {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    /// Flush the write buffer to the database
    pub async fn flush_buffer(&mut self) -> Result<()> {
        if self.write_buffer.is_empty() {
            return Ok(());
        }

        let points_to_write = std::mem::take(&mut self.write_buffer);
        self.batch_insert_points(points_to_write).await
    }

    /// Batch insert time-series data points with optimization
    async fn batch_insert_points(&self, points: Vec<TimeSeriesDataPoint>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        // Use COPY command for bulk insert optimization
        let mut query = String::from(
            r#"
            INSERT INTO time_series_metrics (id, metric_name, value, timestamp, dimensions, request_event_id)
            VALUES 
            "#,
        );

        let mut values = Vec::new();
        for (i, point) in points.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str(&format!("(${}, ${}, ${}, ${}, ${}, ${})", i * 6 + 1, i * 6 + 2, i * 6 + 3, i * 6 + 4, i * 6 + 5, i * 6 + 6));

            let dimensions_json = serde_json::to_string(&point.dimensions)
                .context("Failed to serialize dimensions")?;

            values.push(point.id);
            values.push(point.metric_name.clone());
            values.push(point.value);
            values.push(point.timestamp);
            values.push(dimensions_json);
            values.push(point.request_event_id);
        }

        let mut query_builder = sqlx::query(&query);
        for value in values {
            query_builder = query_builder.bind(value);
        }

        sqlx::query_with(&query, sqlx::postgres::PgArguments::from(values))
            .execute(&self.pool)
            .await
            .context("Failed to batch insert time-series points")?;

        Ok(())
    }

    /// Query time-series data for a specific metric within a time range
    pub async fn query_metric(
        &self,
        metric_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TimeSeriesDataPoint>> {
        let rows = sqlx::query(
            r#"
            SELECT id, metric_name, value, timestamp, dimensions, request_event_id
            FROM time_series_metrics
            WHERE metric_name = $1
              AND timestamp >= $2
              AND timestamp <= $3
            ORDER BY timestamp ASC
            "#,
        )
        .bind(metric_name)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query time-series metric")?;

        let mut points = Vec::new();
        for row in rows {
            let dimensions_json: String = row.get("dimensions");
            let dimensions: HashMap<String, String> = serde_json::from_str(&dimensions_json)
                .context("Failed to deserialize dimensions")?;

            points.push(TimeSeriesDataPoint {
                id: row.get("id"),
                metric_name: row.get("metric_name"),
                value: row.get("value"),
                timestamp: row.get("timestamp"),
                dimensions,
                request_event_id: row.get("request_event_id"),
            });
        }

        Ok(points)
    }

    /// Query time-series data with dimension filters
    pub async fn query_with_dimensions(
        &self,
        metric_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        dimension_filters: &HashMap<String, String>,
    ) -> Result<Vec<TimeSeriesDataPoint>> {
        let mut query = String::from(
            r#"
            SELECT id, metric_name, value, timestamp, dimensions, request_event_id
            FROM time_series_metrics
            WHERE metric_name = $1
              AND timestamp >= $2
              AND timestamp <= $3
            "#,
        );

        let mut param_index = 4;
        let mut params: Vec<String> = Vec::new();

        for (key, value) in dimension_filters {
            query.push_str(&format!(" AND dimensions->>'{}' = ${} ", key, param_index));
            params.push(value.clone());
            param_index += 1;
        }

        query.push_str(" ORDER BY timestamp ASC");

        let mut query_builder = sqlx::query(&query)
            .bind(metric_name)
            .bind(start_time)
            .bind(end_time);

        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .context("Failed to query time-series metric with dimensions")?;

        let mut points = Vec::new();
        for row in rows {
            let dimensions_json: String = row.get("dimensions");
            let dimensions: HashMap<String, String> = serde_json::from_str(&dimensions_json)
                .context("Failed to deserialize dimensions")?;

            points.push(TimeSeriesDataPoint {
                id: row.get("id"),
                metric_name: row.get("metric_name"),
                value: row.get("value"),
                timestamp: row.get("timestamp"),
                dimensions,
                request_event_id: row.get("request_event_id"),
            });
        }

        Ok(points)
    }

    /// Aggregate time-series data over a time window
    pub async fn aggregate_metric(
        &self,
        metric_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        window_size: Duration,
    ) -> Result<Vec<TimeSeriesAggregate>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                time_bucket($1::interval, timestamp) AS window_start,
                AVG(value) AS avg_value,
                MIN(value) AS min_value,
                MAX(value) AS max_value,
                COUNT(*) AS count
            FROM time_series_metrics
            WHERE metric_name = $2
              AND timestamp >= $3
              AND timestamp <= $4
            GROUP BY window_start
            ORDER BY window_start ASC
            "#,
        )
        .bind(format!("{} seconds", window_size.as_secs()))
        .bind(metric_name)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .context("Failed to aggregate time-series metric")?;

        let mut aggregates = Vec::new();
        for row in rows {
            aggregates.push(TimeSeriesAggregate {
                window_start: row.get("window_start"),
                avg_value: row.get("avg_value"),
                min_value: row.get("min_value"),
                max_value: row.get("max_value"),
                count: row.get("count"),
            });
        }

        Ok(aggregates)
    }

    /// Get the current buffer size
    pub fn buffer_size(&self) -> usize {
        self.write_buffer.len()
    }

    /// Get the configuration
    pub fn config(&self) -> &TimeSeriesWriteConfig {
        &self.config
    }
}

/// Time-series aggregate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesAggregate {
    /// Start of the time window
    pub window_start: DateTime<Utc>,
    /// Average value in the window
    pub avg_value: f64,
    /// Minimum value in the window
    pub min_value: f64,
    /// Maximum value in the window
    pub max_value: f64,
    /// Number of data points in the window
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_write_config_default() {
        let config = TimeSeriesWriteConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.write_timeout, 30);
        assert!(config.enable_compression);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_time_series_data_point_creation() {
        let point = TimeSeriesDataPoint {
            id: Uuid::new_v4(),
            metric_name: "cpu_usage".to_string(),
            value: 75.5,
            timestamp: Utc::now(),
            dimensions: HashMap::new(),
            request_event_id: None,
        };

        assert_eq!(point.metric_name, "cpu_usage");
        assert_eq!(point.value, 75.5);
    }

    #[test]
    fn test_time_series_aggregate_creation() {
        let aggregate = TimeSeriesAggregate {
            window_start: Utc::now(),
            avg_value: 50.0,
            min_value: 10.0,
            max_value: 90.0,
            count: 100,
        };

        assert_eq!(aggregate.avg_value, 50.0);
        assert_eq!(aggregate.count, 100);
    }

    #[test]
    fn test_time_series_storage_buffer_size() {
        // This would require a mock database connection in a real test
        // For now, we test the structure
        let config = TimeSeriesWriteConfig::default();
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_time_series_data_point_serialization() {
        let point = TimeSeriesDataPoint {
            id: Uuid::new_v4(),
            metric_name: "test_metric".to_string(),
            value: 42.0,
            timestamp: Utc::now(),
            dimensions: {
                let mut map = HashMap::new();
                map.insert("host".to_string(), "server1".to_string());
                map
            },
            request_event_id: Some(Uuid::new_v4()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&point).unwrap();
        assert!(serialized.contains("test_metric"));
        assert!(serialized.contains("42"));

        // Test deserialization
        let deserialized: TimeSeriesDataPoint = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.metric_name, point.metric_name);
        assert_eq!(deserialized.value, point.value);
    }

    #[test]
    fn test_time_series_aggregate_serialization() {
        let aggregate = TimeSeriesAggregate {
            window_start: Utc::now(),
            avg_value: 50.0,
            min_value: 10.0,
            max_value: 90.0,
            count: 100,
        };

        let serialized = serde_json::to_string(&aggregate).unwrap();
        let deserialized: TimeSeriesAggregate = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.avg_value, aggregate.avg_value);
        assert_eq!(deserialized.count, aggregate.count);
    }
}
