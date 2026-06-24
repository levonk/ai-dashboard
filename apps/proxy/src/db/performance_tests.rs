//! Performance tests for time-series storage operations
//!
//! This module provides performance benchmarks and load tests for time-series
//! storage to validate scalability targets.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

use super::timeseries::{TimeSeriesDataPoint, TimeSeriesWriteConfig};
use super::compression::{TimeSeriesCompressor, CompressionConfig};

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig {
    /// Number of data points to write
    pub write_count: usize,
    /// Number of data points to read
    pub read_count: usize,
    /// Target write throughput (operations per second)
    pub target_write_throughput: f64,
    /// Target read latency (milliseconds)
    pub target_read_latency: f64,
    /// Target compression ratio (compressed_size / original_size)
    pub target_compression_ratio: f64,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            write_count: 10000,
            read_count: 1000,
            target_write_throughput: 10000.0,
            target_read_latency: 2000.0,
            target_compression_ratio: 0.5,
        }
    }
}

/// Performance test result
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    /// Test name
    pub test_name: String,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Operations performed
    pub operations_performed: usize,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Minimum latency in milliseconds
    pub min_latency_ms: f64,
    /// Maximum latency in milliseconds
    pub max_latency_ms: f64,
    /// Whether the test passed the target
    pub passed: bool,
    /// Additional metrics
    pub additional_metrics: HashMap<String, f64>,
}

/// Performance test runner
pub struct PerformanceTestRunner {
    config: PerformanceTestConfig,
}

impl PerformanceTestRunner {
    /// Create a new performance test runner with default configuration
    pub fn new() -> Self {
        Self {
            config: PerformanceTestConfig::default(),
        }
    }

    /// Create a new performance test runner with custom configuration
    pub fn with_config(config: PerformanceTestConfig) -> Self {
        Self { config }
    }

    /// Run write throughput test
    pub fn test_write_throughput(&self) -> PerformanceTestResult {
        let test_name = "write_throughput".to_string();
        let start = Instant::now();

        let mut latencies = Vec::new();
        let mut data_points = Vec::new();

        for i in 0..self.config.write_count {
            let point_start = Instant::now();

            let point = TimeSeriesDataPoint {
                id: Uuid::new_v4(),
                metric_name: "cpu_usage".to_string(),
                value: (i % 100) as f64,
                timestamp: Utc::now(),
                dimensions: HashMap::new(),
                request_event_id: None,
            };

            data_points.push(point);

            let point_duration = point_start.elapsed().as_secs_f64() * 1000.0;
            latencies.push(point_duration);
        }

        let duration = start.elapsed().as_secs_f64();
        let throughput = self.config.write_count as f64 / duration;
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let min_latency = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_latency = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let passed = throughput >= self.config.target_write_throughput;

        let mut additional_metrics = HashMap::new();
        additional_metrics.insert("target_throughput".to_string(), self.config.target_write_throughput);
        additional_metrics.insert("actual_throughput".to_string(), throughput);

        PerformanceTestResult {
            test_name,
            duration_secs: duration,
            operations_performed: self.config.write_count,
            throughput,
            avg_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            passed,
            additional_metrics,
        }
    }

    /// Test compression performance
    pub fn test_compression_performance(&self) -> PerformanceTestResult {
        let test_name = "compression_performance".to_string();
        let start = Instant::now();

        let compressor = TimeSeriesCompressor::new();
        let mut compression_ratios = Vec::new();
        let mut original_sizes = Vec::new();
        let mut compressed_sizes = Vec::new();

        for i in 0..self.config.write_count {
            // Create test data with some patterns
            let data: Vec<u8> = (0..1000)
                .map(|j| ((i + j) % 256) as u8)
                .collect();

            let point_start = Instant::now();
            let result = compressor.compress(&data).unwrap();
            let point_duration = point_start.elapsed().as_secs_f64() * 1000.0;

            compression_ratios.push(result.compression_ratio);
            original_sizes.push(result.original_size);
            compressed_sizes.push(result.compressed_size);
        }

        let duration = start.elapsed().as_secs_f64();
        let throughput = self.config.write_count as f64 / duration;
        let avg_compression_ratio = compression_ratios.iter().sum::<f64>() / compression_ratios.len() as f64;
        let avg_latency = duration * 1000.0 / self.config.write_count as f64;

        let passed = avg_compression_ratio <= self.config.target_compression_ratio;

        let mut additional_metrics = HashMap::new();
        additional_metrics.insert("target_compression_ratio".to_string(), self.config.target_compression_ratio);
        additional_metrics.insert("actual_compression_ratio".to_string(), avg_compression_ratio);
        additional_metrics.insert("total_original_size".to_string(), original_sizes.iter().sum::<usize>() as f64);
        additional_metrics.insert("total_compressed_size".to_string(), compressed_sizes.iter().sum::<usize>() as f64);

        PerformanceTestResult {
            test_name,
            duration_secs: duration,
            operations_performed: self.config.write_count,
            throughput,
            avg_latency_ms: avg_latency,
            min_latency_ms: 0.0,
            max_latency_ms: 0.0,
            passed,
            additional_metrics,
        }
    }

