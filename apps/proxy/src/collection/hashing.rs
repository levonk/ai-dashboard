use analytics_rs::{content_hash, structured_hash, correlation_id, HashCollisionMonitor};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

/// Content hashing for request correlation
/// Provides utilities for generating consistent hashes to track requests through the pipeline
pub struct HashCollector {
    collision_monitor: HashCollisionMonitor,
}

impl HashCollector {
    /// Create a new hash collector
    pub fn new() -> Self {
        Self {
            collision_monitor: HashCollisionMonitor::new(),
        }
    }

    /// Generate a content hash for request body
    pub fn hash_request_content(&self, content: &str) -> String {
        content_hash(content)
    }

    /// Generate a hash for structured request data
    pub fn hash_request_structured<T: Serialize>(&self, data: &T) -> Result<String> {
        Ok(structured_hash(data)?)
    }

    /// Generate a correlation ID from request components
    pub fn generate_correlation_id(&self, components: &[&str]) -> String {
        correlation_id(components)
    }

    /// Generate a request fingerprint from key request attributes
    pub fn generate_request_fingerprint(&self, request: &Value) -> Result<String> {
        let components = self.extract_key_components(request)?;
        let component_refs: Vec<&str> = components.iter().map(|s| s.as_str()).collect();
        Ok(self.generate_correlation_id(&component_refs))
    }

    /// Extract key components for fingerprinting
    fn extract_key_components(&self, request: &Value) -> Result<Vec<String>> {
        let mut components = Vec::new();

        // Add model if present
        if let Some(model) = request.get("model").and_then(|m| m.as_str()) {
            components.push(model.to_string());
        }

        // Add prompt/content if present
        if let Some(prompt) = request.get("prompt").and_then(|p| p.as_str()) {
            components.push(prompt.to_string());
        } else if let Some(content) = request.get("content").and_then(|c| c.as_str()) {
            components.push(content.to_string());
        }

        // Add key parameters
        if let Some(temperature) = request.get("temperature").and_then(|t| t.as_f64()) {
            components.push(format!("temperature:{}", temperature));
        }

        if let Some(max_tokens) = request.get("max_tokens").and_then(|m| m.as_u64()) {
            components.push(format!("max_tokens:{}", max_tokens));
        }

        Ok(components)
    }

    /// Generate a session ID from user and context
    pub fn generate_session_id(&self, user_id: &str, context: &str) -> String {
        self.generate_correlation_id(&[user_id, context])
    }

    /// Check for hash collisions
    pub fn check_collision(&mut self, hash: &str) -> bool {
        self.collision_monitor.record_hash(hash)
    }

    /// Get collision statistics
    pub fn get_collision_stats(&self) -> CollisionStats {
        CollisionStats {
            total_hashes: self.collision_monitor.total_hashes(),
            detected_collisions: self.collision_monitor.detected_collisions(),
            collision_rate: self.collision_monitor.collision_rate(),
        }
    }

    /// Reset collision monitoring
    pub fn reset_collision_monitor(&mut self) {
        self.collision_monitor = HashCollisionMonitor::new();
    }
}

impl Default for HashCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Collision statistics
#[derive(Debug, Clone)]
pub struct CollisionStats {
    pub total_hashes: u64,
    pub detected_collisions: u64,
    pub collision_rate: f64,
}

