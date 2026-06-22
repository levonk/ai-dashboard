//! Emitter mode API endpoints
//! 
//! This module provides HTTP API endpoints for emitter mode health, status,
//! and control operations.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use crate::emitter::{
    MetricsCollector,
    CircuitBreakerState,
    QueueStats,
    FallbackStats,
};

/// Emitter API state
#[derive(Clone)]
pub struct EmitterApiState {
    pub metrics: Arc<MetricsCollector>,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Overall health status
    pub healthy: bool,
    /// Emitter mode status
    pub emitter_enabled: bool,
    /// Circuit breaker state
    pub circuit_breaker_state: String,
    /// Queue connectivity
    pub queue_connected: bool,
    /// Fallback buffer status
    pub fallback_active: bool,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Status response
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Emitter mode enabled
    pub enabled: bool,
    /// Current metrics
    pub metrics: serde_json::Value,
    /// Queue statistics
    pub queue_stats: Option<QueueStats>,
    /// Fallback statistics
    pub fallback_stats: Option<FallbackStats>,
    /// Circuit breaker state
    pub circuit_breaker_state: String,
    /// Success rate
    pub success_rate: f64,
    /// Failure rate
    pub failure_rate: f64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Metrics in JSON format
    pub metrics: serde_json::Value,
    /// Prometheus format metrics
    pub prometheus_metrics: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Control request
#[derive(Debug, Deserialize)]
pub struct ControlRequest {
    /// Action to perform
    pub action: String,
    /// Optional parameters
    pub params: Option<serde_json::Value>,
}

/// Control response
#[derive(Debug, Serialize, Deserialize)]
pub struct ControlResponse {
    /// Success flag
    pub success: bool,
    /// Message
    pub message: String,
    /// Result data
    pub data: Option<serde_json::Value>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Create emitter API router
pub fn create_emitter_router(state: EmitterApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/metrics", get(get_metrics))
        .route("/prometheus", get(get_prometheus_metrics))
        .with_state(state)
}

/// Health check endpoint
async fn health_check(State(state): State<EmitterApiState>) -> Json<HealthCheckResponse> {
    debug!("Health check requested");
    
    let is_healthy = state.metrics.is_healthy().await;
    let metrics = state.metrics.get_metrics().await;
    
    let queue_connected = metrics.queue_stats
        .as_ref()
        .map(|s| s.connected)
        .unwrap_or(false);
    
    let fallback_active = metrics.fallback_stats
        .as_ref()
        .map(|s| s.active)
        .unwrap_or(false);
    
    let response = HealthCheckResponse {
        healthy: is_healthy,
        emitter_enabled: true, // TODO: Get from actual config
        circuit_breaker_state: metrics.circuit_breaker_state.clone(),
        queue_connected,
        fallback_active,
        timestamp: chrono::Utc::now(),
    };
    
    debug!("Health check response: {:?}", response);
    Json(response)
}

/// Status endpoint
async fn get_status(State(state): State<EmitterApiState>) -> Json<StatusResponse> {
    debug!("Status requested");
    
    let metrics = state.metrics.get_metrics().await;
    let success_rate = state.metrics.success_rate().await;
    let failure_rate = state.metrics.failure_rate().await;
    
    let metrics_json = serde_json::to_value(&metrics).unwrap_or_default();
    
    let response = StatusResponse {
        enabled: true, // TODO: Get from actual config
        metrics: metrics_json,
        queue_stats: metrics.queue_stats.clone(),
        fallback_stats: metrics.fallback_stats.clone(),
        circuit_breaker_state: metrics.circuit_breaker_state.clone(),
        success_rate,
        failure_rate,
        uptime_seconds: metrics.uptime_seconds,
        timestamp: chrono::Utc::now(),
    };
    
    debug!("Status response: {:?}", response);
    Json(response)
}

/// Metrics endpoint
async fn get_metrics(State(state): State<EmitterApiState>) -> Json<MetricsResponse> {
    debug!("Metrics requested");
    
    let metrics = state.metrics.get_metrics().await;
    let metrics_json = serde_json::to_value(&metrics).unwrap_or_default();
    let prometheus_metrics = state.metrics.export_prometheus().await;
    
    let response = MetricsResponse {
        metrics: metrics_json,
        prometheus_metrics,
        timestamp: chrono::Utc::now(),
    };
    
    Json(response)
}

/// Prometheus metrics endpoint
async fn get_prometheus_metrics(State(state): State<EmitterApiState>) -> String {
    debug!("Prometheus metrics requested");
    
    state.metrics.export_prometheus().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_response_serialization() {
        let response = HealthCheckResponse {
            healthy: true,
            emitter_enabled: true,
            circuit_breaker_state: "closed".to_string(),
            queue_connected: true,
            fallback_active: false,
            timestamp: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("\"healthy\":true"));
    }

    #[test]
    fn test_status_response_serialization() {
        let response = StatusResponse {
            enabled: true,
            metrics: serde_json::json!({"test": "data"}),
            queue_stats: None,
            fallback_stats: None,
            circuit_breaker_state: "closed".to_string(),
            success_rate: 0.95,
            failure_rate: 0.05,
            uptime_seconds: 3600,
            timestamp: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("\"success_rate\":0.95"));
    }

    #[test]
    fn test_control_request_deserialization() {
        let json = r#"{"action":"reset","params":{"force":true}}"#;
        let request: ControlRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.action, "reset");
        assert!(request.params.is_some());
    }
}
