use analytics_rs::{FilterEngine, Filter, FilterOperator, TelemetryEvent};
use chrono::{Utc, Duration};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;

fn create_test_events(count: usize) -> Vec<TelemetryEvent> {
    (0..count).map(|i| TelemetryEvent {
        event_id: format!("event-{}", i),
        timestamp: Utc::now() + Duration::seconds(i as i64),
        ai_client: if i % 2 == 0 { "claude-code" } else { "codex" }.to_string(),
        ai_provider: if i % 2 == 0 { "anthropic" } else { "openai" }.to_string(),
        model: if i % 3 == 0 { "claude-3-opus" } else if i % 3 == 1 { "gpt-4" } else { "claude-3-sonnet" }.to_string(),
        input_type: "text".to_string(),
        input_tokens: (1000 + (i % 10) * 100) as u32,
        output_tokens: (500 + (i % 5) * 50) as u32,
        duration_ms: (2000 + (i % 3) * 500) as u64,
        cost_usd: 0.03 + (i % 10) as f64 * 0.01,
        metadata: json!({}),
    }).collect()
}

fn bench_filtering_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtering_basic");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filter = Filter {
                    field: "ai_client".to_string(),
                    operator: FilterOperator::Equals,
                    value: json!("claude-code"),
                };
                black_box(FilterEngine::apply(black_box(&events), &[filter]).unwrap())
            })
        });
    }
    
    group.finish();
}

fn bench_filtering_multiple(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtering_multiple");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filters = vec![
                    Filter {
                        field: "ai_client".to_string(),
                        operator: FilterOperator::Equals,
                        value: json!("claude-code"),
                    },
                    Filter {
                        field: "ai_provider".to_string(),
                        operator: FilterOperator::Equals,
                        value: json!("anthropic"),
                    },
                    Filter {
                        field: "input_tokens".to_string(),
                        operator: FilterOperator::GreaterThan,
                        value: json!(1500),
                    },
                ];
                black_box(FilterEngine::apply(black_box(&events), &filters).unwrap())
            })
        });
    }
    
    group.finish();
}

fn bench_filtering_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtering_complex");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filters = vec![
                    Filter {
                        field: "model".to_string(),
                        operator: FilterOperator::In,
                        value: json!(["claude-3-opus", "gpt-4"]),
                    },
                    Filter {
                        field: "cost_usd".to_string(),
                        operator: FilterOperator::GreaterThan,
                        value: json!(0.05),
                    },
                    Filter {
                        field: "duration_ms".to_string(),
                        operator: FilterOperator::LessThan,
                        value: json!(3000),
                    },
                ];
                black_box(FilterEngine::apply(black_box(&events), &filters).unwrap())
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_filtering_basic,
    bench_filtering_multiple,
    bench_filtering_complex
);
criterion_main!(benches);