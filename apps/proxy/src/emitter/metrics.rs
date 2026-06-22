//! Emitter metrics and monitoring
//! 
//! This module provides metrics collection and monitoring for emitter mode,
//! tracking performance, health, and operational statistics.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::info;

use super::{client::CircuitBreakerState, queue::QueueStats, fallback::FallbackStats};

/// Emitter metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterMetrics {
    /// Metrics timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Total events processed
    pub total_events: u64,
    
    /// Successful emissions
    pub successful_emissions: u64,
    
    /// Failed emissions
    pub failed_emissions: u64,
    
    /// Current emission rate (events/second)
    pub emission_rate: f64,
    
    /// Average emission latency in milliseconds
    pub avg_latency_ms: f64,
    
    /// P50 latency in milliseconds
    pub p50_latency_ms: f64,
    
    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,
    
    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,
    
    /// Circuit breaker state
    pub circuit_breaker_state: String,
    
    /// Queue statistics
    pub queue_stats: Option<QueueStats>,
    
    /// Fallback buffer statistics
    pub fallback_stats: Option<FallbackStats>,
    
    /// Active connections
    pub active_connections: usize,
    
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

impl Default for EmitterMetrics {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            total_events: 0,
            successful_emissions: 0,
            failed_emissions: 0,
            emission_rate: 0.0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            circuit_breaker_state: "closed".to_string(),
            queue_stats: None,
            fallback_stats: None,
            active_connections: 0,
            memory_usage_bytes: 0,
            uptime_seconds: 0,
        }
    }
}

/// Metrics collector for emitter mode
pub struct MetricsCollector {
    metrics: Arc<RwLock<EmitterMetrics>>,
    start_time: Instant,
    recent_events: Arc<RwLock<Vec<Instant>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        info!("Created emitter metrics collector");
        
