# Emitter Mode Deployment Guide

## Overview

Emitter mode is a high-scale deployment mode for the AI Analytics Proxy that separates data collection from local processing. Instead of writing telemetry events directly to a local database, the proxy serializes and emits events to external collectors and analytics services via message queues or HTTP endpoints.

## When to Use Emitter Mode

Emitter mode is designed for:

- **Enterprise deployments** with high-volume telemetry (>10,000 events/second)
- **Multi-tenant architectures** requiring centralized analytics
- **Cloud-native environments** with separate collector services
- **Commercial deployments** needing horizontal scalability
- **Distributed systems** requiring event streaming across services

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
│                    │  Serializer     │                            │
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

## Prerequisites

### Required Components

1. **AI Analytics Proxy** (v0.1.0 or later)
2. **Message Queue** (Redis 6.0+ or Kafka 2.8+)
3. **External Collector Service** (HTTP endpoint)
4. **Monitoring Stack** (Prometheus + Grafana recommended)

### System Requirements

- **CPU**: 2+ cores recommended
- **Memory**: 4GB+ RAM
- **Disk**: 10GB+ for fallback buffer
- **Network**: Low latency to collector and queue

## Installation

### 1. Install AI Analytics Proxy

```bash
# Download the latest release
curl -L https://github.com/your-org/ai-dashboard/releases/latest/download/ai-analytics-proxy-linux-amd64 -o ai-analytics-proxy
chmod +x ai-analytics-proxy

# Or build from source
cd ai-dashboard/apps/proxy
cargo build --release
```

### 2. Install Redis (for message queue)

```bash
# Ubuntu/Debian
sudo apt-get install redis-server

# macOS
brew install redis

# Docker
docker run -d -p 6379:6379 redis:7-alpine
```

### 3. Configure Emitter Mode

Create or edit the configuration file:

```toml
[proxy]
mode = "emitter"

[emitter]
enabled = true

[emitter.collector]
url = "https://collector.example.com/api/v1/events"
api_key = "your-api-key-here"
timeout_secs = 30
enabled = true

[emitter.queue]
backend = "redis_stream"
redis_url = "redis://localhost:6379"
queue_name = "ai-analytics-events"
consumer_group = "analytics-consumers"
consumer_name = "proxy-emitter"
stream_max_len = 10000
enabled = true

[emitter.protocol]
format = "json"
enable_compression = false
compression_level = 6
enable_encryption = false

[emitter.fallback]
enabled = true
buffer_path = "/var/lib/ai-analytics-proxy/buffer"
max_buffer_size = "1GB"
enable_rotation = true
rotation_size_mb = 100

[emitter.performance]
batch_size = 100
batch_timeout_secs = 5
max_concurrent_requests = 10
max_retries = 3
retry_delay_secs = 1
max_retry_delay_secs = 60
```

### 4. Start the Proxy

```bash
# Start with emitter mode
./ai-analytics-proxy --config /path/to/config.toml

# Or use systemd service
sudo systemctl start ai-analytics-proxy
```

## Configuration Options

### Collector Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `url` | string | - | HTTP collector endpoint URL |
| `api_key` | string | - | API key for authentication |
| `timeout_secs` | integer | 30 | Request timeout in seconds |
| `enabled` | boolean | false | Enable HTTP collector |

### Queue Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `backend` | string | "redis_stream" | Queue backend: "redis_pubsub", "redis_stream", "kafka" |
| `redis_url` | string | - | Redis connection URL |
| `queue_name` | string | "ai-analytics-events" | Queue/stream name |
| `consumer_group` | string | "analytics-consumers" | Consumer group name |
| `consumer_name` | string | "proxy-emitter" | Consumer name |
| `stream_max_len` | integer | 10000 | Maximum stream length |
| `enabled` | boolean | false | Enable message queue |

### Protocol Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `format` | string | "json" | Serialization format: "json" or "messagepack" |
| `enable_compression` | boolean | false | Enable payload compression |
| `compression_level` | integer | 6 | Compression level (1-9) |
| `enable_encryption` | boolean | false | Enable payload encryption |

### Fallback Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | true | Enable fallback buffering |
| `buffer_path` | string | - | Buffer directory path |
| `max_buffer_size` | string | "1GB" | Maximum buffer size |
| `enable_rotation` | boolean | true | Enable file rotation |
| `rotation_size_mb` | integer | 100 | Rotation file size in MB |

