use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// Content hashing utilities for request fingerprinting and correlation
/// 
/// This module provides functions to generate consistent hashes of request content
/// for tracking requests through the analytics pipeline without exposing sensitive data.

/// Generate a consistent hash fingerprint for request content
/// 
/// # Arguments
/// * `content` - The content to hash (typically request body or key parameters)
/// 
/// # Returns
/// A hexadecimal string representing the SHA-256 hash of the content
/// 
/// # Example
/// ```
/// use analytics_rs::hashing::content_hash;
/// 
/// let hash = content_hash("Hello, world!");
/// assert!(hash.len() == 64); // SHA-256 produces 64 hex characters
/// ```
pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Generate a hash for structured data (JSON serializable)
/// 
/// # Arguments
/// * `data` - Any data that can be serialized to JSON
/// 
/// # Returns
/// A hexadecimal string representing the SHA-256 hash of the JSON representation
/// 
/// # Example
/// ```
/// use analytics_rs::hashing::structured_hash;
/// use serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct Request {
///     model: String,
///     prompt: String,
/// }
/// 
/// let req = Request {
///     model: "claude-3".to_string(),
///     prompt: "Hello".to_string(),
/// };
/// 
/// let hash = structured_hash(&req).unwrap();
/// ```
pub fn structured_hash<T: Serialize>(data: &T) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(data)?;
    Ok(content_hash(&json))
}

/// Generate a correlation ID from multiple request components
/// 
/// This is useful for creating unique identifiers that combine multiple
/// aspects of a request (e.g., model + prompt + parameters) while
/// maintaining consistency across pipeline stages.
/// 
/// # Arguments
/// * `components` - Vector of string components to hash together
/// 
/// # Returns
/// A hexadecimal string representing the combined hash
/// 
/// # Example
/// ```
/// use analytics_rs::hashing::correlation_id;
/// 
/// let corr_id = correlation_id(&vec![
///     "claude-3-opus",
///     "What is AI?",
///     "temperature:0.7"
/// ]);
/// ```
pub fn correlation_id(components: &[&str]) -> String {
    let combined = components.join("||");
    content_hash(&combined)
}

/// Hash collision detection and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashCollisionMonitor {
    total_hashes: u64,
    detected_collisions: u64,
}

impl HashCollisionMonitor {
    pub fn new() -> Self {
        Self {
            total_hashes: 0,
            detected_collisions: 0,
        }
    }

    /// Record a hash and check for collisions
    /// 
    /// In production, this would maintain a set of seen hashes.
    /// For this implementation, we just track statistics.
    pub fn record_hash(&mut self, _hash: &str) -> bool {
        self.total_hashes += 1;
        // In a real implementation, we'd check against a HashSet
        // For now, we assume no collisions for the example
        false
    }

    /// Get collision rate (collisions per 1000 hashes)
    pub fn collision_rate(&self) -> f64 {
        if self.total_hashes == 0 {
            0.0
        } else {
            (self.detected_collisions as f64 / self.total_hashes as f64) * 1000.0
        }
    }
    
    /// Get total number of hashes recorded
    pub fn total_hashes(&self) -> u64 {
        self.total_hashes
    }
    
    /// Get number of detected collisions
    pub fn detected_collisions(&self) -> u64 {
        self.detected_collisions
    }
}

impl Default for HashCollisionMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_consistency() {
        let content = "Hello, world!";
        let hash1 = content_hash(content);
        let hash2 = content_hash(content);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_content_hash_uniqueness() {
        let hash1 = content_hash("Hello");
        let hash2 = content_hash("World");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_structured_hash() {
        #[derive(Serialize)]
        struct Data {
            field1: String,
            field2: i32,
        }

        let data = Data {
            field1: "test".to_string(),
            field2: 42,
        };

        let hash = structured_hash(&data).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_correlation_id() {
        let components = vec!["model1", "prompt1", "param1"];
        let id1 = correlation_id(&components);
        let id2 = correlation_id(&components);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_correlation_id_order_matters() {
        let components1 = vec!["a", "b", "c"];
        let components2 = vec!["c", "b", "a"];
        assert_ne!(correlation_id(&components1), correlation_id(&components2));
    }

    #[test]
    fn test_hash_collision_monitor() {
        let mut monitor = HashCollisionMonitor::new();
        assert_eq!(monitor.collision_rate(), 0.0);
        
        monitor.record_hash("hash1");
        monitor.record_hash("hash2");
        assert_eq!(monitor.total_hashes, 2);
    }
}
