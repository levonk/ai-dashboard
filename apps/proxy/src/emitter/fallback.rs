//! Fallback and error handling for emitter mode
//! 
//! This module provides local buffering and fallback mechanisms to prevent data loss
//! when external collectors or message queues are unavailable.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::protocol::EventMessage;

/// Fallback buffer configuration
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Buffer directory path
    pub buffer_path: PathBuf,
    /// Maximum buffer size in bytes
    pub max_buffer_size: usize,
    /// Enable file rotation
    pub enable_rotation: bool,
    /// Rotation file size in bytes
    pub rotation_size: usize,
    /// Maximum number of rotated files
    pub max_rotated_files: usize,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            buffer_path: PathBuf::from("/tmp/ai-analytics-proxy/buffer"),
            max_buffer_size: 1_000_000_000, // 1GB
            enable_rotation: true,
            rotation_size: 100_000_000, // 100MB
            max_rotated_files: 10,
        }
    }
}

/// Fallback buffer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStats {
    /// Buffer directory path
    pub buffer_path: String,
    /// Current buffer size in bytes
    pub current_size: usize,
    /// Number of buffered events
    pub event_count: usize,
    /// Number of rotated files
    pub rotated_file_count: usize,
    /// Active status
    pub active: bool,
    /// Last write timestamp
    pub last_write_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Fallback buffer for local event storage
pub struct FallbackBuffer {
    config: FallbackConfig,
    current_file: Option<BufWriter<File>>,
    current_file_size: usize,
    stats: Arc<RwLock<FallbackStats>>,
}

impl FallbackBuffer {
    /// Create a new fallback buffer
    pub fn new(config: FallbackConfig) -> Result<Self> {
        // Ensure buffer directory exists
        std::fs::create_dir_all(&config.buffer_path)
            .context("Failed to create buffer directory")?;

        let stats = FallbackStats {
            buffer_path: config.buffer_path.to_string_lossy().to_string(),
            current_size: 0,
            event_count: 0,
            rotated_file_count: 0,
            active: false,
            last_write_at: None,
        };

        info!(
            "Created fallback buffer at: {}",
            config.buffer_path.display()
        );

        Ok(Self {
            config,
            current_file: None,
            current_file_size: 0,
            stats: Arc::new(RwLock::new(stats)),
        })
    }

    /// Write event to fallback buffer
    pub async fn write_event(&mut self, event_message: &EventMessage) -> Result<()> {
        let serialized = serde_json::to_vec(event_message)
            .context("Failed to serialize event for fallback")?;

        // Check if we need to rotate
        if self.config.enable_rotation && self.current_file_size + serialized.len() > self.config.rotation_size {
            self.rotate_file().await?;
        }

        // Ensure current file is open
        if self.current_file.is_none() {
            self.open_current_file().await?;
        }

        // Write event
        if let Some(ref mut writer) = self.current_file {
            writeln!(writer, "{}", serde_json::to_string(event_message)?)
                .context("Failed to write event to buffer")?;
            
            writer.flush()
                .context("Failed to flush buffer")?;

            self.current_file_size += serialized.len();

            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.current_size += serialized.len();
                stats.event_count += 1;
                stats.last_write_at = Some(chrono::Utc::now());
                stats.active = true;
            }

            debug!(
                "Written event {} to fallback buffer (size: {} bytes)",
                event_message.event.event_id,
                serialized.len()
            );
        }

        Ok(())
    }

    /// Open current buffer file
    async fn open_current_file(&mut self) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("fallback_{}.jsonl", timestamp);
        let filepath = self.config.buffer_path.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filepath)
            .context("Failed to open buffer file")?;

        self.current_file = Some(BufWriter::new(file));
        self.current_file_size = 0;

        info!("Opened fallback buffer file: {}", filepath.display());
        Ok(())
    }

    /// Rotate buffer file
    async fn rotate_file(&mut self) -> Result<()> {
        if let Some(mut writer) = self.current_file.take() {
            writer.flush()
                .context("Failed to flush buffer before rotation")?;
            drop(writer);
        }

        self.current_file_size = 0;

        // Clean up old rotated files
        self.cleanup_old_files().await?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.rotated_file_count += 1;
        }

        info!("Rotated fallback buffer file");
        Ok(())
    }

    /// Clean up old rotated files
    async fn cleanup_old_files(&self) -> Result<()> {
        let entries = std::fs::read_dir(&self.config.buffer_path)
            .context("Failed to read buffer directory")?;

        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        // Sort by modification time (oldest first)
        files.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        // Remove old files if we exceed the limit
        while files.len() > self.config.max_rotated_files {
            if let Some(old_file) = files.first() {
                let path = old_file.path().clone();
                std::fs::remove_file(&path)
                    .context("Failed to remove old buffer file")?;
                files.remove(0);
                debug!("Removed old buffer file: {}", path.display());
            }
        }

        Ok(())
    }

    /// Get current statistics
    pub async fn stats(&self) -> FallbackStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Flush and close current buffer file
    pub async fn flush(&mut self) -> Result<()> {
        if let Some(mut writer) = self.current_file.take() {
            writer.flush()
                .context("Failed to flush buffer")?;
            drop(writer);
        }

        info!("Flushed fallback buffer");
        Ok(())
    }

    /// Get buffer size
    pub async fn buffer_size(&self) -> usize {
        let stats = self.stats.read().await;
        stats.current_size
    }

    /// Check if buffer is empty
    pub async fn is_empty(&self) -> bool {
        let stats = self.stats.read().await;
        stats.event_count == 0
    }

    /// Clear all buffer files
    pub async fn clear(&mut self) -> Result<()> {
        // Flush current file
        self.flush().await?;

        // Remove all buffer files
        let entries = std::fs::read_dir(&self.config.buffer_path)
            .context("Failed to read buffer directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
                std::fs::remove_file(&path)
                    .context("Failed to remove buffer file")?;
                debug!("Removed buffer file: {}", path.display());
            }
        }

        // Reset stats
        {
            let mut stats = self.stats.write().await;
            stats.current_size = 0;
            stats.event_count = 0;
            stats.rotated_file_count = 0;
            stats.active = false;
            stats.last_write_at = None;
        }

        info!("Cleared all fallback buffer files");
        Ok(())
    }

    /// Replay buffered events
    pub async fn replay(&self) -> Result<Vec<EventMessage>> {
        let mut events = Vec::new();

        let entries = std::fs::read_dir(&self.config.buffer_path)
            .context("Failed to read buffer directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)
                    .context("Failed to read buffer file")?;

                for line in content.lines() {
                    if let Ok(event) = serde_json::from_str::<EventMessage>(line) {
                        events.push(event);
                    }
                }
            }
        }

        info!("Replayed {} events from fallback buffer", events.len());
        Ok(events)
    }

    /// Get configuration
    pub fn config(&self) -> &FallbackConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_fallback_config_default() {
        let config = FallbackConfig::default();
        assert_eq!(config.max_buffer_size, 1_000_000_000);
        assert_eq!(config.rotation_size, 100_000_000);
        assert!(config.enable_rotation);
    }

    #[test]
    fn test_fallback_stats_serialization() {
        let stats = FallbackStats {
            buffer_path: "/tmp/buffer".to_string(),
            current_size: 1000,
            event_count: 10,
            rotated_file_count: 2,
            active: true,
            last_write_at: Some(Utc::now()),
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: FallbackStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.buffer_path, deserialized.buffer_path);
        assert_eq!(stats.current_size, deserialized.current_size);
    }
}
