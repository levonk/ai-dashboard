# analytics-rs

A reusable analytics processing library for AI telemetry data. This package provides core analytics algorithms that are execution-context agnostic and can be consumed by:

- **Proxy Service** (open-source): Real-time analytics processing
- **Analytics Service** (commercial): Multi-tenant analytics engine
- **Apache Spark Jobs** (commercial): Batch processing pipelines

## Features

- **Aggregation Functions**: Count, sum, average, min, max, and percentile calculations
- **Cost Calculation**: Provider-specific pricing models and cost estimation
- **Multi-dimensional Filtering**: Advanced filtering with multiple operators
- **Time-series Analysis**: Trend detection and anomaly identification
- **Data Processing Pipeline**: Unified processing interface for analytics queries

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
analytics-rs = { path = "../packages/analytics-rs" }
```

## Usage

### Basic Aggregation

```rust
use analytics_rs::{Aggregator, TelemetryEvent, AnalyticsQuery, AggregationType};

let events = vec![/* your telemetry events */];
let query = AnalyticsQuery {
    filters: vec![],
    time_range: TimeRange { start, end },
    aggregation: AggregationType::Count,
    group_by: vec![],
};

let result = Aggregator::aggregate(&events, &query)?;
```

### Cost Calculation

```rust
use analytics_rs::CostCalculator;

let total_cost = CostCalculator::calculate_total_cost(&events)?;
let estimated_cost = CostCalculator::estimate_cost("anthropic", "claude-3-opus", 1000, 500)?;
```

### Filtering

```rust
use analytics_rs::FilterEngine;

let filtered_events = FilterEngine::apply(&events, &filters)?;
```

## Data Models

### TelemetryEvent

Represents a single AI usage event with metadata:

```rust
pub struct TelemetryEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub ai_client: String,      // e.g., "claude-code", "codex"
    pub ai_provider: String,    // e.g., "anthropic", "openai"
    pub model: String,           // e.g., "claude-3-opus"
    pub input_type: String,      // e.g., "text", "image"
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub duration_ms: u64,
    pub cost_usd: f64,
    pub metadata: serde_json::Value,
}
```

## Development

### Running Tests

```bash
cargo test
```

### Running Benchmarks

```bash
cargo bench
```

### Type Checking

```bash
cargo check
```

### Linting

```bash
cargo clippy -- -D warnings
```

## License

AGPL 3.0 for open-source features. Commercial license available for multi-tenant, white-label, or proprietary use.

## Contributing

This package is part of the AI Dashboard project. Please follow the main project's contribution guidelines.
