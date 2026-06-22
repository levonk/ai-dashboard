//! Message queue integration for event streaming
//! 
//! This module handles integration with message queues (Redis, Kafka) for scalable
//! event streaming and buffering. Currently supports Redis with pub/sub and streams.

use anyhow::{Context, Result};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::protocol::EventMessage;

/// Message queue backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueBackend {
    /// Redis pub/sub for real-time streaming
    RedisPubSub,
    /// Redis streams for durable event logs
    RedisStream,
    /// Kafka (future implementation)
    Kafka,
}

impl std::str::FromStr for QueueBackend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "redis_pubsub" | "redis-pubsub" | "pubsub" => Ok(Self::RedisPubSub),
            "redis_stream" | "redis-stream" | "stream" => Ok(Self::RedisStream),
            "kafka" => Ok(Self::Kafka),
            _ => anyhow::bail!("Invalid queue backend: {}. Must be 'redis_pubsub', 'redis_stream', or 'kafka'", s),
        }
    }
}

/// Message queue configuration
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Queue backend type
    pub backend: QueueBackend,
    /// Redis connection URL
    pub redis_url: String,
    /// Queue/stream name
    pub queue_name: String,
    /// Consumer group name (for streams)
    pub consumer_group: Option<String>,
    /// Consumer name (for streams)
    pub consumer_name: Option<String>,
    /// Maximum stream length (for streams)
    pub stream_max_len: Option<usize>,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Maximum pool size
    pub max_pool_size: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            backend: QueueBackend::RedisStream,
            redis_url: String::new(),
            queue_name: "ai-analytics-events".to_string(),
            consumer_group: Some("analytics-consumers".to_string()),
            consumer_name: Some("proxy-emitter".to_string()),
            stream_max_len: Some(10000),
            timeout_secs: 30,
            enable_pooling: true,
            max_pool_size: 10,
        }
    }
}

/// Queue statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// Backend type
    pub backend: String,
    /// Queue/stream name
    pub queue_name: String,
    /// Current queue depth
    pub queue_depth: usize,
    /// Connected status
    pub connected: bool,
    /// Total messages published
    pub total_published: u64,
    /// Total publish errors
    pub total_errors: u64,
    /// Last publish timestamp
    pub last_publish_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Message queue client for event streaming
pub struct MessageQueue {
    config: QueueConfig,
    redis_client: Option<Client>,
    stats: Arc<RwLock<QueueStats>>,
}

impl MessageQueue {
    /// Create a new message queue client
    pub fn new(config: QueueConfig) -> Result<Self> {
        if config.redis_url.is_empty() {
            anyhow::bail!("Redis URL cannot be empty");
        }

        let redis_client = Client::open(config.redis_url.as_str())
            .context("Failed to create Redis client")?;

        let stats = QueueStats {
            backend: format!("{:?}", config.backend),
            queue_name: config.queue_name.clone(),
            queue_depth: 0,
            connected: false,
            total_published: 0,
            total_errors: 0,
            last_publish_at: None,
        };

        info!(
            "Created message queue client: backend={:?}, queue={}",
            config.backend, config.queue_name
        );

        Ok(Self {
            config,
            redis_client: Some(redis_client),
            stats: Arc::new(RwLock::new(stats)),
        })
    }

