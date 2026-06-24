//! Query caching for time-series storage
//!
//! This module provides query result caching to improve response time
//! for repeated time-series queries.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Time-to-live for cache entries
    pub ttl: Duration,
    /// Enable automatic cache cleanup
    pub enable_cleanup: bool,
    /// Cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::minutes(5),
            enable_cleanup: true,
            cleanup_interval: Duration::minutes(10),
        }
    }
}

/// Cache key for time-series queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey {
    /// Metric name
    pub metric_name: String,
    /// Query start time
    pub start_time: DateTime<Utc>,
    /// Query end time
    pub end_time: DateTime<Utc>,
    /// Dimension filters
    pub dimension_filters: HashMap<String, String>,
    /// Aggregation window (if applicable)
    pub aggregation_window: Option<String>,
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.metric_name.hash(state);
        self.start_time.timestamp().hash(state);
        self.end_time.timestamp().hash(state);
        
        // Sort dimension filters for consistent hashing
        let mut sorted_filters: Vec<_> = self.dimension_filters.iter().collect();
        sorted_filters.sort_by_key(|(k, _)| *k);
        for (key, value) in sorted_filters {
            key.hash(state);
            value.hash(state);
        }
        
        if let Some(window) = &self.aggregation_window {
            window.hash(state);
        }
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.metric_name == other.metric_name
            && self.start_time == other.start_time
            && self.end_time == other.end_time
            && self.dimension_filters == other.dimension_filters
            && self.aggregation_window == other.aggregation_window
    }
}

impl Eq for CacheKey {}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// Cache key
    pub key: CacheKey,
    /// Cached data
    pub data: T,
    /// Entry creation time
    pub created_at: DateTime<Utc>,
    /// Number of times this entry has been accessed
    pub access_count: u64,
    /// Last access time
    pub last_accessed_at: DateTime<Utc>,
}

/// Query cache for time-series data
pub struct QueryCache<T> {
    cache: Arc<RwLock<HashMap<CacheKey, CacheEntry<T>>>>,
    config: CacheConfig,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> QueryCache<T> {
    /// Create a new query cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new query cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get data from cache
    pub async fn get(&self, key: &CacheKey) -> Option<T> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(key) {
            // Check if entry has expired
            if Utc::now() - entry.created_at > self.config.ttl {
                cache.remove(key);
                return None;
            }

            // Update access statistics
            entry.access_count += 1;
            entry.last_accessed_at = Utc::now();

            Some(entry.data.clone())
        } else {
            None
        }
    }

    /// Put data into cache
    pub async fn put(&self, key: CacheKey, data: T) -> Result<()> {
        let mut cache = self.cache.write().await;

        // Enforce max entries limit
        if cache.len() >= self.config.max_entries {
            self.evict_lru(&mut cache)?;
        }

        let entry = CacheEntry {
            key: key.clone(),
            data,
            created_at: Utc::now(),
            access_count: 0,
            last_accessed_at: Utc::now(),
        };

        cache.insert(key, entry);

        Ok(())
    }

    /// Remove entry from cache
    pub async fn remove(&self, key: &CacheKey) -> Option<T> {
        let mut cache = self.cache.write().await;
        cache.remove(key).map(|entry| entry.data)
    }

    /// Clear all entries from cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Evict least recently used entry
    fn evict_lru(&self, cache: &mut HashMap<CacheKey, CacheEntry<T>>) -> Result<()> {
        if let Some(lru_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed_at)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&lru_key);
        }

        Ok(())
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut cache = self.cache.write().await;
        let now = Utc::now();
        let expired_keys: Vec<CacheKey> = cache
            .iter()
            .filter(|(_, entry)| now - entry.created_at > self.config.ttl)
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            cache.remove(&key);
        }

        Ok(count)
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_entries = cache.len();
        let total_access_count: u64 = cache.values().map(|entry| entry.access_count).sum();

        let now = Utc::now();
        let expired_count = cache
            .values()
            .filter(|entry| now - entry.created_at > self.config.ttl)
            .count();

        CacheStats {
            total_entries,
            max_entries: self.config.max_entries,
            total_access_count,
            expired_count,
            hit_rate: if total_access_count > 0 {
                (total_entries as f64 / total_access_count as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Start automatic cleanup task
    pub async fn start_cleanup_task(&self) -> Result<()> {
        if !self.config.enable_cleanup {
            return Ok(());
        }

        let cache = self.cache.clone();
        let ttl = self.config.ttl;
        let interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);
            loop {
                timer.tick().await;

                let mut cache_guard = cache.write().await;
                let now = Utc::now();
                let expired_keys: Vec<CacheKey> = cache_guard
                    .iter()
                    .filter(|(_, entry)| now - entry.created_at > ttl)
                    .map(|(key, _)| key.clone())
                    .collect();

                for key in expired_keys {
                    cache_guard.remove(&key);
                }

                // Cache cleanup: removed {} expired entries
                let _ = expired_keys.len();
            }
        });

        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of entries in cache
    pub total_entries: usize,
    /// Maximum number of entries allowed
    pub max_entries: usize,
    /// Total number of cache accesses
    pub total_access_count: u64,
    /// Number of expired entries
    pub expired_count: usize,
    /// Cache hit rate (percentage)
    pub hit_rate: f64,
}

