-- Time-Series Storage Migration
-- This migration adds optimized time-series storage for high-frequency metrics
-- with partitioning, compression support, and aggregation tables

-- ============================================
-- Time-Series Metrics Table
-- ============================================

-- Time-Series Metrics (optimized for high-frequency writes and queries)
CREATE TABLE time_series_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Metric identification
    metric_name VARCHAR(255) NOT NULL,
    value DECIMAL(20, 6) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    
    -- Dimensions for flexible querying
    dimensions JSONB DEFAULT '{}',
    
    -- Optional link to request event
    request_event_id UUID REFERENCES request_events(id) ON DELETE SET NULL,
    
    -- Compression metadata
    compressed BOOLEAN DEFAULT FALSE,
    compression_algorithm VARCHAR(50),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for time-series queries
CREATE INDEX idx_timeseries_metric_name ON time_series_metrics(metric_name);
CREATE INDEX idx_timeseries_timestamp ON time_series_metrics(timestamp DESC);
CREATE INDEX idx_timeseries_metric_timestamp ON time_series_metrics(metric_name, timestamp DESC);

-- GIN index for dimension queries
CREATE INDEX idx_timeseries_dimensions ON time_series_metrics USING GIN(dimensions);

-- Partial index for request event correlation
CREATE INDEX idx_timeseries_request_event ON time_series_metrics(request_event_id) WHERE request_event_id IS NOT NULL;

-- ============================================
-- Time-Series Aggregates Table
-- ============================================

-- Time-Series Aggregates (for efficient historical queries)
CREATE TABLE time_series_aggregates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Metric identification
    metric_name VARCHAR(255) NOT NULL,
    
    -- Time window
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    window_end TIMESTAMP WITH TIME ZONE NOT NULL,
    
    -- Aggregation statistics
    avg_value DECIMAL(20, 6),
    min_value DECIMAL(20, 6),
    max_value DECIMAL(20, 6),
    sum_value DECIMAL(30, 6),
    count BIGINT,
    
    -- Percentiles (p50, p95, p99)
    p50_value DECIMAL(20, 6),
    p95_value DECIMAL(20, 6),
    p99_value DECIMAL(20, 6),
    
    -- Standard deviation
    std_dev DECIMAL(20, 6),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Unique constraint to prevent duplicate aggregates
CREATE UNIQUE INDEX idx_timeseries_aggregates_unique ON time_series_aggregates(metric_name, window_start);

-- Indexes for aggregate queries
CREATE INDEX idx_timeseries_aggregates_window ON time_series_aggregates(window_start, window_end);
CREATE INDEX idx_timeseries_aggregates_metric ON time_series_aggregates(metric_name);

-- ============================================
-- Time-Based Partitioning
-- ============================================

-- Create partitioning function for time-series metrics
CREATE OR REPLACE FUNCTION create_timeseries_partitions()
RETURNS VOID AS $$
DECLARE
    start_date TIMESTAMP WITH TIME ZONE;
    end_date TIMESTAMP WITH TIME ZONE;
    partition_name TEXT;
    i INTEGER;
BEGIN
    -- Create partitions for the next 12 months
    start_date := date_trunc('month', CURRENT_DATE);
    FOR i IN 0..11 LOOP
        end_date := start_date + INTERVAL '1 month';
        partition_name := 'time_series_metrics_' || to_char(start_date, 'YYYY_MM');
        
        -- Check if partition already exists
        IF NOT EXISTS (
            SELECT 1 FROM pg_class 
            WHERE relname = partition_name
        ) THEN
            EXECUTE format(
                'CREATE TABLE %I (LIKE time_series_metrics INCLUDING ALL)',
                partition_name
            );
            
            EXECUTE format(
                'ALTER TABLE %I INHERIT time_series_metrics',
                partition_name
            );
            
            -- Add constraint for partition
            EXECUTE format(
                'ALTER TABLE %I ADD CONSTRAINT %s CHECK (timestamp >= %L AND timestamp < %L)',
                partition_name,
                partition_name || '_check',
                start_date,
                end_date
            );
            
            -- Create indexes for partition
            EXECUTE format(
                'CREATE INDEX idx_%s_timestamp ON %I(timestamp DESC)',
                partition_name,
                partition_name
            );
            
            EXECUTE format(
                'CREATE INDEX idx_%s_metric_timestamp ON %I(metric_name, timestamp DESC)',
                partition_name,
                partition_name
            );
        END IF;
        
        start_date := end_date;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Create initial partitions
