-- Performance Metrics Migration
-- This migration adds performance metrics collection and storage for AI Dashboard
-- Supports 19 performance metrics including AI performance, system resources, and hardware monitoring

-- ============================================
-- Performance Metrics Table
-- ============================================

-- Performance Metrics (Time-series data for high-frequency metrics)
CREATE TABLE performance_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Link to request event
    request_event_id UUID NOT NULL REFERENCES request_events(id) ON DELETE CASCADE,
    
    -- AI Performance Metrics
    ttft_ms DECIMAL(10, 2), -- Time to First Token in milliseconds
    total_processing_time_ms DECIMAL(10, 2), -- Total request processing time in milliseconds
    prefill_token_speed DECIMAL(10, 2), -- Prefill token processing speed (tokens/second)
    decode_token_speed DECIMAL(10, 2), -- Decode token processing speed (tokens/second)
    prompt_token_count INTEGER, -- Prompt token count per request
    output_token_count INTEGER, -- Output token count per request
    context_length INTEGER, -- Total tokens in prompt (context length)
    token_efficiency DECIMAL(5, 4), -- Token efficiency (output tokens / total tokens)
    
    -- System Resource Metrics
    gpu_utilization_percent DECIMAL(5, 2), -- GPU utilization percentage
    cpu_utilization_percent DECIMAL(5, 2), -- CPU utilization percentage
    gpu_memory_usage_mb BIGINT, -- GPU memory usage in megabytes
    cpu_memory_usage_mb BIGINT, -- CPU memory usage in megabytes
    unified_memory_usage_mb BIGINT, -- Unified memory usage in megabytes (where applicable)
    
    -- Hardware Monitoring Metrics
    gpu_temperature_celsius DECIMAL(5, 2), -- GPU temperature in Celsius
    cpu_temperature_celsius DECIMAL(5, 2), -- CPU temperature in Celsius
    power_consumption_watts DECIMAL(10, 2), -- Power consumption in Watts
    software_clock_speed_mhz DECIMAL(10, 2), -- Software clock speed in MHz
    hardware_clock_speed_mhz DECIMAL(10, 2), -- Hardware clock speed in MHz
    sample_rate_hz DECIMAL(10, 2), -- Sample rate in Hz
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for request correlation
CREATE INDEX idx_performance_metrics_request_event ON performance_metrics(request_event_id);

-- Indexes for time-series queries
CREATE INDEX idx_performance_metrics_recorded_at ON performance_metrics(recorded_at DESC);

-- Composite indexes for common query patterns
CREATE INDEX idx_performance_metrics_request_recorded ON performance_metrics(request_event_id, recorded_at DESC);

-- Indexes for AI performance metrics queries
CREATE INDEX idx_performance_metrics_ttft ON performance_metrics(ttft_ms);
CREATE INDEX idx_performance_metrics_processing_time ON performance_metrics(total_processing_time_ms);

-- Indexes for system resource metrics queries
CREATE INDEX idx_performance_metrics_gpu_util ON performance_metrics(gpu_utilization_percent);
CREATE INDEX idx_performance_metrics_cpu_util ON performance_metrics(cpu_utilization_percent);

-- Indexes for hardware monitoring queries
CREATE INDEX idx_performance_metrics_gpu_temp ON performance_metrics(gpu_temperature_celsius);
CREATE INDEX idx_performance_metrics_power ON performance_metrics(power_consumption_watts);

-- ============================================
-- Data Retention Policy Support
-- ============================================

-- Create a function to manage data retention
CREATE OR REPLACE FUNCTION performance_metrics_retention_policy()
RETURNS TRIGGER AS $$
BEGIN
    -- This function can be called by a scheduled job to implement retention policies
    -- Example: DELETE FROM performance_metrics WHERE recorded_at < NOW() - INTERVAL '90 days'
    -- The actual retention period should be configurable
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- Performance Metrics Aggregation Table
-- ============================================

-- Performance Metrics Aggregates (for efficient historical queries)
CREATE TABLE performance_metrics_aggregates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Dimensions (copied from request_events for efficient querying)
    company_id UUID REFERENCES companies(id) ON DELETE CASCADE,
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    ai_client_id UUID REFERENCES ai_clients(id) ON DELETE CASCADE,
    provider_id UUID REFERENCES ai_providers(id) ON DELETE CASCADE,
    model_id UUID REFERENCES ai_models(id) ON DELETE CASCADE,
    
    -- Time Period
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    period_type VARCHAR(20) NOT NULL, -- 'hour', 'day', 'week', 'month'
    
    -- AI Performance Aggregates
    avg_ttft_ms DECIMAL(10, 2),
    p50_ttft_ms DECIMAL(10, 2),
    p95_ttft_ms DECIMAL(10, 2),
    p99_ttft_ms DECIMAL(10, 2),
    avg_processing_time_ms DECIMAL(10, 2),
    avg_prefill_speed DECIMAL(10, 2),
    avg_decode_speed DECIMAL(10, 2),
    total_prompt_tokens BIGINT,
    total_output_tokens BIGINT,
    avg_token_efficiency DECIMAL(5, 4),
    
    -- System Resource Aggregates
    avg_gpu_utilization DECIMAL(5, 2),
    max_gpu_utilization DECIMAL(5, 2),
    avg_cpu_utilization DECIMAL(5, 2),
    max_cpu_utilization DECIMAL(5, 2),
    avg_gpu_memory_mb BIGINT,
    max_gpu_memory_mb BIGINT,
    avg_cpu_memory_mb BIGINT,
    max_cpu_memory_mb BIGINT,
    
    -- Hardware Monitoring Aggregates
    avg_gpu_temperature DECIMAL(5, 2),
    max_gpu_temperature DECIMAL(5, 2),
    avg_cpu_temperature DECIMAL(5, 2),
    max_cpu_temperature DECIMAL(5, 2),
    avg_power_consumption DECIMAL(10, 2),
    max_power_consumption DECIMAL(10, 2),
    
    -- Sample counts
    sample_count INTEGER DEFAULT 0,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for aggregate queries
CREATE INDEX idx_perf_aggregates_period ON performance_metrics_aggregates(period_start, period_end);
CREATE INDEX idx_perf_aggregates_type ON performance_metrics_aggregates(period_type);
CREATE INDEX idx_perf_aggregates_company ON performance_metrics_aggregates(company_id);
CREATE INDEX idx_perf_aggregates_model ON performance_metrics_aggregates(model_id);
CREATE INDEX idx_perf_aggregates_provider ON performance_metrics_aggregates(provider_id);

-- Composite indexes for common aggregate query patterns
CREATE INDEX idx_perf_aggregates_company_period ON performance_metrics_aggregates(company_id, period_start, period_end);
CREATE INDEX idx_perf_aggregates_model_period ON performance_metrics_aggregates(model_id, period_start, period_end);

-- ============================================
-- Comments for Documentation
-- ============================================

COMMENT ON TABLE performance_metrics IS 'Stores high-frequency performance metrics for AI requests including AI performance, system resources, and hardware monitoring data';
COMMENT ON TABLE performance_metrics_aggregates IS 'Stores aggregated performance metrics for efficient historical analysis and dashboard queries';
COMMENT ON COLUMN performance_metrics.ttft_ms IS 'Time to First Token - latency from request start to first token generation';
COMMENT ON COLUMN performance_metrics.token_efficiency IS 'Calculated as output_tokens / (prompt_tokens + output_tokens)';
COMMENT ON COLUMN performance_metrics_aggregates.period_type IS 'Time aggregation period: hour, day, week, or month';
