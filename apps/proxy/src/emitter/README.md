# Emitter Mode Architecture

## Overview

Emitter mode is a high-scale deployment mode for the AI Analytics Proxy that separates data collection from local processing. Instead of writing telemetry events directly to a local database, the proxy serializes and emits events to external collectors and analytics services via message queues or HTTP endpoints.

This mode is designed for:
- **Enterprise deployments** with high-volume telemetry
- **Multi-tenant architectures** requiring centralized analytics
- **Cloud-native environments** with separate collector services
- **Commercial deployments** needing horizontal scalability

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         AI Analytics Proxy                        │
│                                                                    │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Request    │    │   Request    │    │   Request    │       │
│  │   Handler    │    │   Handler    │    │   Handler    │       │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘       │
│         │                   │                   │                 │
│         └───────────────────┼───────────────────┘                 │
│                             │                                     │
│                    ┌────────▼────────┐                            │
│                    │  Event          │                            │
│                    │  Collector      │                            │
│                    │  (Telemetry)    │                            │
│                    └────────┬────────┘                            │
│                             │                                     │
│                    ┌────────▼────────┐                            │
│                    │  Event          │                            │
│                    │  Serializer     │                            │
│                    │  (Protocol)     │                            │
│                    └────────┬────────┘                            │
│                             │                                     │
│                    ┌────────▼────────┐                            │
│                    │  Emitter        │                            │
│                    │  Client         │                            │
│                    └────────┬────────┘                            │
│                             │                                     │
│         ┌───────────────────┼───────────────────┐               │
│         │                   │                   │                 │
│  ┌──────▼──────┐    ┌──────▼──────┐    ┌──────▼──────┐          │
│  │  Message    │    │  HTTP       │    │  Fallback   │          │
│  │  Queue      │    │  Collector  │    │  Buffer     │          │
│  │  (Redis)    │    │  (Direct)   │    │  (Local)    │          │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘          │
│         │                   │                   │                 │
│         └───────────────────┼───────────────────┘                 │
│                             │                                     │
│                    ┌────────▼────────┐                            │
│                    │  External       │                            │
│                    │  Collector      │                            │
│                    │  Service        │                            │
│                    └─────────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Event Serializer (`protocol.rs`)
- **Purpose**: Convert telemetry events to wire format
- **Responsibilities**:
  - Serialize `TelemetryEvent` to JSON/MessagePack
  - Add protocol headers and metadata
  - Handle compression for large payloads
  - Version compatibility for protocol evolution
- **Formats**: JSON (default), MessagePack (optional)

### 2. Emitter Client (`client.rs`)
- **Purpose**: Communicate with external collectors
- **Responsibilities**:
  - HTTP client for direct collector communication
  - Connection pooling and keep-alive
  - Authentication and security headers
  - Retry logic with exponential backoff
  - Circuit breaker for failing endpoints
- **Protocols**: HTTP/HTTPS, WebSocket (future)

### 3. Message Queue Integration (`queue.rs`)
- **Purpose**: Buffer events for scalable processing
- **Responsibilities**:
  - Redis pub/sub for real-time streaming
  - Redis streams for durable event logs
  - Kafka support (future implementation)
  - Queue depth monitoring and backpressure
  - Consumer group management
- **Backends**: Redis (current), Kafka (planned)

### 4. Fallback Handler (`fallback.rs`)
- **Purpose**: Prevent data loss during failures
- **Responsibilities**:
  - Local disk buffering when queue/collector unavailable
  - Automatic retry with exponential backoff
  - Disk space management and rotation
  - Recovery and replay on service restoration
- **Storage**: Local filesystem with rotation

### 5. Configuration (`config/emitter.rs`)
- **Purpose**: Manage emitter mode settings
- **Responsibilities**:
  - Parse and validate emitter configuration
  - Manage collector URLs and credentials
  - Configure queue connection parameters
  - Tune performance parameters (batch size, timeouts)
  - Feature flags for optional capabilities

### 6. Metrics and Monitoring (`metrics.rs`)
- **Purpose**: Track emitter mode health and performance
- **Responsibilities**:
  - Emit Prometheus-style metrics
  - Track emission success/failure rates
  - Monitor queue depth and consumer lag
  - Track buffer utilization and disk usage
  - Performance metrics (latency, throughput)

### 7. API Endpoints (`api/emitter.rs`)
- **Purpose**: Expose emitter mode status and control
- **Responsibilities**:
  - Health check endpoint
  - Status endpoint (queue depth, buffer status)
  - Metrics endpoint for scraping
  - Control endpoints (pause/resume, flush buffer)
  - Configuration validation endpoint

### 8. CLI Integration (`cli/emitter.rs`)
- **Purpose**: Command-line control for emitter mode
- **Responsibilities**:
  - Emitter mode status command
  - Buffer inspection and management
  - Manual flush and recovery commands
  - Configuration validation
  - Diagnostic and troubleshooting tools

## Data Flow

### Normal Operation Flow

1. **Request Processing**: AI requests are processed by request handlers
2. **Event Collection**: Telemetry events are collected by the event collector
3. **Serialization**: Events are serialized to wire format by the protocol handler
4. **Queue Publication**: Serialized events are published to the message queue
5. **External Processing**: External collector service consumes events from queue
6. **Analytics Processing**: Collector service processes events for analytics

### Fallback Flow

1. **Queue Unavailable**: Message queue or collector becomes unavailable
2. **Buffer Activation**: Events are written to local fallback buffer
3. **Retry Logic**: Emitter client attempts to reconnect with backoff
4. **Recovery**: When queue/collector recovers, buffered events are replayed
5. **Normal Operation**: System returns to normal queue-based operation

