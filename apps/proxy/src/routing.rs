use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, warn};

/// Supported AI providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AIProvider {
    Anthropic,
    OpenAI,
    Google,
    Azure,
    AWS,
    OpenRouter,
    Custom(String),
}

impl AIProvider {
    /// Parse provider from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "anthropic" => Ok(AIProvider::Anthropic),
            "openai" => Ok(AIProvider::OpenAI),
            "google" => Ok(AIProvider::Google),
            "azure" => Ok(AIProvider::Azure),
            "aws" => Ok(AIProvider::AWS),
            "openrouter" => Ok(AIProvider::OpenRouter),
            other => Ok(AIProvider::Custom(other.to_string())),
        }
    }

    /// Get base URL for provider
    pub fn base_url(&self) -> &'static str {
        match self {
            AIProvider::Anthropic => "https://api.anthropic.com",
            AIProvider::OpenAI => "https://api.openai.com",
            AIProvider::Google => "https://generativelanguage.googleapis.com",
            AIProvider::Azure => "https://openai.azure.com",
            AIProvider::AWS => "https://bedrock.amazonaws.com",
            AIProvider::OpenRouter => "https://openrouter.ai/api",
            AIProvider::Custom(_) => "https://api.custom.com",
        }
    }

    /// Get API key header name for provider
    pub fn api_key_header(&self) -> &'static str {
        match self {
            AIProvider::Anthropic => "x-api-key",
            AIProvider::OpenAI => "authorization",
            AIProvider::Google => "x-goog-api-key",
            AIProvider::Azure => "api-key",
            AIProvider::AWS => "x-amz-security-token",
            AIProvider::OpenRouter => "authorization",
            AIProvider::Custom(_) => "x-api-key",
        }
    }
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: AIProvider,
    pub api_key: String,
    pub base_url: Option<String>,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::OpenAI,
            api_key: String::new(),
            base_url: None,
            timeout_ms: 30000,
            max_retries: 3,
        }
    }
}

/// Routing configuration
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    pub providers: HashMap<String, ProviderConfig>,
    pub default_provider: String,
    pub enable_load_balancing: bool,
    pub enable_circuit_breaker: bool,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                provider: AIProvider::OpenAI,
                api_key: String::new(),
                base_url: None,
                timeout_ms: 30000,
                max_retries: 3,
            },
        );

        Self {
            providers,
            default_provider: "openai".to_string(),
            enable_load_balancing: false,
            enable_circuit_breaker: true,
        }
    }
}

/// Router state
#[derive(Debug, Clone)]
pub struct RouterState {
    pub config: Arc<RoutingConfig>,
}

