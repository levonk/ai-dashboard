# Performance Metrics Schema Documentation

## Overview

The performance metrics schema extends the AI Dashboard database to support comprehensive performance and resource utilization monitoring. This schema enables collection, storage, and analysis of 19 key performance metrics across AI model performance, system resources, and hardware monitoring.

## Schema Design Principles

1. **Time-Series Optimization**: High-frequency metrics stored in dedicated time-series table with efficient indexing
2. **Request Correlation**: All performance metrics linked to existing `request_events` for comprehensive analysis
3. **Aggregation Support**: Separate aggregate table for efficient historical queries and dashboard performance
4. **Data Retention**: Built-in support for configurable retention policies
5. **Scalability**: Schema designed to handle high-volume metric ingestion without impacting proxy performance

## Tables

### performance_metrics

Stores high-frequency performance metrics for individual AI requests.

#### Columns

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `request_event_id` | UUID | Foreign key to `request_events.id` (CASCADE DELETE) |
| `ttft_ms` | DECIMAL(10,2) | Time to First Token in milliseconds |
| `total_processing_time_ms` | DECIMAL(10,2) | Total request processing time in milliseconds |
| `prefill_token_speed` | DECIMAL(10,2) | Prefill token processing speed (tokens/second) |
| `decode_token_speed` | DECIMAL(10,2) | Decode token processing speed (tokens/second) |
| `prompt_token_count` | INTEGER | Prompt token count per request |
| `output_token_count` | INTEGER | Output token count per request |
| `context_length` | INTEGER | Total tokens in prompt (context length) |
| `token_efficiency` | DECIMAL(5,4) | Token efficiency (output tokens / total tokens) |
| `gpu_utilization_percent` | DECIMAL(5,2) | GPU utilization percentage |
| `cpu_utilization_percent` | DECIMAL(5,2) | CPU utilization percentage |
| `gpu_memory_usage_mb` | BIGINT | GPU memory usage in megabytes |
| `cpu_memory_usage_mb` | BIGINT | CPU memory usage in megabytes |
| `unified_memory_usage_mb` | BIGINT | Unified memory usage in megabytes |
| `gpu_temperature_celsius` | DECIMAL(5,2) | GPU temperature in Celsius |
| `cpu_temperature_celsius` | DECIMAL(5,2) | CPU temperature in Celsius |
| `power_consumption_watts` | DECIMAL(10,2) | Power consumption in Watts |
| `software_clock_speed_mhz` | DECIMAL(10,2) | Software clock speed in MHz |
| `hardware_clock_speed_mhz` | DECIMAL(10,2) | Hardware clock speed in MHz |
| `sample_rate_hz` | DECIMAL(10,2) | Sample rate in Hz |
| `metadata` | JSONB | Additional metadata as JSON |
| `recorded_at` | TIMESTAMP WITH TIME ZONE | When the metrics were recorded |
| `created_at` | TIMESTAMP WITH TIME ZONE | When the record was created |

#### Indexes

- `idx_performance_metrics_request_event`: Request correlation queries
- `idx_performance_metrics_recorded_at`: Time-series queries (DESC)
- `idx_performance_metrics_request_recorded`: Composite request + time queries
- `idx_performance_metrics_ttft`: TTFT analysis queries
- `idx_performance_metrics_processing_time`: Processing time analysis
- `idx_performance_metrics_gpu_util`: GPU utilization queries
- `idx_performance_metrics_cpu_util`: CPU utilization queries
- `idx_performance_metrics_gpu_temp`: GPU temperature monitoring
- `idx_performance_metrics_power`: Power consumption analysis

### performance_metrics_aggregates

Stores aggregated performance metrics for efficient historical analysis and dashboard queries.

