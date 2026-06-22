# AI Analytics Dashboard API Guide

## Overview

The AI Analytics Dashboard API provides comprehensive REST endpoints for AI usage analytics across multiple dimensions: company clients, AI clients, teams, pipeline stages, AI model suppliers, models, and input types.

## Base URL

- **Development**: `http://localhost:3000`
- **Production**: `https://api.ai-dashboard.example.com`

## Authentication

The API supports two authentication methods:

### API Key Authentication

Include your API key in the `x-api-key` header:

```bash
curl -H "x-api-key: your-api-key" https://api.ai-dashboard.example.com/api/analytics/query
```

### JWT Bearer Token Authentication

Include your JWT token in the `Authorization` header:

```bash
curl -H "Authorization: Bearer your-jwt-token" https://api.ai-dashboard.example.com/api/analytics/query
```

## Rate Limiting

API endpoints are rate-limited to prevent abuse. Rate limit information is returned in response headers:

- `X-RateLimit-Limit`: Maximum requests per window
- `X-RateLimit-Remaining`: Remaining requests in current window
- `X-RateLimit-Reset`: Timestamp when the window resets
- `Retry-After`: Seconds to wait before retrying (on 429 responses)

## Endpoints

### Analytics Query

#### Execute Query

```http
POST /api/analytics/query
```

Execute an analytics query with filtering and aggregation.

**Request Body:**
```json
{
  "filters": {
    "provider": "anthropic",
    "model": "claude-3-opus"
  },
  "aggregations": [
    {
      "field": "cost",
      "function": "sum"
    }
  ],
  "timeRange": {
    "startTime": "2024-01-01T00:00:00Z",
    "endTime": "2024-01-31T23:59:59Z"
  },
  "pagination": {
    "page": 1,
    "pageSize": 100
  }
}
```

**Response:**
```json
{
  "queryId": "query-123",
  "data": [...],
  "metadata": {
    "executionTimeMs": 150,
    "recordCount": 1000,
    "cached": false
  }
}
```

#### Health Check

```http
GET /api/analytics/query/health
```

Check the health of the analytics query service.

#### Statistics

```http
GET /api/analytics/query/stats
```

Get analytics query system statistics.

### Cost Analysis

#### Cost Breakdown

```http
POST /api/cost/breakdown
```

Analyze cost breakdown by dimensions.

**Request Body:**
```json
{
  "startTime": "2024-01-01T00:00:00Z",
  "endTime": "2024-01-31T23:59:59Z",
  "dimensions": ["provider", "model"]
}
```

#### Cost Trends

```http
POST /api/cost/trends
```

Analyze cost trends over time.

**Request Body:**
```json
{
  "startTime": "2024-01-01T00:00:00Z",
  "endTime": "2024-01-31T23:59:59Z",
  "granularity": "day",
  "includeForecast": true,
  "forecastPeriod": 7
}
```

#### Cost Forecast

```http
POST /api/cost/forecast
```

Generate cost forecast based on historical data.

#### Cost Optimization

```http
POST /api/cost/optimization
```

Get cost optimization recommendations.

#### Cost Comparison

```http
POST /api/cost/comparison
```

Compare costs across dimensions.

### Filtering

#### Get Filter Dimensions

```http
GET /api/filters/dimensions
```

Get available filter dimensions and their values.

**Response:**
```json
{
  "dimensions": [
    {
      "name": "provider",
      "type": "string",
      "description": "AI model provider",
      "values": ["anthropic", "openai", "google"]
    }
  ]
}
```

#### Validate Filter

```http
POST /api/filters/validate
```

Validate filter configuration.

#### List Saved Filters

```http
GET /api/filters/saved
```

List saved filter configurations.

#### Save Filter

```http
POST /api/filters/saved
```

Save a filter configuration.

**Request Body:**
```json
{
  "name": "My Custom Filter",
  "config": {
    "provider": "anthropic",
    "model": "claude-3-opus"
  }
}
```

#### Delete Saved Filter

```http
DELETE /api/filters/saved/{id}
```

Delete a saved filter configuration.

### Aggregation

#### Get Aggregation Functions

```http
GET /api/aggregation/functions
```

Get available aggregation functions.

**Response:**
```json
{
  "functions": [
    {
      "name": "sum",
      "description": "Sum of values",
      "applicableTypes": ["numeric"]
    }
  ]
}
```

#### Execute Aggregation

```http
POST /api/aggregation/execute
```

Execute aggregation query.

#### Aggregation Trends

```http
POST /api/aggregation/trends
```

Get aggregated trend analysis.

### Export

#### Execute Export

```http
POST /api/export/execute
```

Execute data export.

**Request Body:**
```json
{
  "format": "csv",
  "filters": {
    "provider": "anthropic"
  },
  "fields": ["timestamp", "cost", "tokens"]
}
```

