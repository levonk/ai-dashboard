# Database Schema Design

## Overview

This document describes the core database schema for the AI Analytics Dashboard System, designed to support multi-dimensional analytics across company clients, AI clients, teams, pipeline stages, AI providers, models, and input types.

## Design Principles

1. **Multi-Dimensional Analytics**: Support filtering and aggregation across all key dimensions
2. **Single-Tenant Foundation**: Schema designed for single-tenant with extensibility for future multi-tenant
3. **Performance-First**: Proper indexing for common query patterns
4. **Time-Series Optimized**: Efficient storage and querying of time-series analytics data
5. **Data Integrity**: Foreign key constraints and validation at the database level
6. **Migration-Ready**: Schema versioning and migration support from the start

## Entity-Relationship Model

### Core Dimensions

#### 1. Companies (Clients)
```sql
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    INDEX idx_companies_slug (slug),
    INDEX idx_companies_created (created_at)
);
```

#### 2. AI Clients
```sql
CREATE TABLE ai_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    client_type VARCHAR(50) NOT NULL, -- 'claude-code', 'codex', 'pi', 'devin', etc.
    version VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    INDEX idx_ai_clients_slug (slug),
    INDEX idx_ai_clients_type (client_type)
);
```

#### 3. Teams
```sql
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    parent_team_id UUID REFERENCES teams(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(company_id, slug),
    INDEX idx_teams_company (company_id),
    INDEX idx_teams_parent (parent_team_id),
    INDEX idx_teams_slug (company_id, slug)
);
```

#### 4. AI Providers
```sql
CREATE TABLE ai_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    provider_type VARCHAR(50) NOT NULL, -- 'anthropic', 'openai', 'google', 'microsoft', 'aws', 'openrouter', etc.
    api_endpoint VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    INDEX idx_ai_providers_slug (slug),
    INDEX idx_ai_providers_type (provider_type)
);
```

#### 5. AI Models
```sql
CREATE TABLE ai_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    model_type VARCHAR(50) NOT NULL, -- 'text', 'image', 'audio', 'video', 'code', 'multimodal'
    context_window INTEGER,
    pricing JSONB DEFAULT '{}', -- pricing tiers and costs
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(provider_id, slug),
    INDEX idx_ai_models_provider (provider_id),
    INDEX idx_ai_models_type (model_type),
    INDEX idx_ai_models_slug (provider_id, slug)
);
```

#### 6. Input Types
```sql
CREATE TABLE input_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    category VARCHAR(50) NOT NULL, -- 'text', 'image', 'audio', 'video', 'code', 'multimodal'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    INDEX idx_input_types_slug (slug),
    INDEX idx_input_types_category (category)
);
```

#### 7. Pipeline Stages
```sql
CREATE TABLE pipeline_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    stage_type VARCHAR(50) NOT NULL, -- 'pre-optimization', 'post-optimization', 'headroom', 'omniroute', 'iron-proxy', 'custom'
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    INDEX idx_pipeline_stages_slug (slug),
    INDEX idx_pipeline_stages_type (stage_type)
);
```

### Analytics Events

