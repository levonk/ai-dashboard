//! Database seeding scripts
//!
//! This module provides functions to seed the database with initial reference data
//! for the AI Analytics Dashboard.

use anyhow::Result;
use sqlx::PgPool;

use crate::models::{
    InsertAiClient, InsertAiProvider, InsertAiModel, InsertInputType, InsertPipelineStage,
    AiProvider,
};
use crate::db::queries::{AiClientQueries, AiProviderQueries};

/// Seed the database with initial reference data
pub async fn seed_database(pool: &PgPool) -> Result<()> {
    // Seed AI clients
    seed_ai_clients(pool).await?;
    
    // Seed AI providers and models
    seed_ai_providers_and_models(pool).await?;
    
    // Seed input types
    seed_input_types(pool).await?;
    
    // Seed pipeline stages
    seed_pipeline_stages(pool).await?;
    
    Ok(())
}

/// Seed AI clients
async fn seed_ai_clients(pool: &PgPool) -> Result<()> {
    let clients = vec![
        InsertAiClient {
            name: "Claude Code".to_string(),
            slug: "claude-code".to_string(),
            client_type: "claude-code".to_string(),
            version: Some("1.0.0".to_string()),
            metadata: Some(serde_json::json!({
                "vendor": "Anthropic",
                "capabilities": ["code-generation", "code-analysis", "debugging"]
            })),
        },
        InsertAiClient {
            name: "Codex".to_string(),
            slug: "codex".to_string(),
            client_type: "codex".to_string(),
            version: Some("1.0.0".to_string()),
            metadata: Some(serde_json::json!({
                "vendor": "OpenAI",
                "capabilities": ["code-generation", "code-completion"]
            })),
        },
        InsertAiClient {
            name: "Pi".to_string(),
            slug: "pi".to_string(),
            client_type: "pi".to_string(),
            version: Some("1.0.0".to_string()),
            metadata: Some(serde_json::json!({
                "vendor": "Inflection",
                "capabilities": ["general-purpose", "code-generation"]
            })),
        },
        InsertAiClient {
            name: "Devin".to_string(),
            slug: "devin".to_string(),
            client_type: "devin".to_string(),
            version: Some("1.0.0".to_string()),
            metadata: Some(serde_json::json!({
                "vendor": "Cognition",
                "capabilities": ["autonomous-coding", "debugging", "testing"]
            })),
        },
    ];

    for client in clients {
        // Check if client already exists
        if let Some(_) = AiClientQueries::get_by_slug(pool, &client.slug).await? {
            continue;
        }
        
        AiClientQueries::create(pool, &client).await?;
    }

    Ok(())
}

