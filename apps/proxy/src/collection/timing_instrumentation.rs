use crate::collection::{AIMetricsCollector, TimingCollector};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Request timing instrumentation
/// Provides high-resolution timing for request lifecycle stages
pub struct RequestTimingInstrumentation {
    request_id: String,
    start_time: DateTime<Utc>,
    ai_metrics: AIMetricsCollector,
    timing_collector: TimingCollector,
    stages: Vec<TimingStage>,
}

/// Individual timing stage for request lifecycle
#[derive(Debug, Clone)]
pub struct TimingStage {
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<f64>,
}

impl RequestTimingInstrumentation {
    /// Create new timing instrumentation for a request
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            start_time: Utc::now(),
            ai_metrics: AIMetricsCollector::new(request_id.clone()),
            timing_collector: TimingCollector::new(),
            stages: Vec::new(),
        }
    }

    /// Start a specific timing stage
    pub fn start_stage(&mut self, name: String) {
        self.stages.push(TimingStage {
            name,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
        });
    }

    /// End the most recent timing stage
    pub fn end_current_stage(&mut self) -> Result<()> {
        if let Some(stage) = self.stages.last_mut() {
            stage.end_time = Some(Utc::now());
            stage.duration_ms = Some(
                stage.end_time.unwrap().signed_duration_since(stage.start_time).num_milliseconds() as f64
            );
            Ok(())
        } else {
            anyhow::bail!("No active stage to end")
        }
    }

    /// Get AI metrics collector for direct access
    pub fn ai_metrics(&mut self) -> &mut AIMetricsCollector {
        &mut self.ai_metrics
    }

    /// Get timing collector for direct access
    pub fn timing(&mut self) -> &mut TimingCollector {
        &mut self.timing_collector
    }

    /// Start total request timing
    pub fn start_request(&mut self) {
        self.timing_collector.start_request();
        self.ai_metrics.start_total_timing();
    }

    /// Start provider timing
    pub fn start_provider(&mut self) {
        self.timing_collector.start_provider();
    }

    /// Stop provider timing
    pub fn stop_provider(&mut self) {
        self.timing_collector.stop_provider();
    }

    /// Start TTFT timing
    pub fn start_ttft(&mut self) {
        self.ai_metrics.start_ttft();
    }

    /// Mark first token received
    pub fn mark_first_token(&mut self) {
        self.ai_metrics.mark_first_token();
    }

    /// Start prefill timing
    pub fn start_prefill(&mut self) {
        self.ai_metrics.start_prefill();
    }

    /// Stop prefill timing
    pub fn stop_prefill(&mut self) {
        self.ai_metrics.stop_prefill();
    }

    /// Start decode timing
    pub fn start_decode(&mut self) {
        self.ai_metrics.start_decode();
    }

    /// Stop decode timing
    pub fn stop_decode(&mut self) {
        self.ai_metrics.stop_decode();
    }

    /// Stop total request timing
    pub fn stop_request(&mut self) {
        self.timing_collector.stop();
        self.ai_metrics.stop_total_timing();
    }

    /// Collect all timing metrics
    pub fn collect_timing_summary(&self) -> Result<TimingSummary> {
        Ok(TimingSummary {
            request_id: self.request_id.clone(),
            start_time: self.start_time,
            end_time: Utc::now(),
            total_duration_ms: self.timing_collector.request_duration_ms(),
            provider_latency_ms: self.timing_collector.provider_latency_ms(),
            ai_metrics: self.ai_metrics.collect_metrics()?,
            stages: self.stages.clone(),
        })
    }

    /// Get current elapsed time since start
    pub fn elapsed_ms(&self) -> f64 {
        Utc::now().signed_duration_since(self.start_time).num_milliseconds() as f64
    }
}

/// Summary of all timing metrics for a request
#[derive(Debug, Clone)]
pub struct TimingSummary {
    pub request_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_duration_ms: Option<u64>,
    pub provider_latency_ms: Option<u64>,
    pub ai_metrics: crate::collection::AIMetrics,
    pub stages: Vec<TimingStage>,
}

impl TimingSummary {
    /// Calculate processing overhead (total - provider latency)
    pub fn processing_overhead_ms(&self) -> Option<u64> {
        match (self.total_duration_ms, self.provider_latency_ms) {
            (Some(total), Some(latency)) => Some(total.saturating_sub(latency)),
            _ => None,
        }
    }

