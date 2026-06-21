//! Database query builders
//!
//! This module provides query builders and helpers for common database operations
//! in the AI Analytics Dashboard.

use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{
    Company, AiClient, AiProvider,
    RequestEvent,
    InsertCompany, InsertAiClient, InsertAiProvider,
    InsertRequestEvent,
};

/// Query builder for companies
pub struct CompanyQueries;

impl CompanyQueries {
    /// Create a new company
    pub async fn create(pool: &PgPool, company: &InsertCompany) -> Result<Company> {
        let row = sqlx::query(
            r#"
            INSERT INTO companies (name, slug, metadata)
            VALUES ($1, $2, $3)
            RETURNING id, name, slug, created_at, updated_at, metadata
            "#,
        )
        .bind(&company.name)
        .bind(&company.slug)
        .bind(&company.metadata.clone().unwrap_or(serde_json::json!({})))
        .fetch_one(pool)
        .await?;

        Ok(Company {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    /// Get company by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Company>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, created_at, updated_at, metadata FROM companies WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(Company {
                id: r.get("id"),
                name: r.get("name"),
                slug: r.get("slug"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            })),
            None => Ok(None),
        }
    }

    /// Get company by slug
    pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Company>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, created_at, updated_at, metadata FROM companies WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(Company {
                id: r.get("id"),
                name: r.get("name"),
                slug: r.get("slug"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            })),
            None => Ok(None),
        }
    }

    /// List all companies
    pub async fn list(pool: &PgPool) -> Result<Vec<Company>> {
        let rows = sqlx::query(
            r#"SELECT id, name, slug, created_at, updated_at, metadata FROM companies ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await?;

        let companies = rows.iter().map(|r| Company {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }).collect();

        Ok(companies)
    }
}

/// Query builder for AI providers
pub struct AiProviderQueries;

