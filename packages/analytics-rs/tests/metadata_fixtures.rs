use analytics_rs::{Metadata, validate_metadata, MetadataMigrator};
use serde_json::json;

/// Fixture: Valid metadata for Claude Code in development
pub fn valid_claude_code_metadata() -> Metadata {
    Metadata::new(
        "claude-code".to_string(),
        "development".to_string(),
        "anthropic".to_string(),
        "claude-3-opus".to_string(),
        "text".to_string(),
    )
}

/// Fixture: Valid metadata for Codex in production
pub fn valid_codex_metadata() -> Metadata {
    Metadata::new(
        "codex".to_string(),
        "production".to_string(),
        "openai".to_string(),
        "gpt-4".to_string(),
        "chat".to_string(),
    )
    .with_client_id("client-123".to_string())
    .with_team_id("team-456".to_string())
}

/// Fixture: Metadata with custom fields
pub fn metadata_with_custom_fields() -> Metadata {
    Metadata::new(
        "claude-code".to_string(),
        "staging".to_string(),
        "anthropic".to_string(),
        "claude-3-sonnet".to_string(),
        "image".to_string(),
    )
    .with_custom_field("region".to_string(), json!("us-east"))
    .with_custom_field("project".to_string(), json!("ai-dashboard"))
    .with_custom_field("environment".to_string(), json!("staging"))
}

/// Fixture: Metadata with enriched fields
pub fn metadata_with_enriched_fields() -> Metadata {
    let mut metadata = Metadata::new(
        "pi".to_string(),
        "testing".to_string(),
        "google".to_string(),
        "gemini-pro".to_string(),
        "audio".to_string(),
    );
    
    metadata.enriched_fields.insert(
        "estimated_cost".to_string(),
        json!(0.0015)
    );
    metadata.enriched_fields.insert(
        "token_estimate".to_string(),
        json!(250)
    );
    metadata.enriched_fields.insert(
        "processing_time_ms".to_string(),
        json!(1250)
    );
    
    metadata
}

/// Fixture: Invalid metadata (empty required fields)
pub fn invalid_metadata_empty_fields() -> Metadata {
    Metadata::new(
        "".to_string(),  // Invalid: empty ai_client
        "".to_string(),  // Invalid: empty pipeline_stage
        "".to_string(),  // Invalid: empty provider
        "".to_string(),  // Invalid: empty model
        "".to_string(),  // Invalid: empty input_type
    )
}

/// Fixture: Metadata with unknown input type (generates warning)
pub fn metadata_unknown_input_type() -> Metadata {
    Metadata::new(
        "claude-code".to_string(),
        "development".to_string(),
        "anthropic".to_string(),
        "claude-3-opus".to_string(),
        "unknown-type".to_string(),  // Will generate warning
    )
}

/// Fixture: Old version metadata for migration testing
pub fn old_version_metadata() -> Metadata {
    let mut metadata = Metadata::new(
        "claude-code".to_string(),
        "development".to_string(),
        "anthropic".to_string(),
        "claude-3-opus".to_string(),
        "text".to_string(),
    );
    metadata.schema_version = 0;
    metadata
}

/// Fixture: Complete metadata with all fields
pub fn complete_metadata() -> Metadata {
    Metadata::new(
        "devin".to_string(),
        "production".to_string(),
        "anthropic".to_string(),
        "claude-3-opus".to_string(),
        "chat".to_string(),
    )
    .with_client_id("enterprise-client".to_string())
    .with_team_id("platform-team".to_string())
    .with_custom_field("region".to_string(), json!("eu-west"))
    .with_custom_field("deployment".to_string(), json!("kubernetes"))
    .with_custom_field("version".to_string(), json!("1.2.3"))
}

/// Validator: Check if metadata is valid according to schema
pub fn assert_valid_metadata(metadata: &Metadata) {
    let result = validate_metadata(metadata);
    assert!(
        result.is_valid,
        "Metadata should be valid, but got errors: {:?}",
        result.errors
    );
}

/// Validator: Check if metadata is invalid according to schema
pub fn assert_invalid_metadata(metadata: &Metadata, expected_error_count: usize) {
    let result = validate_metadata(metadata);
    assert!(
        !result.is_valid,
        "Metadata should be invalid, but validation passed"
    );
    assert_eq!(
        result.errors.len(),
        expected_error_count,
        "Expected {} validation errors, got {}",
        expected_error_count,
        result.errors.len()
    );
}

/// Validator: Check if metadata has warnings
pub fn assert_metadata_has_warnings(metadata: &Metadata, expected_warning_count: usize) {
    let result = validate_metadata(metadata);
    assert_eq!(
        result.warnings.len(),
        expected_warning_count,
        "Expected {} warnings, got {}",
        expected_warning_count,
        result.warnings.len()
    );
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[test]
    fn test_valid_claude_code_fixture() {
        let metadata = valid_claude_code_metadata();
        assert_valid_metadata(&metadata);
        assert_eq!(metadata.ai_client, "claude-code");
        assert_eq!(metadata.provider, "anthropic");
    }

    #[test]
    fn test_valid_codex_fixture() {
        let metadata = valid_codex_metadata();
        assert_valid_metadata(&metadata);
        assert_eq!(metadata.client_id, Some("client-123".to_string()));
        assert_eq!(metadata.team_id, Some("team-456".to_string()));
    }

    #[test]
    fn test_custom_fields_fixture() {
        let metadata = metadata_with_custom_fields();
        assert_valid_metadata(&metadata);
        assert_eq!(metadata.custom_fields.len(), 3);
        assert_eq!(
            metadata.custom_fields.get("region"),
            Some(&json!("us-east"))
        );
    }

    #[test]
    fn test_enriched_fields_fixture() {
        let metadata = metadata_with_enriched_fields();
        assert_valid_metadata(&metadata);
        assert_eq!(metadata.enriched_fields.len(), 3);
        assert_eq!(
            metadata.enriched_fields.get("estimated_cost"),
            Some(&json!(0.0015))
        );
    }

    #[test]
    fn test_invalid_metadata_fixture() {
        let metadata = invalid_metadata_empty_fields();
        assert_invalid_metadata(&metadata, 5); // 5 empty required fields
    }

    #[test]
    fn test_unknown_input_type_fixture() {
        let metadata = metadata_unknown_input_type();
        assert_valid_metadata(&metadata); // Still valid, just has warnings
        assert_metadata_has_warnings(&metadata, 1);
    }

    #[test]
    fn test_old_version_metadata_fixture() {
        let metadata = old_version_metadata();
        assert!(MetadataMigrator::needs_migration(&metadata));
        
        let migrated = MetadataMigrator::migrate_to_current(metadata).unwrap();
        assert_eq!(migrated.schema_version, 1);
    }

    #[test]
    fn test_complete_metadata_fixture() {
        let metadata = complete_metadata();
        assert_valid_metadata(&metadata);
        assert_eq!(metadata.ai_client, "devin");
        assert_eq!(metadata.client_id, Some("enterprise-client".to_string()));
        assert_eq!(metadata.custom_fields.len(), 3);
    }
}