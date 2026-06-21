use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Datelike, Timelike};
use std::collections::HashMap;

/// Standardized metadata schema for all collectors
/// This ensures consistent analytics across all dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Unique identifier for the metadata record
    pub metadata_id: String,
    
    /// Timestamp when metadata was created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when metadata was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Schema version for migration support
    pub schema_version: u32,
    
    /// Company client identifier
    pub client_id: Option<String>,
    
    /// AI client (e.g., "claude-code", "codex", "pi", "devin")
    pub ai_client: String,
    
    /// Team identifier within the client
    pub team_id: Option<String>,
    
    /// Pipeline stage (e.g., "development", "staging", "production")
    pub pipeline_stage: String,
    
    /// AI model provider (e.g., "anthropic", "openai", "google", "microsoft")
    pub provider: String,
    
    /// Specific model name (e.g., "claude-3-opus", "gpt-4")
    pub model: String,
    
    /// Input type (e.g., "text", "chat", "image", "audio")
    pub input_type: String,
    
    /// Additional custom metadata fields
    pub custom_fields: HashMap<String, serde_json::Value>,
    
    /// Derived/enriched metadata
    pub enriched_fields: HashMap<String, serde_json::Value>,
}

impl Metadata {
    /// Create a new metadata instance with required fields
    pub fn new(
        ai_client: String,
        pipeline_stage: String,
        provider: String,
        model: String,
        input_type: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            metadata_id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            schema_version: 1,
            client_id: None,
            ai_client,
            team_id: None,
            pipeline_stage,
            provider,
            model,
            input_type,
            custom_fields: HashMap::new(),
            enriched_fields: HashMap::new(),
        }
    }
    
    /// Set optional client identifier
    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }
    
    /// Set optional team identifier
    pub fn with_team_id(mut self, team_id: String) -> Self {
        self.team_id = Some(team_id);
        self
    }
    
    /// Add a custom field
    pub fn with_custom_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_fields.insert(key, value);
        self
    }
    
    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Metadata validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }
    
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
    
    pub fn with_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self.is_valid = false;
        self
    }
}

/// Validate metadata according to schema rules
pub fn validate_metadata(metadata: &Metadata) -> ValidationResult {
    let mut result = ValidationResult::valid();
    
    // Validate required fields
    if metadata.ai_client.is_empty() {
        result = result.with_error("ai_client cannot be empty".to_string());
    }
    
    if metadata.pipeline_stage.is_empty() {
        result = result.with_error("pipeline_stage cannot be empty".to_string());
    }
    
    if metadata.provider.is_empty() {
        result = result.with_error("provider cannot be empty".to_string());
    }
    
    if metadata.model.is_empty() {
        result = result.with_error("model cannot be empty".to_string());
    }
    
    if metadata.input_type.is_empty() {
        result = result.with_error("input_type cannot be empty".to_string());
    }
    
    // Validate known values
    let valid_input_types = vec!["text", "chat", "image", "audio", "video"];
    if !valid_input_types.contains(&metadata.input_type.as_str()) {
        result = result.with_warning(format!(
            "Unknown input_type '{}'. Valid types: {:?}",
            metadata.input_type, valid_input_types
        ));
    }
    
    let valid_pipeline_stages = vec!["development", "staging", "production", "testing"];
    if !valid_pipeline_stages.contains(&metadata.pipeline_stage.as_str()) {
        result = result.with_warning(format!(
            "Unknown pipeline_stage '{}'. Valid stages: {:?}",
            metadata.pipeline_stage, valid_pipeline_stages
        ));
    }
    
    result
}

/// Metadata migration error
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Invalid metadata structure: {0}")]
    InvalidStructure(String),
}

/// Metadata version migrator
pub struct MetadataMigrator;

impl MetadataMigrator {
    /// Current schema version
    pub const CURRENT_VERSION: u32 = 1;
    
    /// Migrate metadata to current version
    pub fn migrate_to_current(mut metadata: Metadata) -> Result<Metadata, MigrationError> {
        if metadata.schema_version > Self::CURRENT_VERSION {
            return Err(MigrationError::UnsupportedVersion(metadata.schema_version));
        }
        
        while metadata.schema_version < Self::CURRENT_VERSION {
            metadata = Self::migrate(metadata)?;
        }
        Ok(metadata)
    }
    
