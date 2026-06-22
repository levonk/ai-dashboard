use analytics_rs::{TextTokenEstimator, TokenEstimate, CostCalculator};
use anyhow::Result;
use serde_json::Value;

/// Token counting and cost estimation
/// Collects token usage and calculates costs for AI requests
pub struct TokenCollector {
    text_estimator: TextTokenEstimator,
}

impl TokenCollector {
    /// Create a new token collector
    pub fn new() -> Self {
        Self {
            text_estimator: TextTokenEstimator::new(),
        }
    }

    /// Estimate tokens for text input
    pub fn estimate_text_tokens(&self, text: &str, model: &str) -> Result<TokenEstimate> {
        Ok(self.text_estimator.estimate_by_chars(text, model))
    }

    /// Estimate tokens for a chat message
    pub fn estimate_chat_tokens(&self, messages: &[Value], model: &str) -> Result<TokenEstimate> {
        let chat_messages: Vec<analytics_rs::ChatMessage> = messages
            .iter()
            .filter_map(|m| {
                Some(analytics_rs::ChatMessage {
                    role: m.get("role")?.as_str()?.to_string(),
                    content: m.get("content")?.as_str()?.to_string(),
                })
            })
            .collect();

        Ok(self.text_estimator.estimate_chat_tokens(&chat_messages, model))
    }

    /// Calculate cost for token usage
    pub fn calculate_cost(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64> {
        let cost_calculator = CostCalculator::new();
        cost_calculator.estimate_cost(provider, model, input_tokens, output_tokens, "text")
    }

    /// Extract and count tokens from request
    pub fn extract_request_tokens(&self, request: &Value, model: &str) -> Result<TokenCount> {
        let input_text = self.extract_input_text(request)?;
        let input_estimate = self.estimate_text_tokens(&input_text, model)?;

        Ok(TokenCount {
            input_tokens: input_estimate.estimated_tokens,
            output_tokens: 0, // Will be updated from response
            total_tokens: input_estimate.estimated_tokens,
            input_characters: input_text.len() as u32,
        })
    }

    /// Update token count with response data
    pub fn update_with_response(&self, token_count: &mut TokenCount, response: &Value, model: &str) -> Result<()> {
        let output_text = self.extract_output_text(response)?;
        let output_estimate = self.estimate_text_tokens(&output_text, model)?;

        token_count.output_tokens = output_estimate.estimated_tokens;
        token_count.total_tokens = token_count.input_tokens + token_count.output_tokens;

        Ok(())
    }

    /// Extract input text from request
    fn extract_input_text(&self, request: &Value) -> Result<String> {
        if let Some(prompt) = request.get("prompt").and_then(|p| p.as_str()) {
            return Ok(prompt.to_string());
        }

        if let Some(messages) = request.get("messages").and_then(|m| m.as_array()) {
            return serde_json::to_string(messages)
                .map_err(|e| anyhow::anyhow!("Failed to serialize messages: {}", e));
        }

        if let Some(content) = request.get("content").and_then(|c| c.as_str()) {
            return Ok(content.to_string());
        }

        Ok(String::new())
    }

    /// Extract output text from response
    fn extract_output_text(&self, response: &Value) -> Result<String> {
        if let Some(content) = response.get("content").and_then(|c| c.as_str()) {
            return Ok(content.to_string());
        }

        if let Some(text) = response.get("text").and_then(|t| t.as_str()) {
            return Ok(text.to_string());
        }

        if let Some(completion) = response.get("completion").and_then(|c| c.as_str()) {
            return Ok(completion.to_string());
        }

        Ok(String::new())
    }

    /// Calculate cost for a complete request/response cycle
    pub fn calculate_total_cost(
        &self,
        provider: &str,
        model: &str,
        token_count: &TokenCount,
    ) -> Result<f64> {
        let cost_calculator = CostCalculator::new();
        cost_calculator.estimate_cost(
            provider,
            model,
            token_count.input_tokens,
            token_count.output_tokens,
            "text",
        )
    }
}

impl Default for TokenCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Token count for a request/response cycle
#[derive(Debug, Clone)]
pub struct TokenCount {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub input_characters: u32,
}

impl TokenCount {
    /// Create a new token count
    pub fn new() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            input_characters: 0,
        }
    }

    /// Calculate token efficiency (tokens per character)
    pub fn token_efficiency(&self) -> f64 {
        if self.input_characters > 0 {
            self.input_tokens as f64 / self.input_characters as f64
        } else {
            0.0
        }
    }

    /// Calculate output ratio (output / input)
    pub fn output_ratio(&self) -> f64 {
        if self.input_tokens > 0 {
            self.output_tokens as f64 / self.input_tokens as f64
        } else {
            0.0
        }
    }
}

