use analytics_rs::{TimeSeriesAnalyzer, TelemetryEvent, TimeGranularity, InterpolationMethod, TimeRange};
use chrono::{Utc, Duration};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;

fn create_test_events(count: usize) -> Vec<TelemetryEvent> {
    (0..count).map(|i| TelemetryEvent {
        event_id: format!("event-{}", i),
        timestamp: Utc::now() + Duration::seconds(i as i64),
        ai_client: "claude-code".to_string(),
        ai_provider: "anthropic".to_string(),
        model: "claude-3-opus".to_string(),
        input_type: "text".to_string(),
        input_tokens: (1000 + (i % 10) * 100) as u32,
        output_tokens: (500 + (i % 5) * 50) as u32,
        duration_ms: (2000 + (i % 3) * 500) as u64,
        cost_usd: 0.03 + (i % 10) as f64 * 0.01,
        metadata: json!({}),
    }).collect()
}

fn bench_time_series_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_series_analysis");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        let time_range = TimeRange {
            start: Utc::now() - Duration::hours(24),
            end: Utc::now() + Duration::hours(1),
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(TimeSeriesAnalyzer::analyze_time_series(black_box(&events), black_box(&time_range)).unwrap())
            })
        });
    }
    
    group.finish();
}

fn bench_trend_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("trend_calculation");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(TimeSeriesAnalyzer::calculate_trends(black_box(&events)).unwrap())
            })
        });
    }
    
    group.finish();
}

fn bench_anomaly_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("anomaly_detection");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(TimeSeriesAnalyzer::detect_anomalies(black_box(&events)).unwrap())
            })
        });
    }
    
    group.finish();
}

fn bench_moving_average(c: &mut Criterion) {
    let mut group = c.benchmark_group("moving_average");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(TimeSeriesAnalyzer::calculate_moving_average(black_box(&events), black_box(10)).unwrap())
            })
        });
    }
    
    group.finish();
}

fn bench_time_grouping(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_grouping");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(TimeSeriesAnalyzer::group_by_time_granularity(black_box(&events), black_box(TimeGranularity::Hour)).unwrap())
            })
        });
    }
    
    group.finish();
}

fn bench_resampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("resampling");
    
    for size in [1000, 10000, 100000].iter() {
        let events = create_test_events(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(TimeSeriesAnalyzer::resample_time_series(black_box(&events), black_box(TimeGranularity::Hour), black_box(InterpolationMethod::Linear)).unwrap())
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_time_series_analysis,
    bench_trend_calculation,
    bench_anomaly_detection,
    bench_moving_average,
    bench_time_grouping,
    bench_resampling
);
criterion_main!(benches);