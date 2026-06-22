//! Emitter mode configuration
//! 
//! This module handles configuration for emitter mode, including parsing,
//! validation, and management of emitter-specific settings.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, warn};
use directories;
use gethostname;

use crate::emitter::{
    EmitterClientConfig, 
    QueueConfig, 
    QueueBackend,
    ProtocolConfig,
    SerializationFormat,
};

/// Emitter mode configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmitterModeConfig {
    /// Enable emitter mode
    pub enabled: bool,
    
    /// HTTP collector configuration
    pub collector: CollectorConfig,
    
    /// Message queue configuration
    pub queue: QueueEmitterConfig,
    
    /// Protocol configuration
    pub protocol: ProtocolEmitterConfig,
    
    /// Fallback configuration
    pub fallback: FallbackConfig,
    
    /// Performance tuning
    pub performance: PerformanceConfig,
}

/// HTTP collector configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectorConfig {
    /// Collector URL
    pub url: String,
    
    /// API key for authentication
    pub api_key: Option<String>,
    
    /// Request timeout in seconds
    pub timeout_secs: u64,
    
    /// Enable HTTP collector
    pub enabled: bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            api_key: None,
            timeout_secs: 30,
            enabled: false,
        }
    }
}

/// Message queue emitter configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueueEmitterConfig {
    /// Queue backend type
    pub backend: String,
    
    /// Redis connection URL
    pub redis_url: String,
    
    /// Queue/stream name
    pub queue_name: String,
    
    /// Consumer group name
    pub consumer_group: Option<String>,
    
    /// Consumer name
    pub consumer_name: Option<String>,
    
    /// Maximum stream length
    pub stream_max_len: Option<usize>,
    
    /// Enable queue
    pub enabled: bool,
}

impl Default for QueueEmitterConfig {
    fn default() -> Self {
        Self {
            backend: "redis_stream".to_string(),
            redis_url: String::new(),
            queue_name: "ai-analytics-events".to_string(),
            consumer_group: Some("analytics-consumers".to_string()),
            consumer_name: Some("proxy-emitter".to_string()),
            stream_max_len: Some(10000),
            enabled: false,
        }
    }
}

/// Protocol emitter configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolEmitterConfig {
    /// Serialization format
    pub format: String,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Compression level (1-9)
    pub compression_level: u32,
    
    /// Enable encryption
    pub enable_encryption: bool,
}

impl Default for ProtocolEmitterConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            enable_compression: false,
            compression_level: 6,
            enable_encryption: false,
        }
    }
}

/// Fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FallbackConfig {
    /// Enable fallback buffering
    pub enabled: bool,
    
    /// Buffer path
    pub buffer_path: Option<PathBuf>,
    
    /// Maximum buffer size in bytes
    pub max_buffer_size: Option<String>,
    
    /// Buffer rotation enabled
    pub enable_rotation: bool,
    
    /// Rotation file size
    pub rotation_size_mb: Option<usize>,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_path: None,
            max_buffer_size: Some("1GB".to_string()),
            enable_rotation: true,
            rotation_size_mb: Some(100),
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceConfig {
    /// Batch size for event emission
    pub batch_size: usize,
    
    /// Batch timeout in seconds
    pub batch_timeout_secs: u64,
    
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    
    /// Maximum retries
    pub max_retries: u32,
    
    /// Retry delay in seconds
    pub retry_delay_secs: u64,
    
    /// Maximum retry delay in seconds
    pub max_retry_delay_secs: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout_secs: 5,
            max_concurrent_requests: 10,
            max_retries: 3,
            retry_delay_secs: 1,
            max_retry_delay_secs: 60,
        }
    }
}

