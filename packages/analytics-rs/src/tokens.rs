use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Token estimation utilities for different AI models and input types
/// 
/// This module provides token counting and estimation for various AI models
/// across different providers (Anthropic, OpenAI, Google, etc.) and input types
/// (text, image, audio, video).

/// Token estimation result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimate {
    pub estimated_tokens: u32,
    pub model: String,
    pub input_type: InputType,
    pub confidence: f64, // 0.0 to 1.0
    pub method: EstimationMethod,
}

/// Input type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputType {
    Text,
    Chat,
    Image,
    Audio,
    Video,
    Multimodal,
}

/// Token estimation method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EstimationMethod {
    CharacterBased, // Rough estimate based on character count
    WordBased,      // Rough estimate based on word count
    ModelSpecific,  // Provider-specific token counting
    Exact,          // Actual token count from API
}

/// Text token estimation for chat and text models
pub struct TextTokenEstimator {
    // Character-to-token ratios for different models
    char_ratios: HashMap<String, f64>,
    // Word-to-token ratios for different models
    word_ratios: HashMap<String, f64>,
}

impl TextTokenEstimator {
    pub fn new() -> Self {
        let mut char_ratios = HashMap::new();
        let mut word_ratios = HashMap::new();
        
        // Anthropic Claude models (approximately 4 chars per token)
        char_ratios.insert("claude-3-opus".to_string(), 4.0);
        char_ratios.insert("claude-3-sonnet".to_string(), 4.0);
        char_ratios.insert("claude-3-haiku".to_string(), 4.0);
        char_ratios.insert("claude-2.1".to_string(), 4.0);
        char_ratios.insert("claude-2.0".to_string(), 4.0);
        char_ratios.insert("claude-instant-1.2".to_string(), 4.0);
        
        // OpenAI GPT models (approximately 4 chars per token)
        char_ratios.insert("gpt-4".to_string(), 4.0);
        char_ratios.insert("gpt-4-turbo".to_string(), 4.0);
        char_ratios.insert("gpt-4o".to_string(), 4.0);
        char_ratios.insert("gpt-3.5-turbo".to_string(), 4.0);
        
        // Google Gemini models (approximately 4 chars per token)
        char_ratios.insert("gemini-pro".to_string(), 4.0);
        char_ratios.insert("gemini-ultra".to_string(), 4.0);
        
        // Word-based ratios (approximately 0.75 words per token)
        word_ratios.insert("default".to_string(), 0.75);
        
        Self {
            char_ratios,
            word_ratios,
        }
    }
    
    /// Estimate tokens for text content using character-based method
    pub fn estimate_by_chars(&self, text: &str, model: &str) -> TokenEstimate {
        let char_count = text.chars().count() as f64;
        let ratio = self.char_ratios.get(model).unwrap_or(&4.0);
        let estimated_tokens = (char_count / ratio).ceil() as u32;
        
        TokenEstimate {
            estimated_tokens,
            model: model.to_string(),
            input_type: InputType::Text,
            confidence: 0.85, // Character-based is reasonably accurate
            method: EstimationMethod::CharacterBased,
        }
    }
    
    /// Estimate tokens for text content using word-based method
    pub fn estimate_by_words(&self, text: &str, model: &str) -> TokenEstimate {
        let word_count = text.split_whitespace().count() as f64;
        let ratio = self.word_ratios.get("default").unwrap_or(&0.75);
        let estimated_tokens = (word_count / ratio).ceil() as u32;
        
        TokenEstimate {
            estimated_tokens,
            model: model.to_string(),
            input_type: InputType::Text,
            confidence: 0.75, // Word-based is less accurate than character-based
            method: EstimationMethod::WordBased,
        }
    }
    
    /// Estimate tokens for chat messages (combines system, user, assistant messages)
    pub fn estimate_chat_tokens(&self, messages: &[ChatMessage], model: &str) -> TokenEstimate {
        let total_text: String = messages.iter()
            .map(|m| format!("{}: {}\n", m.role, m.content))
            .collect();
        
        let mut estimate = self.estimate_by_chars(&total_text, model);
        estimate.input_type = InputType::Chat;
        // Add overhead for message formatting (approximately 3 tokens per message)
        estimate.estimated_tokens += (messages.len() * 3) as u32;
        
        estimate
    }
    
    /// Add or update a model's character-to-token ratio
    pub fn add_model_ratio(&mut self, model: String, ratio: f64) {
        self.char_ratios.insert(model, ratio);
    }
}

/// Chat message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

impl Default for TextTokenEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_by_chars() {
        let estimator = TextTokenEstimator::new();
        let text = "Hello, world! This is a test.";
        let estimate = estimator.estimate_by_chars(text, "gpt-4");
        
        assert!(estimate.estimated_tokens > 0);
        assert_eq!(estimate.model, "gpt-4");
        assert_eq!(estimate.input_type, InputType::Text);
    }
    
    #[test]
    fn test_estimate_by_words() {
        let estimator = TextTokenEstimator::new();
        let text = "Hello world this is a test";
        let estimate = estimator.estimate_by_words(text, "gpt-4");
        
        assert!(estimate.estimated_tokens > 0);
        assert_eq!(estimate.method, EstimationMethod::WordBased);
    }
    
    #[test]
    fn test_estimate_chat_tokens() {
        let estimator = TextTokenEstimator::new();
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
            },
        ];
        
        let estimate = estimator.estimate_chat_tokens(&messages, "claude-3-opus");
        assert_eq!(estimate.input_type, InputType::Chat);
        assert!(estimate.estimated_tokens > 0);
    }
    
    #[test]
    fn test_add_model_ratio() {
        let mut estimator = TextTokenEstimator::new();
        estimator.add_model_ratio("custom-model".to_string(), 5.0);
        
        let text = "Hello, world!";
        let estimate = estimator.estimate_by_chars(text, "custom-model");
        assert_eq!(estimate.model, "custom-model");
    }
    
    #[test]
    fn test_different_models_different_ratios() {
        let estimator = TextTokenEstimator::new();
        let text = "Hello, world!";
        
        let gpt_estimate = estimator.estimate_by_chars(text, "gpt-4");
        let claude_estimate = estimator.estimate_by_chars(text, "claude-3-opus");
        
        // Should be similar since both use 4.0 ratio
        assert_eq!(gpt_estimate.estimated_tokens, claude_estimate.estimated_tokens);
    }
}
