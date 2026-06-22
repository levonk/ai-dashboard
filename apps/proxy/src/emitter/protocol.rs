//! Event emission protocol and serialization
//! 
//! This module handles the conversion of telemetry events to wire format
//! for transmission to external collectors and message queues.

use anyhow::{Context, Result};
use analytics_rs::TelemetryEvent;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Protocol version for event serialization
pub const PROTOCOL_VERSION: u32 = 1;

/// Serialization format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// JSON format (human-readable, widely supported)
    Json,
    /// MessagePack format (binary, more compact)
    MessagePack,
}

impl Default for SerializationFormat {
    fn default() -> Self {
        Self::Json
    }
}

impl std::str::FromStr for SerializationFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "messagepack" | "msgpack" => Ok(Self::MessagePack),
            _ => anyhow::bail!("Invalid serialization format: {}. Must be 'json' or 'messagepack'", s),
        }
    }
}

/// Protocol header for event messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolHeader {
    /// Protocol version
    pub version: u32,
    /// Message type (event, heartbeat, control)
    pub message_type: String,
    /// Timestamp when message was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Source identifier (proxy instance)
    pub source_id: String,
    /// Sequence number for ordering
    pub sequence: u64,
}

/// Serialized event message with protocol header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    /// Protocol header
    pub header: ProtocolHeader,
    /// Telemetry event payload
    pub event: TelemetryEvent,
    /// Additional metadata for transmission
    pub metadata: TransmissionMetadata,
}

/// Transmission metadata for event messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionMetadata {
    /// Content hash for integrity verification
    pub content_hash: String,
    /// Payload size in bytes
    pub payload_size: usize,
    /// Compression flag
    pub compressed: bool,
    /// Encryption flag
    pub encrypted: bool,
    /// Retry count (for retransmission)
    pub retry_count: u32,
}

impl Default for TransmissionMetadata {
    fn default() -> Self {
        Self {
            content_hash: String::new(),
            payload_size: 0,
            compressed: false,
            encrypted: false,
            retry_count: 0,
        }
    }
}

/// Protocol configuration
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Serialization format
    pub format: SerializationFormat,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (1-9, for gzip/zstd)
    pub compression_level: u32,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Source identifier for this proxy instance
    pub source_id: String,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            format: SerializationFormat::Json,
            enable_compression: false,
            compression_level: 6,
            enable_encryption: false,
            source_id: gethostname::gethostname()
                .to_string_lossy()
                .to_string(),
        }
    }
}

/// Event serializer for converting telemetry events to wire format
pub struct EventSerializer {
    config: ProtocolConfig,
    sequence: u64,
}

impl EventSerializer {
    /// Create a new event serializer with default configuration
    pub fn new() -> Self {
        Self {
            config: ProtocolConfig::default(),
            sequence: 0,
        }
    }

    /// Create a new event serializer with custom configuration
    pub fn with_config(config: ProtocolConfig) -> Self {
        Self {
            config,
            sequence: 0,
        }
    }

    /// Serialize a telemetry event to wire format
    pub fn serialize_event(&mut self, event: TelemetryEvent) -> Result<Vec<u8>> {
        let event_id = event.event_id.clone();
        debug!("Serializing event: {}", event_id);

        // Create protocol header
        let header = ProtocolHeader {
            version: PROTOCOL_VERSION,
            message_type: "event".to_string(),
            created_at: chrono::Utc::now(),
            source_id: self.config.source_id.clone(),
            sequence: self.next_sequence(),
        };

        // Calculate content hash
        let event_json = serde_json::to_string(&event)
            .context("Failed to serialize event for hashing")?;
        let content_hash = self.calculate_hash(&event_json);

        // Create transmission metadata
        let metadata = TransmissionMetadata {
            content_hash,
            payload_size: event_json.len(),
            compressed: self.config.enable_compression,
            encrypted: self.config.enable_encryption,
            retry_count: 0,
        };

        // Create event message
        let message = EventMessage {
            header,
            event,
            metadata,
        };

        // Serialize to wire format
        let serialized = match self.config.format {
            SerializationFormat::Json => self.serialize_json(&message)?,
            SerializationFormat::MessagePack => self.serialize_messagepack(&message)?,
        };

        // Apply compression if enabled
        let final_bytes = if self.config.enable_compression {
            self.compress(&serialized)?
        } else {
            serialized
        };

        debug!("Successfully serialized event: {} ({} bytes)", event_id, final_bytes.len());
        Ok(final_bytes)
    }

    /// Serialize to JSON format
    fn serialize_json(&self, message: &EventMessage) -> Result<Vec<u8>> {
        serde_json::to_vec(message)
            .context("Failed to serialize event message to JSON")
    }

    /// Serialize to MessagePack format
    fn serialize_messagepack(&self, message: &EventMessage) -> Result<Vec<u8>> {
        // Note: This would require rmp_serde dependency
        // For now, fall back to JSON with a warning
        tracing::warn!("MessagePack serialization not yet implemented, falling back to JSON");
        self.serialize_json(message)
    }

