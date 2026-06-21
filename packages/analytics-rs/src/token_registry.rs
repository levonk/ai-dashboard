use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::tokens::{InputType, EstimationMethod};

/// Model-specific token estimation registry
/// 
/// This registry maintains model-specific token counting characteristics
/// and allows easy addition of new models without modifying core estimation logic.

/// Model configuration for token estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_id: String,
    pub provider: String,
    pub supported_input_types: Vec<InputType>,
    pub char_to_token_ratio: f64,
    pub word_to_token_ratio: f64,
    pub image_base_tokens: u32,
    pub audio_tokens_per_second: f64,
    pub video_tokens_per_second: f64,
    pub estimation_method: EstimationMethod,
}

/// Token estimation registry
pub struct TokenRegistry {
    models: HashMap<String, ModelConfig>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        
        // Initialize with default models
        registry.register_default_models();
        
        registry
    }
    
    /// Register a new model configuration
    pub fn register_model(&mut self, config: ModelConfig) {
        self.models.insert(config.model_id.clone(), config);
    }
    
    /// Get configuration for a specific model
    pub fn get_model_config(&self, model_id: &str) -> Option<&ModelConfig> {
        self.models.get(model_id)
    }
    
    /// Check if a model is registered
    pub fn has_model(&self, model_id: &str) -> bool {
        self.models.contains_key(model_id)
    }
    
    /// Get all registered models
    pub fn get_all_models(&self) -> Vec<&ModelConfig> {
        self.models.values().collect()
    }
    
    /// Get models by provider
    pub fn get_models_by_provider(&self, provider: &str) -> Vec<&ModelConfig> {
        self.models
            .values()
            .filter(|config| config.provider == provider)
            .collect()
    }
    
    /// Get models that support a specific input type
    pub fn get_models_by_input_type(&self, input_type: &InputType) -> Vec<&ModelConfig> {
        self.models
            .values()
            .filter(|config| config.supported_input_types.contains(input_type))
            .collect()
    }
    
    /// Remove a model from the registry
    pub fn unregister_model(&mut self, model_id: &str) -> Option<ModelConfig> {
        self.models.remove(model_id)
    }
    
    /// Initialize registry with default models
    fn register_default_models(&mut self) {
        // Anthropic Claude models
        self.register_model(ModelConfig {
            model_id: "claude-3-opus".to_string(),
            provider: "anthropic".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 0.0, // Not supported
            video_tokens_per_second: 0.0, // Not supported
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        self.register_model(ModelConfig {
            model_id: "claude-3-sonnet".to_string(),
            provider: "anthropic".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        self.register_model(ModelConfig {
            model_id: "claude-3-haiku".to_string(),
            provider: "anthropic".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        // OpenAI GPT models
        self.register_model(ModelConfig {
            model_id: "gpt-4".to_string(),
            provider: "openai".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 0,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        self.register_model(ModelConfig {
            model_id: "gpt-4-turbo".to_string(),
            provider: "openai".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        self.register_model(ModelConfig {
            model_id: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image, InputType::Audio, InputType::Video],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 32.0,
            video_tokens_per_second: 162.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        self.register_model(ModelConfig {
            model_id: "gpt-3.5-turbo".to_string(),
            provider: "openai".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 0,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        // Google Gemini models
        self.register_model(ModelConfig {
            model_id: "gemini-pro".to_string(),
            provider: "google".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        self.register_model(ModelConfig {
            model_id: "gemini-ultra".to_string(),
            provider: "google".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image, InputType::Video],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 130.0,
            estimation_method: EstimationMethod::ModelSpecific,
        });
        
        // OpenRouter (aggregator)
        self.register_model(ModelConfig {
            model_id: "openrouter/*".to_string(),
            provider: "openrouter".to_string(),
            supported_input_types: vec![InputType::Text, InputType::Chat, InputType::Image],
            char_to_token_ratio: 4.0,
            word_to_token_ratio: 0.75,
            image_base_tokens: 765,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::CharacterBased, // Fallback to character-based
        });
    }
}

impl Default for TokenRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_initialization() {
        let registry = TokenRegistry::new();
        assert!(registry.has_model("claude-3-opus"));
        assert!(registry.has_model("gpt-4"));
        assert!(registry.has_model("gemini-pro"));
    }
    
    #[test]
    fn test_get_model_config() {
        let registry = TokenRegistry::new();
        let config = registry.get_model_config("claude-3-opus");
        
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.char_to_token_ratio, 4.0);
    }
    
    #[test]
    fn test_register_custom_model() {
        let mut registry = TokenRegistry::new();
        
        let custom_config = ModelConfig {
            model_id: "custom-model".to_string(),
            provider: "custom".to_string(),
            supported_input_types: vec![InputType::Text],
            char_to_token_ratio: 5.0,
            word_to_token_ratio: 0.8,
            image_base_tokens: 0,
            audio_tokens_per_second: 0.0,
            video_tokens_per_second: 0.0,
            estimation_method: EstimationMethod::CharacterBased,
        };
        
        registry.register_model(custom_config);
        assert!(registry.has_model("custom-model"));
    }
    
    #[test]
    fn test_get_models_by_provider() {
        let registry = TokenRegistry::new();
        let anthropic_models = registry.get_models_by_provider("anthropic");
        
        assert!(!anthropic_models.is_empty());
        assert!(anthropic_models.iter().all(|m| m.provider == "anthropic"));
    }
    
    #[test]
    fn test_get_models_by_input_type() {
        let registry = TokenRegistry::new();
        let image_models = registry.get_models_by_input_type(&InputType::Image);
        
        assert!(!image_models.is_empty());
        assert!(image_models.iter().all(|m| m.supported_input_types.contains(&InputType::Image)));
    }
    
    #[test]
    fn test_unregister_model() {
        let mut registry = TokenRegistry::new();
        assert!(registry.has_model("claude-3-opus"));
        
        registry.unregister_model("claude-3-opus");
        assert!(!registry.has_model("claude-3-opus"));
    }
    
    #[test]
    fn test_get_all_models() {
        let registry = TokenRegistry::new();
        let all_models = registry.get_all_models();
        
        assert!(!all_models.is_empty());
        // Should have at least the default models we registered
        assert!(all_models.len() >= 10);
    }
}
