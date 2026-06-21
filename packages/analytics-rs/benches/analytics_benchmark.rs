use criterion::{black_box, criterion_group, criterion_main, Criterion};
use analytics_rs::{Aggregator, CostCalculator, FilterEngine};
use analytics_rs::models::{TelemetryEvent, AnalyticsQuery, AggregationType, TimeRange, Filter, FilterOperator};
use chrono::Utc;

fn bench_aggregation(c: &mut Criterion) {
    let events: Vec<TelemetryEvent> = (0..1000)
        .map(|i| TelemetryEvent {
            event_id: i.to_string(),
            timestamp: Utc::now(),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        })
        .collect();

    let query = AnalyticsQuery {
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - chrono::Duration::hours(1),
            end: Utc::now(),
        },
        aggregation: AggregationType::Count,
        group_by: vec![],
    };

    c.bench_function("aggregation_count_1000_events", |b| {
        b.iter(|| Aggregator::aggregate(black_box(&events), black_box(&query)))
    });
}

fn bench_cost_calculation(c: &mut Criterion) {
    let events: Vec<TelemetryEvent> = (0..1000)
        .map(|i| TelemetryEvent {
            event_id: i.to_string(),
            timestamp: Utc::now(),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        })
        .collect();

    c.bench_function("cost_calculation_1000_events", |b| {
        b.iter(|| CostCalculator::calculate_total_cost(black_box(&events)))
    });
}

fn bench_filtering(c: &mut Criterion) {
    let events: Vec<TelemetryEvent> = (0..1000)
        .map(|i| TelemetryEvent {
            event_id: i.to_string(),
            timestamp: Utc::now(),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        })
        .collect();

    let filters = vec![
        Filter {
            field: "ai_provider".to_string(),
            operator: FilterOperator::Equals,
            value: serde_json::json!("anthropic"),
        }
    ];

    c.bench_function("filtering_1000_events", |b| {
        b.iter(|| FilterEngine::apply(black_box(&events), black_box(&filters)))
    });
}

criterion_group!(benches, bench_aggregation, bench_cost_calculation, bench_filtering);
criterion_main!(benches);
