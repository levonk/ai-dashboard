use analytics_rs::{Timer, TimingStats};
use anyhow::Result;
use std::time::Duration;

/// Timing metrics collection
/// Collects request duration, provider latency, and timing statistics
pub struct TimingCollector {
    request_timer: Timer,
    provider_timer: Timer,
}

impl TimingCollector {
    /// Create a new timing collector
    pub fn new() -> Self {
        Self {
            request_timer: Timer::new(),
            provider_timer: Timer::new(),
        }
    }

    /// Start timing a request
    pub fn start_request(&mut self) {
        self.request_timer.start_timer();
    }

    /// Start timing provider interaction
    pub fn start_provider(&mut self) {
        self.provider_timer.start_timer();
    }

    /// Stop timing provider interaction
    pub fn stop_provider(&mut self) {
        self.provider_timer.stop();
    }

    /// Get request duration in milliseconds
    pub fn request_duration_ms(&self) -> Option<u64> {
        if self.request_timer.is_running() {
            Some(self.request_timer.elapsed_millis() as u64)
        } else {
            Some(self.request_timer.elapsed_millis() as u64)
        }
    }

    /// Get provider latency in milliseconds
    pub fn provider_latency_ms(&self) -> Option<u64> {
        if self.provider_timer.is_running() {
            Some(self.provider_timer.elapsed_millis() as u64)
        } else {
            Some(self.provider_timer.elapsed_millis() as u64)
        }
    }

    /// Calculate total request duration
    pub fn calculate_request_duration(&self) -> Duration {
        self.request_timer.elapsed()
    }

    /// Calculate provider latency
    pub fn calculate_provider_latency(&self) -> Duration {
        self.provider_timer.elapsed()
    }

    /// Collect timing statistics for a request
    pub fn collect_timing_stats(&self) -> Result<TimingStats> {
        let request_duration = self.calculate_request_duration();
        let _provider_latency = self.calculate_provider_latency();

        let mut stats = TimingStats::new();
        stats.record(request_duration);

        Ok(stats)
    }

    /// Reset all timing measurements
    pub fn reset(&mut self) {
        self.request_timer.reset();
        self.provider_timer.reset();
    }
}

impl Default for TimingCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Timing metrics for a single request
#[derive(Debug, Clone)]
pub struct RequestTiming {
    pub request_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: u64,
    pub provider_latency_ms: Option<u64>,
    pub processing_overhead_ms: u64,
}

impl RequestTiming {
    /// Create a new request timing record
    pub fn new(request_id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            request_id,
            start_time: now,
            end_time: None,
            duration_ms: 0,
            provider_latency_ms: None,
            processing_overhead_ms: 0,
        }
    }

    /// Mark the request as completed
    pub fn complete(&mut self, duration_ms: u64, provider_latency_ms: Option<u64>) {
        self.end_time = Some(chrono::Utc::now());
        self.duration_ms = duration_ms;
        self.provider_latency_ms = provider_latency_ms;
        self.processing_overhead_ms = if let Some(latency) = provider_latency_ms {
            duration_ms.saturating_sub(latency)
        } else {
            0
        };
    }

    /// Calculate processing overhead
    pub fn calculate_overhead(&self) -> u64 {
        if let Some(latency) = self.provider_latency_ms {
            self.duration_ms.saturating_sub(latency)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_collector_creation() {
        let collector = TimingCollector::new();
        assert!(collector.request_start.is_none());
        assert!(collector.provider_start.is_none());
    }

    #[test]
    fn test_start_request() {
        let mut collector = TimingCollector::new();
        collector.start_request();
        assert!(collector.request_start.is_some());
    }

    #[test]
    fn test_start_provider() {
        let mut collector = TimingCollector::new();
        collector.start_provider();
        assert!(collector.provider_start.is_some());
    }

    #[test]
    fn test_request_duration() {
        let mut collector = TimingCollector::new();
        collector.start_request();
        std::thread::sleep(Duration::from_millis(10));
        
        let duration = collector.calculate_request_duration();
        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_provider_latency() {
        let mut collector = TimingCollector::new();
        collector.start_provider();
        std::thread::sleep(Duration::from_millis(5));
        
        let latency = collector.calculate_provider_latency();
        assert!(latency.as_millis() >= 5);
    }

    #[test]
    fn test_collect_timing_stats() {
        let mut collector = TimingCollector::new();
        collector.start_request();
        collector.start_provider();
        std::thread::sleep(Duration::from_millis(10));
        
        let stats = collector.collect_timing_stats().unwrap();
        assert!(stats.total_duration.as_millis() >= 10);
    }

    #[test]
    fn test_reset() {
        let mut collector = TimingCollector::new();
        collector.start_request();
        collector.start_provider();
        
        collector.reset();
        assert!(collector.request_start.is_none());
        assert!(collector.provider_start.is_none());
    }

    #[test]
    fn test_request_timing_creation() {
        let timing = RequestTiming::new("test-request".to_string());
        assert_eq!(timing.request_id, "test-request");
        assert!(timing.end_time.is_none());
        assert_eq!(timing.duration_ms, 0);
    }

    #[test]
    fn test_request_timing_complete() {
        let mut timing = RequestTiming::new("test-request".to_string());
        timing.complete(100, Some(50));
        
        assert!(timing.end_time.is_some());
        assert_eq!(timing.duration_ms, 100);
        assert_eq!(timing.provider_latency_ms, Some(50));
        assert_eq!(timing.processing_overhead_ms, 50);
    }

    #[test]
    fn test_request_timing_overhead_calculation() {
        let mut timing = RequestTiming::new("test-request".to_string());
        timing.complete(100, Some(30));
        
        assert_eq!(timing.calculate_overhead(), 70);
    }

    #[test]
    fn test_request_timing_no_provider_latency() {
        let mut timing = RequestTiming::new("test-request".to_string());
        timing.complete(100, None);
        
        assert_eq!(timing.provider_latency_ms, None);
        assert_eq!(timing.processing_overhead_ms, 0);
    }
}