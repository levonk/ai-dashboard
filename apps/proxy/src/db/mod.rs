//! Database module
//!
//! This module provides database connection management, queries, and related utilities
//! for the AI Analytics Dashboard.

pub mod cache;
pub mod cleanup;
pub mod compression;
pub mod connection;
pub mod monitoring;
pub mod performance_tests;
pub mod queries;
pub mod retention;
pub mod seeds;
pub mod timeseries;

pub use cache::{CacheConfig, CacheKey, CacheStats, QueryCache, CacheTracker};
pub use cleanup::{CleanupJobConfig, CleanupJobManager, CleanupJobResult, CleanupStats};
pub use compression::{CompressionAlgorithm, CompressionConfig, CompressionResult, CompressionStats, TimeSeriesCompressor};
pub use connection::{DatabaseConfig, DatabaseManager, PoolStats};
pub use monitoring::{Bottleneck, BottleneckCategory, BottleneckSeverity, OperationStats, OperationTracker, PerformanceSummary, StorageMetrics, StorageMonitor};
pub use performance_tests::{PerformanceTestConfig, PerformanceTestResult, PerformanceTestRunner};
pub use queries::{CompanyQueries, AiClientQueries, AiProviderQueries, RequestEventQueries};
pub use retention::{AggregationPolicy, RetentionPolicy, RetentionPolicyManager, RetentionEnforcementResult, RetentionStats, default_retention_policies};
pub use seeds::seed_database;
pub use timeseries::{TimeSeriesDataPoint, TimeSeriesStorage, TimeSeriesWriteConfig, TimeSeriesAggregate};