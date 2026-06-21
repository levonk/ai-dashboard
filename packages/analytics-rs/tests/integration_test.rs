use analytics_rs::{Aggregator, CostCalculator, FilterEngine, Processor, TimeSeriesAnalyzer};
use analytics_rs::models::{TelemetryEvent, AnalyticsQuery, AggregationType, TimeRange, Filter, FilterOperator};
use chrono::{Utc, Duration};

#[test]
fn test_basic_aggregation() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
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
        }
    ];

    let query = AnalyticsQuery {
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now(),
        },
        aggregation: AggregationType::Count,
        group_by: vec![],
    };

    let result = Aggregator::aggregate(&events, &query);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_sum_aggregation() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
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
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now(),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 2000,
            output_tokens: 1000,
            duration_ms: 2000,
            cost_usd: 0.02,
            metadata: serde_json::json!({}),
        }
    ];

    let query = AnalyticsQuery {
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now(),
        },
        aggregation: AggregationType::Sum,
        group_by: vec![],
    };

    let result = Aggregator::aggregate(&events, &query);
    assert!(result.is_ok());
    let result_json = result.unwrap();
    assert_eq!(result_json["total_cost_usd"].as_f64().unwrap(), 0.03);
    assert_eq!(result_json["total_input_tokens"].as_u64().unwrap(), 3000);
    assert_eq!(result_json["total_output_tokens"].as_u64().unwrap(), 1500);
}

#[test]
fn test_average_aggregation() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
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
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now(),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 2000,
            output_tokens: 1000,
            duration_ms: 2000,
            cost_usd: 0.02,
            metadata: serde_json::json!({}),
        }
    ];

    let query = AnalyticsQuery {
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now(),
        },
        aggregation: AggregationType::Average,
        group_by: vec![],
    };

    let result = Aggregator::aggregate(&events, &query);
    assert!(result.is_ok());
    let result_json = result.unwrap();
    assert_eq!(result_json["avg_cost_usd"].as_f64().unwrap(), 0.015);
    assert_eq!(result_json["avg_input_tokens"].as_f64().unwrap(), 1500.0);
    assert_eq!(result_json["avg_output_tokens"].as_f64().unwrap(), 750.0);
}

#[test]
fn test_cost_calculation() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
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
        }
    ];

    let total_cost = CostCalculator::calculate_total_cost(&events);
    assert!(total_cost.is_ok());
    assert_eq!(total_cost.unwrap(), 0.01);
}

#[test]
fn test_cost_estimation() {
    let estimated_cost = CostCalculator::estimate_cost("anthropic", "claude-3-opus", 1000, 500);
    assert!(estimated_cost.is_ok());
    // Anthropic Claude-3 Opus: $15/1k input tokens, $75/1k output tokens
    // 1000 input tokens = $15.0, 500 output tokens = $37.5, total = $52.5
    let cost = estimated_cost.unwrap();
    assert!((cost - 52.5).abs() < 0.0001);
}

#[test]
fn test_cost_by_provider() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
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
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now(),
            ai_client: "codex".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.02,
            metadata: serde_json::json!({}),
        }
    ];

    let costs_by_provider = CostCalculator::calculate_cost_by_provider(&events);
    assert!(costs_by_provider.is_ok());
    let costs = costs_by_provider.unwrap();
    assert_eq!(costs.get("anthropic").unwrap(), &0.01);
    assert_eq!(costs.get("openai").unwrap(), &0.02);
}

#[test]
fn test_filtering() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
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
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now(),
            ai_client: "codex".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.02,
            metadata: serde_json::json!({}),
        }
    ];

    let filters = vec![
        Filter {
            field: "ai_provider".to_string(),
            operator: FilterOperator::Equals,
            value: serde_json::json!("anthropic"),
        }
    ];

    let filtered = FilterEngine::apply(&events, &filters);
    assert!(filtered.is_ok());
    let filtered_events = filtered.unwrap();
    assert_eq!(filtered_events.len(), 1);
    assert_eq!(filtered_events[0].ai_provider, "anthropic");
}

#[test]
fn test_time_range_filter() {
    let now = Utc::now();
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
            timestamp: now - Duration::hours(2),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: now - Duration::minutes(30),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        }
    ];

    let filtered = FilterEngine::apply_time_range_filter(
        &events,
        now - Duration::hours(1),
        now
    );
    assert!(filtered.is_ok());
    let filtered_events = filtered.unwrap();
    assert_eq!(filtered_events.len(), 1);
}

#[test]
fn test_processing() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
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
        }
    ];

    let query = AnalyticsQuery {
        filters: vec![],
        time_range: TimeRange {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now(),
        },
        aggregation: AggregationType::Count,
        group_by: vec![],
    };

    let result = Processor::process_events(&events, &query);
    assert!(result.is_ok());
    let analytics_result = result.unwrap();
    assert_eq!(analytics_result.metadata.events_processed, 1);
}

#[test]
fn test_time_series() {
    let now = Utc::now();
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
            timestamp: now - Duration::minutes(30),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: now - Duration::minutes(15),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        }
    ];

    let time_range = TimeRange {
        start: now - Duration::hours(1),
        end: now,
    };

    let result = TimeSeriesAnalyzer::analyze_time_series(&events, &time_range);
    assert!(result.is_ok());
    let result_json = result.unwrap();
    assert!(result_json["time_series"].is_array());
}

#[test]
fn test_trends() {
    let events = vec![
        TelemetryEvent {
            event_id: "1".to_string(),
            timestamp: Utc::now() - Duration::hours(2),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 1000,
            cost_usd: 0.01,
            metadata: serde_json::json!({}),
        },
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now() - Duration::hours(1),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 2000,
            output_tokens: 1000,
            duration_ms: 2000,
            cost_usd: 0.02,
            metadata: serde_json::json!({}),
        }
    ];

    let result = TimeSeriesAnalyzer::calculate_trends(&events);
    assert!(result.is_ok());
    let result_json = result.unwrap();
    assert!(result_json["cost_trend"].is_object());
}

#[test]
fn test_anomaly_detection() {
    let events: Vec<TelemetryEvent> = (0..20)
        .map(|i| TelemetryEvent {
            event_id: i.to_string(),
            timestamp: Utc::now() - Duration::minutes(i),
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

    let result = TimeSeriesAnalyzer::detect_anomalies(&events);
    assert!(result.is_ok());
    let anomalies = result.unwrap();
    // With consistent data, we expect "No significant anomalies detected"
    assert!(anomalies.iter().any(|a| a.contains("No significant anomalies detected")));
}