impl Default for EmitterModeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            collector: CollectorConfig::default(),
            queue: QueueEmitterConfig::default(),
            protocol: ProtocolEmitterConfig::default(),
            fallback: FallbackConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl EmitterModeConfig {
    /// Validate emitter configuration
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            debug!("Emitter mode is disabled, skipping validation");
            return Ok(());
        }

        // Check that at least one output is enabled
        if !self.collector.enabled && !self.queue.enabled {
            anyhow::bail!(
                "At least one output must be enabled (collector or queue)"
            );
        }

        // Validate collector configuration
        if self.collector.enabled {
            if self.collector.url.is_empty() {
                anyhow::bail!("Collector URL is required when collector is enabled");
            }
            if !self.collector.url.starts_with("http://") && !self.collector.url.starts_with("https://") {
                anyhow::bail!("Collector URL must start with http:// or https://");
            }
        }

        // Validate queue configuration
        if self.queue.enabled {
            if self.queue.redis_url.is_empty() {
                anyhow::bail!("Redis URL is required when queue is enabled");
            }
            
            // Validate backend type
            let backend: QueueBackend = self.queue.backend.parse()
                .context("Invalid queue backend type")?;
            
            if matches!(backend, QueueBackend::Kafka) {
                warn!("Kafka backend is not yet implemented");
            }
        }

        // Validate protocol configuration
        let format: SerializationFormat = self.protocol.format.parse()
            .context("Invalid serialization format")?;
        
        if self.protocol.compression_level < 1 || self.protocol.compression_level > 9 {
            anyhow::bail!("Compression level must be between 1 and 9");
        }

        // Validate performance configuration
        if self.performance.batch_size == 0 {
            anyhow::bail!("Batch size must be greater than 0");
        }
        if self.performance.max_concurrent_requests == 0 {
            anyhow::bail!("Max concurrent requests must be greater than 0");
        }

        info!("Emitter configuration validation passed");
        Ok(())
    }

    /// Convert to EmitterClientConfig
    pub fn to_client_config(&self) -> Result<EmitterClientConfig> {
        if !self.collector.enabled {
            anyhow::bail!("Collector is not enabled");
        }

        Ok(EmitterClientConfig {
            collector_url: self.collector.url.clone(),
            api_key: self.collector.api_key.clone(),
            timeout_secs: self.collector.timeout_secs,
            max_retries: self.performance.max_retries,
            retry_delay_secs: self.performance.retry_delay_secs,
            max_retry_delay_secs: self.performance.max_retry_delay_secs,
            max_concurrent_requests: self.performance.max_concurrent_requests,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
        })
    }

    /// Convert to QueueConfig
    pub fn to_queue_config(&self) -> Result<QueueConfig> {
        if !self.queue.enabled {
            anyhow::bail!("Queue is not enabled");
        }

        let backend: QueueBackend = self.queue.backend.parse()
            .context("Invalid queue backend type")?;

        Ok(QueueConfig {
            backend,
            redis_url: self.queue.redis_url.clone(),
            queue_name: self.queue.queue_name.clone(),
            consumer_group: self.queue.consumer_group.clone(),
            consumer_name: self.queue.consumer_name.clone(),
            stream_max_len: self.queue.stream_max_len,
            timeout_secs: self.collector.timeout_secs,
            enable_pooling: true,
            max_pool_size: self.performance.max_concurrent_requests,
        })
    }

    /// Convert to ProtocolConfig
    pub fn to_protocol_config(&self) -> Result<ProtocolConfig> {
        let _format: SerializationFormat = self.protocol.format.parse()
            .context("Invalid serialization format")?;

        Ok(ProtocolConfig {
            format: match self.protocol.format.as_str() {
                "json" => SerializationFormat::Json,
                "messagepack" | "msgpack" => SerializationFormat::MessagePack,
                _ => SerializationFormat::Json,
            },
            enable_compression: self.protocol.enable_compression,
            compression_level: self.protocol.compression_level,
            enable_encryption: self.protocol.enable_encryption,
            source_id: gethostname::gethostname()
                .to_string_lossy()
                .to_string(),
        })
    }

    /// Check if emitter mode is properly configured
    pub fn is_configured(&self) -> bool {
        self.enabled && (self.collector.enabled || self.queue.enabled)
    }

    /// Get default buffer path
    pub fn default_buffer_path() -> PathBuf {
        let mut path = directories::ProjectDirs::from("com", "myorg", "ai-analytics-proxy")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        path.push("buffer");
        path
    }

    /// Ensure buffer directory exists
    pub fn ensure_buffer_dir(&self) -> Result<()> {
        if !self.fallback.enabled {
            return Ok(());
        }

        let buffer_path = self.fallback.buffer_path
            .clone()
            .unwrap_or_else(Self::default_buffer_path);

        std::fs::create_dir_all(&buffer_path)
            .context("Failed to create buffer directory")?;

        info!("Buffer directory ensured: {}", buffer_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_config_default() {
        let config = EmitterModeConfig::default();
        assert!(!config.enabled);
        assert!(!config.collector.enabled);
        assert!(!config.queue.enabled);
    }

    #[test]
    fn test_emitter_config_validation_disabled() {
        let config = EmitterModeConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_emitter_config_validation_no_outputs() {
        let mut config = EmitterModeConfig::default();
        config.enabled = true;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_emitter_config_validation_collector_missing_url() {
        let mut config = EmitterModeConfig::default();
        config.enabled = true;
        config.collector.enabled = true;
        config.collector.url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_emitter_config_validation_invalid_url() {
        let mut config = EmitterModeConfig::default();
        config.enabled = true;
        config.collector.enabled = true;
        config.collector.url = "invalid-url".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_emitter_config_validation_valid() {
        let mut config = EmitterModeConfig::default();
        config.enabled = true;
        config.collector.enabled = true;
        config.collector.url = "https://collector.example.com".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_is_configured() {
        let mut config = EmitterModeConfig::default();
        assert!(!config.is_configured());

        config.enabled = true;
        assert!(!config.is_configured());

        config.collector.enabled = true;
        config.collector.url = "https://example.com".to_string();
        assert!(config.is_configured());
    }

    #[test]
    fn test_to_client_config_requires_enabled() {
        let config = EmitterModeConfig::default();
        assert!(config.to_client_config().is_err());
    }
}
