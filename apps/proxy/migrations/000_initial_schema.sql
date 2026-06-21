-- Initial Schema Migration
-- This migration creates the core database schema for AI Analytics Dashboard
-- Supports multi-dimensional analytics across clients, teams, providers, models, and pipeline stages

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================
-- Core Dimension Tables
-- ============================================

-- Companies (Clients)
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_companies_slug ON companies(slug);
CREATE INDEX idx_companies_created ON companies(created_at);

-- AI Clients
CREATE TABLE ai_clients (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    client_type VARCHAR(50) NOT NULL,
    version VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_ai_clients_slug ON ai_clients(slug);
CREATE INDEX idx_ai_clients_type ON ai_clients(client_type);

-- Teams
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    parent_team_id UUID REFERENCES teams(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(company_id, slug)
);

CREATE INDEX idx_teams_company ON teams(company_id);
CREATE INDEX idx_teams_parent ON teams(parent_team_id);
CREATE INDEX idx_teams_slug ON teams(company_id, slug);

-- AI Providers
CREATE TABLE ai_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    provider_type VARCHAR(50) NOT NULL,
    api_endpoint VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_ai_providers_slug ON ai_providers(slug);
CREATE INDEX idx_ai_providers_type ON ai_providers(provider_type);

-- AI Models
CREATE TABLE ai_models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    model_type VARCHAR(50) NOT NULL,
    context_window INTEGER,
    pricing JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(provider_id, slug)
);

CREATE INDEX idx_ai_models_provider ON ai_models(provider_id);
CREATE INDEX idx_ai_models_type ON ai_models(model_type);
CREATE INDEX idx_ai_models_slug ON ai_models(provider_id, slug);

-- Input Types
CREATE TABLE input_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    category VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_input_types_slug ON input_types(slug);
CREATE INDEX idx_input_types_category ON input_types(category);

-- Pipeline Stages
CREATE TABLE pipeline_stages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    stage_type VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_pipeline_stages_slug ON pipeline_stages(slug);
CREATE INDEX idx_pipeline_stages_type ON pipeline_stages(stage_type);

-- ============================================
-- Core Analytics Table
-- ============================================

-- Request Events
CREATE TABLE request_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Dimensional Foreign Keys
    company_id UUID REFERENCES companies(id) ON DELETE SET NULL,
    team_id UUID REFERENCES teams(id) ON DELETE SET NULL,
    ai_client_id UUID REFERENCES ai_clients(id) ON DELETE SET NULL,
    provider_id UUID REFERENCES ai_providers(id) ON DELETE SET NULL,
    model_id UUID REFERENCES ai_models(id) ON DELETE SET NULL,
    input_type_id UUID REFERENCES input_types(id) ON DELETE SET NULL,
    pipeline_stage_id UUID REFERENCES pipeline_stages(id) ON DELETE SET NULL,
    
    -- Request Identification
    request_hash VARCHAR(64) NOT NULL,
    request_id VARCHAR(255),
    parent_request_id UUID REFERENCES request_events(id) ON DELETE SET NULL,
    
    -- Request Content
    input_text TEXT,
    input_tokens INTEGER,
    output_text TEXT,
    output_tokens INTEGER,
    total_tokens INTEGER,
    
    -- Performance Metrics
    latency_ms INTEGER,
    timing_breakdown JSONB DEFAULT '{}',
    
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
    processed_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for dimensional queries
CREATE INDEX idx_request_events_company ON request_events(company_id);
CREATE INDEX idx_request_events_team ON request_events(team_id);
CREATE INDEX idx_request_events_ai_client ON request_events(ai_client_id);
CREATE INDEX idx_request_events_provider ON request_events(provider_id);
CREATE INDEX idx_request_events_model ON request_events(model_id);
CREATE INDEX idx_request_events_input_type ON request_events(input_type_id);
CREATE INDEX idx_request_events_pipeline_stage ON request_events(pipeline_stage_id);

-- Indexes for request correlation
CREATE INDEX idx_request_events_hash ON request_events(request_hash);
CREATE INDEX idx_request_events_parent ON request_events(parent_request_id);

-- Indexes for time-series queries
CREATE INDEX idx_request_events_created ON request_events(created_at);

-- Composite indexes for multi-dimensional queries
CREATE INDEX idx_request_events_company_created ON request_events(company_id, created_at);
CREATE INDEX idx_request_events_team_created ON request_events(team_id, created_at);
CREATE INDEX idx_request_events_provider_created ON request_events(provider_id, created_at);
CREATE INDEX idx_request_events_model_created ON request_events(model_id, created_at);
CREATE INDEX idx_request_events_client_created ON request_events(ai_client_id, created_at);

-- Performance optimization for time-series queries
CREATE INDEX idx_request_events_created_company ON request_events(created_at DESC, company_id);

-- ============================================
-- Aggregation Tables
-- ============================================

-- Daily Aggregates
CREATE TABLE daily_aggregates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
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
    UNIQUE(company_id, team_id, ai_client_id, provider_id, model_id, input_type_id, pipeline_stage_id, date)
);

-- Indexes for daily aggregates
CREATE INDEX idx_daily_aggregates_date ON daily_aggregates(date);
CREATE INDEX idx_daily_aggregates_company_date ON daily_aggregates(company_id, date);
CREATE INDEX idx_daily_aggregates_team_date ON daily_aggregates(team_id, date);
CREATE INDEX idx_daily_aggregates_provider_date ON daily_aggregates(provider_id, date);
CREATE INDEX idx_daily_aggregates_model_date ON daily_aggregates(model_id, date);

-- Hourly Aggregates
CREATE TABLE hourly_aggregates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
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
    UNIQUE(company_id, team_id, ai_client_id, provider_id, model_id, input_type_id, pipeline_stage_id, hour)
);

-- Indexes for hourly aggregates
CREATE INDEX idx_hourly_aggregates_hour ON hourly_aggregates(hour);
CREATE INDEX idx_hourly_aggregates_company_hour ON hourly_aggregates(company_id, hour);
CREATE INDEX idx_hourly_aggregates_provider_hour ON hourly_aggregates(provider_id, hour);
CREATE INDEX idx_hourly_aggregates_model_hour ON hourly_aggregates(model_id, hour);

-- ============================================
-- Triggers for Automatic Timestamp Updates
-- ============================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to tables with updated_at
CREATE TRIGGER update_companies_updated_at BEFORE UPDATE ON companies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ai_clients_updated_at BEFORE UPDATE ON ai_clients
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_teams_updated_at BEFORE UPDATE ON teams
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ai_providers_updated_at BEFORE UPDATE ON ai_providers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ai_models_updated_at BEFORE UPDATE ON ai_models
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pipeline_stages_updated_at BEFORE UPDATE ON pipeline_stages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_daily_aggregates_updated_at BEFORE UPDATE ON daily_aggregates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_hourly_aggregates_updated_at BEFORE UPDATE ON hourly_aggregates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();