    /// Publish event message to queue
    pub async fn publish_event(&self, event_message: &EventMessage) -> Result<()> {
        let serialized = serde_json::to_vec(event_message)
            .context("Failed to serialize event message")?;

        match self.config.backend {
            QueueBackend::RedisPubSub => {
                self.publish_pubsub(&serialized).await?;
            }
            QueueBackend::RedisStream => {
                self.publish_stream(&serialized).await?;
            }
            QueueBackend::Kafka => {
                anyhow::bail!("Kafka backend not yet implemented");
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_published += 1;
            stats.last_publish_at = Some(chrono::Utc::now());
        }

        debug!(
            "Published event {} to queue {}",
            event_message.event.event_id, self.config.queue_name
        );

        Ok(())
    }

    /// Publish to Redis pub/sub
    async fn publish_pubsub(&self, data: &[u8]) -> Result<()> {
        let client = self.redis_client.as_ref()
            .context("Redis client not initialized")?;

        let mut conn = client
            .get_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let _: () = conn
            .publish(&self.config.queue_name, data)
            .await
            .context("Failed to publish to Redis pub/sub")?;

        // Update connection status
        {
            let mut stats = self.stats.write().await;
            stats.connected = true;
        }

        Ok(())
    }

    /// Publish to Redis stream
    async fn publish_stream(&self, data: &[u8]) -> Result<()> {
        let client = self.redis_client.as_ref()
            .context("Redis client not initialized")?;

        let mut conn = client
            .get_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        // Create consumer group if it doesn't exist
        if let Some(ref group) = self.config.consumer_group {
            let _: Result<(), redis::RedisError> = conn
                .xgroup_create_mkstream(
                    &self.config.queue_name,
                    group,
                    "0",
                )
                .await;

            // Ignore error if group already exists
        }

        // Add event to stream
        let event_data = [("data", data)];
        let _: () = conn
            .xadd(
                &self.config.queue_name,
                "*",  // Auto-generated ID
                &event_data,
            )
            .await
            .context("Failed to add event to Redis stream")?;

        // Trim stream if max length is configured
        if let Some(max_len) = self.config.stream_max_len {
            let _: () = conn
                .xtrim(
                    &self.config.queue_name,
                    redis::streams::StreamMaxlen::Approx(max_len),
                )
                .await
                .context("Failed to trim Redis stream")?;
        }

        // Update queue depth
        let length: usize = conn
            .xlen(&self.config.queue_name)
            .await
            .context("Failed to get stream length")?;

        {
            let mut stats = self.stats.write().await;
            stats.queue_depth = length;
            stats.connected = true;
        }

        Ok(())
    }

    /// Get current queue statistics
    pub async fn stats(&self) -> QueueStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Health check for queue connectivity
    pub async fn health_check(&self) -> Result<bool> {
        let client = self.redis_client.as_ref()
            .context("Redis client not initialized")?;

        let mut conn = client
            .get_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("Redis PING failed")?;

        {
            let mut stats = self.stats.write().await;
            stats.connected = true;
        }

        Ok(true)
    }

    /// Get queue depth (number of pending messages)
    pub async fn queue_depth(&self) -> Result<usize> {
        let client = self.redis_client.as_ref()
            .context("Redis client not initialized")?;

        let mut conn = client
            .get_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let depth = match self.config.backend {
            QueueBackend::RedisPubSub => {
                // Pub/sub doesn't have a concept of queue depth
                0
            }
            QueueBackend::RedisStream => {
                conn.xlen(&self.config.queue_name)
                    .await
                    .context("Failed to get stream length")?
            }
            QueueBackend::Kafka => {
                anyhow::bail!("Kafka backend not yet implemented");
            }
        };

        Ok(depth)
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.total_published = 0;
        stats.total_errors = 0;
        stats.last_publish_at = None;
    }

    /// Get configuration
    pub fn config(&self) -> &QueueConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_queue_backend_from_str() {
        assert_eq!(
            QueueBackend::from_str("redis_pubsub").unwrap(),
            QueueBackend::RedisPubSub
        );
        assert_eq!(
            QueueBackend::from_str("REDIS-PUBSUB").unwrap(),
            QueueBackend::RedisPubSub
        );
        assert_eq!(
            QueueBackend::from_str("redis_stream").unwrap(),
            QueueBackend::RedisStream
        );
        assert_eq!(
            QueueBackend::from_str("kafka").unwrap(),
            QueueBackend::Kafka
        );
        assert!(QueueBackend::from_str("invalid").is_err());
    }

    #[test]
    fn test_queue_config_default() {
        let config = QueueConfig::default();
        assert_eq!(config.backend, QueueBackend::RedisStream);
        assert_eq!(config.queue_name, "ai-analytics-events");
        assert_eq!(config.timeout_secs, 30);
        assert!(config.enable_pooling);
    }

    #[test]
    fn test_message_queue_requires_url() {
        let config = QueueConfig {
            redis_url: String::new(),
            ..Default::default()
        };
        assert!(MessageQueue::new(config).is_err());
    }

    #[test]
    fn test_queue_stats_serialization() {
        let stats = QueueStats {
            backend: "RedisStream".to_string(),
            queue_name: "test-queue".to_string(),
            queue_depth: 100,
            connected: true,
            total_published: 1000,
            total_errors: 5,
            last_publish_at: Some(Utc::now()),
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: QueueStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.backend, deserialized.backend);
        assert_eq!(stats.queue_name, deserialized.queue_name);
        assert_eq!(stats.queue_depth, deserialized.queue_depth);
    }
}