/// Route request to appropriate AI provider
pub async fn route_request(
    State(state): State<RouterState>,
    mut req: Request,
) -> Result<Response, StatusCode> {
    // Extract provider from request path or headers
    let provider_name = extract_provider(&req).unwrap_or_else(|| state.config.default_provider.clone());
    
    // Get provider configuration
    let provider_config = state.config.providers.get(&provider_name)
        .ok_or_else(|| {
            warn!("Provider not found: {}", provider_name);
            StatusCode::NOT_FOUND
        })?;

    debug!("Routing request to provider: {}", provider_name);

    // Build target URL
    let base_url = provider_config.base_url
        .as_deref()
        .unwrap_or_else(|| provider_config.provider.base_url());
    
    let path = req.uri().path();
    let query = req.uri().query().unwrap_or("");
    let target_url = if query.is_empty() {
        format!("{}{}", base_url, path)
    } else {
        format!("{}{}?{}", base_url, path, query)
    };

    // Update request URI and headers
    *req.uri_mut() = target_url.parse()
        .map_err(|e| {
            error!("Failed to parse target URL: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // Add API key header if not present
    let headers = req.headers_mut();
    let api_key_header = provider_config.provider.api_key_header();
    if !headers.contains_key(api_key_header) {
        headers.insert(
            api_key_header,
            provider_config.api_key.parse().unwrap_or_else(|_| {
                warn!("Failed to parse API key for provider: {}", provider_name);
                "".parse().unwrap()
            }),
        );
    }

    // Forward request to provider
    forward_request(req, provider_config).await
        .map_err(|e| {
            error!("Failed to forward request: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Extract provider name from request
fn extract_provider(req: &Request) -> Option<String> {
    // Try to extract from path first
    let path = req.uri().path();
    let segments: Vec<&str> = path.split('/').collect();
    
    if segments.len() > 1 {
        // Check if first segment is a known provider
        let potential_provider = segments.get(1)?;
        if matches!(potential_provider.to_lowercase().as_str(), 
            "anthropic" | "openai" | "google" | "azure" | "aws" | "openrouter") {
            return Some((*potential_provider).to_string());
        }
    }

    // Try to extract from header
    req.headers()
        .get("x-ai-provider")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Forward request to AI provider
async fn forward_request(
    req: Request,
    config: &ProviderConfig,
) -> Result<Response> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(config.timeout_ms))
        .build()
        .context("Failed to create HTTP client")?;

    // Convert axum Request to reqwest Request
    let method = req.method().clone();
    let url = req.uri().to_string();
    let headers = req.headers().clone();
    
    // Extract body
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Err(anyhow::anyhow!("Failed to read request body"));
        }
    };

    // Build reqwest request
    let mut request_builder = client.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes())
            .map_err(|e| anyhow::anyhow!("Invalid method: {}", e))?,
        &url,
    );

    // Add headers
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            request_builder = request_builder.header(name.as_str(), value_str);
        }
    }

    // Add body if present
    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes.to_vec());
    }

    // Execute request with retries
    let mut last_error = None;
    for attempt in 0..=config.max_retries {
        match request_builder.try_clone()
            .ok_or_else(|| anyhow::anyhow!("Failed to clone request"))?
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();
                let body = response.bytes().await
                    .context("Failed to read response body")?;

                // Build axum response
                let mut response_builder = Response::builder()
                    .status(StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));

                // Copy headers (convert reqwest headers to axum headers)
                for (name, value) in headers.iter() {
                    let name_str = name.as_str();
                    if let Ok(value_str) = value.to_str() {
                        if let Ok(header_name) = axum::http::HeaderName::try_from(name_str) {
                            if let Ok(header_value) = axum::http::HeaderValue::try_from(value_str) {
                                response_builder = response_builder.header(header_name, header_value);
                            }
                        }
                    }
                }

                return response_builder
                    .body(Body::from(body))
                    .context("Failed to build response");
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_retries {
                    debug!("Retry attempt {} for request to {}", attempt + 1, url);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                }
            }
        }
    }

    Err(last_error.map(|e| anyhow::anyhow!("Request failed after retries: {}", e))
        .unwrap_or_else(|| anyhow::anyhow!("Request failed after retries")))
}

/// Health check for routing service
pub async fn routing_health_check() -> &'static str {
    "Routing service is healthy"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_str() {
        assert_eq!(AIProvider::from_str("anthropic").unwrap(), AIProvider::Anthropic);
        assert_eq!(AIProvider::from_str("OPENAI").unwrap(), AIProvider::OpenAI);
        assert_eq!(AIProvider::from_str("custom").unwrap(), AIProvider::Custom("custom".to_string()));
    }

    #[test]
    fn test_provider_base_url() {
        assert_eq!(AIProvider::Anthropic.base_url(), "https://api.anthropic.com");
        assert_eq!(AIProvider::OpenAI.base_url(), "https://api.openai.com");
    }

    #[test]
    fn test_provider_api_key_header() {
        assert_eq!(AIProvider::Anthropic.api_key_header(), "x-api-key");
        assert_eq!(AIProvider::OpenAI.api_key_header(), "authorization");
    }

    #[test]
    fn test_routing_config_default() {
        let config = RoutingConfig::default();
        assert_eq!(config.default_provider, "openai");
        assert!(config.providers.contains_key("openai"));
        assert!(config.enable_circuit_breaker);
    }
}