        Self {
            metrics: Arc::new(RwLock::new(EmitterMetrics::default())),
            start_time: Instant::now(),
            recent_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a successful event emission
    pub async fn record_success(&self, latency_ms: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_events += 1;
        metrics.successful_emissions += 1;
        metrics.timestamp = chrono::Utc::now();
        
        // Simple latency tracking
        metrics.avg_latency_ms = (metrics.avg_latency_ms * (metrics.successful_emissions - 1) as f64 + latency_ms) / metrics.successful_emissions as f64;
        
        // Track recent events for rate calculation
        let mut recent = self.recent_events.write().await;
        recent.push(Instant::now());
        // Keep only last 60 seconds of events
        recent.retain(|t| t.elapsed() < Duration::from_secs(60));
        
        // Calculate emission rate
        metrics.emission_rate = recent.len() as f64;
    }

    /// Record a failed event emission
    pub async fn record_failure(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_events += 1;
        metrics.failed_emissions += 1;
        metrics.timestamp = chrono::Utc::now();
    }

    /// Update circuit breaker state
    pub async fn update_circuit_breaker_state(&self, state: CircuitBreakerState) {
        let mut metrics = self.metrics.write().await;
        metrics.circuit_breaker_state = match state {
            CircuitBreakerState::Closed => "closed".to_string(),
            CircuitBreakerState::Open => "open".to_string(),
            CircuitBreakerState::HalfOpen => "half_open".to_string(),
        };
    }

    /// Update queue statistics
    pub async fn update_queue_stats(&self, stats: QueueStats) {
        let mut metrics = self.metrics.write().await;
        metrics.queue_stats = Some(stats);
    }

    /// Update fallback buffer statistics
    pub async fn update_fallback_stats(&self, stats: FallbackStats) {
        let mut metrics = self.metrics.write().await;
        metrics.fallback_stats = Some(stats);
    }

    /// Update active connections count
    pub async fn update_active_connections(&self, count: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.active_connections = count;
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self, bytes: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.memory_usage_bytes = bytes;
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> EmitterMetrics {
        let mut metrics = self.metrics.write().await;
        
        // Update uptime
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        
        metrics.clone()
    }

    /// Reset metrics
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = EmitterMetrics::default();
        metrics.timestamp = chrono::Utc::now();
        
        let mut recent = self.recent_events.write().await;
        recent.clear();
        
        info!("Emitter metrics reset");
    }

    /// Get success rate (0.0 to 1.0)
    pub async fn success_rate(&self) -> f64 {
        let metrics = self.metrics.read().await;
        if metrics.total_events == 0 {
            return 1.0;
        }
        metrics.successful_emissions as f64 / metrics.total_events as f64
    }

    /// Get failure rate (0.0 to 1.0)
    pub async fn failure_rate(&self) -> f64 {
        let metrics = self.metrics.read().await;
        if metrics.total_events == 0 {
            return 0.0;
        }
        metrics.failed_emissions as f64 / metrics.total_events as f64
    }

    /// Check if emitter is healthy
    pub async fn is_healthy(&self) -> bool {
        let metrics = self.metrics.read().await;
        
        // Check circuit breaker state
        if metrics.circuit_breaker_state == "open" {
            return false;
        }
        
        // Check failure rate (threshold: 50%)
        let failure_rate = if metrics.total_events > 0 {
            metrics.failed_emissions as f64 / metrics.total_events as f64
        } else {
            0.0
        };
        
        if failure_rate > 0.5 {
            return false;
        }
        
        // Check if queue is available (if configured)
        if let Some(ref queue_stats) = metrics.queue_stats {
            if !queue_stats.connected {
                return false;
            }
        }
        
        true
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.metrics.read().await;
        
        let mut output = String::new();
        
        output.push_str("# HELP emitter_total_events Total number of events processed\n");
        output.push_str("# TYPE emitter_total_events counter\n");
        output.push_str(&format!("emitter_total_events {}\n", metrics.total_events));
        
        output.push_str("\n# HELP emitter_successful_emissions Total successful emissions\n");
        output.push_str("# TYPE emitter_successful_emissions counter\n");
        output.push_str(&format!("emitter_successful_emissions {}\n", metrics.successful_emissions));
        
        output.push_str("\n# HELP emitter_failed_emissions Total failed emissions\n");
        output.push_str("# TYPE emitter_failed_emissions counter\n");
        output.push_str(&format!("emitter_failed_emissions {}\n", metrics.failed_emissions));
        
        output.push_str("\n# HELP emitter_emission_rate Current emission rate (events/second)\n");
        output.push_str("# TYPE emitter_emission_rate gauge\n");
        output.push_str(&format!("emitter_emission_rate {}\n", metrics.emission_rate));
        
        output.push_str("\n# HELP emitter_avg_latency_ms Average emission latency in milliseconds\n");
        output.push_str("# TYPE emitter_avg_latency_ms gauge\n");
        output.push_str(&format!("emitter_avg_latency_ms {}\n", metrics.avg_latency_ms));
        
        output.push_str("\n# HELP emitter_p95_latency_ms P95 emission latency in milliseconds\n");
        output.push_str("# TYPE emitter_p95_latency_ms gauge\n");
        output.push_str(&format!("emitter_p95_latency_ms {}\n", metrics.p95_latency_ms));
        
        output.push_str("\n# HELP emitter_active_connections Current number of active connections\n");
        output.push_str("# TYPE emitter_active_connections gauge\n");
        output.push_str(&format!("emitter_active_connections {}\n", metrics.active_connections));
        
        output.push_str("\n# HELP emitter_memory_usage_bytes Current memory usage in bytes\n");
        output.push_str("# TYPE emitter_memory_usage_bytes gauge\n");
        output.push_str(&format!("emitter_memory_usage_bytes {}\n", metrics.memory_usage_bytes));
        
        output.push_str("\n# HELP emitter_uptime_seconds Uptime in seconds\n");
        output.push_str("# TYPE emitter_uptime_seconds gauge\n");
        output.push_str(&format!("emitter_uptime_seconds {}\n", metrics.uptime_seconds));
        
        if let Some(ref queue_stats) = metrics.queue_stats {
            output.push_str("\n# HELP emitter_queue_depth Current queue depth\n");
            output.push_str("# TYPE emitter_queue_depth gauge\n");
            output.push_str(&format!("emitter_queue_depth {}\n", queue_stats.queue_depth));
        }
        
        if let Some(ref fallback_stats) = metrics.fallback_stats {
            output.push_str("\n# HELP emitter_fallback_size Current fallback buffer size in bytes\n");
            output.push_str("# TYPE emitter_fallback_size gauge\n");
            output.push_str(&format!("emitter_fallback_size {}\n", fallback_stats.current_size));
            
            output.push_str("\n# HELP emitter_fallback_event_count Number of events in fallback buffer\n");
            output.push_str("# TYPE emitter_fallback_event_count gauge\n");
            output.push_str(&format!("emitter_fallback_event_count {}\n", fallback_stats.event_count));
        }
        
        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_metrics_default() {
        let metrics = EmitterMetrics::default();
        assert_eq!(metrics.total_events, 0);
        assert_eq!(metrics.successful_emissions, 0);
        assert_eq!(metrics.failed_emissions, 0);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.record_success(10.0).await;
        collector.record_success(20.0).await;
        collector.record_failure().await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_events, 3);
        assert_eq!(metrics.successful_emissions, 2);
        assert_eq!(metrics.failed_emissions, 1);
        assert_eq!(metrics.avg_latency_ms, 15.0); // (10.0 + 20.0) / 2
    }

    #[tokio::test]
    async fn test_success_rate() {
        let collector = MetricsCollector::new();
        
        assert_eq!(collector.success_rate().await, 1.0);
        
        collector.record_success(10.0).await;
        collector.record_failure().await;
        
        assert_eq!(collector.success_rate().await, 0.5);
    }

    #[tokio::test]
    async fn test_health_check() {
        let collector = MetricsCollector::new();
        
        assert!(collector.is_healthy().await);
        
        collector.update_circuit_breaker_state(CircuitBreakerState::Open).await;
        assert!(!collector.is_healthy().await);
    }
}
