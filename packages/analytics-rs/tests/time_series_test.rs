use analytics_rs::{TimeSeriesAnalyzer, TelemetryEvent, TimeGranularity, InterpolationMethod, TimeRange};
use chrono::{Utc, Duration};
use serde_json::json;

fn create_test_event(id: &str, timestamp: i64, cost: f64) -> TelemetryEvent {
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
        cost_usd: cost,
        metadata: json!({}),
    }
}

#[test]
fn test_analyze_time_series() {
    let events = vec![
        create_test_event("1", -3600, 0.03),
        create_test_event("2", -1800, 0.05),
        create_test_event("3", 0, 0.04),
    ];
    
    let time_range = TimeRange {
        start: Utc::now() - Duration::hours(2),
        end: Utc::now() + Duration::hours(1),
    };
    
    let result = TimeSeriesAnalyzer::analyze_time_series(&events, &time_range).unwrap();
    assert!(result["time_series"].as_array().unwrap().len() > 0);
    assert_eq!(result["summary"]["total_events"].as_u64().unwrap(), 3);
}

#[test]
fn test_analyze_time_series_empty() {
    let events = vec![];
    
    let time_range = TimeRange {
        start: Utc::now() - Duration::hours(2),
        end: Utc::now() + Duration::hours(1),
    };
    
    let result = TimeSeriesAnalyzer::analyze_time_series(&events, &time_range).unwrap();
    assert_eq!(result["time_series"].as_array().unwrap().len(), 0);
    assert_eq!(result["summary"]["total_events"].as_u64().unwrap(), 0);
}

#[test]
fn test_calculate_trends() {
    let events = vec![
        create_test_event("1", -3600, 0.01),
        create_test_event("2", -1800, 0.02),
        create_test_event("3", 0, 0.03),
    ];
    
    let result = TimeSeriesAnalyzer::calculate_trends(&events).unwrap();
    assert!(result["cost_trend"]["direction"].is_string());
    assert!(result["input_tokens_trend"]["direction"].is_string());
}

#[test]
fn test_calculate_trends_insufficient_data() {
    let events = vec![create_test_event("1", 0, 0.03)];
    
    let result = TimeSeriesAnalyzer::calculate_trends(&events).unwrap();
    assert_eq!(result["trend"], "insufficient_data");
}

#[test]
fn test_detect_anomalies() {
    let events = vec![
        create_test_event("1", -3600, 0.03),
        create_test_event("2", -1800, 0.03),
        create_test_event("3", 0, 0.03),
        create_test_event("4", 1800, 0.03),
        create_test_event("5", 3600, 0.03),
        create_test_event("6", 5400, 0.03),
        create_test_event("7", 7200, 0.03),
        create_test_event("8", 9000, 0.03),
        create_test_event("9", 10800, 0.03),
        create_test_event("10", 12600, 5.0), // Anomaly
    ];
    
    let result = TimeSeriesAnalyzer::detect_anomalies(&events).unwrap();
    assert!(result.len() > 0);
}

#[test]
fn test_detect_anomalies_insufficient_data() {
    let events = vec![
        create_test_event("1", 0, 0.03),
        create_test_event("2", 1800, 0.03),
    ];
    
    let result = TimeSeriesAnalyzer::detect_anomalies(&events).unwrap();
    assert!(result[0].contains("Insufficient data"));
}

#[test]
fn test_calculate_moving_average() {
    let events = vec![
        create_test_event("1", -3600, 0.01),
        create_test_event("2", -1800, 0.02),
        create_test_event("3", 0, 0.03),
        create_test_event("4", 1800, 0.04),
        create_test_event("5", 3600, 0.05),
    ];
    
    let result = TimeSeriesAnalyzer::calculate_moving_average(&events, 3).unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn test_calculate_moving_average_insufficient_data() {
    let events = vec![
        create_test_event("1", 0, 0.03),
        create_test_event("2", 1800, 0.03),
    ];
    
    let result = TimeSeriesAnalyzer::calculate_moving_average(&events, 3).unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_group_by_time_granularity_hour() {
    let events = vec![
        create_test_event("1", -3600, 0.03),
        create_test_event("2", -1800, 0.05),
        create_test_event("3", 0, 0.04),
    ];
    
    let result = TimeSeriesAnalyzer::group_by_time_granularity(&events, TimeGranularity::Hour).unwrap();
    assert!(result.len() > 0);
}

#[test]
fn test_group_by_time_granularity_day() {
    let events = vec![
        create_test_event("1", -86400, 0.03),
        create_test_event("2", -43200, 0.05),
        create_test_event("3", 0, 0.04),
    ];
    
    let result = TimeSeriesAnalyzer::group_by_time_granularity(&events, TimeGranularity::Day).unwrap();
    assert!(result.len() > 0);
}

#[test]
fn test_group_by_time_granularity_empty() {
    let events = vec![];
    
    let result = TimeSeriesAnalyzer::group_by_time_granularity(&events, TimeGranularity::Hour).unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_resample_time_series_linear() {
    let events = vec![
        create_test_event("1", -3600, 0.03),
        create_test_event("2", 0, 0.05),
        create_test_event("3", 3600, 0.04),
    ];
    
    let result = TimeSeriesAnalyzer::resample_time_series(&events, TimeGranularity::Hour, InterpolationMethod::Linear).unwrap();
    assert!(result.len() >= 1);
    // Check that interpolated field exists
    assert!(result[0].get("interpolated").is_some());
}

#[test]
fn test_resample_time_series_forward_fill() {
    let events = vec![
        create_test_event("1", -3600, 0.03),
        create_test_event("2", 0, 0.05),
        create_test_event("3", 3600, 0.04),
    ];
    
    let result = TimeSeriesAnalyzer::resample_time_series(&events, TimeGranularity::Hour, InterpolationMethod::ForwardFill).unwrap();
    assert!(result.len() >= 1);
    assert!(result[0].get("interpolated").is_some());
}

#[test]
fn test_resample_time_series_zero() {
    let events = vec![
        create_test_event("1", -3600, 0.03),
        create_test_event("2", 0, 0.05),
        create_test_event("3", 3600, 0.04),
    ];
    
    let result = TimeSeriesAnalyzer::resample_time_series(&events, TimeGranularity::Hour, InterpolationMethod::Zero).unwrap();
    assert!(result.len() >= 1);
    assert!(result[0].get("interpolated").is_some());
}

#[test]
fn test_resample_time_series_empty() {
    let events = vec![];
    
    let result = TimeSeriesAnalyzer::resample_time_series(&events, TimeGranularity::Hour, InterpolationMethod::Linear).unwrap();
    assert_eq!(result.len(), 0);
}