    /// Compress data using gzip
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Note: This would require flate2 dependency
        // For now, return uncompressed data with a warning
        tracing::warn!("Compression not yet implemented, returning uncompressed data");
        Ok(data.to_vec())
    }

    /// Calculate content hash for integrity verification
    fn calculate_hash(&self, data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get next sequence number
    fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        self.sequence
    }

    /// Reset sequence number
    pub fn reset_sequence(&mut self) {
        self.sequence = 0;
    }
}

impl Default for EventSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Event deserializer for converting wire format back to telemetry events
pub struct EventDeserializer {
    config: ProtocolConfig,
}

impl EventDeserializer {
    /// Create a new event deserializer with default configuration
    pub fn new() -> Self {
        Self {
            config: ProtocolConfig::default(),
        }
    }

    /// Create a new event deserializer with custom configuration
    pub fn with_config(config: ProtocolConfig) -> Self {
        Self { config }
    }

    /// Deserialize wire format to telemetry event
    pub fn deserialize_event(&self, data: &[u8]) -> Result<TelemetryEvent> {
        debug!("Deserializing event ({} bytes)", data.len());

        // Decompress if needed
        let data = if self.config.enable_compression {
            self.decompress(data)?
        } else {
            data.to_vec()
        };

        // Deserialize based on format
        let message: EventMessage = match self.config.format {
            SerializationFormat::Json => self.deserialize_json(&data)?,
            SerializationFormat::MessagePack => self.deserialize_messagepack(&data)?,
        };

        // Verify content hash if present
        if !message.metadata.content_hash.is_empty() {
            self.verify_hash(&message)?;
        }

        debug!("Successfully deserialized event: {}", message.event.event_id);
        Ok(message.event)
    }

    /// Deserialize from JSON format
    fn deserialize_json(&self, data: &[u8]) -> Result<EventMessage> {
        serde_json::from_slice(data)
            .context("Failed to deserialize event message from JSON")
    }

    /// Deserialize from MessagePack format
    fn deserialize_messagepack(&self, data: &[u8]) -> Result<EventMessage> {
        // Note: This would require rmp_serde dependency
        // For now, fall back to JSON with a warning
        tracing::warn!("MessagePack deserialization not yet implemented, falling back to JSON");
        self.deserialize_json(data)
    }

    /// Decompress data using gzip
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Note: This would require flate2 dependency
        // For now, return data as-is with a warning
        tracing::warn!("Decompression not yet implemented, returning data as-is");
        Ok(data.to_vec())
    }

    /// Verify content hash for integrity
    fn verify_hash(&self, message: &EventMessage) -> Result<()> {
        let event_json = serde_json::to_string(&message.event)
            .context("Failed to serialize event for hash verification")?;
        
        let calculated_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            event_json.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };

        if calculated_hash != message.metadata.content_hash {
            anyhow::bail!(
                "Content hash mismatch: expected {}, got {}",
                message.metadata.content_hash,
                calculated_hash
            );
        }

        Ok(())
    }
}

impl Default for EventDeserializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_serialization_format_from_str() {
        assert_eq!(
            SerializationFormat::from_str("json").unwrap(),
            SerializationFormat::Json
        );
        assert_eq!(
            SerializationFormat::from_str("JSON").unwrap(),
            SerializationFormat::Json
        );
        assert_eq!(
            SerializationFormat::from_str("messagepack").unwrap(),
            SerializationFormat::MessagePack
        );
        assert_eq!(
            SerializationFormat::from_str("msgpack").unwrap(),
            SerializationFormat::MessagePack
        );
        assert!(SerializationFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_event_serializer_sequence() {
        let mut serializer = EventSerializer::new();
        assert_eq!(serializer.current_sequence(), 0);
        
        let event = create_test_event();
        serializer.serialize_event(event).unwrap();
        assert_eq!(serializer.current_sequence(), 1);
        
        serializer.reset_sequence();
        assert_eq!(serializer.current_sequence(), 0);
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let mut serializer = EventSerializer::new();
        let deserializer = EventDeserializer::new();

        let original_event = create_test_event();
        let serialized = serializer.serialize_event(original_event.clone()).unwrap();
        let deserialized_event = deserializer.deserialize_event(&serialized).unwrap();

        assert_eq!(original_event.event_id, deserialized_event.event_id);
        assert_eq!(original_event.ai_client, deserialized_event.ai_client);
        assert_eq!(original_event.ai_provider, deserialized_event.ai_provider);
    }

    #[test]
    fn test_protocol_header_creation() {
        let config = ProtocolConfig {
            source_id: "test-proxy".to_string(),
            ..Default::default()
        };
        let mut serializer = EventSerializer::with_config(config.clone());

        let event = create_test_event();
        let serialized = serializer.serialize_event(event).unwrap();
        let message: EventMessage = serde_json::from_slice(&serialized).unwrap();

        assert_eq!(message.header.version, PROTOCOL_VERSION);
        assert_eq!(message.header.message_type, "event");
        assert_eq!(message.header.source_id, config.source_id);
        assert_eq!(message.header.sequence, 0);
    }

    fn create_test_event() -> TelemetryEvent {
        TelemetryEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            ai_client: "test-client".to_string(),
            ai_provider: "test-provider".to_string(),
            model: Some("test-model".to_string()),
            input_type: "text".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            duration_ms: 1000,
            cost_usd: Some(0.01),
            metadata: serde_json::json!({"test": "data"}),
        }
    }
}