## Configuration

### Required Configuration

```toml
[proxy]
mode = "emitter"

[emitter]
# Primary collector endpoint (HTTP)
collector_url = "https://collector.example.com/api/v1/events"

# Message queue (Redis)
message_queue_url = "redis://localhost:6379"
queue_name = "ai-analytics-events"

# Authentication
collector_api_key = "your-api-key"
```

### Optional Configuration

```toml
[emitter]
# Performance tuning
batch_size = 100
batch_timeout = "5s"
max_buffer_size = "1GB"
buffer_path = "/var/lib/ai-analytics-proxy/buffer"

# Retry configuration
max_retries = 5
retry_delay = "1s"
max_retry_delay = "60s"

# Queue configuration
queue_stream_max_len = 10000
consumer_group = "analytics-consumers"

# Protocol settings
serialization_format = "json"  # or "messagepack"
enable_compression = true
compression_level = 6

# Feature flags
enable_direct_http = true
enable_queue_fallback = true
enable_metrics = true
```

## Integration Points

### With Existing Analytics Mode

- **Shared Telemetry Event Structure**: Both modes use the same `TelemetryEvent` type
- **Configuration Toggle**: Mode selection via `proxy_mode` configuration
- **Backward Compatibility**: Existing analytics mode deployments unaffected
- **Migration Path**: Gradual migration from analytics to emitter mode

### With External Services

- **Collector Service**: HTTP endpoint for receiving events
- **Message Queue**: Redis/Kafka for scalable event streaming
- **Monitoring Stack**: Prometheus/Grafana for metrics
- **Log Aggregation**: Centralized logging for emitter mode logs

## Security Considerations

### Data in Transit
- **TLS Encryption**: All HTTP communication uses HTTPS
- **Authentication**: API keys or OAuth2 for collector access
- **Certificate Validation**: Strict certificate validation for HTTPS

### Data at Rest
- **Buffer Encryption**: Optional encryption for local fallback buffer
- **Secure Storage**: Proper permissions on buffer directory
- **Key Management**: Secure storage of API keys and credentials

### Access Control
- **Network Security**: Firewall rules for collector access
- **API Security**: Rate limiting and authentication on API endpoints
- **Audit Logging**: All configuration changes and mode switches logged

## Performance Considerations

### Throughput Optimization
- **Batching**: Events batched for efficient queue publishing
- **Async Processing**: Non-blocking event emission
- **Connection Pooling**: Reused HTTP connections
- **Compression**: Optional payload compression

### Latency Minimization
- **Direct HTTP Mode**: Option to bypass queue for low-latency needs
- **Local Buffering**: Minimal overhead for fallback mode
- **Protocol Efficiency**: Binary serialization option (MessagePack)

### Resource Management
- **Memory Limits**: Configurable buffer size limits
- **Disk Management**: Automatic buffer rotation and cleanup
- **Connection Limits**: Configurable connection pool sizes
- **Backpressure**: Queue depth monitoring to prevent overload

## Migration Path

### From Analytics Mode to Emitter Mode

1. **Configuration Update**: Change `proxy_mode` from "analytics" to "emitter"
2. **Collector Setup**: Deploy external collector service
3. **Queue Setup**: Deploy Redis/Kafka cluster
4. **Validation**: Use configuration validation endpoint
5. **Gradual Rollout**: Deploy emitter mode alongside analytics mode
6. **Cutover**: Switch traffic to emitter mode
7. **Decommission**: Remove analytics mode after validation

### Hybrid Deployment
- **Dual Mode**: Run both modes in parallel for validation
- **Selective Emission**: Emit specific event types to external collector
- **A/B Testing**: Compare analytics and emitter mode performance

## Monitoring and Observability

### Key Metrics
- **Emission Success Rate**: Percentage of successfully emitted events
- **Queue Depth**: Current number of events in queue
- **Buffer Utilization**: Disk space used by fallback buffer
- **Emission Latency**: Time from event creation to queue publication
- **Retry Rate**: Frequency of retry attempts
- **Connection Health**: Status of collector connections

### Health Checks
- **Liveness**: Emitter process is running
- **Readiness**: Collector and queue are accessible
- **Configuration**: Valid configuration for emitter mode
- **Buffer Status**: Fallback buffer health and capacity

### Alerting
- **High Failure Rate**: Emission failures above threshold
- **Queue Backlog**: Queue depth exceeds warning level
- **Buffer Full**: Fallback buffer approaching capacity
- **Collector Unavailable**: Extended collector unavailability

## Testing Strategy

### Unit Tests
- Event serialization/deserialization
- Protocol format validation
- Configuration parsing and validation
- Individual component logic

### Integration Tests
- Message queue integration (Redis)
- HTTP collector communication
- Fallback buffer activation
- End-to-end event flow

### Performance Tests
- Throughput under load
- Latency measurements
- Resource utilization
- Failure recovery performance

### Reliability Tests
- Collector failure scenarios
- Queue unavailability
- Network partition handling
- Buffer overflow conditions

## Future Enhancements

### Planned Features
- **Kafka Support**: Alternative message queue backend
- **WebSocket Emission**: Real-time event streaming
- **Event Filtering**: Client-side event filtering
- **Multi-Collector**: Support for multiple collector endpoints
- **Event Transformation**: Client-side event modification

### Evolution Path
- **Protocol Versioning**: Support for protocol evolution
- **Schema Registry**: Integration with schema registry
- **Event Enrichment**: Automatic event enrichment
- **Dynamic Configuration**: Runtime configuration updates
