use analytics_rs::models::{TelemetryEvent, AnalyticsQuery, AggregationType, TimeRange};
use analytics_rs::Aggregator;
use chrono::{Utc, Duration};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn create_large_test_events(count: usize) -> Vec<TelemetryEvent> {
    let now = Utc::now();
    (0..count).map(|i| TelemetryEvent {
        event_id: format!("event-{}", i),
        timestamp: now + Duration::seconds(i as i64),
        ai_client: if i % 2 == 0 { "claude-code" } else { "codex" }.to_string(),
        ai_provider: if i % 3 == 0 { "anthropic" } else if i % 3 == 1 { "openai" } else { "google" }.to_string(),
        model: if i % 4 == 0 { "claude-3-5-sonnet" } else if i % 4 == 1 { "gpt-4" } else if i % 4 == 2 { "gemini-pro" } else { "claude-3-opus" }.to_string(),
        input_type: "text".to_string(),
        input_tokens: (i % 1000 + 100) as u32,
        output_tokens: (i % 500 + 50) as u32,
        cost_usd: (i as f64 * 0.0001),
        duration_ms: (i % 2000 + 500) as u64,
        metadata: serde_json::json!({}),
    }).collect()
}

fn benchmark_count_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("count_aggregation");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let events = create_large_test_events(*size);
        let aggregator = Aggregator::new();
        let query = AnalyticsQuery {
            aggregation: AggregationType::Count,
            group_by: vec![],
            filters: vec![],
            time_range: TimeRange {
                start: Utc::now() - Duration::hours(1),
                end: Utc::now() + Duration::hours(1),
            },
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| aggregator.aggregate(black_box(&events), black_box(&query)));
        });
    }
    
    group.finish();
}

fn benchmark_sum_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_aggregation");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let events = create_large_test_events(*size);
        let aggregator = Aggregator::new();
        let query = AnalyticsQuery {
            aggregation: AggregationType::Sum,
            group_by: vec![],
            filters: vec![],
            time_range: TimeRange {
                start: Utc::now() - Duration::hours(1),
                end: Utc::now() + Duration::hours(1),
            },
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| aggregator.aggregate(black_box(&events), black_box(&query)));
        });
    }
    
    group.finish();
}

fn benchmark_average_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("average_aggregation");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let events = create_large_test_events(*size);
        let aggregator = Aggregator::new();
        let query = AnalyticsQuery {
            aggregation: AggregationType::Average,
            group_by: vec![],
            filters: vec![],
            time_range: TimeRange {
                start: Utc::now() - Duration::hours(1),
                end: Utc::now() + Duration::hours(1),
            },
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| aggregator.aggregate(black_box(&events), black_box(&query)));
        });
    }
    
    group.finish();
}

fn benchmark_percentile_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("percentile_aggregation");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let events = create_large_test_events(*size);
        let aggregator = Aggregator::new();
        let query = AnalyticsQuery {
            aggregation: AggregationType::Percentile(0.5),
            group_by: vec![],
            filters: vec![],
            time_range: TimeRange {
                start: Utc::now() - Duration::hours(1),
                end: Utc::now() + Duration::hours(1),
            },
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| aggregator.aggregate(black_box(&events), black_box(&query)));
        });
    }
    
    group.finish();
}

fn benchmark_grouped_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouped_aggregation");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let events = create_large_test_events(*size);
        let aggregator = Aggregator::new();
        let query = AnalyticsQuery {
            aggregation: AggregationType::Sum,
            group_by: vec!["ai_provider".to_string()],
            filters: vec![],
            time_range: TimeRange {
                start: Utc::now() - Duration::hours(1),
                end: Utc::now() + Duration::hours(1),
            },
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| aggregator.aggregate(black_box(&events), black_box(&query)));
        });
    }
    
    group.finish();
}

fn benchmark_histogram(c: &mut Criterion) {
    let mut group = c.benchmark_group("histogram");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let events = create_large_test_events(*size);
        let aggregator = Aggregator::new();
        let buckets = vec![0.001, 0.005, 0.01, 0.05, 0.1];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| aggregator.histogram(black_box(&events), black_box("cost_usd"), black_box(buckets.clone())));
        });
    }
    
    group.finish();
}

fn benchmark_rate_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_calculations");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let events = create_large_test_events(*size);
        let aggregator = Aggregator::new();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| aggregator.rate_calculations(black_box(&events), black_box(60)));
        });
    }
    
    group.finish();
}

fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    
    let events = create_large_test_events(10000);
    let aggregator = Aggregator::with_cache_ttl(60);
    let query = AnalyticsQuery {
        aggregation: AggregationType::Count,
        group_by: vec![],
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now() + Duration::hours(1),
        },
    };
    
    // First call (cache miss)
    group.bench_function("cache_miss", |b| {
        b.iter(|| aggregator.aggregate(black_box(&events), black_box(&query)));
    });
    
    // Second call (cache hit)
    group.bench_function("cache_hit", |b| {
        b.iter(|| aggregator.aggregate(black_box(&events), black_box(&query)));
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_count_aggregation,
    benchmark_sum_aggregation,
    benchmark_average_aggregation,
    benchmark_percentile_aggregation,
    benchmark_grouped_aggregation,
    benchmark_histogram,
    benchmark_rate_calculations,
    benchmark_cache_performance
);
criterion_main!(benches);