impl Default for TokenCount {
    fn default() -> Self {
        Self::new()
    }
}

/// Cost information for a request
#[derive(Debug, Clone)]
pub struct CostInfo {
    pub input_cost_usd: f64,
    pub output_cost_usd: f64,
    pub total_cost_usd: f64,
    pub cost_per_1k_input_tokens: f64,
    pub cost_per_1k_output_tokens: f64,
}

impl CostInfo {
    /// Create new cost info
    pub fn new(
        input_cost_usd: f64,
        output_cost_usd: f64,
        cost_per_1k_input_tokens: f64,
        cost_per_1k_output_tokens: f64,
    ) -> Self {
        Self {
            input_cost_usd,
            output_cost_usd,
            total_cost_usd: input_cost_usd + output_cost_usd,
            cost_per_1k_input_tokens,
            cost_per_1k_output_tokens,
        }
    }

    /// Calculate cost per token
    pub fn cost_per_token(&self, total_tokens: u32) -> f64 {
        if total_tokens > 0 {
            self.total_cost_usd / total_tokens as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_collector_creation() {
        let collector = TokenCollector::new();
        // Collector is created successfully
    }

    #[test]
    fn test_estimate_text_tokens() {
        let collector = TokenCollector::new();
        let text = "Hello, world!";
        let estimate = collector.estimate_text_tokens(text, "gpt-4").unwrap();
        
        assert!(estimate.estimated_tokens > 0);
    }

    #[test]
    fn test_estimate_chat_tokens() {
        let collector = TokenCollector::new();
        let messages = serde_json::json!([
            {"role": "user", "content": "Hello"},
            {"role": "assistant", "content": "Hi there!"}
        ]);
        
        let estimate = collector.estimate_chat_tokens(messages.as_array().unwrap(), "claude-3-opus").unwrap();
        assert!(estimate.estimated_tokens > 0);
    }

    #[test]
    fn test_calculate_cost() {
        let cost = CostCalculator::estimate_cost("anthropic", "claude-3-opus", 1000, 500).unwrap();
        
        assert!(cost > 0.0);
    }

    #[test]
    fn test_extract_request_tokens() {
        let collector = TokenCollector::new();
        let request = serde_json::json!({
            "prompt": "Hello, world!"
        });
        
        let token_count = collector.extract_request_tokens(&request, "gpt-4").unwrap();
        assert!(token_count.input_tokens > 0);
        assert_eq!(token_count.output_tokens, 0);
    }

    #[test]
    fn test_update_with_response() {
        let collector = TokenCollector::new();
        let mut token_count = TokenCount {
            input_tokens: 10,
            output_tokens: 0,
            total_tokens: 10,
            input_characters: 50,
        };
        
        let response = serde_json::json!({
            "content": "Hi there!"
        });
        
        collector.update_with_response(&mut token_count, &response, "gpt-4").unwrap();
        assert!(token_count.output_tokens > 0);
        assert!(token_count.total_tokens > token_count.input_tokens);
    }

    #[test]
    fn test_token_count_new() {
        let token_count = TokenCount::new();
        assert_eq!(token_count.input_tokens, 0);
        assert_eq!(token_count.output_tokens, 0);
        assert_eq!(token_count.total_tokens, 0);
    }

    #[test]
    fn test_token_efficiency() {
        let token_count = TokenCount {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            input_characters: 500,
        };
        
        let efficiency = token_count.token_efficiency();
        assert_eq!(efficiency, 0.2);
    }

    #[test]
    fn test_output_ratio() {
        let token_count = TokenCount {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            input_characters: 500,
        };
        
        let ratio = token_count.output_ratio();
        assert_eq!(ratio, 0.5);
    }

    #[test]
    fn test_cost_info_creation() {
        let cost_info = CostInfo::new(0.01, 0.02, 0.01, 0.02);
        assert_eq!(cost_info.input_cost_usd, 0.01);
        assert_eq!(cost_info.output_cost_usd, 0.02);
        assert_eq!(cost_info.total_cost_usd, 0.03);
    }

    #[test]
    fn test_cost_per_token() {
        let cost_info = CostInfo::new(0.01, 0.02, 0.01, 0.02);
        let cost_per_token = cost_info.cost_per_token(100);
        assert_eq!(cost_per_token, 0.0003);
    }

    #[test]
    fn test_calculate_total_cost() {
        let collector = TokenCollector::new();
        let token_count = TokenCount {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            input_characters: 5000,
        };
        
        let cost = collector.calculate_total_cost("anthropic", "claude-3-opus", &token_count).unwrap();
        assert!(cost > 0.0);
    }
}