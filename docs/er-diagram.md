# Entity Relationship Diagram

```mermaid
erDiagram
    companies ||--o{ teams : "has"
    companies ||--o{ request_events : "tracks"
    
    teams ||--o{ teams : "parent-child"
    teams ||--o{ request_events : "owns"
    
    ai_clients ||--o{ request_events : "generates"
    
    ai_providers ||--o{ ai_models : "provides"
    ai_models ||--o{ request_events : "used in"
    
    input_types ||--o{ request_events : "classifies"
    
    pipeline_stages ||--o{ request_events : "processes"
    
    request_events ||--o{ request_events : "pipeline correlation"
    request_events ||--o{ daily_aggregates : "aggregates to"
    request_events ||--o{ hourly_aggregates : "aggregates to"
    
    companies {
        uuid id PK
        string name
        string slug UK
        timestamp created_at
        timestamp updated_at
        jsonb metadata
    }
    
    ai_clients {
        uuid id PK
        string name
        string slug UK
        string client_type
        string version
        timestamp created_at
        timestamp updated_at
        jsonb metadata
    }
    
    teams {
        uuid id PK
        uuid company_id FK
        string name
        string slug
        uuid parent_team_id FK
        timestamp created_at
        timestamp updated_at
        jsonb metadata
    }
    
    ai_providers {
        uuid id PK
        string name
        string slug UK
        string provider_type
        string api_endpoint
        timestamp created_at
        timestamp updated_at
        jsonb metadata
    }
    
    ai_models {
        uuid id PK
        uuid provider_id FK
        string name
        string slug
        string model_type
        integer context_window
        jsonb pricing
        timestamp created_at
        timestamp updated_at
        jsonb metadata
    }
    
    input_types {
        uuid id PK
        string name
        string slug UK
        string category
        timestamp created_at
    }
    
    pipeline_stages {
        uuid id PK
        string name
        string slug UK
        string stage_type
        text description
        timestamp created_at
        timestamp updated_at
        jsonb metadata
    }
    
    request_events {
        uuid id PK
        uuid company_id FK
        uuid team_id FK
        uuid ai_client_id FK
        uuid provider_id FK
        uuid model_id FK
        uuid input_type_id FK
        uuid pipeline_stage_id FK
        string request_hash
        string request_id
        uuid parent_request_id FK
        text input_text
        integer input_tokens
        text output_text
        integer output_tokens
        integer total_tokens
        integer latency_ms
        jsonb timing_breakdown
        decimal input_cost
        decimal output_cost
        decimal total_cost
        string error_type
        text error_message
        jsonb error_details
        jsonb metadata
        jsonb custom_attributes
        timestamp created_at
        timestamp processed_at
    }
    
    daily_aggregates {
        uuid id PK
        uuid company_id FK
        uuid team_id FK
        uuid ai_client_id FK
        uuid provider_id FK
        uuid model_id FK
        uuid input_type_id FK
        uuid pipeline_stage_id FK
        date date
        integer request_count
        bigint total_input_tokens
        bigint total_output_tokens
        bigint total_tokens
        decimal avg_latency_ms
        decimal total_cost
        integer error_count
        jsonb metadata
        timestamp created_at
        timestamp updated_at
    }
    
    hourly_aggregates {
        uuid id PK
        uuid company_id FK
        uuid team_id FK
        uuid ai_client_id FK
        uuid provider_id FK
        uuid model_id FK
        uuid input_type_id FK
        uuid pipeline_stage_id FK
        timestamp hour
        integer request_count
        bigint total_input_tokens
        bigint total_output_tokens
        bigint total_tokens
        decimal avg_latency_ms
        decimal total_cost
        integer error_count
        jsonb metadata
        timestamp created_at
        timestamp updated_at
    }
```

## Key Relationships

### Core Dimensional Relationships
- **Companies → Teams**: One-to-many hierarchy for organizational structure
- **Teams → Request Events**: Many-to-one for team-based analytics
- **AI Clients → Request Events**: Many-to-one for client-based tracking
- **AI Providers → AI Models**: One-to-many for provider model catalog
- **AI Models → Request Events**: Many-to-one for model-specific analytics
- **Input Types → Request Events**: Many-to-one for input classification
- **Pipeline Stages → Request Events**: Many-to-one for pipeline analytics

### Self-Referential Relationships
- **Request Events → Request Events**: Parent-child relationship for pipeline correlation (e.g., pre-optimization → post-optimization)

### Aggregation Relationships
- **Request Events → Daily Aggregates**: One-to-many for pre-computed daily statistics
- **Request Events → Hourly Aggregates**: One-to-many for real-time hourly statistics

## Query Patterns Supported

### Dimensional Filtering
```sql
-- Filter by company
SELECT * FROM request_events WHERE company_id = ?;

-- Filter by team
SELECT * FROM request_events WHERE team_id = ?;

-- Filter by AI client
SELECT * FROM request_events WHERE ai_client_id = ?;

-- Filter by provider
SELECT * FROM request_events WHERE provider_id = ?;

-- Filter by model
SELECT * FROM request_events WHERE model_id = ?;

-- Multi-dimensional filter
SELECT * FROM request_events 
WHERE company_id = ? 
  AND provider_id = ? 
  AND model_id = ? 
  AND created_at >= ? 
  AND created_at <= ?;
```

### Pipeline Correlation
```sql
-- Track request through pipeline stages
SELECT * FROM request_events 
WHERE request_hash = ? 
ORDER BY created_at;

-- Find parent-child relationships
SELECT parent.*, child.* 
FROM request_events parent
JOIN request_events child ON parent.id = child.parent_request_id
WHERE parent.request_id = ?;
```

### Time-Series Analytics
```sql
-- Daily aggregation from pre-computed table
SELECT * FROM daily_aggregates
WHERE company_id = ? 
  AND date >= ? 
  AND date <= ?;

-- Hourly real-time analytics
SELECT * FROM hourly_aggregates
WHERE model_id = ? 
  AND hour >= ? 
  AND hour <= ?;
```

### Cost Analysis
```sql
-- Cost by provider
SELECT provider_id, SUM(total_cost) as total_cost
FROM request_events
WHERE created_at >= ? AND created_at <= ?
GROUP BY provider_id;

-- Cost by model
SELECT model_id, SUM(total_cost) as total_cost, AVG(total_cost) as avg_cost
FROM request_events
WHERE created_at >= ? AND created_at <= ?
GROUP BY model_id;
```

## Performance Optimizations

### Index Coverage
- All foreign keys have indexes
- Composite indexes for common query patterns
- Time-series indexes for date range queries
- Hash indexes for request correlation

### Query Optimization
- Use aggregation tables for summary queries
- Leverage composite indexes for multi-dimensional filters
- Partition by date for large time-series data
- Materialized views for complex aggregations