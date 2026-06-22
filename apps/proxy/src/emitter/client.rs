//! Emitter client for external collector communication
//! 
//! This module handles HTTP communication with external collector services,
//! including connection pooling, authentication, retry logic, and circuit breaker patterns.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use super::protocol::{EventMessage, PROTOCOL_VERSION};

/// Collector response from external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorResponse {
    /// Success flag
    pub success: bool,
    /// Response message
    pub message: String,
    /// Event ID if applicable
    pub event_id: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Emitter client configuration
#[derive(Debug, Clone)]
pub struct EmitterClientConfig {
    /// Collector URL (HTTP endpoint)
    pub collector_url: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial retry delay in seconds
    pub retry_delay_secs: u64,
    /// Maximum retry delay in seconds
    pub max_retry_delay_secs: u64,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker timeout in seconds
    pub circuit_breaker_timeout_secs: u64,
}

impl Default for EmitterClientConfig {
    fn default() -> Self {
        Self {
            collector_url: String::new(),
            api_key: None,
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_secs: 1,
            max_retry_delay_secs: 60,
            max_concurrent_requests: 10,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures
struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    threshold: u32,
    last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            threshold,
            last_failure_time: None,
            timeout,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        if self.state == CircuitBreakerState::HalfOpen {
            self.state = CircuitBreakerState::Closed;
            info!("Circuit breaker closed after successful request");
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(chrono::Utc::now());

        if self.failure_count >= self.threshold {
            self.state = CircuitBreakerState::Open;
            warn!(
                "Circuit breaker opened after {} failures",
                self.failure_count
            );
        }
    }

    fn allow_request(&self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    let elapsed = chrono::Utc::now() - last_failure;
                    if elapsed > chrono::Duration::from_std(self.timeout).unwrap_or_default() {
                        return true; // Allow request to test if circuit should close
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn attempt_reset(&mut self) {
        if self.state == CircuitBreakerState::Open {
            if let Some(last_failure) = self.last_failure_time {
                let elapsed = chrono::Utc::now() - last_failure;
                if elapsed > chrono::Duration::from_std(self.timeout).unwrap_or_default() {
                    self.state = CircuitBreakerState::HalfOpen;
                    info!("Circuit breaker transitioning to half-open state");
                }
            }
        }
    }
}

/// Emitter client for HTTP communication with external collectors
pub struct EmitterClient {
    config: EmitterClientConfig,
    http_client: Client,
    semaphore: Arc<Semaphore>,
    circuit_breaker: Arc<tokio::sync::Mutex<CircuitBreaker>>,
}

impl EmitterClient {
    /// Create a new emitter client with default configuration
    pub fn new(config: EmitterClientConfig) -> Result<Self> {
        if config.collector_url.is_empty() {
            anyhow::bail!("Collector URL cannot be empty");
        }

        let timeout = Duration::from_secs(config.timeout_secs);
        let http_client = Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .context("Failed to create HTTP client")?;

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        let circuit_breaker = if config.enable_circuit_breaker {
            Arc::new(tokio::sync::Mutex::new(CircuitBreaker::new(
                config.circuit_breaker_threshold,
                Duration::from_secs(config.circuit_breaker_timeout_secs),
            )))
        } else {
            // Disabled circuit breaker - always allows requests
            Arc::new(tokio::sync::Mutex::new(CircuitBreaker::new(u32::MAX, Duration::from_secs(0))))
        };

        info!(
            "Created emitter client for collector: {}",
            config.collector_url
        );

        Ok(Self {
            config,
            http_client,
            semaphore,
            circuit_breaker,
        })
    }

    /// Emit a single event message to the collector
    pub async fn emit_event(&self, event_message: &EventMessage) -> Result<CollectorResponse> {
        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.lock().await;
            cb.attempt_reset();
            if !cb.allow_request() {
                anyhow::bail!("Circuit breaker is open, rejecting request");
            }
        }

        // Acquire semaphore for concurrency control
        let permit = self.semaphore
            .acquire()
            .await
            .context("Failed to acquire semaphore")?;

        let result = self.emit_event_with_retry(event_message).await;

        // Update circuit breaker based on result
        {
            let mut cb = self.circuit_breaker.lock().await;
            if result.is_ok() {
                cb.record_success();
            } else {
                cb.record_failure();
            }
        }

        drop(permit);
        result
    }

    /// Emit event with retry logic
    async fn emit_event_with_retry(&self, event_message: &EventMessage) -> Result<CollectorResponse> {
        let mut last_error_msg = String::new();
        let mut retry_delay = Duration::from_secs(self.config.retry_delay_secs);

        for attempt in 0..=self.config.max_retries {
            debug!(
                "Emitting event {} (attempt {}/{})",
                event_message.event.event_id,
                attempt + 1,
                self.config.max_retries + 1
            );

            match self.send_event(event_message).await {
                Ok(response) => {
                    if response.success {
                        debug!(
                            "Successfully emitted event: {}",
                            event_message.event.event_id
                        );
                        return Ok(response);
                    } else {
                        warn!(
                            "Collector returned error for event {}: {}",
                            event_message.event.event_id,
                            response.message
                        );
                        // Don't retry on business logic errors
                        return Ok(response);
                    }
                }
                Err(e) => {
                    last_error_msg = e.to_string();
                    warn!(
                        "Failed to emit event {} (attempt {}/{}): {}",
                        event_message.event.event_id,
                        attempt + 1,
                        self.config.max_retries + 1,
                        e
                    );

                    // Don't retry on last attempt
                    if attempt < self.config.max_retries {
                        // Exponential backoff with jitter
                        let jitter = rand::random::<f64>() * 0.1; // 10% jitter
                        let delay_ms = (retry_delay.as_millis() as f64 * (1.0 + jitter)) as u64;
                        let delay = Duration::from_millis(delay_ms);
                        
                        debug!("Retrying after {:?}", delay);
                        tokio::time::sleep(delay).await;

                        // Exponential backoff
                        retry_delay = std::cmp::min(
                            retry_delay * 2,
                            Duration::from_secs(self.config.max_retry_delay_secs),
                        );
                    }
                }
            }
        }

        if last_error_msg.is_empty() {
            Err(anyhow::anyhow!("Max retries exceeded"))
        } else {
            Err(anyhow::anyhow!("Max retries exceeded: {}", last_error_msg))
        }
    }

    /// Send event to collector
    async fn send_event(&self, event_message: &EventMessage) -> Result<CollectorResponse> {
        let url = format!("{}/api/v1/events", self.config.collector_url);

        let mut request = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Protocol-Version", PROTOCOL_VERSION.to_string())
            .header("X-Source-ID", &event_message.header.source_id)
            .json(event_message);

        // Add authentication if configured
        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .context("Failed to send request to collector")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            debug!("Collector returned success status: {}", status);
            
            // Parse response
            match serde_json::from_str::<CollectorResponse>(&response_text) {
                Ok(collector_response) => Ok(collector_response),
                Err(e) => {
                    // If response is not in expected format, create a default success response
                    warn!("Failed to parse collector response: {}, using default success", e);
                    Ok(CollectorResponse {
                        success: true,
                        message: "Event received".to_string(),
                        event_id: Some(event_message.event.event_id.clone()),
                        metadata: serde_json::json!({}),
                    })
                }
            }
        } else {
            error!(
                "Collector returned error status: {}, body: {}",
                status, response_text
            );
            anyhow::bail!(
                "Collector returned error status: {}, body: {}",
                status,
                response_text
            );
        }
    }

    /// Health check for collector connectivity
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.config.collector_url);

        let mut request = self.http_client.get(&url);
        
        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .context("Failed to send health check request")?;

        Ok(response.status().is_success())
    }