    /// Get duration of a specific stage
    pub fn stage_duration(&self, stage_name: &str) -> Option<f64> {
        self.stages.iter()
            .find(|s| s.name == stage_name)
            .and_then(|s| s.duration_ms)
    }
}

/// Shared timing instrumentation for concurrent access
pub type SharedTimingInstrumentation = Arc<RwLock<RequestTimingInstrumentation>>;

/// Create shared timing instrumentation
pub fn create_shared_timing(request_id: String) -> SharedTimingInstrumentation {
    Arc::new(RwLock::new(RequestTimingInstrumentation::new(request_id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_instrumentation_creation() {
        let timing = RequestTimingInstrumentation::new("test-request".to_string());
        assert_eq!(timing.request_id, "test-request");
        assert!(timing.stages.is_empty());
    }

    #[test]
    fn test_stage_timing() {
        let mut timing = RequestTimingInstrumentation::new("test-request".to_string());
        timing.start_stage("test-stage".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        timing.end_current_stage().unwrap();
        
        assert_eq!(timing.stages.len(), 1);
        assert!(timing.stages[0].duration_ms.unwrap() >= 10.0);
    }

    #[test]
    fn test_multiple_stages() {
        let mut timing = RequestTimingInstrumentation::new("test-request".to_string());
        
        timing.start_stage("stage1".to_string());
        std::thread::sleep(std::time::Duration::from_millis(5));
        timing.end_current_stage().unwrap();
        
        timing.start_stage("stage2".to_string());
        std::thread::sleep(std::time::Duration::from_millis(5));
        timing.end_current_stage().unwrap();
        
        assert_eq!(timing.stages.len(), 2);
        assert_eq!(timing.stages[0].name, "stage1");
        assert_eq!(timing.stages[1].name, "stage2");
    }

    #[test]
    fn test_ai_metrics_integration() {
        let mut timing = RequestTimingInstrumentation::new("test-request".to_string());
        timing.start_request();
        timing.start_ttft();
        std::thread::sleep(std::time::Duration::from_millis(10));
        timing.mark_first_token();
        timing.stop_request();
        
        let summary = timing.collect_timing_summary().unwrap();
        assert!(summary.ai_metrics.ttft_ms.is_some());
        assert!(summary.ai_metrics.ttft_ms.unwrap() >= 10.0);
    }

    #[test]
    fn test_timing_summary() {
        let mut timing = RequestTimingInstrumentation::new("test-request".to_string());
        timing.start_request();
        timing.start_provider();
        std::thread::sleep(std::time::Duration::from_millis(20));
        timing.stop_provider();
        timing.stop_request();
        
        let summary = timing.collect_timing_summary().unwrap();
        assert!(summary.total_duration_ms.is_some());
        assert!(summary.provider_latency_ms.is_some());
    }

    #[test]
    fn test_processing_overhead() {
        let mut timing = RequestTimingInstrumentation::new("test-request".to_string());
        timing.start_request();
        timing.start_provider();
        std::thread::sleep(std::time::Duration::from_millis(30));
        timing.stop_provider();
        timing.stop_request();
        
        let summary = timing.collect_timing_summary().unwrap();
        let overhead = summary.processing_overhead_ms();
        assert!(overhead.is_some());
        // Overhead should be less than total duration
        assert!(overhead.unwrap() <= summary.total_duration_ms.unwrap());
    }

    #[test]
    fn test_elapsed_time() {
        let timing = RequestTimingInstrumentation::new("test-request".to_string());
        std::thread::sleep(std::time::Duration::from_millis(15));
        let elapsed = timing.elapsed_ms();
        assert!(elapsed >= 15.0);
    }

    #[test]
    fn test_stage_duration_lookup() {
        let mut timing = RequestTimingInstrumentation::new("test-request".to_string());
        timing.start_stage("lookup-test".to_string());
        std::thread::sleep(std::time::Duration::from_millis(8));
        timing.end_current_stage().unwrap();
        
        let summary = timing.collect_timing_summary().unwrap();
        let duration = summary.stage_duration("lookup-test");
        assert!(duration.is_some());
        assert!(duration.unwrap() >= 8.0);
    }
}
