//! Data validation layer
//!
//! This module provides validation schemas and functions for all entities
//! in the AI Analytics Dashboard.

use anyhow::{Result, Context};
use regex::Regex;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{
    InsertCompany, InsertAiClient, InsertAiProvider,
    InsertRequestEvent,
};

/// Validation trait for entities
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

/// Validate slug format (kebab-case, lowercase, alphanumeric with hyphens)
fn validate_slug(slug: &str) -> Result<()> {
    let slug_regex = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")?;
    if !slug_regex.is_match(slug) {
        anyhow::bail!("Invalid slug format: must be kebab-case (lowercase alphanumeric with hyphens)");
    }
    if slug.len() < 2 || slug.len() > 100 {
        anyhow::bail!("Slug must be between 2 and 100 characters");
    }
    Ok(())
}

/// Validate string length
fn validate_string_length(value: &str, min: usize, max: usize, field_name: &str) -> Result<()> {
    if value.len() < min {
        anyhow::bail!("{} must be at least {} characters", field_name, min);
    }
    if value.len() > max {
        anyhow::bail!("{} must be at most {} characters", field_name, max);
    }
    Ok(())
}

/// Validate UUID format
fn validate_uuid(uuid: &str) -> Result<()> {
    Uuid::parse_str(uuid).context("Invalid UUID format")?;
    Ok(())
}

/// Validate date range
fn validate_date_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<()> {
    if start > end {
        anyhow::bail!("Start date must be before end date");
    }
    Ok(())
}

impl Validate for InsertCompany {
    fn validate(&self) -> Result<()> {
        validate_string_length(&self.name, 1, 255, "Company name")?;
        validate_slug(&self.slug)?;
        Ok(())
    }
}

impl Validate for InsertAiClient {
    fn validate(&self) -> Result<()> {
        validate_string_length(&self.name, 1, 100, "AI client name")?;
        validate_slug(&self.slug)?;
        validate_string_length(&self.client_type, 1, 50, "Client type")?;
        
        if let Some(version) = &self.version {
            validate_string_length(version, 1, 50, "Version")?;
        }
        
        Ok(())
    }
}

impl Validate for InsertAiProvider {
    fn validate(&self) -> Result<()> {
        validate_string_length(&self.name, 1, 100, "Provider name")?;
        validate_slug(&self.slug)?;
        validate_string_length(&self.provider_type, 1, 50, "Provider type")?;
        
        if let Some(api_endpoint) = &self.api_endpoint {
            // Basic URL validation
            if !api_endpoint.starts_with("http://") && !api_endpoint.starts_with("https://") {
                anyhow::bail!("API endpoint must be a valid URL");
            }
        }
        
        Ok(())
    }
}

impl Validate for InsertRequestEvent {
    fn validate(&self) -> Result<()> {
        // Validate request hash (SHA-256 should be 64 hex characters)
        if self.request_hash.len() != 64 {
            anyhow::bail!("Request hash must be 64 characters (SHA-256)");
        }
        
        // Validate request hash is hex
        if !self.request_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("Request hash must be hexadecimal");
        }
        
        // Validate foreign keys if present
        if let Some(company_id) = self.company_id {
            validate_uuid(&company_id.to_string())?;
        }
        
        if let Some(team_id) = self.team_id {
            validate_uuid(&team_id.to_string())?;
        }
        
        if let Some(ai_client_id) = self.ai_client_id {
            validate_uuid(&ai_client_id.to_string())?;
        }
        
        if let Some(provider_id) = self.provider_id {
            validate_uuid(&provider_id.to_string())?;
        }
        
        if let Some(model_id) = self.model_id {
            validate_uuid(&model_id.to_string())?;
        }
        
        if let Some(input_type_id) = self.input_type_id {
            validate_uuid(&input_type_id.to_string())?;
        }
        
