use analytics_rs::models::{TelemetryEvent, AnalyticsQuery, AggregationType, TimeRange};
use analytics_rs::Aggregator;
use chrono::{Utc, Duration};

fn create_test_events() -> Vec<TelemetryEvent> {
    let now = Utc::now();
    vec![
        TelemetryEvent {
            event_id: "1".to_string(),
            timestamp: now,
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            input_type: "text".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cost_usd: 0.001,
            duration_ms: 1000,
            metadata: serde_json::json!({}),
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: now + Duration::seconds(1),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            input_type: "text".to_string(),
            input_tokens: 200,
            output_tokens: 100,
            cost_usd: 0.002,
            duration_ms: 2000,
            metadata: serde_json::json!({}),
        },
        TelemetryEvent {
            event_id: "3".to_string(),
            timestamp: now + Duration::seconds(2),
            ai_client: "codex".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            input_tokens: 150,
            output_tokens: 75,
            cost_usd: 0.003,
            duration_ms: 1500,
            metadata: serde_json::json!({}),
        },
    ]
}

#[test]
fn test_count_aggregation() {
    let events = create_test_events();
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
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    assert_eq!(result.as_i64().unwrap(), 3);
}

#[test]
fn test_sum_aggregation() {
    let events = create_test_events();
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
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    assert_eq!(result["total_cost_usd"].as_f64().unwrap(), 0.006);
    assert_eq!(result["total_input_tokens"].as_u64().unwrap(), 450);
    assert_eq!(result["total_output_tokens"].as_u64().unwrap(), 225);
}

#[test]
fn test_average_aggregation() {
    let events = create_test_events();
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
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    assert!((result["avg_cost_usd"].as_f64().unwrap() - 0.002).abs() < 0.0001);
    assert!((result["avg_input_tokens"].as_f64().unwrap() - 150.0).abs() < 0.0001);
    assert!((result["avg_output_tokens"].as_f64().unwrap() - 75.0).abs() < 0.0001);
}

#[test]
fn test_min_aggregation() {
    let events = create_test_events();
    let aggregator = Aggregator::new();
    let query = AnalyticsQuery {
        aggregation: AggregationType::Min,
        group_by: vec![],
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now() + Duration::hours(1),
        },
    };
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    assert_eq!(result["min_cost_usd"].as_f64().unwrap(), 0.001);
    assert_eq!(result["min_input_tokens"].as_u64().unwrap(), 100);
    assert_eq!(result["min_output_tokens"].as_u64().unwrap(), 50);
}

#[test]
fn test_max_aggregation() {
    let events = create_test_events();
    let aggregator = Aggregator::new();
    let query = AnalyticsQuery {
        aggregation: AggregationType::Max,
        group_by: vec![],
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now() + Duration::hours(1),
        },
    };
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    assert_eq!(result["max_cost_usd"].as_f64().unwrap(), 0.003);
    assert_eq!(result["max_input_tokens"].as_u64().unwrap(), 200);
    assert_eq!(result["max_output_tokens"].as_u64().unwrap(), 100);
}

#[test]
fn test_percentile_aggregation() {
    let events = create_test_events();
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
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    // Should return p50, p90, p95, p99
    assert!(result.get("p50_cost_usd").is_some());
    assert!(result.get("p90_cost_usd").is_some());
    assert!(result.get("p95_cost_usd").is_some());
    assert!(result.get("p99_cost_usd").is_some());
}

#[test]
fn test_grouped_aggregation() {
    let events = create_test_events();
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
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    // Should have grouped results by provider
    assert!(result.is_object());
    assert!(result.get("anthropic").is_some());
    assert!(result.get("openai").is_some());
}

#[test]
fn test_histogram() {
    let events = create_test_events();
    let aggregator = Aggregator::new();
    let buckets = vec![0.0015, 0.0025, 0.0035];
    
    let result = aggregator.histogram(&events, "cost_usd", buckets).unwrap();
    assert!(result["buckets"].is_object());
}

#[test]
fn test_rate_calculations() {
    let events = create_test_events();
    let aggregator = Aggregator::new();
    
    let result = aggregator.rate_calculations(&events, 10).unwrap();
    assert!(result["requests_per_second"].as_f64().unwrap() > 0.0);
    assert!(result["input_tokens_per_second"].as_f64().unwrap() > 0.0);
    assert!(result["output_tokens_per_second"].as_f64().unwrap() > 0.0);
}

#[test]
fn test_empty_events() {
    let events: Vec<TelemetryEvent> = vec![];
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
    
    let result = aggregator.aggregate(&events, &query).unwrap();
    assert_eq!(result.as_i64().unwrap(), 0);
}

#[test]
fn test_cache_functionality() {
    let events = create_test_events();
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
    
    // First call
    let result1 = aggregator.aggregate(&events, &query).unwrap();
    // Second call should use cache
    let result2 = aggregator.aggregate(&events, &query).unwrap();
    
    assert_eq!(result1, result2);
}