/// Request fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFingerprint {
    pub fingerprint: String,
    pub components: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RequestFingerprint {
    /// Create a new request fingerprint
    pub fn new(fingerprint: String, components: Vec<String>) -> Self {
        Self {
            fingerprint,
            components,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if two fingerprints match
    pub fn matches(&self, other: &RequestFingerprint) -> bool {
        self.fingerprint == other.fingerprint
    }

    /// Calculate similarity score (0.0 to 1.0)
    pub fn similarity(&self, other: &RequestFingerprint) -> f64 {
        if self.fingerprint == other.fingerprint {
            1.0
        } else {
            // Calculate component overlap
            let intersection: usize = self.components
                .iter()
                .filter(|c| other.components.contains(c))
                .count();
            
            let union = self.components.len() + other.components.len() - intersection;
            
            if union > 0 {
                intersection as f64 / union as f64
            } else {
                0.0
            }
        }
    }
}

/// Correlation context for tracking requests across pipeline stages
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    pub correlation_id: String,
    pub request_id: String,
    pub stage: String,
    pub metadata: HashMap<String, String>,
}

impl CorrelationContext {
    /// Create a new correlation context
    pub fn new(correlation_id: String, request_id: String, stage: String) -> Self {
        Self {
            correlation_id,
            request_id,
            stage,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Create a new context for the next stage
    pub fn for_next_stage(&self, next_stage: String) -> Self {
        Self {
            correlation_id: self.correlation_id.clone(),
            request_id: self.request_id.clone(),
            stage: next_stage,
            metadata: self.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_collector_creation() {
        let collector = HashCollector::new();
        // Collector created successfully
    }

    #[test]
    fn test_hash_request_content() {
        let collector = HashCollector::new();
        let content = "Hello, world!";
        let hash = collector.hash_request_content(content);
        
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_consistency() {
        let collector = HashCollector::new();
        let content = "Test content";
        let hash1 = collector.hash_request_content(content);
        let hash2 = collector.hash_request_content(content);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_uniqueness() {
        let collector = HashCollector::new();
        let hash1 = collector.hash_request_content("Content 1");
        let hash2 = collector.hash_request_content("Content 2");
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_correlation_id() {
        let collector = HashCollector::new();
        let components = vec!["model1", "prompt1", "param1"];
        let id1 = collector.generate_correlation_id(&components);
        let id2 = collector.generate_correlation_id(&components);
        
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_request_fingerprint() {
        let collector = HashCollector::new();
        let request = serde_json::json!({
            "model": "claude-3-opus",
            "prompt": "Hello, world!",
            "temperature": 0.7
        });
        
        let fingerprint = collector.generate_request_fingerprint(&request).unwrap();
        assert_eq!(fingerprint.len(), 64);
    }

    #[test]
    fn test_generate_session_id() {
        let collector = HashCollector::new();
        let session_id = collector.generate_session_id("user123", "production");
        
        assert_eq!(session_id.len(), 64);
    }

    #[test]
    fn test_collision_stats() {
        let mut collector = HashCollector::new();
        collector.check_collision("hash1");
        collector.check_collision("hash2");
        
        let stats = collector.get_collision_stats();
        assert_eq!(stats.total_hashes, 2);
    }

    #[test]
    fn test_request_fingerprint() {
        let fingerprint = RequestFingerprint::new(
            "abc123".to_string(),
            vec!["component1".to_string(), "component2".to_string()],
        );
        
        assert_eq!(fingerprint.fingerprint, "abc123");
        assert_eq!(fingerprint.components.len(), 2);
    }

    #[test]
    fn test_fingerprint_matches() {
        let fp1 = RequestFingerprint::new(
            "abc123".to_string(),
            vec!["component1".to_string()],
        );
        
        let fp2 = RequestFingerprint::new(
            "abc123".to_string(),
            vec!["component1".to_string()],
        );
        
        assert!(fp1.matches(&fp2));
    }

    #[test]
    fn test_fingerprint_similarity() {
        let fp1 = RequestFingerprint::new(
            "abc123".to_string(),
            vec!["component1".to_string(), "component2".to_string()],
        );
        
        let fp2 = RequestFingerprint::new(
            "def456".to_string(),
            vec!["component1".to_string(), "component3".to_string()],
        );
        
        let similarity = fp1.similarity(&fp2);
        assert!(similarity > 0.0 && similarity < 1.0);
    }

    #[test]
    fn test_correlation_context() {
        let context = CorrelationContext::new(
            "corr-123".to_string(),
            "req-456".to_string(),
            "routing".to_string(),
        );
        
        assert_eq!(context.correlation_id, "corr-123");
        assert_eq!(context.stage, "routing");
    }

    #[test]
    fn test_correlation_context_with_metadata() {
        let context = CorrelationContext::new(
            "corr-123".to_string(),
            "req-456".to_string(),
            "routing".to_string(),
        ).with_metadata("key".to_string(), "value".to_string());
        
        assert!(context.metadata.contains_key("key"));
    }

    #[test]
    fn test_correlation_context_next_stage() {
        let context = CorrelationContext::new(
            "corr-123".to_string(),
            "req-456".to_string(),
            "routing".to_string(),
        ).with_metadata("key".to_string(), "value".to_string());
        
        let next_context = context.for_next_stage("processing".to_string());
        assert_eq!(next_context.stage, "processing");
        assert_eq!(next_context.correlation_id, context.correlation_id);
        assert!(next_context.metadata.contains_key("key"));
    }
}