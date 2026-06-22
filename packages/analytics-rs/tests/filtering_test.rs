use analytics_rs::{FilterEngine, Filter, FilterOperator, TelemetryEvent};
use chrono::{Utc, Duration};
use serde_json::json;

fn create_test_event(id: &str, timestamp: i64) -> TelemetryEvent {
    TelemetryEvent {
        event_id: id.to_string(),
        timestamp: Utc::now() + Duration::seconds(timestamp),
        ai_client: "claude-code".to_string(),
        ai_provider: "anthropic".to_string(),
        model: "claude-3-opus".to_string(),
        input_type: "text".to_string(),
        input_tokens: 1000,
        output_tokens: 500,
        duration_ms: 2000,
        cost_usd: 0.03,
        metadata: json!({}),
    }
}

#[test]
fn test_filter_equals() {
    let events = vec![
        create_test_event("1", 0),
        create_test_event("2", 1),
    ];
    
    let filter = Filter {
        field: "ai_client".to_string(),
        operator: FilterOperator::Equals,
        value: json!("claude-code"),
    };
    
    let result = FilterEngine::apply(&events, &[filter]).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_filter_not_equals() {
    let events = vec![
        create_test_event("1", 0),
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now() + Duration::seconds(1),
            ai_client: "codex".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 2000,
            cost_usd: 0.03,
            metadata: json!({}),
        },
    ];
    
    let filter = Filter {
        field: "ai_client".to_string(),
        operator: FilterOperator::NotEquals,
        value: json!("claude-code"),
    };
    
    let result = FilterEngine::apply(&events, &[filter]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ai_client, "codex");
}

#[test]
fn test_filter_contains() {
    let events = vec![
        create_test_event("1", 0),
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now() + Duration::seconds(1),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-sonnet".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 2000,
            cost_usd: 0.03,
            metadata: json!({}),
        },
    ];
    
    let filter = Filter {
        field: "model".to_string(),
        operator: FilterOperator::Contains,
        value: json!("sonnet"),
    };
    
    let result = FilterEngine::apply(&events, &[filter]).unwrap();
    assert_eq!(result.len(), 1);
    assert!(result[0].model.contains("sonnet"));
}

#[test]
fn test_filter_greater_than() {
    let events = vec![
        create_test_event("1", 0),
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now() + Duration::seconds(1),
            ai_client: "claude-code".to_string(),
            ai_provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_type: "text".to_string(),
            input_tokens: 5000,
            output_tokens: 2500,
            duration_ms: 2000,
            cost_usd: 0.15,
            metadata: json!({}),
        },
    ];
    
    let filter = Filter {
        field: "input_tokens".to_string(),
        operator: FilterOperator::GreaterThan,
        value: json!(2000),
    };
    
    let result = FilterEngine::apply(&events, &[filter]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].input_tokens, 5000);
}

#[test]
fn test_filter_in() {
    let events = vec![
        create_test_event("1", 0),
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now() + Duration::seconds(1),
            ai_client: "codex".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 2000,
            cost_usd: 0.03,
            metadata: json!({}),
        },
    ];
    
    let filter = Filter {
        field: "ai_client".to_string(),
        operator: FilterOperator::In,
        value: json!(["claude-code", "codex"]),
    };
    
    let result = FilterEngine::apply(&events, &[filter]).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_multiple_filters() {
    let events = vec![
        create_test_event("1", 0),
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now() + Duration::seconds(1),
            ai_client: "codex".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 2000,
            cost_usd: 0.03,
            metadata: json!({}),
        },
    ];
    
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
    ];
    
    let result = FilterEngine::apply(&events, &filters).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ai_client, "claude-code");
}

#[test]
fn test_time_range_filter() {
    let events = vec![
        create_test_event("1", -3600), // 1 hour ago
        create_test_event("2", 0),      // now
        create_test_event("3", 3600),   // 1 hour from now
    ];
    
    let start = Utc::now() - Duration::seconds(1800); // 30 min ago
    let end = Utc::now() + Duration::seconds(1800);   // 30 min from now
    
    let result = FilterEngine::apply_time_range_filter(&events, start, end).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event_id, "2");
}

#[test]
fn test_client_filter() {
    let events = vec![
        create_test_event("1", 0),
        TelemetryEvent {
            event_id: "2".to_string(),
            timestamp: Utc::now() + Duration::seconds(1),
            ai_client: "codex".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_type: "text".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            duration_ms: 2000,
            cost_usd: 0.03,
            metadata: json!({}),
        },
    ];
    
    let result = FilterEngine::apply_client_filter(&events, &["claude-code".to_string()]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ai_client, "claude-code");
}