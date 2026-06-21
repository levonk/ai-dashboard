use analytics_rs::Metadata;
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Request metadata extractor
/// Extracts standardized metadata from proxy requests
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract metadata from a proxy request
    pub fn extract_from_request(
        ai_client: &str,
        pipeline_stage: &str,
        provider: &str,
        model: &str,
        input_type: &str,
        additional_fields: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Metadata> {
        let mut metadata = Metadata::new(
            ai_client.to_string(),
            pipeline_stage.to_string(),
            provider.to_string(),
            model.to_string(),
            input_type.to_string(),
        );

        // Add additional custom fields if provided
        if let Some(fields) = additional_fields {
            for (key, value) in fields {
                metadata = metadata.with_custom_field(key, value);
            }
        }

        // Validate the metadata
        let validation = analytics_rs::validate_metadata(&metadata);
        if !validation.is_valid {
            anyhow::bail!(
                "Metadata validation failed: {}",
                validation.errors.join(", ")
            );
        }

        Ok(metadata)
    }

    /// Extract AI client from request headers
    pub fn extract_ai_client(headers: &HashMap<String, String>) -> Result<String> {
        headers
            .get("x-ai-client")
            .or_else(|| headers.get("user-agent"))
            .cloned()
            .context("Failed to extract AI client from headers")
    }

    /// Extract pipeline stage from request context
    pub fn extract_pipeline_stage(context: &HashMap<String, String>) -> String {
        context
            .get("pipeline_stage")
            .cloned()
            .unwrap_or_else(|| "development".to_string())
    }

    /// Extract provider from request
    pub fn extract_provider(request: &serde_json::Value) -> Result<String> {
        request
            .get("provider")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .context("Failed to extract provider from request")
    }

    /// Extract model from request
    pub fn extract_model(request: &serde_json::Value) -> Result<String> {
        request
            .get("model")
            .and_then(|m| m.as_str())
            .map(|s| s.to_string())
            .context("Failed to extract model from request")
    }

    /// Detect input type from request content
    pub fn detect_input_type(request: &serde_json::Value) -> String {
        // Check for image content
        if request.get("image").is_some() || request.get("images").is_some() {
            return "image".to_string();
        }

        // Check for audio content
        if request.get("audio").is_some() || request.get("audio_url").is_some() {
            return "audio".to_string();
        }

        // Check for video content
        if request.get("video").is_some() || request.get("video_url").is_some() {
            return "video".to_string();
        }

        // Default to chat/text
        "chat".to_string()
    }

    /// Extract user identifier from request
    pub fn extract_user_id(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("x-user-id").cloned()
    }

    /// Extract team identifier from request
    pub fn extract_team_id(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("x-team-id").cloned()
    }

    /// Extract client identifier from request
    pub fn extract_client_id(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("x-client-id").cloned()
    }

    /// Build complete metadata from request components
    pub fn build_metadata(
        headers: &HashMap<String, String>,
        request: &serde_json::Value,
        context: &HashMap<String, String>,
    ) -> Result<Metadata> {
        let ai_client = Self::extract_ai_client(headers)?;
        let pipeline_stage = Self::extract_pipeline_stage(context);
        let provider = Self::extract_provider(request)?;
        let model = Self::extract_model(request)?;
        let input_type = Self::detect_input_type(request);

        let mut metadata = Self::extract_from_request(
            &ai_client,
            &pipeline_stage,
            &provider,
            &model,
            &input_type,
            None,
        )?;

        // Add optional identifiers if present
        if let Some(client_id) = Self::extract_client_id(headers) {
            metadata = metadata.with_client_id(client_id);
        }

        if let Some(team_id) = Self::extract_team_id(headers) {
            metadata = metadata.with_team_id(team_id);
        }

        if let Some(user_id) = Self::extract_user_id(headers) {
            metadata = metadata.with_custom_field("user_id".to_string(), serde_json::json!(user_id));
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ai_client() {
        let mut headers = HashMap::new();
        headers.insert("x-ai-client".to_string(), "claude-code".to_string());

        let result = MetadataExtractor::extract_ai_client(&headers).unwrap();
        assert_eq!(result, "claude-code");
    }

    #[test]
    fn test_extract_pipeline_stage() {
        let mut context = HashMap::new();
        context.insert("pipeline_stage".to_string(), "production".to_string());

        let result = MetadataExtractor::extract_pipeline_stage(&context);
        assert_eq!(result, "production");
    }

    #[test]
    fn test_extract_pipeline_stage_default() {
        let context = HashMap::new();

        let result = MetadataExtractor::extract_pipeline_stage(&context);
        assert_eq!(result, "development");
    }

    #[test]
    fn test_detect_input_type_image() {
        let request = serde_json::json!({
            "image": "base64_encoded_image"
        });

        let result = MetadataExtractor::detect_input_type(&request);
        assert_eq!(result, "image");
    }

    #[test]
    fn test_detect_input_type_audio() {
        let request = serde_json::json!({
            "audio_url": "https://example.com/audio.mp3"
        });

        let result = MetadataExtractor::detect_input_type(&request);
        assert_eq!(result, "audio");
    }

    #[test]
    fn test_detect_input_type_default() {
        let request = serde_json::json!({
            "prompt": "Hello world"
        });

        let result = MetadataExtractor::detect_input_type(&request);
        assert_eq!(result, "chat");
    }

    #[test]
    fn test_extract_provider() {
        let request = serde_json::json!({
            "provider": "anthropic"
        });

        let result = MetadataExtractor::extract_provider(&request).unwrap();
        assert_eq!(result, "anthropic");
    }

    #[test]
    fn test_extract_model() {
        let request = serde_json::json!({
            "model": "claude-3-opus"
        });

        let result = MetadataExtractor::extract_model(&request).unwrap();
        assert_eq!(result, "claude-3-opus");
    }

    #[test]
    fn test_extract_user_id() {
        let mut headers = HashMap::new();
        headers.insert("x-user-id".to_string(), "user123".to_string());

        let result = MetadataExtractor::extract_user_id(&headers);
        assert_eq!(result, Some("user123".to_string()));
    }

    #[test]
    fn test_extract_team_id() {
        let mut headers = HashMap::new();
        headers.insert("x-team-id".to_string(), "team456".to_string());

        let result = MetadataExtractor::extract_team_id(&headers);
        assert_eq!(result, Some("team456".to_string()));
    }

    #[test]
    fn test_extract_client_id() {
        let mut headers = HashMap::new();
        headers.insert("x-client-id".to_string(), "client789".to_string());

        let result = MetadataExtractor::extract_client_id(&headers);
        assert_eq!(result, Some("client789".to_string()));
    }

    #[test]
    fn test_build_metadata() {
        let mut headers = HashMap::new();
        headers.insert("x-ai-client".to_string(), "claude-code".to_string());
        headers.insert("x-user-id".to_string(), "user123".to_string());

        let request = serde_json::json!({
            "provider": "anthropic",
            "model": "claude-3-opus"
        });

        let mut context = HashMap::new();
        context.insert("pipeline_stage".to_string(), "production".to_string());

        let result = MetadataExtractor::build_metadata(&headers, &request, &context).unwrap();

        assert_eq!(result.ai_client, "claude-code");
        assert_eq!(result.pipeline_stage, "production");
        assert_eq!(result.provider, "anthropic");
        assert_eq!(result.model, "claude-3-opus");
        assert_eq!(result.input_type, "chat");
        assert!(result.custom_fields.contains_key("user_id"));
    }
}