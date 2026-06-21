use ai_analytics_proxy::collection::{
    MetadataExtractor, TimingCollector, TokenCollector, ErrorCollector,
    DatabaseWriter, HashCollector, DimensionCollector, DimensionExtractor,
    TokenCount, RequestTiming, ErrorRecord, AttributeRecord,
};
use analytics_rs::TelemetryEvent;
use std::collections::HashMap;

#[test]
fn test_full_collection_workflow() {
    // Simulate a complete request collection workflow
    
    // 1. Extract metadata
    let mut headers = HashMap::new();
    headers.insert("x-ai-client".to_string(), "claude-code".to_string());
    headers.insert("x-user-id".to_string(), "user123".to_string());
    
    let request = serde_json::json!({
        "provider": "anthropic",
        "model": "claude-3-opus",
        "prompt": "Hello, world!"
    });
    
    let mut context = HashMap::new();
    context.insert("pipeline_stage".to_string(), "production".to_string());
    
    let metadata = MetadataExtractor::build_metadata(&headers, &request, &context).unwrap();
    assert_eq!(metadata.ai_client, "claude-code");
    assert_eq!(metadata.pipeline_stage, "production");
    
    // 2. Collect timing metrics
    let mut timing_collector = TimingCollector::new();
    timing_collector.start_request();
    std::thread::sleep(std::time::Duration::from_millis(10));
    timing_collector.start_provider();
    std::thread::sleep(std::time::Duration::from_millis(5));
    
    let request_duration = timing_collector.calculate_request_duration();
    assert!(request_duration.as_millis() >= 10);
    
    // 3. Collect token counts
    let token_collector = TokenCollector::new();
    let token_count = token_collector.extract_request_tokens(&request, "claude-3-opus").unwrap();
    assert!(token_count.input_tokens > 0);
    
    // 4. Generate hash for correlation
    let hash_collector = HashCollector::new();
    let fingerprint = hash_collector.generate_request_fingerprint(&request).unwrap();
    assert_eq!(fingerprint.len(), 64);
    
    // 5. Collect dimensions
    let mut dimension_collector = DimensionCollector::new();
    dimension_collector.initialize_default_dimensions();
    
    let dimensions = DimensionExtractor::extract_all_dimensions(&headers, &context, &request);
    assert!(dimensions.contains_key("environment"));
    assert!(dimensions.contains_key("time_of_day"));
    
    // 6. Create telemetry event
    let db_writer = DatabaseWriter::new();
    let event = db_writer.create_telemetry_event(
        "req-123".to_string(),
        metadata.ai_client.clone(),
        metadata.provider.clone(),
        metadata.model.clone(),
        metadata.input_type.clone(),
        token_count.input_tokens,
        token_count.output_tokens,
        request_duration.as_millis() as u64,
        0.0, // Cost would be calculated
        HashMap::new(),
    );
    
    assert_eq!(event.ai_client, "claude-code");
    assert_eq!(event.ai_provider, "anthropic");
}

#[test]
fn test_error_collection_workflow() {
    let mut error_collector = ErrorCollector::new();
    
    // Simulate different types of errors
    let auth_error = ErrorRecord::new(
        "Unauthorized access".to_string(),
        Some(401),
        Some("req-1".to_string()),
        Some("anthropic".to_string()),
        Some("claude-3-opus".to_string()),
    );
    
    let network_error = ErrorRecord::new(
        "Network timeout".to_string(),
        None,
        Some("req-2".to_string()),
        Some("openai".to_string()),
        Some("gpt-4".to_string()),
    );
    
    error_collector.record_error(auth_error);
    error_collector.record_error(network_error);
    
    assert_eq!(error_collector.total_errors(), 2);
    
    let common_errors = error_collector.get_common_errors(5);
    assert_eq!(common_errors.len(), 2);
}

#[test]
fn test_request_timing_lifecycle() {
    let mut timing = RequestTiming::new("test-request".to_string());
    
    assert_eq!(timing.duration_ms, 0);
    assert!(timing.end_time.is_none());
    
    timing.complete(100, Some(50));
    
    assert_eq!(timing.duration_ms, 100);
    assert_eq!(timing.provider_latency_ms, Some(50));
    assert_eq!(timing.processing_overhead_ms, 50);
    assert!(timing.end_time.is_some());
}

#[test]
fn test_token_count_lifecycle() {
    let mut token_count = TokenCount::new();
    
    assert_eq!(token_count.input_tokens, 0);
    assert_eq!(token_count.output_tokens, 0);
    
    token_count.input_tokens = 100;
    token_count.output_tokens = 50;
    token_count.total_tokens = 150;
    token_count.input_characters = 500;
    
    assert_eq!(token_count.token_efficiency(), 0.2);
    assert_eq!(token_count.output_ratio(), 0.5);
}

#[test]
fn test_attribute_record_collection() {
    let record = AttributeRecord::new("req-123".to_string())
        .with_dimension("geo".to_string(), "us".to_string())
        .with_dimension("segment".to_string(), "pro".to_string())
        .with_dimension("environment".to_string(), "production".to_string());
    
    assert_eq!(record.dimensions.len(), 3);
    assert_eq!(record.get_dimension("geo"), Some(&"us".to_string()));
}

