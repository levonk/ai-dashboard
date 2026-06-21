# Metadata Schema Documentation

## Overview

The metadata schema provides a standardized structure for tracking AI usage across all dimensions in the AI Analytics Dashboard. This ensures consistent analytics across all collectors regardless of the AI client, provider, or input type.

## Schema Structure

### Core Fields

```rust
pub struct Metadata {
    pub metadata_id: String,              // Unique identifier
    pub created_at: DateTime<Utc>,        // Creation timestamp
    pub updated_at: DateTime<Utc>,        // Last update timestamp
    pub schema_version: u32,              // Schema version for migration
    pub client_id: Option<String>,        // Company client identifier
    pub ai_client: String,                // AI client (e.g., "claude-code", "codex")
    pub team_id: Option<String>,          // Team identifier
    pub pipeline_stage: String,           // Pipeline stage (e.g., "development")
    pub provider: String,                 // AI provider (e.g., "anthropic", "openai")
    pub model: String,                    // Model name (e.g., "claude-3-opus")
    pub input_type: String,               // Input type (e.g., "text", "image")
    pub custom_fields: HashMap<String, serde_json::Value>,  // Custom metadata
    pub enriched_fields: HashMap<String, serde_json::Value>, // Derived metadata
}
```

### Field Descriptions

#### Required Fields

- **`metadata_id`**: UUID-based unique identifier for the metadata record
- **`created_at`**: ISO 8601 timestamp when the metadata was created
- **`updated_at`**: ISO 8601 timestamp when the metadata was last modified
- **`schema_version`**: Current schema version (starts at 1, used for migrations)
- **`ai_client`**: The AI client being used (e.g., "claude-code", "codex", "pi", "devin")
- **`pipeline_stage`**: The development pipeline stage (e.g., "development", "staging", "production")
- **`provider`**: The AI model provider (e.g., "anthropic", "openai", "google", "microsoft")
- **`model`**: The specific model name (e.g., "claude-3-opus", "gpt-4", "gemini-pro")
- **`input_type`**: The type of input (e.g., "text", "chat", "image", "audio", "video")

#### Optional Fields

- **`client_id`**: Company or organization client identifier
- **`team_id`**: Team identifier within the client organization

#### Extensible Fields

- **`custom_fields`**: Custom key-value pairs for collector-specific metadata
- **`enriched_fields`**: Derived metadata added by the enrichment system

## Usage Examples

### Basic Metadata Creation

```rust
use analytics_rs::Metadata;

let metadata = Metadata::new(
    "claude-code".to_string(),
    "development".to_string(),
    "anthropic".to_string(),
    "claude-3-opus".to_string(),
    "text".to_string(),
);
```

### Metadata with Optional Fields

```rust
use analytics_rs::Metadata;

let metadata = Metadata::new(
    "codex".to_string(),
    "production".to_string(),
    "openai".to_string(),
    "gpt-4".to_string(),
    "chat".to_string(),
)
.with_client_id("client-123".to_string())
.with_team_id("team-456".to_string());
```

### Metadata with Custom Fields

```rust
use analytics_rs::Metadata;
use serde_json::json;

let metadata = Metadata::new(
    "claude-code".to_string(),
    "development".to_string(),
    "anthropic".to_string(),
    "claude-3-opus".to_string(),
    "text".to_string(),
)
.with_custom_field("region".to_string(), json!("us-west"))
.with_custom_field("environment".to_string(), json!("dev"));
```

## Validation

The metadata schema includes built-in validation to ensure data consistency:

### Validation Rules

1. **Required fields**: All required fields must be non-empty
2. **Known values**: Warnings are issued for unknown input types or pipeline stages
3. **Type safety**: All fields are strongly typed via Rust's type system

### Validation Example