    /// Perform single version migration
    fn migrate(metadata: Metadata) -> Result<Metadata, MigrationError> {
        match metadata.schema_version {
            0 => Self::migrate_v0_to_v1(metadata),
            1 => Ok(metadata), // Already at version 1
            _ => Err(MigrationError::UnsupportedVersion(metadata.schema_version)),
        }
    }
    
    /// Migration from version 0 to 1
    /// This is a placeholder for future schema changes
    fn migrate_v0_to_v1(mut metadata: Metadata) -> Result<Metadata, MigrationError> {
        // Example migration: add default enriched fields
        if metadata.enriched_fields.is_empty() {
            metadata.enriched_fields.insert(
                "migration_timestamp".to_string(),
                serde_json::json!(Utc::now().to_rfc3339()),
            );
        }
        
        metadata.schema_version = 1;
        metadata.touch();
        Ok(metadata)
    }
    
    /// Check if metadata needs migration
    pub fn needs_migration(metadata: &Metadata) -> bool {
        metadata.schema_version < Self::CURRENT_VERSION
    }
}

/// Metadata enrichment error
#[derive(Debug, thiserror::Error)]
pub enum EnrichmentError {
    #[error("Missing required field for enrichment: {0}")]
    MissingField(String),
    
    #[error("Enrichment calculation failed: {0}")]
    CalculationFailed(String),
    
    #[error("Invalid enrichment data: {0}")]
    InvalidData(String),
}

/// Metadata enricher for adding derived metadata
pub struct MetadataEnricher;

impl MetadataEnricher {
    /// Add cost estimation to metadata
    pub fn enrich_with_cost_estimate(
        metadata: &mut Metadata,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<(), EnrichmentError> {
        let cost = estimate_cost(&metadata.provider, &metadata.model, input_tokens, output_tokens)?;
        metadata.enriched_fields.insert(
            "estimated_cost".to_string(),
            serde_json::json!(cost),
        );
        metadata.enriched_fields.insert(
            "input_tokens".to_string(),
            serde_json::json!(input_tokens),
        );
        metadata.enriched_fields.insert(
            "output_tokens".to_string(),
            serde_json::json!(output_tokens),
        );
        metadata.touch();
        Ok(())
    }
    
    /// Add geographic region metadata
    pub fn enrich_with_region(metadata: &mut Metadata, region: String) {
        metadata.enriched_fields.insert(
            "region".to_string(),
            serde_json::json!(region),
        );
        metadata.touch();
    }
    
    /// Add session metadata
    pub fn enrich_with_session(metadata: &mut Metadata, session_id: String) {
        metadata.enriched_fields.insert(
            "session_id".to_string(),
            serde_json::json!(session_id),
        );
        metadata.touch();
    }
    
    /// Add user metadata
    pub fn enrich_with_user(metadata: &mut Metadata, user_id: String) {
        metadata.enriched_fields.insert(
            "user_id".to_string(),
            serde_json::json!(user_id),
        );
        metadata.touch();
    }
    
    /// Add timestamp-based enrichment
    pub fn enrich_with_timestamps(metadata: &mut Metadata) {
        let now = Utc::now();
        metadata.enriched_fields.insert(
            "enrichment_timestamp".to_string(),
            serde_json::json!(now.to_rfc3339()),
        );
        metadata.enriched_fields.insert(
            "hour_of_day".to_string(),
            serde_json::json!(now.hour()),
        );
        metadata.enriched_fields.insert(
            "day_of_week".to_string(),
            serde_json::json!(now.weekday().number_from_monday()),
        );
        metadata.touch();
    }
    
    /// Add model category enrichment
    pub fn enrich_with_model_category(metadata: &mut Metadata) {
        let category = categorize_model(&metadata.model);
        metadata.enriched_fields.insert(
            "model_category".to_string(),
            serde_json::json!(category),
        );
        metadata.touch();
    }
    
    /// Add provider category enrichment
    pub fn enrich_with_provider_category(metadata: &mut Metadata) {
        let category = categorize_provider(&metadata.provider);
        metadata.enriched_fields.insert(
            "provider_category".to_string(),
            serde_json::json!(category),
        );
        metadata.touch();
    }
    
    /// Apply all standard enrichments
    pub fn apply_standard_enrichments(
        metadata: &mut Metadata,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
    ) -> Result<(), EnrichmentError> {
        MetadataEnricher::enrich_with_timestamps(metadata);
        MetadataEnricher::enrich_with_model_category(metadata);
        MetadataEnricher::enrich_with_provider_category(metadata);
        
        if let (Some(input), Some(output)) = (input_tokens, output_tokens) {
            MetadataEnricher::enrich_with_cost_estimate(metadata, input, output)?;
        }
        
        Ok(())
    }
}

/// Estimate cost based on provider and model
fn estimate_cost(provider: &str, model: &str, input_tokens: u32, output_tokens: u32) -> Result<f64, EnrichmentError> {
    // Simplified cost estimation - in production, this would use a pricing database
    let (input_cost_per_1k, output_cost_per_1k) = match provider {
        "anthropic" => match model {
            "claude-3-opus" => (15.0, 75.0),
            "claude-3-sonnet" => (3.0, 15.0),
            "claude-3-haiku" => (0.25, 1.25),
            _ => return Err(EnrichmentError::MissingField(format!("Unknown model: {}", model))),
        },
        "openai" => match model {
            "gpt-4" => (30.0, 60.0),
            "gpt-4-turbo" => (10.0, 30.0),
            "gpt-3.5-turbo" => (0.5, 1.5),
            _ => return Err(EnrichmentError::MissingField(format!("Unknown model: {}", model))),
        },
        "google" => match model {
            "gemini-pro" => (0.5, 1.5),
            "gemini-ultra" => (2.0, 4.0),
            _ => return Err(EnrichmentError::MissingField(format!("Unknown model: {}", model))),
        },
        _ => return Err(EnrichmentError::MissingField(format!("Unknown provider: {}", provider))),
    };
    
    let input_cost = (input_tokens as f64 / 1000.0) * input_cost_per_1k;
    let output_cost = (output_tokens as f64 / 1000.0) * output_cost_per_1k;
    
    Ok(input_cost + output_cost)
}

/// Categorize model by type
fn categorize_model(model: &str) -> &'static str {
    if model.contains("opus") || model.contains("ultra") {
        "flagship"
    } else if model.contains("sonnet") || (model.contains("turbo") && !model.contains("3.5")) {
        "balanced"
    } else if model.contains("haiku") || model.contains("3.5") {
        "fast"
    } else {
        "unknown"
    }
}