#### Columns

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `company_id` | UUID | Foreign key to `companies.id` (CASCADE DELETE) |
| `team_id` | UUID | Foreign key to `teams.id` (CASCADE DELETE) |
| `ai_client_id` | UUID | Foreign key to `ai_clients.id` (CASCADE DELETE) |
| `provider_id` | UUID | Foreign key to `ai_providers.id` (CASCADE DELETE) |
| `model_id` | UUID | Foreign key to `ai_models.id` (CASCADE DELETE) |
| `period_start` | TIMESTAMP WITH TIME ZONE | Start of aggregation period |
| `period_end` | TIMESTAMP WITH TIME ZONE | End of aggregation period |
| `period_type` | VARCHAR(20) | Aggregation period: 'hour', 'day', 'week', 'month' |
| `avg_ttft_ms` | DECIMAL(10,2) | Average TTFT |
| `p50_ttft_ms` | DECIMAL(10,2) | 50th percentile TTFT |
| `p95_ttft_ms` | DECIMAL(10,2) | 95th percentile TTFT |
| `p99_ttft_ms` | DECIMAL(10,2) | 99th percentile TTFT |
| `avg_processing_time_ms` | DECIMAL(10,2) | Average processing time |
| `avg_prefill_speed` | DECIMAL(10,2) | Average prefill speed |
| `avg_decode_speed` | DECIMAL(10,2) | Average decode speed |
| `total_prompt_tokens` | BIGINT | Total prompt tokens in period |
| `total_output_tokens` | BIGINT | Total output tokens in period |
| `avg_token_efficiency` | DECIMAL(5,4) | Average token efficiency |
| `avg_gpu_utilization` | DECIMAL(5,2) | Average GPU utilization |
| `max_gpu_utilization` | DECIMAL(5,2) | Maximum GPU utilization |
| `avg_cpu_utilization` | DECIMAL(5,2) | Average CPU utilization |
| `max_cpu_utilization` | DECIMAL(5,2) | Maximum CPU utilization |
| `avg_gpu_memory_mb` | BIGINT | Average GPU memory usage |
| `max_gpu_memory_mb` | BIGINT | Maximum GPU memory usage |
| `avg_cpu_memory_mb` | BIGINT | Average CPU memory usage |
| `max_cpu_memory_mb` | BIGINT | Maximum CPU memory usage |
| `avg_gpu_temperature` | DECIMAL(5,2) | Average GPU temperature |
| `max_gpu_temperature` | DECIMAL(5,2) | Maximum GPU temperature |
| `avg_cpu_temperature` | DECIMAL(5,2) | Average CPU temperature |
| `max_cpu_temperature` | DECIMAL(5,2) | Maximum CPU temperature |
| `avg_power_consumption` | DECIMAL(10,2) | Average power consumption |
| `max_power_consumption` | DECIMAL(10,2) | Maximum power consumption |
| `sample_count` | INTEGER | Number of samples in aggregation |
| `metadata` | JSONB | Additional metadata as JSON |
| `created_at` | TIMESTAMP WITH TIME ZONE | When the aggregate was created |
| `updated_at` | TIMESTAMP WITH TIME ZONE | When the aggregate was last updated |

#### Indexes

- `idx_perf_aggregates_period`: Time range queries
- `idx_perf_aggregates_type`: Period type filtering
- `idx_perf_aggregates_company`: Company-level analysis
- `idx_perf_aggregates_model`: Model-level analysis
- `idx_perf_aggregates_provider`: Provider-level analysis
- `idx_perf_aggregates_company_period`: Composite company + time queries
- `idx_perf_aggregates_model_period`: Composite model + time queries

## Relationships

### Foreign Key Relationships

```
performance_metrics.request_event_id → request_events.id (CASCADE DELETE)
performance_metrics_aggregates.company_id → companies.id (CASCADE DELETE)
performance_metrics_aggregates.team_id → teams.id (CASCADE DELETE)
performance_metrics_aggregates.ai_client_id → ai_clients.id (CASCADE DELETE)
performance_metrics_aggregates.provider_id → ai_providers.id (CASCADE DELETE)
performance_metrics_aggregates.model_id → ai_models.id (CASCADE DELETE)
```

### Data Flow

1. **Request Processing**: AI requests are logged in `request_events`
2. **Metrics Collection**: Performance metrics are collected and stored in `performance_metrics` linked to the request event
3. **Aggregation**: Scheduled jobs aggregate raw metrics into `performance_metrics_aggregates` for efficient querying
4. **Dashboard Queries**: Web dashboard queries aggregate tables for historical analysis and raw metrics table for detailed inspection

## Metric Categories

### AI Performance Metrics (8 metrics)

- **TTFT (Time to First Token)**: Latency from request start to first token generation
- **Total Processing Time**: Complete request processing duration
- **Prefill Token Speed**: Token processing speed during prefill phase
- **Decode Token Speed**: Token processing speed during decode phase
- **Prompt Token Count**: Number of tokens in the input prompt
- **Output Token Count**: Number of tokens in the model output
- **Context Length**: Total tokens in prompt (same as prompt token count)
- **Token Efficiency**: Calculated as output_tokens / (prompt_tokens + output_tokens)

### System Resource Metrics (5 metrics)

- **GPU Utilization**: GPU utilization percentage
- **CPU Utilization**: CPU utilization percentage
- **GPU Memory Usage**: GPU memory consumption in megabytes
- **CPU Memory Usage**: CPU memory consumption in megabytes
- **Unified Memory Usage**: Unified memory consumption (where applicable)