```rust
use analytics_rs::{Metadata, validate_metadata};

let metadata = Metadata::new(
    "claude-code".to_string(),
    "development".to_string(),
    "anthropic".to_string(),
    "claude-3-opus".to_string(),
    "text".to_string(),
);

let result = validate_metadata(&metadata);
if result.is_valid {
    println!("Metadata is valid");
} else {
    println!("Validation errors: {:?}", result.errors);
}
```

### Valid Input Types

- `text` - Plain text input
- `chat` - Conversational/chat input
- `image` - Image input
- `audio` - Audio input
- `video` - Video input

### Valid Pipeline Stages

- `development` - Development environment
- `staging` - Staging/pre-production environment
- `production` - Production environment
- `testing` - Testing environment

## Serialization

The metadata schema supports JSON serialization via Serde:

```rust
use analytics_rs::Metadata;
use serde_json;

let metadata = Metadata::new(
    "claude-code".to_string(),
    "development".to_string(),
    "anthropic".to_string(),
    "claude-3-opus".to_string(),
    "text".to_string(),
);

// Serialize to JSON
let json = serde_json::to_string(&metadata).unwrap();

// Deserialize from JSON
let deserialized: Metadata = serde_json::from_str(&json).unwrap();
```

## Versioning and Migration

The metadata schema includes versioning support to handle schema evolution:

### Migration Example

```rust
use analytics_rs::{Metadata, MetadataMigrator};

let mut metadata = Metadata::new(
    "claude-code".to_string(),
    "development".to_string(),
    "anthropic".to_string(),
    "claude-3-opus".to_string(),
    "text".to_string(),
);
metadata.schema_version = 0; // Old version

// Check if migration is needed
if MetadataMigrator::needs_migration(&metadata) {
    // Migrate to current version
    metadata = MetadataMigrator::migrate_to_current(metadata).unwrap();
    println!("Migrated to version {}", metadata.schema_version);
}
```

### Migration Process

1. Check if migration is needed using `MetadataMigrator::needs_migration()`
2. If needed, call `MetadataMigrator::migrate_to_current()`
3. The migrator will apply all necessary version upgrades
4. Updated metadata will have the current schema version

## Enrichment

The `enriched_fields` section allows for automatic metadata enrichment:

```rust
use analytics_rs::Metadata;
use serde_json::json;

let mut metadata = Metadata::new(
    "claude-code".to_string(),
    "development".to_string(),
    "anthropic".to_string(),
    "claude-3-opus".to_string(),
    "text".to_string(),
);

// Add enriched metadata
metadata.enriched_fields.insert(
    "cost_estimate".to_string(),
    json!(0.0025)
);
metadata.enriched_fields.insert(
    "token_count".to_string(),
    json!(150)
);
```

## Best Practices

1. **Always validate metadata** before using it in production
2. **Use custom fields** for collector-specific metadata
3. **Keep schema version updated** when making structural changes
4. **Use migration system** for schema evolution
5. **Document custom fields** in your collector implementation
6. **Use enriched fields** for derived data (costs, token counts, etc.)

## Error Handling

The metadata system uses Rust's `Result` type for error handling:

```rust
use analytics_rs::{Metadata, MetadataMigrator, MigrationError};

match MetadataMigrator::migrate_to_current(metadata) {
    Ok(migrated) => println!("Migration successful"),
    Err(MigrationError::UnsupportedVersion(v)) => {
        eprintln!("Unsupported version: {}", v);
    }
    Err(MigrationError::MigrationFailed(e)) => {
        eprintln!("Migration failed: {}", e);
    }
    Err(MigrationError::InvalidStructure(e)) => {
        eprintln!("Invalid structure: {}", e);
    }
}
```

## Performance Considerations

- Metadata validation is fast (sub-millisecond for typical records)
- Serialization/deserialization is optimized via Serde
- Custom fields use HashMap for O(1) lookups
- Consider the size of custom fields for performance-sensitive applications

## Future Enhancements

Planned enhancements for the metadata schema:

- Support for protobuf serialization
- Additional validation rules
- More enrichment fields
- Schema registry integration
- Metadata templates for common use cases