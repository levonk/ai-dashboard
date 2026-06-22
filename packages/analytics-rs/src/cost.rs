use crate::models::TelemetryEvent;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct CostCalculator {
    pricing_data: RwLock<PricingDatabase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModel {
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub input_type_factors: HashMap<String, f64>, // Multipliers for different input types
    pub cache_enabled: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingDatabase {
    pub models: HashMap<String, PricingModel>,
    pub providers: HashMap<String, ProviderConfig>,
    pub version: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub default_model: String,
    pub base_pricing: PricingModel,
    pub supported_input_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub input_cost: f64,
    pub output_cost: f64,
    pub input_type_multiplier: f64,
    pub total_cost: f64,
}

#[derive(Debug, Clone)]
pub struct BudgetAlert {
    pub budget_limit: f64,
    pub current_spend: f64,
    pub percentage_used: f64,
    pub alert_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct CostOptimizationSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub potential_savings: f64,
    pub confidence: f64,
}

impl CostCalculator {
    pub fn new() -> Self {
        Self {
            pricing_data: RwLock::new(Self::initialize_pricing_database()),
        }
    }

    fn initialize_pricing_database() -> PricingDatabase {
        let mut models = HashMap::new();
        let mut providers = HashMap::new();

        // Anthropic models
        models.insert("anthropic:claude-3-opus".to_string(), PricingModel {
            input_price_per_1k: 15.0,
            output_price_per_1k: 75.0,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });
        models.insert("anthropic:claude-3-sonnet".to_string(), PricingModel {
            input_price_per_1k: 3.0,
            output_price_per_1k: 15.0,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });
        models.insert("anthropic:claude-3-haiku".to_string(), PricingModel {
            input_price_per_1k: 0.25,
            output_price_per_1k: 1.25,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });

        // OpenAI models
        models.insert("openai:gpt-4".to_string(), PricingModel {
            input_price_per_1k: 30.0,
            output_price_per_1k: 60.0,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });
        models.insert("openai:gpt-4-turbo".to_string(), PricingModel {
            input_price_per_1k: 10.0,
            output_price_per_1k: 30.0,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });
        models.insert("openai:gpt-3.5-turbo".to_string(), PricingModel {
            input_price_per_1k: 0.5,
            output_price_per_1k: 1.5,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });

        // Google models
        models.insert("google:gemini-pro".to_string(), PricingModel {
            input_price_per_1k: 0.5,
            output_price_per_1k: 1.5,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });
        models.insert("google:gemini-ultra".to_string(), PricingModel {
            input_price_per_1k: 2.0,
            output_price_per_1k: 4.0,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });

        // Azure OpenAI models
        models.insert("azure:gpt-4".to_string(), PricingModel {
            input_price_per_1k: 30.0,
            output_price_per_1k: 60.0,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });
        models.insert("azure:gpt-35-turbo".to_string(), PricingModel {
            input_price_per_1k: 0.5,
            output_price_per_1k: 1.5,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });

        // AWS Bedrock models
        models.insert("aws:anthropic-claude".to_string(), PricingModel {
            input_price_per_1k: 8.0,
            output_price_per_1k: 24.0,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });
        models.insert("aws:ai21-jurassic".to_string(), PricingModel {
            input_price_per_1k: 2.5,
            output_price_per_1k: 2.5,
            input_type_factors: Self::default_input_factors(),
            cache_enabled: true,
            version: "2024-01".to_string(),
        });

        // Provider configurations
        providers.insert("anthropic".to_string(), ProviderConfig {
            default_model: "claude-3-sonnet".to_string(),
            base_pricing: PricingModel {
                input_price_per_1k: 3.0,
                output_price_per_1k: 15.0,
                input_type_factors: Self::default_input_factors(),
                cache_enabled: true,
                version: "2024-01".to_string(),
            },
            supported_input_types: vec!["text".to_string(), "image".to_string()],
        });

        providers.insert("openai".to_string(), ProviderConfig {
            default_model: "gpt-3.5-turbo".to_string(),
            base_pricing: PricingModel {
                input_price_per_1k: 0.5,
                output_price_per_1k: 1.5,
                input_type_factors: Self::default_input_factors(),
                cache_enabled: true,
                version: "2024-01".to_string(),
            },
            supported_input_types: vec!["text".to_string(), "image".to_string(), "audio".to_string()],
        });

        providers.insert("google".to_string(), ProviderConfig {
            default_model: "gemini-pro".to_string(),
            base_pricing: PricingModel {
                input_price_per_1k: 0.5,
                output_price_per_1k: 1.5,
                input_type_factors: Self::default_input_factors(),
                cache_enabled: true,
                version: "2024-01".to_string(),
            },
            supported_input_types: vec!["text".to_string(), "image".to_string(), "audio".to_string(), "video".to_string()],
        });

        providers.insert("azure".to_string(), ProviderConfig {
            default_model: "gpt-35-turbo".to_string(),
            base_pricing: PricingModel {
                input_price_per_1k: 0.5,
                output_price_per_1k: 1.5,
                input_type_factors: Self::default_input_factors(),
                cache_enabled: true,
                version: "2024-01".to_string(),
            },
            supported_input_types: vec!["text".to_string(), "image".to_string()],
        });

        providers.insert("aws".to_string(), ProviderConfig {
            default_model: "anthropic-claude".to_string(),
            base_pricing: PricingModel {
                input_price_per_1k: 8.0,
                output_price_per_1k: 24.0,
                input_type_factors: Self::default_input_factors(),
                cache_enabled: true,
                version: "2024-01".to_string(),
            },
            supported_input_types: vec!["text".to_string(), "image".to_string()],
        });

        providers.insert("openrouter".to_string(), ProviderConfig {
            default_model: "default".to_string(),
            base_pricing: PricingModel {
                input_price_per_1k: 1.0,
                output_price_per_1k: 2.0,
                input_type_factors: Self::default_input_factors(),
                cache_enabled: true,
                version: "2024-01".to_string(),
            },
            supported_input_types: vec!["text".to_string()],
        });

        PricingDatabase {
            models,
            providers,
            version: "1.0.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn default_input_factors() -> HashMap<String, f64> {
        let mut factors = HashMap::new();
        factors.insert("text".to_string(), 1.0);
        factors.insert("image".to_string(), 1.5); // Images typically cost more
        factors.insert("audio".to_string(), 2.0); // Audio processing is more expensive
        factors.insert("video".to_string(), 5.0); // Video is most expensive
        factors
    }
    pub fn calculate_cost(&self, event: &TelemetryEvent) -> Result<f64> {
        // If the event already has a calculated cost, use it
        if event.cost_usd > 0.0 {
            return Ok(event.cost_usd);
        }
        
        // Otherwise, estimate based on provider and model
        self.estimate_cost(&event.ai_provider, &event.model, event.input_tokens, event.output_tokens, &event.input_type)
    }

    pub fn calculate_cost_with_breakdown(&self, event: &TelemetryEvent) -> Result<CostBreakdown> {
        let pricing = self.get_pricing_model(&event.ai_provider, &event.model)?;
        
        let input_type_factor = pricing.input_type_factors
            .get(&event.input_type)
            .unwrap_or(&1.0);
        
        let input_cost = (event.input_tokens as f64 / 1000.0) * pricing.input_price_per_1k * input_type_factor;
        let output_cost = (event.output_tokens as f64 / 1000.0) * pricing.output_price_per_1k;
        let total_cost = input_cost + output_cost;
        
        Ok(CostBreakdown {
            input_cost,
            output_cost,
            input_type_multiplier: *input_type_factor,
            total_cost,
        })
    }

    pub fn calculate_total_cost(&self, events: &[TelemetryEvent]) -> Result<f64> {
        let total: f64 = events.iter()
            .map(|e| self.calculate_cost(e).unwrap_or(0.0))
            .sum();
        Ok(total)
    }

    pub fn estimate_cost(&self, provider: &str, model: &str, input_tokens: u32, output_tokens: u32, input_type: &str) -> Result<f64> {
        let pricing = self.get_pricing_model(provider, model)?;
        
        let input_type_factor = pricing.input_type_factors
            .get(input_type)
            .unwrap_or(&1.0);
        
        let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_price_per_1k * input_type_factor;
        let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_price_per_1k;
        
        Ok(input_cost + output_cost)
    }
    
    pub fn get_pricing_model(&self, provider: &str, model: &str) -> Result<PricingModel> {
        let pricing_db = self.pricing_data.read().unwrap();
        
        let key = format!("{}:{}", provider.to_lowercase(), model.to_lowercase());
        
        pricing_db.models.get(&key)
            .cloned()
            .or_else(|| {
                // Try provider-level default pricing
                pricing_db.providers.get(&provider.to_lowercase())
                    .map(|config| config.base_pricing.clone())
            })
            .ok_or_else(|| anyhow::anyhow!("No pricing model found for provider: {}, model: {}", provider, model))
    }
    
    pub fn calculate_cost_by_provider(&self, events: &[TelemetryEvent]) -> Result<HashMap<String, f64>> {
        let mut costs_by_provider: HashMap<String, f64> = HashMap::new();
        
        for event in events {
            let cost = self.calculate_cost(event)?;
            *costs_by_provider.entry(event.ai_provider.clone()).or_insert(0.0) += cost;
        }
        
        Ok(costs_by_provider)
    }
    
    pub fn calculate_cost_by_model(&self, events: &[TelemetryEvent]) -> Result<HashMap<String, f64>> {
        let mut costs_by_model: HashMap<String, f64> = HashMap::new();
        
        for event in events {
            let cost = self.calculate_cost(event)?;
            let key = format!("{}:{}", event.ai_provider, event.model);
            *costs_by_model.entry(key).or_insert(0.0) += cost;
        }
        
        Ok(costs_by_model)
    }

    pub fn calculate_cost_by_input_type(&self, events: &[TelemetryEvent]) -> Result<HashMap<String, f64>> {
        let mut costs_by_type: HashMap<String, f64> = HashMap::new();
        
        for event in events {
            let cost = self.calculate_cost(event)?;
            *costs_by_type.entry(event.input_type.clone()).or_insert(0.0) += cost;
        }
        
        Ok(costs_by_type)
    }

    pub fn check_budget_alert(&self, events: &[TelemetryEvent], budget_limit: f64, alert_threshold: f64) -> Result<BudgetAlert> {
        let current_spend = self.calculate_total_cost(events)?;
        let percentage_used = (current_spend / budget_limit) * 100.0;
        
        Ok(BudgetAlert {
            budget_limit,
            current_spend,
            percentage_used,
            alert_threshold,
        })
    }

    pub fn should_trigger_alert(&self, alert: &BudgetAlert) -> bool {
        alert.percentage_used >= alert.alert_threshold
    }

    pub fn generate_cost_optimization_suggestions(&self, events: &[TelemetryEvent]) -> Result<Vec<CostOptimizationSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Analyze by provider
        let costs_by_provider = self.calculate_cost_by_provider(events)?;
        if let Some((provider, cost)) = costs_by_provider.iter().max_by(|a, b| a.1.total_cmp(b.1)) {
            if cost > &100.0 {
                suggestions.push(CostOptimizationSuggestion {
                    suggestion_type: "provider_optimization".to_string(),
                    description: format!("Consider optimizing usage of {} which accounts for ${:.2} of spend", provider, cost),
                    potential_savings: cost * 0.2, // Estimate 20% potential savings
                    confidence: 0.7,
                });
            }
        }
        
        // Analyze by model
        let costs_by_model = self.calculate_cost_by_model(events)?;
        if let Some((model, cost)) = costs_by_model.iter().max_by(|a, b| a.1.total_cmp(b.1)) {
            if cost > &50.0 {
                suggestions.push(CostOptimizationSuggestion {
                    suggestion_type: "model_optimization".to_string(),
                    description: format!("Consider using a smaller model instead of {} to save ${:.2}", model, cost * 0.3),
                    potential_savings: cost * 0.3,
                    confidence: 0.8,
                });
            }
        }
        
        // Analyze by input type
        let costs_by_type = self.calculate_cost_by_input_type(events)?;
        if let Some((input_type, cost)) = costs_by_type.iter().max_by(|a, b| a.1.total_cmp(b.1)) {
            if input_type != "text" && cost > &20.0 {
                suggestions.push(CostOptimizationSuggestion {
                    suggestion_type: "input_type_optimization".to_string(),
                    description: format!("Consider reducing {} processing which costs ${:.2}", input_type, cost),
                    potential_savings: cost * 0.4,
                    confidence: 0.6,
                });
            }
        }
        
        Ok(suggestions)
    }

    pub fn update_pricing_data(&self, new_pricing: PricingDatabase) -> Result<()> {
        let mut pricing_db = self.pricing_data.write().unwrap();
        *pricing_db = new_pricing;
        Ok(())
    }

    pub fn get_pricing_database(&self) -> PricingDatabase {
        self.pricing_data.read().unwrap().clone()
    }

    pub fn export_pricing_data(&self) -> Result<String> {
        let pricing_db = self.get_pricing_database();
        serde_json::to_string_pretty(&pricing_db)
            .map_err(|e| anyhow::anyhow!("Failed to serialize pricing data: {}", e))
    }

    pub fn import_pricing_data(&self, json_data: &str) -> Result<()> {
        let new_pricing: PricingDatabase = serde_json::from_str(json_data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize pricing data: {}", e))?;
        self.update_pricing_data(new_pricing)
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}