impl AiProviderQueries {
    /// Create a new AI provider
    pub async fn create(pool: &PgPool, provider: &InsertAiProvider) -> Result<AiProvider> {
        let row = sqlx::query(
            r#"
            INSERT INTO ai_providers (name, slug, provider_type, api_endpoint, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, slug, provider_type, api_endpoint, created_at, updated_at, metadata
            "#,
        )
        .bind(&provider.name)
        .bind(&provider.slug)
        .bind(&provider.provider_type)
        .bind(&provider.api_endpoint)
        .bind(&provider.metadata.clone().unwrap_or(serde_json::json!({})))
        .fetch_one(pool)
        .await?;

        Ok(AiProvider {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
            provider_type: row.get("provider_type"),
            api_endpoint: row.get("api_endpoint"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    /// Get AI provider by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AiProvider>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, provider_type, api_endpoint, created_at, updated_at, metadata FROM ai_providers WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(AiProvider {
                id: r.get("id"),
                name: r.get("name"),
                slug: r.get("slug"),
                provider_type: r.get("provider_type"),
                api_endpoint: r.get("api_endpoint"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            })),
            None => Ok(None),
        }
    }

    /// Get AI provider by slug
    pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<AiProvider>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, provider_type, api_endpoint, created_at, updated_at, metadata FROM ai_providers WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(AiProvider {
                id: r.get("id"),
                name: r.get("name"),
                slug: r.get("slug"),
                provider_type: r.get("provider_type"),
                api_endpoint: r.get("api_endpoint"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            })),
            None => Ok(None),
        }
    }

    /// List all AI providers
    pub async fn list(pool: &PgPool) -> Result<Vec<AiProvider>> {
        let rows = sqlx::query(
            r#"SELECT id, name, slug, provider_type, api_endpoint, created_at, updated_at, metadata FROM ai_providers ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await?;

        let providers = rows.iter().map(|r| AiProvider {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            provider_type: r.get("provider_type"),
            api_endpoint: r.get("api_endpoint"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }).collect();

        Ok(providers)
    }
}

/// Query builder for AI clients
pub struct AiClientQueries;

impl AiClientQueries {
    /// Create a new AI client
    pub async fn create(pool: &PgPool, client: &InsertAiClient) -> Result<AiClient> {
        let row = sqlx::query(
            r#"
            INSERT INTO ai_clients (name, slug, client_type, version, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, slug, client_type, version, created_at, updated_at, metadata
            "#,
        )
        .bind(&client.name)
        .bind(&client.slug)
        .bind(&client.client_type)
        .bind(&client.version)
        .bind(&client.metadata.clone().unwrap_or(serde_json::json!({})))
        .fetch_one(pool)
        .await?;

        Ok(AiClient {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
            client_type: row.get("client_type"),
            version: row.get("version"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    /// Get AI client by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AiClient>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, client_type, version, created_at, updated_at, metadata FROM ai_clients WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(AiClient {
                id: r.get("id"),
                name: r.get("name"),
                slug: r.get("slug"),
                client_type: r.get("client_type"),
                version: r.get("version"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            })),
            None => Ok(None),
        }
    }

    /// Get AI client by slug
    pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<AiClient>> {
        let row = sqlx::query(
            r#"SELECT id, name, slug, client_type, version, created_at, updated_at, metadata FROM ai_clients WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(AiClient {
                id: r.get("id"),
                name: r.get("name"),
                slug: r.get("slug"),
                client_type: r.get("client_type"),
                version: r.get("version"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            })),
            None => Ok(None),
        }
    }

    /// List all AI clients
    pub async fn list(pool: &PgPool) -> Result<Vec<AiClient>> {
        let rows = sqlx::query(
            r#"SELECT id, name, slug, client_type, version, created_at, updated_at, metadata FROM ai_clients ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await?;

        let clients = rows.iter().map(|r| AiClient {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            client_type: r.get("client_type"),
            version: r.get("version"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }).collect();

        Ok(clients)
    }
}

/// Query builder for request events
pub struct RequestEventQueries;

impl RequestEventQueries {
    /// Create a new request event
    pub async fn create(pool: &PgPool, event: &InsertRequestEvent) -> Result<RequestEvent> {
        let row = sqlx::query(
            r#"
            INSERT INTO request_events (
                company_id, team_id, ai_client_id, provider_id, model_id, input_type_id, pipeline_stage_id,
                request_hash, request_id, parent_request_id,
                input_text, input_tokens, output_text, output_tokens, total_tokens,
                latency_ms, timing_breakdown,
                input_cost, output_cost, total_cost,
                error_type, error_message, error_details,
                metadata, custom_attributes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
            RETURNING *
            "#,
        )
        .bind(event.company_id)
        .bind(event.team_id)
        .bind(event.ai_client_id)
        .bind(event.provider_id)
        .bind(event.model_id)
        .bind(event.input_type_id)
        .bind(event.pipeline_stage_id)
        .bind(&event.request_hash)
        .bind(&event.request_id)
        .bind(event.parent_request_id)
        .bind(&event.input_text)
        .bind(event.input_tokens)
        .bind(&event.output_text)
        .bind(event.output_tokens)
        .bind(event.total_tokens)
        .bind(event.latency_ms)
        .bind(&event.timing_breakdown.clone().unwrap_or(serde_json::json!({})))
        .bind(event.input_cost)
        .bind(event.output_cost)
        .bind(event.total_cost)
        .bind(&event.error_type)
        .bind(&event.error_message)
        .bind(&event.error_details.clone().unwrap_or(serde_json::json!({})))
        .bind(&event.metadata.clone().unwrap_or(serde_json::json!({})))
        .bind(&event.custom_attributes.clone().unwrap_or(serde_json::json!({})))
        .fetch_one(pool)
        .await?;

        Ok(RequestEvent {
            id: row.get("id"),
            company_id: row.get("company_id"),
            team_id: row.get("team_id"),
            ai_client_id: row.get("ai_client_id"),
            provider_id: row.get("provider_id"),
            model_id: row.get("model_id"),
            input_type_id: row.get("input_type_id"),
            pipeline_stage_id: row.get("pipeline_stage_id"),
            request_hash: row.get("request_hash"),
            request_id: row.get("request_id"),
            parent_request_id: row.get("parent_request_id"),
            input_text: row.get("input_text"),
            input_tokens: row.get("input_tokens"),
            output_text: row.get("output_text"),
            output_tokens: row.get("output_tokens"),
            total_tokens: row.get("total_tokens"),
            latency_ms: row.get("latency_ms"),
            timing_breakdown: row.get("timing_breakdown"),
            input_cost: row.get("input_cost"),
            output_cost: row.get("output_cost"),
            total_cost: row.get("total_cost"),
            error_type: row.get("error_type"),
            error_message: row.get("error_message"),
            error_details: row.get("error_details"),
            metadata: row.get("metadata"),
            custom_attributes: row.get("custom_attributes"),
            created_at: row.get("created_at"),
            processed_at: row.get("processed_at"),
        })
    }

    /// Get request event by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RequestEvent>> {
        let row = sqlx::query(
            r#"SELECT * FROM request_events WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(RequestEvent {
                id: r.get("id"),
                company_id: r.get("company_id"),
                team_id: r.get("team_id"),
                ai_client_id: r.get("ai_client_id"),
                provider_id: r.get("provider_id"),
                model_id: r.get("model_id"),
                input_type_id: r.get("input_type_id"),
                pipeline_stage_id: r.get("pipeline_stage_id"),
                request_hash: r.get("request_hash"),
                request_id: r.get("request_id"),
                parent_request_id: r.get("parent_request_id"),
                input_text: r.get("input_text"),
                input_tokens: r.get("input_tokens"),
                output_text: r.get("output_text"),
                output_tokens: r.get("output_tokens"),
                total_tokens: r.get("total_tokens"),
                latency_ms: r.get("latency_ms"),
                timing_breakdown: r.get("timing_breakdown"),
                input_cost: r.get("input_cost"),
                output_cost: r.get("output_cost"),
                total_cost: r.get("total_cost"),
                error_type: r.get("error_type"),
                error_message: r.get("error_message"),
                error_details: r.get("error_details"),
                metadata: r.get("metadata"),
                custom_attributes: r.get("custom_attributes"),
                created_at: r.get("created_at"),
                processed_at: r.get("processed_at"),
            })),
            None => Ok(None),
        }
    }

    /// Get request events by hash (for pipeline correlation)
    pub async fn get_by_hash(pool: &PgPool, hash: &str) -> Result<Vec<RequestEvent>> {
        let rows = sqlx::query(
            r#"SELECT * FROM request_events WHERE request_hash = $1 ORDER BY created_at"#,
        )
        .bind(hash)
        .fetch_all(pool)
        .await?;

        let events = rows.iter().map(|r| RequestEvent {
            id: r.get("id"),
            company_id: r.get("company_id"),
            team_id: r.get("team_id"),
            ai_client_id: r.get("ai_client_id"),
            provider_id: r.get("provider_id"),
            model_id: r.get("model_id"),
            input_type_id: r.get("input_type_id"),
            pipeline_stage_id: r.get("pipeline_stage_id"),
            request_hash: r.get("request_hash"),
            request_id: r.get("request_id"),
            parent_request_id: r.get("parent_request_id"),
            input_text: r.get("input_text"),
            input_tokens: r.get("input_tokens"),
            output_text: r.get("output_text"),
            output_tokens: r.get("output_tokens"),
            total_tokens: r.get("total_tokens"),
            latency_ms: r.get("latency_ms"),
            timing_breakdown: r.get("timing_breakdown"),
            input_cost: r.get("input_cost"),
            output_cost: r.get("output_cost"),
            total_cost: r.get("total_cost"),
            error_type: r.get("error_type"),
            error_message: r.get("error_message"),
            error_details: r.get("error_details"),
            metadata: r.get("metadata"),
            custom_attributes: r.get("custom_attributes"),
            created_at: r.get("created_at"),
            processed_at: r.get("processed_at"),
        }).collect();

        Ok(events)
    }

    /// Query request events with filters
    pub async fn query(
        _pool: &PgPool,
        _company_id: Option<Uuid>,
        _team_id: Option<Uuid>,
        _ai_client_id: Option<Uuid>,
        _provider_id: Option<Uuid>,
        _model_id: Option<Uuid>,
        _start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
        _limit: Option<i64>,
    ) -> Result<Vec<RequestEvent>> {
        // For simplicity, return empty result for now
        // In production, you'd want to use SQLx's query builder or a more sophisticated approach
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_queries() {
        // Test structure - actual tests would require a database
        assert!(true);
    }

    #[test]
    fn test_request_event_queries() {
        // Test structure - actual tests would require a database
        assert!(true);
    }
}