#### 8. Request Events (Core Analytics Table)
```sql
CREATE TABLE request_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Dimensional Foreign Keys
    company_id UUID REFERENCES companies(id) ON DELETE SET NULL,
    team_id UUID REFERENCES teams(id) ON DELETE SET NULL,
    ai_client_id UUID REFERENCES ai_clients(id) ON DELETE SET NULL,
    provider_id UUID REFERENCES ai_providers(id) ON DELETE SET NULL,
    model_id UUID REFERENCES ai_models(id) ON DELETE SET NULL,
    input_type_id UUID REFERENCES input_types(id) ON DELETE SET NULL,
    pipeline_stage_id UUID REFERENCES pipeline_stages(id) ON DELETE SET NULL,
    
    -- Request Identification
    request_hash VARCHAR(64) NOT NULL, -- SHA-256 hash for correlation
    request_id VARCHAR(255), -- Original request ID from client
    parent_request_id UUID REFERENCES request_events(id) ON DELETE SET NULL, -- For pipeline correlation
    
    -- Request Content
    input_text TEXT,
    input_tokens INTEGER,
    output_text TEXT,
    output_tokens INTEGER,
    total_tokens INTEGER,
    
    -- Performance Metrics
    latency_ms INTEGER,
    timing_breakdown JSONB DEFAULT '{}', -- Detailed timing per stage
    
    -- Cost Metrics
    input_cost DECIMAL(10, 6) DEFAULT 0,
    output_cost DECIMAL(10, 6) DEFAULT 0,
    total_cost DECIMAL(10, 6) DEFAULT 0,
    
    -- Error Tracking
    error_type VARCHAR(100),
    error_message TEXT,
    error_details JSONB DEFAULT '{}',
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    custom_attributes JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE,
    
    -- Indexes for common query patterns
    INDEX idx_request_events_company (company_id),
    INDEX idx_request_events_team (team_id),
    INDEX idx_request_events_ai_client (ai_client_id),
    INDEX idx_request_events_provider (provider_id),
    INDEX idx_request_events_model (model_id),
    INDEX idx_request_events_input_type (input_type_id),
    INDEX idx_request_events_pipeline_stage (pipeline_stage_id),
    INDEX idx_request_events_hash (request_hash),
    INDEX idx_request_events_created (created_at),
    INDEX idx_request_events_parent (parent_request_id),
    
    -- Composite indexes for multi-dimensional queries
    INDEX idx_request_events_company_created (company_id, created_at),
    INDEX idx_request_events_team_created (team_id, created_at),
    INDEX idx_request_events_provider_created (provider_id, created_at),
    INDEX idx_request_events_model_created (model_id, created_at),
    INDEX idx_request_events_client_created (ai_client_id, created_at),
    
    -- Performance optimization for time-series queries
    INDEX idx_request_events_created_company (created_at DESC, company_id)
);
```

### Aggregation Tables (for Performance)

#### 9. Daily Aggregations
```sql
CREATE TABLE daily_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Dimensions
    company_id UUID REFERENCES companies(id) ON DELETE CASCADE,
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    ai_client_id UUID REFERENCES ai_clients(id) ON DELETE CASCADE,
    provider_id UUID REFERENCES ai_providers(id) ON DELETE CASCADE,
    model_id UUID REFERENCES ai_models(id) ON DELETE CASCADE,
    input_type_id UUID REFERENCES input_types(id) ON DELETE CASCADE,
    pipeline_stage_id UUID REFERENCES pipeline_stages(id) ON DELETE CASCADE,
    
    -- Time Period
    date DATE NOT NULL,
    
    -- Aggregated Metrics
    request_count INTEGER DEFAULT 0,
    total_input_tokens BIGINT DEFAULT 0,
    total_output_tokens BIGINT DEFAULT 0,
    total_tokens BIGINT DEFAULT 0,
    avg_latency_ms DECIMAL(10, 2),
    total_cost DECIMAL(12, 6) DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Unique constraint for upsert operations
    UNIQUE(company_id, team_id, ai_client_id, provider_id, model_id, input_type_id, pipeline_stage_id, date),
    
    -- Indexes
    INDEX idx_daily_aggregates_date (date),
    INDEX idx_daily_aggregates_company_date (company_id, date),
    INDEX idx_daily_aggregates_team_date (team_id, date),
    INDEX idx_daily_aggregates_provider_date (provider_id, date),
    INDEX idx_daily_aggregates_model_date (model_id, date)
);
```