/// Seed AI providers and models
async fn seed_ai_providers_and_models(pool: &PgPool) -> Result<()> {
    // Seed providers
    let providers = vec![
        InsertAiProvider {
            name: "Anthropic".to_string(),
            slug: "anthropic".to_string(),
            provider_type: "anthropic".to_string(),
            api_endpoint: Some("https://api.anthropic.com".to_string()),
            metadata: Some(serde_json::json!({
                "region": "us-east-1",
                "tier": "enterprise"
            })),
        },
        InsertAiProvider {
            name: "OpenAI".to_string(),
            slug: "openai".to_string(),
            provider_type: "openai".to_string(),
            api_endpoint: Some("https://api.openai.com".to_string()),
            metadata: Some(serde_json::json!({
                "region": "global",
                "tier": "standard"
            })),
        },
        InsertAiProvider {
            name: "Google".to_string(),
            slug: "google".to_string(),
            provider_type: "google".to_string(),
            api_endpoint: Some("https://generativelanguage.googleapis.com".to_string()),
            metadata: Some(serde_json::json!({
                "region": "global",
                "tier": "standard"
            })),
        },
    ];

    let mut provider_ids = std::collections::HashMap::new();
    
    for provider in providers {
        // Check if provider already exists
        if let Some(existing) = AiProviderQueries::get_by_slug(pool, &provider.slug).await? {
            provider_ids.insert(provider.slug.clone(), existing.id);
            continue;
        }
        
        let created: AiProvider = AiProviderQueries::create(pool, &provider).await?;
        provider_ids.insert(provider.slug.clone(), created.id);
    }

    // Seed models for each provider
    if let Some(anthropic_id) = provider_ids.get("anthropic") {
        let models = vec![
            InsertAiModel {
                provider_id: *anthropic_id,
                name: "Claude 3.5 Sonnet".to_string(),
                slug: "claude-3-5-sonnet".to_string(),
                model_type: "text".to_string(),
                context_window: Some(200000),
                pricing: Some(serde_json::json!({
                    "input": 3.0,
                    "output": 15.0,
                    "currency": "USD",
                    "unit": "per_1m_tokens"
                })),
                metadata: Some(serde_json::json!({
                    "capabilities": ["text", "code", "vision"],
                    "release_date": "2024-06-20"
                })),
            },
            InsertAiModel {
                provider_id: *anthropic_id,
                name: "Claude 3 Opus".to_string(),
                slug: "claude-3-opus".to_string(),
                model_type: "text".to_string(),
                context_window: Some(200000),
                pricing: Some(serde_json::json!({
                    "input": 15.0,
                    "output": 75.0,
                    "currency": "USD",
                    "unit": "per_1m_tokens"
                })),
                metadata: Some(serde_json::json!({
                    "capabilities": ["text", "code", "vision"],
                    "release_date": "2024-03-01"
                })),
            },
        ];

        for model in models {
            // Check if model already exists
            let existing = sqlx::query(
                "SELECT id FROM ai_models WHERE provider_id = $1 AND slug = $2",
            )
            .bind(model.provider_id)
            .bind(&model.slug)
            .fetch_optional(pool)
            .await?;

            if existing.is_none() {
                sqlx::query(
                    r#"
                    INSERT INTO ai_models (provider_id, name, slug, model_type, context_window, pricing, metadata)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                )
                .bind(model.provider_id)
                .bind(&model.name)
                .bind(&model.slug)
                .bind(&model.model_type)
                .bind(model.context_window)
                .bind(&model.pricing)
                .bind(&model.metadata)
                .execute(pool)
                .await?;
            }
        }
    }

    if let Some(openai_id) = provider_ids.get("openai") {
        let models = vec![
            InsertAiModel {
                provider_id: *openai_id,
                name: "GPT-4 Turbo".to_string(),
                slug: "gpt-4-turbo".to_string(),
                model_type: "text".to_string(),
                context_window: Some(128000),
                pricing: Some(serde_json::json!({
                    "input": 10.0,
                    "output": 30.0,
                    "currency": "USD",
                    "unit": "per_1m_tokens"
                })),
                metadata: Some(serde_json::json!({
                    "capabilities": ["text", "code", "vision"],
                    "release_date": "2024-04-01"
                })),
            },
            InsertAiModel {
                provider_id: *openai_id,
                name: "GPT-3.5 Turbo".to_string(),
                slug: "gpt-3-5-turbo".to_string(),
                model_type: "text".to_string(),
                context_window: Some(16385),
                pricing: Some(serde_json::json!({
                    "input": 0.5,
                    "output": 1.5,
                    "currency": "USD",
                    "unit": "per_1m_tokens"
                })),
                metadata: Some(serde_json::json!({
                    "capabilities": ["text", "code"],
                    "release_date": "2023-11-01"
                })),
            },
        ];

        for model in models {
            let existing = sqlx::query(
                "SELECT id FROM ai_models WHERE provider_id = $1 AND slug = $2",
            )
            .bind(model.provider_id)
            .bind(&model.slug)
            .fetch_optional(pool)
            .await?;

            if existing.is_none() {
                sqlx::query(
                    r#"
                    INSERT INTO ai_models (provider_id, name, slug, model_type, context_window, pricing, metadata)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                )
                .bind(model.provider_id)
                .bind(&model.name)
                .bind(&model.slug)
                .bind(&model.model_type)
                .bind(model.context_window)
                .bind(&model.pricing)
                .bind(&model.metadata)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}

/// Seed input types
async fn seed_input_types(pool: &PgPool) -> Result<()> {
    let input_types = vec![
        InsertInputType {
            name: "Text/Chat".to_string(),
            slug: "text-chat".to_string(),
            category: "text".to_string(),
        },
        InsertInputType {
            name: "Image".to_string(),
            slug: "image".to_string(),
            category: "image".to_string(),
        },
        InsertInputType {
            name: "Audio".to_string(),
            slug: "audio".to_string(),
            category: "audio".to_string(),
        },
        InsertInputType {
            name: "Code".to_string(),
            slug: "code".to_string(),
            category: "code".to_string(),
        },
        InsertInputType {
            name: "Multimodal".to_string(),
            slug: "multimodal".to_string(),
            category: "multimodal".to_string(),
        },
    ];

    for input_type in input_types {
        let existing = sqlx::query(
            "SELECT id FROM input_types WHERE slug = $1",
        )
        .bind(&input_type.slug)
        .fetch_optional(pool)
        .await?;

        if existing.is_none() {
            sqlx::query(
                "INSERT INTO input_types (name, slug, category) VALUES ($1, $2, $3)",
            )
            .bind(&input_type.name)
            .bind(&input_type.slug)
            .bind(&input_type.category)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

/// Seed pipeline stages
async fn seed_pipeline_stages(pool: &PgPool) -> Result<()> {
    let stages = vec![
        InsertPipelineStage {
            name: "Pre-Optimization".to_string(),
            slug: "pre-optimization".to_string(),
            stage_type: "pre-optimization".to_string(),
            description: Some("Original requests before optimization".to_string()),
            metadata: Some(serde_json::json!({
                "position": 1,
                "optional": false
            })),
        },
        InsertPipelineStage {
            name: "Post-Optimization".to_string(),
            slug: "post-optimization".to_string(),
            stage_type: "post-optimization".to_string(),
            description: Some("Requests after compression/transformation".to_string()),
            metadata: Some(serde_json::json!({
                "position": 2,
                "optional": false
            })),
        },
        InsertPipelineStage {
            name: "Headroom".to_string(),
            slug: "headroom".to_string(),
            stage_type: "headroom".to_string(),
            description: Some("Headroom optimization stage".to_string()),
            metadata: Some(serde_json::json!({
                "position": 3,
                "optional": true
            })),
        },
        InsertPipelineStage {
            name: "OmniRoute".to_string(),
            slug: "omniroute".to_string(),
            stage_type: "omniroute".to_string(),
            description: Some("OmniRoute routing stage".to_string()),
            metadata: Some(serde_json::json!({
                "position": 4,
                "optional": true
            })),
        },
        InsertPipelineStage {
            name: "Iron-Proxy".to_string(),
            slug: "iron-proxy".to_string(),
            stage_type: "iron-proxy".to_string(),
            description: Some("Iron-Proxy processing stage".to_string()),
            metadata: Some(serde_json::json!({
                "position": 5,
                "optional": true
            })),
        },
    ];

    for stage in stages {
        let existing = sqlx::query(
            "SELECT id FROM pipeline_stages WHERE slug = $1",
        )
        .bind(&stage.slug)
        .fetch_optional(pool)
        .await?;

        if existing.is_none() {
            sqlx::query(
                r#"
                INSERT INTO pipeline_stages (name, slug, stage_type, description, metadata)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(&stage.name)
            .bind(&stage.slug)
            .bind(&stage.stage_type)
            .bind(&stage.description)
            .bind(&stage.metadata)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_structure() {
        // Test structure - actual tests would require a database
        assert!(true);
    }
}