#[test]
fn test_dimension_tracking() {
    let mut collector = DimensionCollector::new();
    collector.initialize_default_dimensions();
    
    // Record some values
    collector.record_dimension_value("geography", "us".to_string()).unwrap();
    collector.record_dimension_value("geography", "us".to_string()).unwrap();
    collector.record_dimension_value("geography", "eu".to_string()).unwrap();
    
    let stats = collector.get_dimension_stats("geography").unwrap();
    assert_eq!(stats.total_count, 3);
    assert_eq!(stats.unique_values, 2);
}

#[test]
fn test_hash_correlation_tracking() {
    let mut hash_collector = HashCollector::new();
    
    // Generate correlation IDs for related requests
    let components1 = vec!["claude-3-opus", "What is AI?"];
    let components2 = vec!["claude-3-opus", "What is AI?"];
    let components3 = vec!["claude-3-opus", "What is ML?"];
    
    let id1 = hash_collector.generate_correlation_id(&components1);
    let id2 = hash_collector.generate_correlation_id(&components2);
    let id3 = hash_collector.generate_correlation_id(&components3);
    
    // Same components should produce same ID
    assert_eq!(id1, id2);
    
    // Different components should produce different ID
    assert_ne!(id1, id3);
}

#[test]
fn test_database_write_validation() {
    let db_writer = DatabaseWriter::new();
    
    // Test valid event creation
    let event = db_writer.create_telemetry_event(
        "req-123".to_string(),
        "claude-code".to_string(),
        "anthropic".to_string(),
        "claude-3-opus".to_string(),
        "chat".to_string(),
        100,
        50,
        1000,
        0.015,
        HashMap::new(),
    );
    
    assert!(db_writer.write_telemetry_event(event).is_ok());
    
    // Test invalid event (empty client)
    let invalid_event = db_writer.create_telemetry_event(
        "req-456".to_string(),
        "".to_string(), // Invalid: empty client
        "anthropic".to_string(),
        "claude-3-opus".to_string(),
        "chat".to_string(),
        100,
        50,
        1000,
        0.015,
        HashMap::new(),
    );
    
    assert!(db_writer.write_telemetry_event(invalid_event).is_err());
}

#[test]
fn test_batch_collection() {
    let db_writer = DatabaseWriter::new();
    
    let mut events = Vec::new();
    
    for i in 0..5 {
        let event = db_writer.create_telemetry_event(
            format!("req-{}", i),
            "claude-code".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "chat".to_string(),
            100,
            50,
            1000,
            0.015,
            HashMap::new(),
        );
        events.push(event);
    }
    
    let result = db_writer.write_batch(events).unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_error_severity_classification() {
    use ai_analytics_proxy::collection::{ErrorType, ErrorSeverity};
    
    // Test different error types and their severity
    let auth_severity = ErrorSeverity::from_error_type(&ErrorType::Authentication, false);
    assert_eq!(auth_severity, ErrorSeverity::High);
    
    let internal_severity = ErrorSeverity::from_error_type(&ErrorType::Internal, false);
    assert_eq!(internal_severity, ErrorSeverity::Critical);
    
    let validation_severity = ErrorSeverity::from_error_type(&ErrorType::Validation, false);
    assert_eq!(validation_severity, ErrorSeverity::Low);
}

#[test]
fn test_metadata_validation() {
    let mut headers = HashMap::new();
    headers.insert("x-ai-client".to_string(), "claude-code".to_string());
    
    let request = serde_json::json!({
        "provider": "anthropic",
        "model": "claude-3-opus",
        "prompt": "Hello"
    });
    
    let context = HashMap::new();
    
    // Valid metadata
    let valid_metadata = MetadataExtractor::build_metadata(&headers, &request, &context);
    assert!(valid_metadata.is_ok());
    
    // Invalid metadata (empty client)
    let empty_headers = HashMap::new();
    let invalid_metadata = MetadataExtractor::build_metadata(&empty_headers, &request, &context);
    assert!(invalid_metadata.is_err());
}

#[test]
fn test_integration_with_analytics_mode() {
    // This test verifies that the collection module integrates properly
    // with the analytics mode infrastructure
    
    let db_writer = DatabaseWriter::new();
    
    // Create a realistic telemetry event
    let event = db_writer.create_telemetry_event(
        "req-integration-test".to_string(),
        "claude-code".to_string(),
        "anthropic".to_string(),
        "claude-3-opus".to_string(),
        "chat".to_string(),
        150, // input tokens
        75,  // output tokens
        2500, // duration in ms
        0.0225, // estimated cost
        {
            let mut metadata = HashMap::new();
            metadata.insert("user_id".to_string(), serde_json::json!("user123"));
            metadata.insert("team_id".to_string(), serde_json::json!("team456"));
            metadata
        },
    );
    
    // Verify event structure matches TelemetryEvent expectations
    assert_eq!(event.event_id.len(), 36); // UUID format
    assert_eq!(event.ai_client, "claude-code");
    assert_eq!(event.ai_provider, "anthropic");
    assert_eq!(event.input_tokens, Some(150));
    assert_eq!(event.output_tokens, Some(75));
    assert_eq!(event.duration_ms, Some(2500));
    assert_eq!(event.cost_usd, Some(0.0225));
}