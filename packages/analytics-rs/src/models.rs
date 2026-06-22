use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub ai_client: String,
    pub ai_provider: String,
    pub model: String,
    pub input_type: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub duration_ms: u64,
    pub cost_usd: f64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub filters: Vec<Filter>,
    pub time_range: TimeRange,
    pub aggregation: AggregationType,
    pub group_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub data: serde_json::Value,
    pub metadata: QueryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub query_duration_ms: u64,
    pub events_processed: u64,
    pub time_range: TimeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Count,
    Sum,
    Average,
    Min,
    Max,
    Percentile(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    NotIn,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeGranularity {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InterpolationMethod {
    Linear,
    ForwardFill,
    Zero,
}