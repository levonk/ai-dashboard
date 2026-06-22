use analytics_rs::{CostCalculator, TelemetryEvent};
use chrono::Utc;

fn create_test_event(provider: &str, model: &str, input_tokens: u32, output_tokens: u32, input_type: &str) -> TelemetryEvent {
    TelemetryEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        ai_client: "test_client".to_string(),
        ai_provider: provider.to_string(),
        model: model.to_string(),
        input_type: input_type.to_string(),
        input_tokens,
        output_tokens,
        duration_ms: 1000,
        cost_usd: 0.0,
        metadata: serde_json::json!({}),
    }
}

#[test]
fn test_cost_calculator_initialization() {
    let calculator = CostCalculator::new();
    let pricing_db = calculator.get_pricing_database();
    assert!(!pricing_db.models.is_empty());
    assert!(!pricing_db.providers.is_empty());
}

#[test]
fn test_basic_cost_calculation() {
    let calculator = CostCalculator::new();
    
    let event = create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "text");
    
    let cost = calculator.calculate_cost(&event).unwrap();
    assert!(cost > 0.0);
    
    // Expected: (1000/1000) * 3.0 + (500/1000) * 15.0 = 3.0 + 7.5 = 10.5
    assert!((cost - 10.5).abs() < 0.01);
}

#[test]
fn test_cost_calculation_with_existing_cost() {
    let calculator = CostCalculator::new();
    
    let mut event = create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "text");
    event.cost_usd = 25.0; // Pre-calculated cost
    
    let cost = calculator.calculate_cost(&event).unwrap();
    assert_eq!(cost, 25.0);
}

#[test]
fn test_cost_breakdown() {
    let calculator = CostCalculator::new();
    
    let event = create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "text");
    
    let breakdown = calculator.calculate_cost_with_breakdown(&event).unwrap();
    assert!(breakdown.input_cost > 0.0);
    assert!(breakdown.output_cost > 0.0);
    assert_eq!(breakdown.input_type_multiplier, 1.0);
    assert_eq!(breakdown.total_cost, breakdown.input_cost + breakdown.output_cost);
}

#[test]
fn test_input_type_multiplier() {
    let calculator = CostCalculator::new();
    
    let event = create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "image");
    
    let breakdown = calculator.calculate_cost_with_breakdown(&event).unwrap();
    assert_eq!(breakdown.input_type_multiplier, 1.5);
    assert!(breakdown.input_cost > 3.0); // Should be higher due to multiplier
}

#[test]
fn test_total_cost_calculation() {
    let calculator = CostCalculator::new();
    
    let events = vec![
        create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "text"),
        create_test_event("openai", "gpt-3.5-turbo", 2000, 1000, "text"),
    ];
    
    let total_cost = calculator.calculate_total_cost(&events).unwrap();
    assert!(total_cost > 0.0);
}

#[test]
fn test_cost_by_provider() {
    let calculator = CostCalculator::new();
    
    let events = vec![
        create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "text"),
        create_test_event("openai", "gpt-3.5-turbo", 2000, 1000, "text"),
    ];
    
    let costs_by_provider = calculator.calculate_cost_by_provider(&events).unwrap();
    assert_eq!(costs_by_provider.len(), 2);
    assert!(costs_by_provider.contains_key("anthropic"));
    assert!(costs_by_provider.contains_key("openai"));
}

#[test]
fn test_cost_by_model() {
    let calculator = CostCalculator::new();
    
    let events = vec![
        create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "text"),
        create_test_event("anthropic", "claude-3-haiku", 2000, 1000, "text"),
    ];
    
    let costs_by_model = calculator.calculate_cost_by_model(&events).unwrap();
    assert_eq!(costs_by_model.len(), 2);
    assert!(costs_by_model.contains_key("anthropic:claude-3-sonnet"));
    assert!(costs_by_model.contains_key("anthropic:claude-3-haiku"));
}

#[test]
fn test_cost_by_input_type() {
    let calculator = CostCalculator::new();
    
    let events = vec![
        create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "text"),
        create_test_event("anthropic", "claude-3-sonnet", 1000, 500, "image"),
    ];
    
    let costs_by_type = calculator.calculate_cost_by_input_type(&events).unwrap();
    assert_eq!(costs_by_type.len(), 2);
    assert!(costs_by_type.contains_key("text"));
    assert!(costs_by_type.contains_key("image"));
}

#[test]
fn test_budget_alert() {
    let calculator = CostCalculator::new();
    
    let events = vec![
        create_test_event("anthropic", "claude-3-sonnet", 10000, 5000, "text"),
    ];
    
    let alert = calculator.check_budget_alert(&events, 100.0, 80.0).unwrap();
    assert!(alert.percentage_used > 0.0);
    assert_eq!(alert.budget_limit, 100.0);
}

#[test]
fn test_alert_triggering() {
    let calculator = CostCalculator::new();
    
    let events = vec![
        create_test_event("anthropic", "claude-3-sonnet", 10000, 5000, "text"),
    ];
    
    let alert = calculator.check_budget_alert(&events, 100.0, 80.0).unwrap();
    let should_trigger = calculator.should_trigger_alert(&alert);
    
    if alert.percentage_used >= 80.0 {
        assert!(should_trigger);
    } else {
        assert!(!should_trigger);
    }
}

#[test]
fn test_cost_optimization_suggestions() {
    let calculator = CostCalculator::new();
    
    let events = vec![
        create_test_event("anthropic", "claude-3-opus", 10000, 5000, "text"),
    ];
    
    let suggestions = calculator.generate_cost_optimization_suggestions(&events).unwrap();
    assert!(!suggestions.is_empty());
}

#[test]
fn test_pricing_data_export_import() {
    let calculator = CostCalculator::new();
    
    let json_data = calculator.export_pricing_data().unwrap();
    assert!(!json_data.is_empty());
    
    let new_calculator = CostCalculator::new();
    let result = new_calculator.import_pricing_data(&json_data);
    assert!(result.is_ok());
}

#[test]
fn test_unknown_provider() {
    let calculator = CostCalculator::new();
    
    let event = create_test_event("unknown_provider", "unknown_model", 1000, 500, "text");
    
    let result = calculator.calculate_cost(&event);
    assert!(result.is_err());
}

#[test]
fn test_zero_tokens() {
    let calculator = CostCalculator::new();
    
    let event = create_test_event("anthropic", "claude-3-sonnet", 0, 0, "text");
    
    let cost = calculator.calculate_cost(&event).unwrap();
    assert_eq!(cost, 0.0);
}

#[test]
fn test_large_token_counts() {
    let calculator = CostCalculator::new();
    
    let event = create_test_event("anthropic", "claude-3-sonnet", 1_000_000, 500_000, "text");
    
    let cost = calculator.calculate_cost(&event).unwrap();
    assert!(cost > 0.0);
    assert!(cost.is_finite());
}