SELECT create_timeseries_partitions();

-- ============================================
-- Compression Support Functions
-- ============================================

-- Function to compress time-series data
CREATE OR REPLACE FUNCTION compress_timeseries_data()
RETURNS INTEGER AS $$
DECLARE
    compressed_count INTEGER;
BEGIN
    -- Compress data older than 7 days
    UPDATE time_series_metrics
    SET compressed = TRUE,
        compression_algorithm = 'delta'
    WHERE timestamp < NOW() - INTERVAL '7 days'
      AND compressed = FALSE;
    
    GET DIAGNOSTICS compressed_count = ROW_COUNT;
    RETURN compressed_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- Data Retention Functions
-- ============================================

-- Function to enforce retention policies
CREATE OR REPLACE FUNCTION enforce_retention_policy(metric_pattern TEXT, retention_days INTEGER)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM time_series_metrics
    WHERE metric_name LIKE metric_pattern
      AND timestamp < NOW() - (retention_days || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- Query Optimization Functions
-- ============================================

-- Function to aggregate time-series data
CREATE OR REPLACE FUNCTION aggregate_timeseries_data(
    p_metric_name TEXT,
    p_window_start TIMESTAMP WITH TIME ZONE,
    p_window_end TIMESTAMP WITH TIME ZONE
)
RETURNS VOID AS $$
BEGIN
    INSERT INTO time_series_aggregates (
        metric_name, window_start, window_end,
        avg_value, min_value, max_value, sum_value, count,
        p50_value, p95_value, p99_value, std_dev
    )
    SELECT 
        metric_name,
        p_window_start,
        p_window_end,
        AVG(value) as avg_value,
        MIN(value) as min_value,
        MAX(value) as max_value,
        SUM(value) as sum_value,
        COUNT(*) as count,
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY value) as p50_value,
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY value) as p95_value,
        PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY value) as p99_value,
        STDDEV(value) as std_dev
    FROM time_series_metrics
    WHERE metric_name = p_metric_name
      AND timestamp >= p_window_start
      AND timestamp < p_window_end
    GROUP BY metric_name
    ON CONFLICT (metric_name, window_start) DO UPDATE SET
        avg_value = EXCLUDED.avg_value,
        min_value = EXCLUDED.min_value,
        max_value = EXCLUDED.max_value,
        sum_value = EXCLUDED.sum_value,
        count = EXCLUDED.count,
        p50_value = EXCLUDED.p50_value,
        p95_value = EXCLUDED.p95_value,
        p99_value = EXCLUDED.p99_value,
        std_dev = EXCLUDED.std_dev,
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- Maintenance Jobs
-- ============================================

-- Create a scheduled job for partition maintenance
-- This would typically be managed by pg_cron or similar extension
-- Example: SELECT cron.schedule('create-timeseries-partitions', '0 0 1 * *', 'SELECT create_timeseries_partitions()');

-- ============================================
-- Comments for Documentation
-- ============================================

COMMENT ON TABLE time_series_metrics IS 'Optimized time-series storage for high-frequency metrics with partitioning and compression support';
COMMENT ON TABLE time_series_aggregates IS 'Aggregated time-series data for efficient historical queries and dashboard visualization';
COMMENT ON COLUMN time_series_metrics.dimensions IS 'JSONB field for flexible dimension filtering (e.g., labels, tags, hostnames)';
COMMENT ON COLUMN time_series_metrics.compressed IS 'Flag indicating whether the data point is compressed';
COMMENT ON COLUMN time_series_aggregates.p50_value IS '50th percentile (median) value in the time window';
COMMENT ON COLUMN time_series_aggregates.p95_value IS '95th percentile value in the time window';
COMMENT ON COLUMN time_series_aggregates.p99_value IS '99th percentile value in the time window';
