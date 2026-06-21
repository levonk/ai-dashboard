use crate::models::TelemetryEvent;
use anyhow::Result;
use std::collections::HashMap;

pub struct CostCalculator;

#[derive(Debug, Clone)]
pub struct PricingModel {
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
}

impl CostCalculator {
    pub fn calculate_cost(event: &TelemetryEvent) -> Result<f64> {
        // If the event already has a calculated cost, use it
        if event.cost_usd > 0.0 {
            return Ok(event.cost_usd);
        }
        
        // Otherwise, estimate based on provider and model
        Self::estimate_cost(&event.ai_provider, &event.model, event.input_tokens, event.output_tokens)
    }

    pub fn calculate_total_cost(events: &[TelemetryEvent]) -> Result<f64> {
        let total: f64 = events.iter()
            .map(|e| Self::calculate_cost(e).unwrap_or(0.0))
            .sum();
        Ok(total)
    }

    pub fn estimate_cost(provider: &str, model: &str, input_tokens: u32, output_tokens: u32) -> Result<f64> {
        let pricing = Self::get_pricing_model(provider, model)?;
        
        let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_price_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_price_per_1k;
        
        Ok(input_cost + output_cost)
    }
    
    pub fn get_pricing_model(provider: &str, model: &str) -> Result<PricingModel> {
        let pricing = Self::get_pricing_database();
        
        let key = format!("{}:{}", provider.to_lowercase(), model.to_lowercase());
        
        pricing.get(&key)
            .cloned()
            .or_else(|| {
                // Try provider-level default pricing
                pricing.get(&provider.to_lowercase())
                    .cloned()
            })
            .ok_or_else(|| anyhow::anyhow!("No pricing model found for provider: {}, model: {}", provider, model))
    }
    
    fn get_pricing_database() -> HashMap<String, PricingModel> {
        let mut pricing = HashMap::new();
        
        // Anthropic pricing (as of 2024)
        pricing.insert("anthropic:claude-3-opus".to_string(), PricingModel {
            input_price_per_1k: 15.0,
            output_price_per_1k: 75.0,
        });
        pricing.insert("anthropic:claude-3-sonnet".to_string(), PricingModel {
            input_price_per_1k: 3.0,
            output_price_per_1k: 15.0,
        });
        pricing.insert("anthropic:claude-3-haiku".to_string(), PricingModel {
            input_price_per_1k: 0.25,
            output_price_per_1k: 1.25,
        });
        pricing.insert("anthropic".to_string(), PricingModel {
            input_price_per_1k: 3.0,  // Default to Sonnet pricing
            output_price_per_1k: 15.0,
        });
        
        // OpenAI pricing (as of 2024)
        pricing.insert("openai:gpt-4".to_string(), PricingModel {
            input_price_per_1k: 30.0,
            output_price_per_1k: 60.0,
        });
        pricing.insert("openai:gpt-4-turbo".to_string(), PricingModel {
            input_price_per_1k: 10.0,
            output_price_per_1k: 30.0,
        });
        pricing.insert("openai:gpt-3.5-turbo".to_string(), PricingModel {
            input_price_per_1k: 0.5,
            output_price_per_1k: 1.5,
        });
        pricing.insert("openai".to_string(), PricingModel {
            input_price_per_1k: 0.5,  // Default to GPT-3.5 pricing
            output_price_per_1k: 1.5,
        });
        
        // Google pricing (as of 2024)
        pricing.insert("google:gemini-pro".to_string(), PricingModel {
            input_price_per_1k: 0.5,
            output_price_per_1k: 1.5,
        });
        pricing.insert("google:gemini-ultra".to_string(), PricingModel {
            input_price_per_1k: 2.0,
            output_price_per_1k: 4.0,
        });
        pricing.insert("google".to_string(), PricingModel {
            input_price_per_1k: 0.5,  // Default to Gemini Pro pricing
            output_price_per_1k: 1.5,
        });
        
        // Azure OpenAI pricing (similar to OpenAI)
        pricing.insert("azure:gpt-4".to_string(), PricingModel {
            input_price_per_1k: 30.0,
            output_price_per_1k: 60.0,
        });
        pricing.insert("azure:gpt-35-turbo".to_string(), PricingModel {
            input_price_per_1k: 0.5,
            output_price_per_1k: 1.5,
        });
        pricing.insert("azure".to_string(), PricingModel {
            input_price_per_1k: 0.5,
            output_price_per_1k: 1.5,
        });
        
        // AWS Bedrock pricing (varies by provider)
        pricing.insert("aws:anthropic-claude".to_string(), PricingModel {
            input_price_per_1k: 8.0,
            output_price_per_1k: 24.0,
        });
        pricing.insert("aws:ai21-jurassic".to_string(), PricingModel {
            input_price_per_1k: 2.5,
            output_price_per_1k: 2.5,
        });
        pricing.insert("aws".to_string(), PricingModel {
            input_price_per_1k: 8.0,  // Default to Claude pricing
            output_price_per_1k: 24.0,
        });
        
        // OpenRouter pricing (varies)
        pricing.insert("openrouter".to_string(), PricingModel {
            input_price_per_1k: 1.0,  // Conservative default
            output_price_per_1k: 2.0,
        });
        
        pricing
    }
    
    pub fn calculate_cost_by_provider(events: &[TelemetryEvent]) -> Result<HashMap<String, f64>> {
        let mut costs_by_provider: HashMap<String, f64> = HashMap::new();
        
        for event in events {
            let cost = Self::calculate_cost(event)?;
            *costs_by_provider.entry(event.ai_provider.clone()).or_insert(0.0) += cost;
        }
        
        Ok(costs_by_provider)
    }
    
    pub fn calculate_cost_by_model(events: &[TelemetryEvent]) -> Result<HashMap<String, f64>> {
        let mut costs_by_model: HashMap<String, f64> = HashMap::new();
        
        for event in events {
            let cost = Self::calculate_cost(event)?;
            let key = format!("{}:{}", event.ai_provider, event.model);
            *costs_by_model.entry(key).or_insert(0.0) += cost;
        }
        
        Ok(costs_by_model)
    }
}