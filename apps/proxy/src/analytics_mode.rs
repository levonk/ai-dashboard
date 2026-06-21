use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use analytics_rs::TelemetryEvent;
use crate::config::Config;

/// Analytics mode configuration
#[derive(Debug, Clone)]
pub struct AnalyticsModeConfig {
    pub database_url: String,
    pub pool_max_connections: u32,
    pub pool_min_connections: u32,
    pub connection_timeout: Duration,
    pub enable_batch_writes: bool,
    pub batch_size: usize,
    pub batch_timeout: Duration,
}

impl Default for AnalyticsModeConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            pool_max_connections: 10,
            pool_min_connections: 2,
            connection_timeout: Duration::from_secs(30),
            enable_batch_writes: true,
            batch_size: 100,
            batch_timeout: Duration::from_secs(5),
        }
    }
}

/// Analytics mode handler
pub struct AnalyticsMode {
    config: AnalyticsModeConfig,
    pool: Arc<PgPool>,
    event_buffer: Vec<TelemetryEvent>,
}

impl AnalyticsMode {
    /// Create new analytics mode handler
    pub async fn new(config: AnalyticsModeConfig) -> Result<Self> {
        info!("Connecting to analytics database...");

        let pool = PgPoolOptions::new()
            .max_connections(config.pool_max_connections)
            .min_connections(config.pool_min_connections)
            .acquire_timeout(config.connection_timeout)
            .connect(&config.database_url)
            .await
            .context("Failed to connect to database")?;

        info!("Successfully connected to analytics database");

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(Self {
            config,
            pool: Arc::new(pool),
            event_buffer: Vec::new(),
        })
    }

    /// Create from app config
    pub async fn from_config(app_config: &Config) -> Result<Self> {
        let database_url = app_config.database_url
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Database URL not configured"))?;

        let config = AnalyticsModeConfig {
            database_url,
            ..Default::default()
        };

        Self::new(config).await
    }

    /// Run database migrations
    async fn run_migrations(pool: &PgPool) -> Result<()> {
        info!("Running database migrations...");
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS telemetry_events (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                event_id TEXT NOT NULL UNIQUE,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                ai_client TEXT NOT NULL,
                ai_provider TEXT NOT NULL,
                model TEXT,
                input_type TEXT NOT NULL,
                input_tokens INTEGER,
                output_tokens INTEGER,
                duration_ms BIGINT,
                cost_usd DECIMAL(10, 4),
                metadata JSONB,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_telemetry_events_timestamp ON telemetry_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_telemetry_events_ai_client ON telemetry_events(ai_client);
            CREATE INDEX IF NOT EXISTS idx_telemetry_events_ai_provider ON telemetry_events(ai_provider);
            CREATE INDEX IF NOT EXISTS idx_telemetry_events_model ON telemetry_events(model);
            "#
        )
        .execute(pool)
        .await
        .context("Failed to create telemetry_events table")?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Write telemetry event to database
    pub async fn write_event(&mut self, event: TelemetryEvent) -> Result<()> {
        if self.config.enable_batch_writes {
            self.event_buffer.push(event);
            
            if self.event_buffer.len() >= self.config.batch_size {
                self.flush_batch().await?;
            }
        } else {
            self.write_single_event(event).await?;
        }

        Ok(())
    }

    /// Write single event to database
    async fn write_single_event(&self, event: TelemetryEvent) -> Result<()> {
        debug!("Writing telemetry event: {}", event.event_id);

        let metadata_json = serde_json::to_value(&event.metadata)
            .context("Failed to serialize metadata")?;

        sqlx::query(
            r#"
            INSERT INTO telemetry_events (
                event_id, timestamp, ai_client, ai_provider,
                model, input_type, input_tokens, output_tokens,
                duration_ms, cost_usd, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (event_id) DO UPDATE SET
                timestamp = EXCLUDED.timestamp,
                duration_ms = EXCLUDED.duration_ms,
                output_tokens = EXCLUDED.output_tokens,
                cost_usd = EXCLUDED.cost_usd,
                metadata = EXCLUDED.metadata
            "#
        )
        .bind(&event.event_id)
        .bind(event.timestamp)
        .bind(&event.ai_client)
        .bind(&event.ai_provider)
        .bind(&event.model)
        .bind(&event.input_type)
        .bind(event.input_tokens as i32)
        .bind(event.output_tokens as i32)
        .bind(event.duration_ms as i64)
        .bind(event.cost_usd)
        .bind(metadata_json)
        .execute(&*self.pool)
        .await
        .context("Failed to insert telemetry event")?;

        debug!("Successfully wrote telemetry event: {}", event.event_id);
        Ok(())
    }

    /// Flush batch of events to database
    pub async fn flush_batch(&mut self) -> Result<()> {
        if self.event_buffer.is_empty() {
            return Ok(());
        }

        info!("Flushing {} telemetry events to database", self.event_buffer.len());

        let mut tx = self.pool.begin()
            .await
            .context("Failed to begin transaction")?;

        for event in self.event_buffer.drain(..) {
            let metadata_json = serde_json::to_value(&event.metadata)
                .context("Failed to serialize metadata")?;

            sqlx::query(
                r#"
                INSERT INTO telemetry_events (
                    event_id, timestamp, ai_client, ai_provider,
                    model, input_type, input_tokens, output_tokens,
                    duration_ms, cost_usd, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (event_id) DO UPDATE SET
                    timestamp = EXCLUDED.timestamp,
                    duration_ms = EXCLUDED.duration_ms,
                    output_tokens = EXCLUDED.output_tokens,
                    cost_usd = EXCLUDED.cost_usd,
                    metadata = EXCLUDED.metadata
                "#
            )
            .bind(&event.event_id)
            .bind(event.timestamp)
            .bind(&event.ai_client)
            .bind(&event.ai_provider)
            .bind(&event.model)
            .bind(&event.input_type)
            .bind(event.input_tokens as i32)
            .bind(event.output_tokens as i32)
            .bind(event.duration_ms as i64)
            .bind(event.cost_usd)
            .bind(metadata_json)
            .execute(&mut *tx)
            .await
            .context("Failed to insert telemetry event in batch")?;
        }

        tx.commit().await
            .context("Failed to commit transaction")?;

        info!("Successfully flushed telemetry events to database");
        Ok(())
    }

    /// Force flush any remaining events
    pub async fn flush(&mut self) -> Result<()> {
        self.flush_batch().await
    }

    /// Get database pool reference
    pub fn pool(&self) -> Arc<PgPool> {
        self.pool.clone()
    }

    /// Health check for analytics mode
    pub async fn health_check(&self) -> Result<bool> {
        sqlx::query("SELECT 1")
            .fetch_one(&*self.pool)
            .await
            .map(|_| true)
            .map_err(|e| {
                error!("Analytics mode health check failed: {}", e);
                anyhow::anyhow!("Health check failed")
            })
    }

    /// Get statistics about buffered events
    pub fn buffer_stats(&self) -> (usize, usize) {
        (self.event_buffer.len(), self.config.batch_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_mode_config_default() {
        let config = AnalyticsModeConfig::default();
        assert_eq!(config.pool_max_connections, 10);
        assert_eq!(config.pool_min_connections, 2);
        assert!(config.enable_batch_writes);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_buffer_stats() {
        let config = AnalyticsModeConfig::default();
        
        // Note: This test only checks the config values, not actual pool functionality
        // In integration tests, use a real test database
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.pool_max_connections, 10);
        assert!(config.enable_batch_writes);
    }
}