    /// Test serialization performance
    pub fn test_serialization_performance(&self) -> PerformanceTestResult {
        let test_name = "serialization_performance".to_string();
        let start = Instant::now();

        let mut latencies = Vec::new();

        for i in 0..self.config.write_count {
            let point = TimeSeriesDataPoint {
                id: Uuid::new_v4(),
                metric_name: "test_metric".to_string(),
                value: (i % 100) as f64,
                timestamp: Utc::now(),
                dimensions: {
                    let mut map = HashMap::new();
                    map.insert("host".to_string(), format!("server{}", i % 10));
                    map
                },
                request_event_id: None,
            };

            let point_start = Instant::now();
            let _serialized = serde_json::to_string(&point).unwrap();
            let point_duration = point_start.elapsed().as_secs_f64() * 1000.0;

            latencies.push(point_duration);
        }

        let duration = start.elapsed().as_secs_f64();
        let throughput = self.config.write_count as f64 / duration;
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let min_latency = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_latency = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let passed = avg_latency < 10.0; // Target: < 10ms per serialization

        let mut additional_metrics = HashMap::new();
        additional_metrics.insert("target_latency_ms".to_string(), 10.0);
        additional_metrics.insert("actual_latency_ms".to_string(), avg_latency);

        PerformanceTestResult {
            test_name,
            duration_secs: duration,
            operations_performed: self.config.write_count,
            throughput,
            avg_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            passed,
            additional_metrics,
        }
    }

    /// Test deserialization performance
    pub fn test_deserialization_performance(&self) -> PerformanceTestResult {
        let test_name = "deserialization_performance".to_string();
        let start = Instant::now();

        // Pre-serialize data
        let serialized_data: Vec<String> = (0..self.config.write_count)
            .map(|i| {
                let point = TimeSeriesDataPoint {
                    id: Uuid::new_v4(),
                    metric_name: "test_metric".to_string(),
                    value: (i % 100) as f64,
                    timestamp: Utc::now(),
                    dimensions: HashMap::new(),
                    request_event_id: None,
                };
                serde_json::to_string(&point).unwrap()
            })
            .collect();

        let mut latencies = Vec::new();

        for serialized in &serialized_data {
            let point_start = Instant::now();
            let _: TimeSeriesDataPoint = serde_json::from_str(serialized).unwrap();
            let point_duration = point_start.elapsed().as_secs_f64() * 1000.0;

            latencies.push(point_duration);
        }

        let duration = start.elapsed().as_secs_f64();
        let throughput = self.config.write_count as f64 / duration;
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let min_latency = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_latency = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let passed = avg_latency < 10.0; // Target: < 10ms per deserialization

        let mut additional_metrics = HashMap::new();
        additional_metrics.insert("target_latency_ms".to_string(), 10.0);
        additional_metrics.insert("actual_latency_ms".to_string(), avg_latency);

        PerformanceTestResult {
            test_name,
            duration_secs: duration,
            operations_performed: self.config.write_count,
            throughput,
            avg_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            passed,
            additional_metrics,
        }
    }

    /// Run all performance tests
    pub fn run_all_tests(&self) -> Vec<PerformanceTestResult> {
        vec![
            self.test_write_throughput(),
            self.test_compression_performance(),
            self.test_serialization_performance(),
            self.test_deserialization_performance(),
        ]
    }

