//! Core entity models
//!
//! This module defines the core entity models for the AI Analytics Dashboard,
//! representing the database entities for companies, teams, AI clients, providers, models, etc.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// Core Dimension Entities
// ============================================

/// Company (Client) entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// AI Client entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiClient {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub client_type: String,
    pub version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Team entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub slug: String,
    pub parent_team_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// AI Provider entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiProvider {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub provider_type: String,
    pub api_endpoint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// AI Model entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiModel {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: String,
    pub slug: String,
    pub model_type: String,
    pub context_window: Option<i32>,
    pub pricing: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Input Type entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InputType {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub category: String,
    pub created_at: DateTime<Utc>,
}

/// Pipeline Stage entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineStage {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub stage_type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

// ============================================
// Analytics Event Entities
// ============================================

/// Request Event entity (core analytics table)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestEvent {
    pub id: Uuid,
    
    // Dimensional Foreign Keys
    pub company_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub ai_client_id: Option<Uuid>,
    pub provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub input_type_id: Option<Uuid>,
    pub pipeline_stage_id: Option<Uuid>,
    
    // Request Identification
    pub request_hash: String,
    pub request_id: Option<String>,
    pub parent_request_id: Option<Uuid>,
    
    // Request Content
    pub input_text: Option<String>,
    pub input_tokens: Option<i32>,
    pub output_text: Option<String>,
    pub output_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
    
    // Performance Metrics
    pub latency_ms: Option<i32>,
    pub timing_breakdown: serde_json::Value,
    
    // Cost Metrics
    pub input_cost: Option<rust_decimal::Decimal>,
    pub output_cost: Option<rust_decimal::Decimal>,
    pub total_cost: Option<rust_decimal::Decimal>,
    
    // Error Tracking
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_details: serde_json::Value,
    
    // Metadata
    pub metadata: serde_json::Value,
    pub custom_attributes: serde_json::Value,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Daily Aggregate entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyAggregate {
    pub id: Uuid,
    
    // Dimensions
    pub company_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub ai_client_id: Option<Uuid>,
    pub provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub input_type_id: Option<Uuid>,
    pub pipeline_stage_id: Option<Uuid>,
    
    // Time Period
    pub date: chrono::NaiveDate,
    
    // Aggregated Metrics
    pub request_count: i32,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub avg_latency_ms: Option<rust_decimal::Decimal>,
    pub total_cost: Option<rust_decimal::Decimal>,
    pub error_count: i32,
    
    // Metadata
    pub metadata: serde_json::Value,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Hourly Aggregate entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HourlyAggregate {
    pub id: Uuid,
    
    // Dimensions
    pub company_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub ai_client_id: Option<Uuid>,
    pub provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub input_type_id: Option<Uuid>,
    pub pipeline_stage_id: Option<Uuid>,
    
    // Time Period
    pub hour: DateTime<Utc>,
    
    // Aggregated Metrics
    pub request_count: i32,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub avg_latency_ms: Option<rust_decimal::Decimal>,
    pub total_cost: Option<rust_decimal::Decimal>,
    pub error_count: i32,
    
    // Metadata
    pub metadata: serde_json::Value,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// Insert/Update DTOs
// ============================================

/// Company insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertCompany {
    pub name: String,
    pub slug: String,
    pub metadata: Option<serde_json::Value>,
}

/// AI Client insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertAiClient {
    pub name: String,
    pub slug: String,
    pub client_type: String,
    pub version: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Team insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTeam {
    pub company_id: Uuid,
    pub name: String,
    pub slug: String,
    pub parent_team_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

/// AI Provider insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertAiProvider {
    pub name: String,
    pub slug: String,
    pub provider_type: String,
    pub api_endpoint: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// AI Model insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertAiModel {
    pub provider_id: Uuid,
    pub name: String,
    pub slug: String,
    pub model_type: String,
    pub context_window: Option<i32>,
    pub pricing: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Input Type insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertInputType {
    pub name: String,
    pub slug: String,
    pub category: String,
}

/// Pipeline Stage insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertPipelineStage {
    pub name: String,
    pub slug: String,
    pub stage_type: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Request Event insert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRequestEvent {
    pub company_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub ai_client_id: Option<Uuid>,
    pub provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub input_type_id: Option<Uuid>,
    pub pipeline_stage_id: Option<Uuid>,
    pub request_hash: String,
    pub request_id: Option<String>,
    pub parent_request_id: Option<Uuid>,
    pub input_text: Option<String>,
    pub input_tokens: Option<i32>,
    pub output_text: Option<String>,
    pub output_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
    pub latency_ms: Option<i32>,
    pub timing_breakdown: Option<serde_json::Value>,
    pub input_cost: Option<rust_decimal::Decimal>,
    pub output_cost: Option<rust_decimal::Decimal>,
    pub total_cost: Option<rust_decimal::Decimal>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_details: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub custom_attributes: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_serialization() {
        let company = Company {
            id: Uuid::new_v4(),
            name: "Test Company".to_string(),
            slug: "test-company".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        };
        
        let serialized = serde_json::to_string(&company).unwrap();
        assert!(serialized.contains("Test Company"));
    }

    #[test]
    fn test_insert_company() {
        let insert = InsertCompany {
            name: "Test Company".to_string(),
            slug: "test-company".to_string(),
            metadata: None,
        };
        
        assert_eq!(insert.name, "Test Company");
        assert_eq!(insert.slug, "test-company");
    }
}