### Hardware Monitoring Metrics (6 metrics)

- **GPU Temperature**: GPU temperature in Celsius
- **CPU Temperature**: CPU temperature in Celsius
- **Power Consumption**: Power consumption in Watts
- **Software Clock Speed**: Software-configured clock speed in MHz
- **Hardware Clock Speed**: Actual hardware clock speed in MHz
- **Sample Rate**: Monitoring sample rate in Hz

## Data Retention Policy

The schema includes support for configurable data retention policies:

### Retention Function

A PostgreSQL function `performance_metrics_retention_policy()` is provided for implementing retention policies. This function can be called by scheduled jobs to:

- Delete raw metrics older than a configured period (e.g., 90 days)
- Archive aggregated metrics for longer periods (e.g., 1 year)
- Implement tiered retention based on metric importance

### Recommended Retention Periods

- **Raw performance_metrics**: 90 days (high-frequency data)
- **Hourly aggregates**: 6 months
- **Daily aggregates**: 1 year
- **Weekly/monthly aggregates**: Indefinite

## Query Patterns

### Common Query Patterns

1. **Request Correlation**: Get performance metrics for a specific request
   ```sql
   SELECT * FROM performance_metrics 
   WHERE request_event_id = ? 
   ORDER BY recorded_at;
   ```

2. **Time Range Analysis**: Get metrics for a time period
   ```sql
   SELECT * FROM performance_metrics 
   WHERE recorded_at BETWEEN ? AND ? 
   ORDER BY recorded_at DESC;
   ```

3. **Model Performance**: Analyze performance by model
   ```sql
   SELECT pm.*, re.model_id 
   FROM performance_metrics pm
   JOIN request_events re ON pm.request_event_id = re.id
   WHERE re.model_id = ? 
   AND pm.recorded_at BETWEEN ? AND ?;
   ```

4. **Aggregated Historical Data**: Get aggregated metrics for dashboard
   ```sql
   SELECT * FROM performance_metrics_aggregates
   WHERE model_id = ? 
   AND period_start BETWEEN ? AND ?
   AND period_type = 'day'
   ORDER BY period_start;
   ```

5. **Performance Percentiles**: Analyze TTFT distribution
   ```sql
   SELECT 
     percentile_cont(0.5) WITHIN GROUP (ORDER BY ttft_ms) as p50,
     percentile_cont(0.95) WITHIN GROUP (ORDER BY ttft_ms) as p95,
     percentile_cont(0.99) WITHIN GROUP (ORDER BY ttft_ms) as p99
   FROM performance_metrics
   WHERE recorded_at BETWEEN ? AND ?;
   ```

## Performance Considerations

### Write Performance

- **Target**: <5ms overhead for metrics collection
- **Strategy**: Batch inserts for high-frequency metrics, async writes where possible
- **Monitoring**: Track write latency and connection pool utilization

### Read Performance

- **Target**: <2s response time for dashboard queries
- **Strategy**: Use aggregate tables for historical queries, proper indexing for raw metrics
- **Monitoring**: Track query performance and optimize slow queries

### Storage Optimization

- **Compression**: Consider table compression for historical data
- **Partitioning**: Consider time-based partitioning for `performance_metrics` table
- **Vacuum**: Regular vacuum operations to reclaim space

## Migration Notes

### Migration File

- **File**: `migrations/001_performance_metrics.sql`
- **Dependencies**: Requires `000_initial_schema.sql` to be applied first
- **Rollback**: Manual rollback script should be created for production deployments

### Breaking Changes

None - this migration only adds new tables and does not modify existing schema.

### Data Migration

No data migration required as this is a new feature addition.

## Security Considerations

### Data Privacy

- Hardware metrics may contain sensitive system information
- Implement appropriate access controls on performance metrics endpoints
- Consider data anonymization for multi-tenant deployments

### Access Control

- Performance metrics should respect existing company/team access controls
- Aggregated data should follow the same dimension-based access rules
- Hardware metrics may need additional restrictions for security-sensitive environments

## Future Enhancements

### Potential Future Additions

1. **Real-time Streaming**: Integration with time-series databases for real-time metrics
2. **Custom Metrics**: Support for user-defined custom performance metrics
3. **Alerting Integration**: Database-level triggers for threshold-based alerting
4. **Machine Learning**: Storage of ML model performance metrics and predictions
5. **Cost Attribution**: Extended cost metrics based on performance data

### Schema Evolution

The schema is designed to be extensible:
- New metrics can be added via ALTER TABLE
- JSONB metadata fields provide flexibility for ad-hoc data
- Aggregate table can be extended with new aggregation types