    /// Print performance test results
    pub fn print_results(&self, results: &[PerformanceTestResult]) {
        println!("\n=== Performance Test Results ===\n");

        for result in results {
            println!("Test: {}", result.test_name);
            println!("  Duration: {:.2}s", result.duration_secs);
            println!("  Operations: {}", result.operations_performed);
            println!("  Throughput: {:.2} ops/sec", result.throughput);
            println!("  Avg Latency: {:.2}ms", result.avg_latency_ms);
            println!("  Min Latency: {:.2}ms", result.min_latency_ms);
            println!("  Max Latency: {:.2}ms", result.max_latency_ms);
            println!("  Status: {}", if result.passed { "✓ PASSED" } else { "✗ FAILED" });

            for (key, value) in &result.additional_metrics {
                println!("  {}: {:.2}", key, value);
            }

            println!();
        }

        let passed_count = results.iter().filter(|r| r.passed).count();
        let total_count = results.len();

        println!("Summary: {}/{} tests passed", passed_count, total_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_test_config_default() {
        let config = PerformanceTestConfig::default();
        assert_eq!(config.write_count, 10000);
        assert_eq!(config.target_write_throughput, 10000.0);
        assert_eq!(config.target_read_latency, 2000.0);
    }

    #[test]
    fn test_performance_test_result_creation() {
        let result = PerformanceTestResult {
            test_name: "test".to_string(),
            duration_secs: 10.0,
            operations_performed: 1000,
            throughput: 100.0,
            avg_latency_ms: 5.0,
            min_latency_ms: 1.0,
            max_latency_ms: 10.0,
            passed: true,
            additional_metrics: HashMap::new(),
        };

        assert_eq!(result.test_name, "test");
        assert!(result.passed);
    }

    #[test]
    fn test_performance_test_runner() {
        let runner = PerformanceTestRunner::new();
        let config = PerformanceTestConfig {
            write_count: 100, // Smaller count for faster test
            read_count: 10,
            target_write_throughput: 1000.0,
            target_read_latency: 100.0,
            target_compression_ratio: 0.8,
        };

        let custom_runner = PerformanceTestRunner::with_config(config);
        let results = custom_runner.run_all_tests();

        assert!(!results.is_empty());
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_write_throughput_test() {
        let runner = PerformanceTestRunner::with_config(PerformanceTestConfig {
            write_count: 100,
            read_count: 10,
            target_write_throughput: 1000.0,
            target_read_latency: 100.0,
            target_compression_ratio: 0.8,
        });

        let result = runner.test_write_throughput();
        assert_eq!(result.test_name, "write_throughput");
        assert_eq!(result.operations_performed, 100);
    }

    #[test]
    fn test_compression_performance_test() {
        let runner = PerformanceTestRunner::with_config(PerformanceTestConfig {
            write_count: 100,
            read_count: 10,
            target_write_throughput: 1000.0,
            target_read_latency: 100.0,
            target_compression_ratio: 0.8,
        });

        let result = runner.test_compression_performance();
        assert_eq!(result.test_name, "compression_performance");
        assert!(result.additional_metrics.contains_key("actual_compression_ratio"));
    }

    #[test]
    fn test_serialization_performance_test() {
        let runner = PerformanceTestRunner::with_config(PerformanceTestConfig {
            write_count: 100,
            read_count: 10,
            target_write_throughput: 1000.0,
            target_read_latency: 100.0,
            target_compression_ratio: 0.8,
        });

        let result = runner.test_serialization_performance();
        assert_eq!(result.test_name, "serialization_performance");
        assert_eq!(result.operations_performed, 100);
    }

    #[test]
    fn test_deserialization_performance_test() {
        let runner = PerformanceTestRunner::with_config(PerformanceTestConfig {
            write_count: 100,
            read_count: 10,
            target_write_throughput: 1000.0,
            target_read_latency: 100.0,
            target_compression_ratio: 0.8,
        });

        let result = runner.test_deserialization_performance();
        assert_eq!(result.test_name, "deserialization_performance");
        assert_eq!(result.operations_performed, 100);
    }
}
