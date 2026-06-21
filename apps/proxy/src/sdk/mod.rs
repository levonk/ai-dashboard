//! Proxy SDK for AI client integration
//! 
//! This SDK provides a simple interface for AI clients (Claude Code, Codex, Pi, Devin, etc.)
//! to integrate with the AI Analytics Proxy service.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// SDK configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySdkConfig {
    /// Proxy endpoint URL
    pub proxy_url: String,
    /// API key for proxy authentication
    pub api_key: Option<String>,
    /// Client identifier
    pub client_id: String,
    /// User identifier (optional)
    pub user_id: Option<String>,
    /// Session identifier (optional)
    pub session_id: Option<String>,
    /// Enable telemetry collection
    pub enable_telemetry: bool,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ProxySdkConfig {
    fn default() -> Self {
        Self {
            proxy_url: "http://localhost:8080".to_string(),
            api_key: None,
            client_id: "default_client".to_string(),
            user_id: None,
            session_id: None,
            enable_telemetry: true,
            timeout_ms: 30000,
        }
    }
}

/// Proxy SDK client
pub struct ProxyClient {
    config: ProxySdkConfig,
    http_client: reqwest::Client,
}

impl ProxyClient {
    /// Create a new proxy client
    pub fn new(config: ProxySdkConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .context("Failed to create HTTP client")?;

        info!("Created proxy client for: {}", config.proxy_url);

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = ProxySdkConfig {
            proxy_url: std::env::var("AI_PROXY_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("AI_PROXY_API_KEY").ok(),
            client_id: std::env::var("AI_PROXY_CLIENT_ID")
                .unwrap_or_else(|_| "default_client".to_string()),
            user_id: std::env::var("AI_PROXY_USER_ID").ok(),
            session_id: std::env::var("AI_PROXY_SESSION_ID").ok(),
            enable_telemetry: std::env::var("AI_PROXY_ENABLE_TELEMETRY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            timeout_ms: std::env::var("AI_PROXY_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
        };

        Self::new(config)
    }

    /// Send a request to an AI provider through the proxy
    pub async fn send_request(
        &self,
        provider: &str,
        path: &str,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Result<ProxyResponse> {
        let url = format!("{}/{}{}", self.config.proxy_url, provider, path);
        debug!("Sending proxy request to: {}", url);

        let mut request_builder = self.http_client
            .request(reqwest::Method::POST, &url);

        // Add SDK headers
        request_builder = request_builder.header("x-client-id", &self.config.client_id);
        if let Some(ref user_id) = self.config.user_id {
            request_builder = request_builder.header("x-user-id", user_id);
        }
        if let Some(ref session_id) = self.config.session_id {
            request_builder = request_builder.header("x-session-id", session_id);
        }
        if let Some(ref api_key) = self.config.api_key {
            request_builder = request_builder.header("authorization", format!("Bearer {}", api_key));
        }

        // Add custom headers
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }

        // Add body if present
        if let Some(body) = body {
            request_builder = request_builder.header("content-type", "application/json");
            request_builder = request_builder.body(body);
        }

        let response = request_builder
            .send()
            .await
            .context("Failed to send proxy request")?;

        let status = response.status();
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();
        let response_body = response.text().await
            .context("Failed to read response body")?;

        debug!("Received proxy response with status: {}", status);

        Ok(ProxyResponse {
            status: status.as_u16(),
            headers: response_headers,
            body: response_body,
        })
    }

    /// Send a request to OpenAI through the proxy
    pub async fn send_openai_request(
        &self,
        path: &str,
        headers: HashMap<String, String>,
        body: String,
    ) -> Result<ProxyResponse> {
        self.send_request("openai", path, headers, Some(body)).await
    }

    /// Send a request to Anthropic through the proxy
    pub async fn send_anthropic_request(
        &self,
        path: &str,
        headers: HashMap<String, String>,
        body: String,
    ) -> Result<ProxyResponse> {
        self.send_request("anthropic", path, headers, Some(body)).await
    }

    /// Send a request to Google through the proxy
    pub async fn send_google_request(
        &self,
        path: &str,
        headers: HashMap<String, String>,
        body: String,
    ) -> Result<ProxyResponse> {
        self.send_request("google", path, headers, Some(body)).await
    }

    /// Check proxy health
    pub async fn health_check(&self) -> Result<ProxyHealth> {
        let url = format!("{}/proxy/health", self.config.proxy_url);
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to check proxy health")?;

        if response.status().is_success() {
            let health: ProxyHealth = response.json().await
                .context("Failed to parse health check response")?;
            Ok(health)
        } else {
            anyhow::bail!("Health check failed with status: {}", response.status());
        }
    }

    /// Get SDK configuration
    pub fn config(&self) -> &ProxySdkConfig {
        &self.config
    }

    /// Update client ID
    pub fn set_client_id(&mut self, client_id: String) {
        self.config.client_id = client_id;
    }

    /// Update user ID
    pub fn set_user_id(&mut self, user_id: Option<String>) {
        self.config.user_id = user_id;
    }

    /// Update session ID
    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.config.session_id = session_id;
    }
}

/// Proxy response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Proxy health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHealth {
    pub status: String,
    pub proxy_mode: String,
    pub timestamp: String,
    pub version: String,
    pub analytics_mode: bool,
    pub emitter_mode: bool,
    pub database_connected: bool,
}

/// Builder for creating proxy clients
pub struct ProxyClientBuilder {
    config: ProxySdkConfig,
}

impl ProxyClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: ProxySdkConfig::default(),
        }
    }

    /// Set proxy URL
    pub fn proxy_url(mut self, url: String) -> Self {
        self.config.proxy_url = url;
        self
    }

    /// Set API key
    pub fn api_key(mut self, key: String) -> Self {
        self.config.api_key = Some(key);
        self
    }

    /// Set client ID
    pub fn client_id(mut self, id: String) -> Self {
        self.config.client_id = id;
        self
    }

    /// Set user ID
    pub fn user_id(mut self, id: String) -> Self {
        self.config.user_id = Some(id);
        self
    }

    /// Set session ID
    pub fn session_id(mut self, id: String) -> Self {
        self.config.session_id = Some(id);
        self
    }

    /// Enable or disable telemetry
    pub fn enable_telemetry(mut self, enabled: bool) -> Self {
        self.config.enable_telemetry = enabled;
        self
    }

    /// Set timeout
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.config.timeout_ms = timeout;
        self
    }

    /// Build the client
    pub fn build(self) -> Result<ProxyClient> {
        ProxyClient::new(self.config)
    }
}

impl Default for ProxyClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_sdk_config_default() {
        let config = ProxySdkConfig::default();
        assert_eq!(config.proxy_url, "http://localhost:8080");
        assert_eq!(config.client_id, "default_client");
        assert!(config.enable_telemetry);
        assert_eq!(config.timeout_ms, 30000);
    }

    #[test]
    fn test_proxy_client_builder() {
        let client = ProxyClientBuilder::new()
            .proxy_url("http://custom:9090".to_string())
            .client_id("test_client".to_string())
            .enable_telemetry(false)
            .timeout_ms(60000);

        assert_eq!(client.config.proxy_url, "http://custom:9090");
        assert_eq!(client.config.client_id, "test_client");
        assert!(!client.config.enable_telemetry);
        assert_eq!(client.config.timeout_ms, 60000);
    }

    #[test]
    fn test_proxy_response_structure() {
        let response = ProxyResponse {
            status: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
        };

        assert_eq!(response.status, 200);
        assert!(response.headers.is_empty());
        assert_eq!(response.body, "{}");
    }
}