#### 10. Hourly Aggregates (for real-time dashboards)
```sql
CREATE TABLE hourly_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Dimensions
    company_id UUID REFERENCES companies(id) ON DELETE CASCADE,
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    ai_client_id UUID REFERENCES ai_clients(id) ON DELETE CASCADE,
    provider_id UUID REFERENCES ai_providers(id) ON DELETE CASCADE,
    model_id UUID REFERENCES ai_models(id) ON DELETE CASCADE,
    input_type_id UUID REFERENCES input_types(id) ON DELETE CASCADE,
    pipeline_stage_id UUID REFERENCES pipeline_stages(id) ON DELETE CASCADE,
    
    -- Time Period
    hour TIMESTAMP WITH TIME ZONE NOT NULL,
    
    -- Aggregated Metrics
    request_count INTEGER DEFAULT 0,
    total_input_tokens BIGINT DEFAULT 0,
    total_output_tokens BIGINT DEFAULT 0,
    total_tokens BIGINT DEFAULT 0,
    avg_latency_ms DECIMAL(10, 2),
    total_cost DECIMAL(12, 6) DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Unique constraint for upsert operations
    UNIQUE(company_id, team_id, ai_client_id, provider_id, model_id, input_type_id, pipeline_stage_id, hour),
    
    -- Indexes
    INDEX idx_hourly_aggregates_hour (hour),
    INDEX idx_hourly_aggregates_company_hour (company_id, hour),
    INDEX idx_hourly_aggregates_provider_hour (provider_id, hour),
    INDEX idx_hourly_aggregates_model_hour (model_id, hour)
);
```

## Entity Relationship Diagram

```
companies (1) ----< (N) teams
teams (1) ----< (N) request_events

ai_clients (1) ----< (N) request_events

ai_providers (1) ----< (N) ai_models
ai_models (1) ----< (N) request_events

input_types (1) ----< (N) request_events

pipeline_stages (1) ----< (N) request_events

request_events (1) ----< (N) request_events (self-referential for pipeline correlation)
request_events (1) ----< (N) daily_aggregates
request_events (1) ----< (N) hourly_aggregates
```

## Migration Strategy

### Version 1: Initial Schema
- Create all dimension tables (companies, ai_clients, teams, ai_providers, ai_models, input_types, pipeline_stages)
- Create request_events table with core indexes
- Seed initial reference data (default AI clients, common providers, input types, pipeline stages)

### Version 2: Performance Indexes
- Add composite indexes for multi-dimensional queries
- Add time-series optimized indexes
- Create aggregation tables (daily_aggregates, hourly_aggregates)

### Version 3: Data Retention and Cleanup
- Add partitioning for request_events by date
- Create cleanup jobs for old data
- Add data retention policies

## Future Multi-Tenant Considerations

The schema is designed to support future multi-tenant capabilities through:

1. **Tenant Column**: Add `tenant_id` column to all tables when multi-tenant is needed
2. **Row-Level Security**: Implement RLS policies for tenant isolation
3. **Index Optimization**: Add tenant_id to composite indexes
4. **Connection Pooling**: Design connection pool strategy for multi-tenant access

## Performance Considerations

### Index Strategy
- **Foreign Key Indexes**: All foreign keys have indexes
- **Query Pattern Indexes**: Composite indexes for common query patterns
- **Time-Series Indexes**: Optimized indexes for time-based queries
- **Hash Indexes**: For request_hash lookups

### Partitioning Strategy (Future)
- Partition request_events by date (monthly partitions)
- Partition aggregation tables by time period
- Automatic partition management

### Query Performance Targets
- Simple dimension filter: <100ms
- Multi-dimensional filter: <500ms
- Time-series range query: <2 seconds
- Aggregation query: <5 seconds

## Data Validation

### Constraints
- Foreign key constraints for referential integrity
- Unique constraints for natural keys (slugs)
- NOT NULL constraints for required fields
- CHECK constraints for data validation

### Triggers
- Automatic timestamp updates (updated_at)
- Data validation triggers
- Aggregation table maintenance triggers

## Security Considerations

### Data Encryption
- Plan for encryption at rest (column-level encryption)
- SSL/TLS for data in transit
- Secure key management

### Access Control
- Role-based access control (RBAC)
- Row-level security for future multi-tenant
- Audit logging for sensitive operations

### Data Privacy
- Support for data deletion requests (GDPR)
- Data anonymization capabilities
- Configurable data retention policies

## Backup and Recovery

### Backup Strategy
- Daily full backups
- Continuous WAL archiving
- Point-in-time recovery capability

### Recovery Procedures
- Database restoration procedures
- Data validation after recovery
- Minimal downtime recovery strategy