    /// Get circuit breaker state as string
    pub async fn circuit_breaker_state(&self) -> String {
        let cb = self.circuit_breaker.lock().await;
        match cb.state {
            CircuitBreakerState::Closed => "closed".to_string(),
            CircuitBreakerState::Open => "open".to_string(),
            CircuitBreakerState::HalfOpen => "half_open".to_string(),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &EmitterClientConfig {
        &self.config
    }

    /// Reset circuit breaker (for testing/recovery)
    pub async fn reset_circuit_breaker(&self) {
        let mut cb = self.circuit_breaker.lock().await;
        cb.state = CircuitBreakerState::Closed;
        cb.failure_count = 0;
        cb.last_failure_time = None;
        info!("Circuit breaker manually reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_emitter_client_config_default() {
        let config = EmitterClientConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.max_concurrent_requests, 10);
        assert!(config.enable_circuit_breaker);
    }

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));
        
        assert_eq!(cb.state, CircuitBreakerState::Closed);
        assert!(cb.allow_request());

        // Record failures up to threshold
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitBreakerState::Closed);
        assert!(cb.allow_request());

        // Trigger circuit breaker
        cb.record_failure();
        assert_eq!(cb.state, CircuitBreakerState::Open);
        assert!(!cb.allow_request());

        // Reset after timeout
        cb.last_failure_time = Some(chrono::Utc::now() - chrono::Duration::seconds(61));
        cb.attempt_reset();
        assert_eq!(cb.state, CircuitBreakerState::HalfOpen);
        assert!(cb.allow_request());

        // Close after success
        cb.record_success();
        assert_eq!(cb.state, CircuitBreakerState::Closed);
    }

    #[test]
    fn test_emitter_client_requires_url() {
        let config = EmitterClientConfig {
            collector_url: String::new(),
            ..Default::default()
        };
        assert!(EmitterClient::new(config).is_err());
    }
}