### Performance Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `batch_size` | integer | 100 | Batch size for event emission |
| `batch_timeout_secs` | integer | 5 | Batch timeout in seconds |
| `max_concurrent_requests` | integer | 10 | Maximum concurrent requests |
| `max_retries` | integer | 3 | Maximum retry attempts |
| `retry_delay_secs` | integer | 1 | Initial retry delay in seconds |
| `max_retry_delay_secs` | integer | 60 | Maximum retry delay in seconds |

## Deployment Strategies

### Single Instance Deployment

For small to medium deployments:

```bash
# Start single instance
./ai-analytics-proxy --config emitter-config.toml
```

### Multi-Instance Deployment

For high availability and scalability:

```bash
# Start multiple instances behind a load balancer
./ai-analytics-proxy --config emitter-config.toml --port 3090 &
./ai-analytics-proxy --config emitter-config.toml --port 3091 &
./ai-analytics-proxy --config emitter-config.toml --port 3092 &
```

### Docker Deployment

```dockerfile
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY ai-analytics-proxy /usr/local/bin/

# Create buffer directory
RUN mkdir -p /var/lib/ai-analytics-proxy/buffer

# Expose metrics port
EXPOSE 9090

# Run proxy
CMD ["ai-analytics-proxy", "--config", "/etc/ai-analytics-proxy/config.toml"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ai-analytics-proxy
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ai-analytics-proxy
  template:
    metadata:
      labels:
        app: ai-analytics-proxy
    spec:
      containers:
      - name: proxy
        image: ai-analytics-proxy:latest
        ports:
        - containerPort: 3090
        - containerPort: 9090
        volumeMounts:
        - name: config
          mountPath: /etc/ai-analytics-proxy
        - name: buffer
          mountPath: /var/lib/ai-analytics-proxy/buffer
        env:
        - name: PROXY_MODE
          value: "emitter"
      volumes:
      - name: config
        configMap:
          name: proxy-config
      - name: buffer
        emptyDir: {}
```

## Monitoring

### Health Check Endpoint

```bash
curl http://localhost:3090/api/emitter/health
```

Response:
```json
{
  "healthy": true,
  "emitter_enabled": true,
  "circuit_breaker_state": "closed",
  "queue_connected": true,
  "fallback_active": false,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Status Endpoint

```bash
curl http://localhost:3090/api/emitter/status
```

Response:
```json
{
  "enabled": true,
  "metrics": {
    "total_events": 10000,
    "successful_emissions": 9950,
    "failed_emissions": 50,
    "emission_rate": 150.5,
    "avg_latency_ms": 25.3
  },
  "queue_stats": {
    "queue_depth": 100,
    "connected": true
  },
  "fallback_stats": {
    "current_size": 0,
    "event_count": 0,
    "active": false
  },
  "circuit_breaker_state": "closed",
  "success_rate": 0.995,
  "failure_rate": 0.005,
  "uptime_seconds": 3600
}
```

### Prometheus Metrics

```bash
curl http://localhost:3090/api/emitter/prometheus
```

Available metrics:
- `emitter_total_events` - Total events processed
- `emitter_successful_emissions` - Successful emissions
- `emitter_failed_emissions` - Failed emissions
- `emitter_emission_rate` - Current emission rate
- `emitter_avg_latency_ms` - Average latency
- `emitter_p95_latency_ms` - P95 latency
- `emitter_queue_depth` - Queue depth
- `emitter_fallback_size` - Fallback buffer size

## Troubleshooting

### Common Issues

#### 1. Connection Refused to Redis

**Problem**: Proxy cannot connect to Redis

**Solution**:
```bash
# Check Redis is running
redis-cli ping

# Check connection string
redis-cli -h localhost -p 6379 ping

# Verify firewall rules
sudo ufw allow 6379
```

#### 2. Circuit Breaker Open

**Problem**: Circuit breaker is open, rejecting requests

**Solution**:
```bash
# Check circuit breaker state
curl http://localhost:3090/api/emitter/status

# Reset circuit breaker (if safe)
curl -X POST http://localhost:3090/api/emitter/control \
  -H "Content-Type: application/json" \
  -d '{"action": "reset_circuit_breaker"}'
```

#### 3. Fallback Buffer Growing

**Problem**: Fallback buffer is growing continuously

**Solution**:
```bash
# Check buffer size
curl http://localhost:3090/api/emitter/status