/// Cache hit/miss tracker
#[derive(Debug, Clone, Default)]
pub struct CacheTracker {
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl CacheTracker {
    /// Create a new cache tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit
    pub async fn record_hit(&self) {
        let mut hits = self.hits.write().await;
        *hits += 1;
    }

    /// Record a cache miss
    pub async fn record_miss(&self) {
        let mut misses = self.misses.write().await;
        *misses += 1;
    }

    /// Get hit rate
    pub async fn hit_rate(&self) -> f64 {
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Get total hits
    pub async fn hits(&self) -> u64 {
        *self.hits.read().await
    }

    /// Get total misses
    pub async fn misses(&self) -> u64 {
        *self.misses.read().await
    }

    /// Reset statistics
    pub async fn reset(&self) {
        let mut hits = self.hits.write().await;
        let mut misses = self.misses.write().await;
        *hits = 0;
        *misses = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.ttl, Duration::minutes(5));
        assert!(config.enable_cleanup);
    }

    #[test]
    fn test_cache_key_hash() {
        let key1 = CacheKey {
            metric_name: "cpu_usage".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: HashMap::new(),
            aggregation_window: None,
        };

        let key2 = CacheKey {
            metric_name: "cpu_usage".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: HashMap::new(),
            aggregation_window: None,
        };

        // Same keys should have same hash
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CacheKey {
            metric_name: "cpu_usage".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: HashMap::new(),
            aggregation_window: None,
        };

        let key2 = CacheKey {
            metric_name: "cpu_usage".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: HashMap::new(),
            aggregation_window: None,
        };

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_tracker() {
        let tracker = CacheTracker::new();
        
        // In a real async test, we would use tokio::test
        // For now, just verify the structure
        assert_eq!(tracker.hits().await, 0);
        assert_eq!(tracker.misses().await, 0);
    }

    #[test]
    fn test_cache_key_serialization() {
        let key = CacheKey {
            metric_name: "test_metric".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: {
                let mut map = HashMap::new();
                map.insert("host".to_string(), "server1".to_string());
                map
            },
            aggregation_window: Some("1h".to_string()),
        };

        let serialized = serde_json::to_string(&key).unwrap();
        let deserialized: CacheKey = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.metric_name, key.metric_name);
        assert_eq!(deserialized.aggregation_window, key.aggregation_window);
    }

    #[test]
    fn test_cache_key_with_dimensions() {
        let mut dimensions = HashMap::new();
        dimensions.insert("host".to_string(), "server1".to_string());
        dimensions.insert("region".to_string(), "us-west".to_string());

        let key = CacheKey {
            metric_name: "cpu_usage".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: dimensions,
            aggregation_window: None,
        };

        assert_eq!(key.dimension_filters.len(), 2);
    }

    #[test]
    fn test_cache_config_custom() {
        let config = CacheConfig {
            max_entries: 500,
            ttl: Duration::minutes(10),
            enable_cleanup: false,
            cleanup_interval: Duration::minutes(5),
        };

        assert_eq!(config.max_entries, 500);
        assert_eq!(config.ttl, Duration::minutes(10));
        assert!(!config.enable_cleanup);
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            total_entries: 100,
            max_entries: 1000,
            total_access_count: 1000,
            expired_count: 10,
            hit_rate: 75.0,
        };

        assert_eq!(stats.total_entries, 100);
        assert_eq!(stats.hit_rate, 75.0);
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry {
            key: CacheKey {
                metric_name: "test".to_string(),
                start_time: Utc::now(),
                end_time: Utc::now(),
                dimension_filters: HashMap::new(),
                aggregation_window: None,
            },
            data: vec![1, 2, 3],
            created_at: Utc::now(),
            access_count: 5,
            last_accessed_at: Utc::now(),
        };

        assert_eq!(entry.access_count, 5);
        assert_eq!(entry.data.len(), 3);
    }

    #[test]
    fn test_cache_key_inequality() {
        let key1 = CacheKey {
            metric_name: "cpu_usage".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: HashMap::new(),
            aggregation_window: None,
        };

        let key2 = CacheKey {
            metric_name: "memory_usage".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            dimension_filters: HashMap::new(),
            aggregation_window: None,
        };

        assert_ne!(key1, key2);
    }
}