/// Categorize provider by type
fn categorize_provider(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "proprietary",
        "openai" => "proprietary",
        "google" => "proprietary",
        "microsoft" => "proprietary",
        "aws" => "cloud",
        "openrouter" => "aggregator",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_creation() {
        let metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        assert_eq!(metadata.ai_client, "claude-code");
        assert_eq!(metadata.schema_version, 1);
        assert!(metadata.client_id.is_none());
    }
    
    #[test]
    fn test_metadata_builder() {
        let metadata = Metadata::new(
            "codex".to_string(),
            "production".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
            "chat".to_string(),
        )
        .with_client_id("client-123".to_string())
        .with_team_id("team-456".to_string())
        .with_custom_field("region".to_string(), serde_json::json!("us-west"));
        
        assert_eq!(metadata.client_id, Some("client-123".to_string()));
        assert_eq!(metadata.team_id, Some("team-456".to_string()));
        assert_eq!(metadata.custom_fields.get("region"), Some(&serde_json::json!("us-west")));
    }
    
    #[test]
    fn test_metadata_validation_valid() {
        let metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        let result = validate_metadata(&metadata);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
    
    #[test]
    fn test_metadata_validation_invalid() {
        let metadata = Metadata::new(
            "".to_string(),  // Invalid: empty ai_client
            "".to_string(),  // Invalid: empty pipeline_stage
            "".to_string(),  // Invalid: empty provider
            "".to_string(),  // Invalid: empty model
            "".to_string(),  // Invalid: empty input_type
        );
        
        let result = validate_metadata(&metadata);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
    
    #[test]
    fn test_metadata_validation_warnings() {
        let metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "unknown-type".to_string(),  // Will generate warning
        );
        
        let result = validate_metadata(&metadata);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }
    
    #[test]
    fn test_metadata_migration_current_version() {
        let metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        assert!(!MetadataMigrator::needs_migration(&metadata));
    }
    
    #[test]
    fn test_metadata_migration_needed() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        metadata.schema_version = 0;
        
        assert!(MetadataMigrator::needs_migration(&metadata));
    }
    
    #[test]
    fn test_metadata_migration_success() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        metadata.schema_version = 0;
        
        let migrated = MetadataMigrator::migrate_to_current(metadata).unwrap();
        assert_eq!(migrated.schema_version, 1);
        assert!(!migrated.enriched_fields.is_empty());
    }
    
    #[test]
    fn test_metadata_migration_unsupported_version() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        metadata.schema_version = 99; // Unsupported version
        
        let result = MetadataMigrator::migrate(metadata);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_enrichment_cost_estimate() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        let result = MetadataEnricher::enrich_with_cost_estimate(&mut metadata, 1000, 500);
        assert!(result.is_ok());
        
        assert!(metadata.enriched_fields.contains_key("estimated_cost"));
        assert!(metadata.enriched_fields.contains_key("input_tokens"));
        assert!(metadata.enriched_fields.contains_key("output_tokens"));
    }
    
    #[test]
    fn test_enrichment_region() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        MetadataEnricher::enrich_with_region(&mut metadata, "us-west".to_string());
        
        assert_eq!(
            metadata.enriched_fields.get("region"),
            Some(&serde_json::json!("us-west"))
        );
    }
    
    #[test]
    fn test_enrichment_timestamps() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        MetadataEnricher::enrich_with_timestamps(&mut metadata);
        
        assert!(metadata.enriched_fields.contains_key("enrichment_timestamp"));
        assert!(metadata.enriched_fields.contains_key("hour_of_day"));
        assert!(metadata.enriched_fields.contains_key("day_of_week"));
    }
    
    #[test]
    fn test_enrichment_model_category() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        MetadataEnricher::enrich_with_model_category(&mut metadata);
        
        assert_eq!(
            metadata.enriched_fields.get("model_category"),
            Some(&serde_json::json!("flagship"))
        );
    }
    
    #[test]
    fn test_enrichment_provider_category() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        MetadataEnricher::enrich_with_provider_category(&mut metadata);
        
        assert_eq!(
            metadata.enriched_fields.get("provider_category"),
            Some(&serde_json::json!("proprietary"))
        );
    }
    
    #[test]
    fn test_standard_enrichments() {
        let mut metadata = Metadata::new(
            "claude-code".to_string(),
            "development".to_string(),
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            "text".to_string(),
        );
        
        let result = MetadataEnricher::apply_standard_enrichments(&mut metadata, Some(1000), Some(500));
        assert!(result.is_ok());
        
        // Check that standard enrichments were applied
        assert!(metadata.enriched_fields.contains_key("enrichment_timestamp"));
        assert!(metadata.enriched_fields.contains_key("model_category"));
        assert!(metadata.enriched_fields.contains_key("provider_category"));
        assert!(metadata.enriched_fields.contains_key("estimated_cost"));
    }
    
    #[test]
    fn test_cost_estimation_anthropic() {
        let cost = estimate_cost("anthropic", "claude-3-opus", 1000, 500).unwrap();
        // Claude-3 Opus: $15/1k input, $75/1k output
        // 1000 input = $15, 500 output = $37.5, total = $52.5
        assert!((cost - 52.5).abs() < 0.01);
    }
    
    #[test]
    fn test_cost_estimation_openai() {
        let cost = estimate_cost("openai", "gpt-4", 1000, 500).unwrap();
        // GPT-4: $30/1k input, $60/1k output
        // 1000 input = $30, 500 output = $30, total = $60
        assert!((cost - 60.0).abs() < 0.01);
    }
    
    #[test]
    fn test_model_categorization() {
        assert_eq!(categorize_model("claude-3-opus"), "flagship");
        assert_eq!(categorize_model("claude-3-sonnet"), "balanced");
        assert_eq!(categorize_model("claude-3-haiku"), "fast");
        assert_eq!(categorize_model("gpt-4-turbo"), "balanced");
        assert_eq!(categorize_model("gpt-3.5-turbo"), "fast");
    }
    
    #[test]
    fn test_provider_categorization() {
        assert_eq!(categorize_provider("anthropic"), "proprietary");
        assert_eq!(categorize_provider("openai"), "proprietary");
        assert_eq!(categorize_provider("google"), "proprietary");
        assert_eq!(categorize_provider("aws"), "cloud");
        assert_eq!(categorize_provider("openrouter"), "aggregator");
    }
}