# Manually flush buffer
curl -X POST http://localhost:3090/api/emitter/control \
  -H "Content-Type: application/json" \
  -d '{"action": "flush_buffer"}'

# Check collector connectivity
curl https://collector.example.com/health
```

#### 4. High Latency

**Problem**: Emission latency is high

**Solution**:
- Check network latency to collector
- Increase `max_concurrent_requests` in config
- Enable compression for large payloads
- Consider using Redis pub/sub instead of streams for lower latency

### Debug Mode

Enable debug logging:

```bash
./ai-analytics-proxy --config emitter-config.toml --log-level debug
```

## Performance Tuning

### Throughput Optimization

1. **Increase batch size**: Set `batch_size` to 500-1000 for high throughput
2. **Enable compression**: Reduce network bandwidth usage
3. **Use Redis pub/sub**: Lower latency than streams
4. **Increase concurrent requests**: Set `max_concurrent_requests` to 20-50

### Latency Optimization

1. **Reduce batch timeout**: Set `batch_timeout_secs` to 1-2 seconds
2. **Use Redis pub/sub**: Direct streaming vs durable logging
3. **Disable compression**: Trade bandwidth for lower CPU usage
4. **Optimize network**: Ensure low latency to collector

### Resource Optimization

1. **Limit buffer size**: Set `max_buffer_size` to control disk usage
2. **Enable rotation**: Prevent single large buffer files
3. **Monitor memory usage**: Set appropriate `max_concurrent_requests`
4. **Tune connection pooling**: Balance between performance and resource usage

## Security Considerations

### Authentication

- Use API keys for collector authentication
- Rotate API keys regularly
- Store API keys in environment variables or secret management
- Use TLS for all network communication

### Data Security

- Enable encryption for sensitive data
- Use secure Redis connections (TLS)
- Implement proper access controls on collector
- Audit log all configuration changes

### Network Security

- Use firewall rules to restrict access
- Implement network segmentation
- Use VPN or private networks for internal communication
- Monitor for unauthorized access attempts

## Migration from Analytics Mode

### Step 1: Deploy Emitter Mode

1. Deploy external collector service
2. Set up Redis/Kafka cluster
3. Configure emitter mode
4. Test with sample traffic

### Step 2: Gradual Migration

1. Run both modes in parallel
2. Compare analytics results
3. Gradually shift traffic to emitter mode
4. Monitor for discrepancies

### Step 3: Cutover

1. Switch all traffic to emitter mode
2. Validate analytics consistency
3. Decommission analytics mode
4. Clean up old infrastructure

## Backup and Recovery

### Backup Configuration

```bash
# Backup configuration
cp /etc/ai-analytics-proxy/config.toml /backup/config.toml.$(date +%Y%m%d)

# Backup buffer data
tar -czf /backup/buffer-$(date +%Y%m%d).tar.gz /var/lib/ai-analytics-proxy/buffer
```

### Recovery

```bash
# Restore configuration
cp /backup/config.toml.20240115 /etc/ai-analytics-proxy/config.toml

# Restore buffer data
tar -xzf /backup/buffer-20240115.tar.gz -C /

# Restart service
sudo systemctl restart ai-analytics-proxy
```

## Scaling Considerations

### Horizontal Scaling

- Deploy multiple proxy instances behind load balancer
- Use consistent queue configuration across instances
- Monitor queue depth and consumer lag
- Scale collector service proportionally

### Vertical Scaling

- Increase CPU for higher throughput
- Add memory for larger buffers
- Use faster storage for buffer I/O
- Optimize network bandwidth

## Best Practices

1. **Monitor continuously**: Set up alerts for key metrics
2. **Test failover**: Regularly test fallback and recovery
3. **Validate data**: Compare emitter and analytics mode results
4. **Plan capacity**: Size infrastructure for peak load
5. **Document changes**: Maintain configuration documentation
6. **Security first**: Implement proper authentication and encryption
7. **Performance testing**: Load test before production deployment
8. **Regular maintenance**: Update components and rotate credentials

## Support

For issues and questions:

- **Documentation**: https://docs.ai-dashboard.example.com
- **GitHub Issues**: https://github.com/your-org/ai-dashboard/issues
- **Community**: https://community.ai-dashboard.example.com
- **Enterprise Support**: support@ai-dashboard.example.com