        if let Some(pipeline_stage_id) = self.pipeline_stage_id {
            validate_uuid(&pipeline_stage_id.to_string())?;
        }
        
        if let Some(parent_request_id) = self.parent_request_id {
            validate_uuid(&parent_request_id.to_string())?;
        }
        
        // Validate token counts are non-negative
        if let Some(input_tokens) = self.input_tokens {
            if input_tokens < 0 {
                anyhow::bail!("Input tokens must be non-negative");
            }
        }
        
        if let Some(output_tokens) = self.output_tokens {
            if output_tokens < 0 {
                anyhow::bail!("Output tokens must be non-negative");
            }
        }
        
        if let Some(total_tokens) = self.total_tokens {
            if total_tokens < 0 {
                anyhow::bail!("Total tokens must be non-negative");
            }
        }
        
        // Validate latency is non-negative
        if let Some(latency_ms) = self.latency_ms {
            if latency_ms < 0 {
                anyhow::bail!("Latency must be non-negative");
            }
        }
        
        // Validate costs are non-negative
        if let Some(input_cost) = self.input_cost {
            if input_cost < rust_decimal::Decimal::ZERO {
                anyhow::bail!("Input cost must be non-negative");
            }
        }
        
        if let Some(output_cost) = self.output_cost {
            if output_cost < rust_decimal::Decimal::ZERO {
                anyhow::bail!("Output cost must be non-negative");
            }
        }
        
        if let Some(total_cost) = self.total_cost {
            if total_cost < rust_decimal::Decimal::ZERO {
                anyhow::bail!("Total cost must be non-negative");
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_slug_valid() {
        assert!(validate_slug("valid-slug").is_ok());
        assert!(validate_slug("another-valid-slug-123").is_ok());
    }

    #[test]
    fn test_validate_slug_invalid() {
        assert!(validate_slug("Invalid_Slug").is_err());
        assert!(validate_slug("invalid slug").is_err());
        assert!(validate_slug("INVALID").is_err());
        assert!(validate_slug("").is_err());
    }

    #[test]
    fn test_validate_company() {
        let company = InsertCompany {
            name: "Test Company".to_string(),
            slug: "test-company".to_string(),
            metadata: None,
        };
        assert!(company.validate().is_ok());
    }

    #[test]
    fn test_validate_company_invalid_slug() {
        let company = InsertCompany {
            name: "Test Company".to_string(),
            slug: "invalid_slug".to_string(),
            metadata: None,
        };
        assert!(company.validate().is_err());
    }

    #[test]
    fn test_validate_request_event() {
        let event = InsertRequestEvent {
            company_id: None,
            team_id: None,
            ai_client_id: None,
            provider_id: None,
            model_id: None,
            input_type_id: None,
            pipeline_stage_id: None,
            request_hash: "a".repeat(64),
            request_id: None,
            parent_request_id: None,
            input_text: None,
            input_tokens: None,
            output_text: None,
            output_tokens: None,
            total_tokens: None,
            latency_ms: None,
            timing_breakdown: None,
            input_cost: None,
            output_cost: None,
            total_cost: None,
            error_type: None,
            error_message: None,
            error_details: None,
            metadata: None,
            custom_attributes: None,
        };
        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_validate_request_event_invalid_hash() {
        let event = InsertRequestEvent {
            company_id: None,
            team_id: None,
            ai_client_id: None,
            provider_id: None,
            model_id: None,
            input_type_id: None,
            pipeline_stage_id: None,
            request_hash: "invalid".to_string(),
            request_id: None,
            parent_request_id: None,
            input_text: None,
            input_tokens: None,
            output_text: None,
            output_tokens: None,
            total_tokens: None,
            latency_ms: None,
            timing_breakdown: None,
            input_cost: None,
            output_cost: None,
            total_cost: None,
            error_type: None,
            error_message: None,
            error_details: None,
            metadata: None,
            custom_attributes: None,
        };
        assert!(event.validate().is_err());
    }
}