pub mod aggregation;
pub mod cost;
pub mod filtering;
pub mod metadata;
pub mod models;
pub mod processing;
pub mod time_series;

pub use aggregation::Aggregator;
pub use cost::CostCalculator;
pub use filtering::FilterEngine;
pub use metadata::{Metadata, ValidationResult, validate_metadata, MetadataMigrator, MigrationError, MetadataEnricher, EnrichmentError};
pub use models::{TelemetryEvent, AnalyticsQuery, AnalyticsResult, Filter};
pub use processing::Processor;
pub use time_series::TimeSeriesAnalyzer;