**Response:**
```json
{
  "exportId": "export-123",
  "status": "processing",
  "format": "csv",
  "createdAt": "2024-01-15T10:00:00Z",
  "estimatedCompletion": "2024-01-15T10:00:30Z"
}
```

#### Get Export Formats

```http
GET /api/export/formats
```

Get available export formats.

**Response:**
```json
{
  "formats": [
    {
      "name": "json",
      "description": "JSON format",
      "mimeType": "application/json",
      "extension": ".json"
    },
    {
      "name": "csv",
      "description": "CSV format",
      "mimeType": "text/csv",
      "extension": ".csv"
    },
    {
      "name": "pdf",
      "description": "PDF report",
      "mimeType": "application/pdf",
      "extension": ".pdf"
    }
  ]
}
```

#### Get Export Status

```http
GET /api/export/status/{id}
```

Get export job status.

**Response:**
```json
{
  "exportId": "export-123",
  "status": "completed",
  "progress": 100,
  "completedAt": "2024-01-15T10:00:30Z",
  "downloadUrl": "/api/export/download/export-123"
}
```

#### Download Export

```http
GET /api/export/download/{id}
```

Download exported data.

### Webhooks

#### List Webhooks

```http
GET /api/webhooks
```

List configured webhooks.

#### Create Webhook

```http
POST /api/webhooks
```

Create a new webhook.

**Request Body:**
```json
{
  "url": "https://example.com/webhook",
  "events": ["cost.alert", "usage.threshold"],
  "headers": {
    "X-Custom-Header": "value"
  },
  "secret": "webhook-secret"
}
```

**Response:**
```json
{
  "id": "webhook-123",
  "url": "https://example.com/webhook",
  "events": ["cost.alert", "usage.threshold"],
  "headers": {
    "X-Custom-Header": "value"
  },
  "secret": "webhook-secret",
  "enabled": true,
  "createdAt": "2024-01-15T10:00:00Z"
}
```

#### Get Webhook

```http
GET /api/webhooks/{id}
```

Get webhook details.

#### Update Webhook

```http
PUT /api/webhooks/{id}
```

Update webhook configuration.

#### Delete Webhook

```http
DELETE /api/webhooks/{id}
```

Delete a webhook.

#### Test Webhook

```http
POST /api/webhooks/{id}/test
```

Test webhook delivery.

**Response:**
```json
{
  "webhookId": "webhook-123",
  "testResult": {
    "success": true,
    "statusCode": 200,
    "responseTimeMs": 150,
    "timestamp": "2024-01-15T10:00:00Z"
  }
}
```

#### Disable Webhook

```http
POST /api/webhooks/{id}/disable
```

Disable a webhook.

#### Enable Webhook

```http
POST /api/webhooks/{id}/enable
```

Enable a webhook.

### Metadata

#### API Metadata

```http
GET /api
```

Get API metadata and available endpoints.

#### API Health Check

```http
GET /api/health
```

Overall API health check.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:00:00Z",
  "services": {
    "analytics": true,
    "cost": true
  }
}
```

## Error Handling

The API uses standard HTTP status codes and returns error details in the response body:

**Error Response Format:**
```json
{
  "error": "Error code",
  "message": "Human-readable error message",
  "code": "ERROR_CODE",
  "timestamp": "2024-01-15T10:00:00Z",
  "details": {}
}
```

**Common Error Codes:**

- `AUTH_REQUIRED` (401): Authentication required
- `FORBIDDEN` (403): Insufficient permissions
- `NOT_FOUND` (404): Resource not found
- `VALIDATION_ERROR` (400): Invalid request data
- `RATE_LIMIT_EXCEEDED` (429): Too many requests
- `INTERNAL_ERROR` (500): Internal server error

## Pagination

Large result sets support pagination via the `pagination` parameter:

```json
{
  "pagination": {
    "page": 1,
    "pageSize": 100
  }
}
```

## Caching

The API implements caching for expensive operations. Cached responses include a `cached` flag in the metadata:

```json
{
  "metadata": {
    "cached": true,
    "executionTimeMs": 5
  }
}
```

## Webhook Events

The following events can trigger webhooks:

- `cost.alert`: Cost threshold exceeded
- `usage.threshold`: Usage threshold exceeded
- `export.completed`: Export job completed
- `export.failed`: Export job failed

## SDK and Client Libraries

Official SDKs are available for:

- JavaScript/TypeScript
- Python
- Go

See the SDK documentation for installation and usage instructions.

## Support

For API support, contact:
- Email: support@ai-dashboard.example.com
- Documentation: https://docs.ai-dashboard.example.com
- OpenAPI Spec: https://api.ai-dashboard.example.com/docs/api-spec.yaml
