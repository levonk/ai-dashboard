# AI Agent API Guide

## Overview

The AI Dashboard provides optimized API endpoints specifically designed for AI agent consumption. These endpoints support multiple output formats to maximize token efficiency while maintaining data fidelity and human readability.

## Base URL

```
http://your-dashboard.com/api/ai-agents
```

## Authentication

All API endpoints require authentication via API key in the header:

```
Authorization: Bearer YOUR_API_KEY
```

## Available Endpoints

### 1. Query Endpoint

**POST** `/api/ai-agents/query`

Execute analytics queries with AI-optimized output formats.

#### Request Body

```json
{
  "timeRange": {
    "start": "2024-01-01T00:00:00Z",
    "end": "2024-01-31T23:59:59Z"
  },
  "dimensions": ["model", "provider", "environment"],
  "metrics": ["tokens", "cost", "latency"],
  "format": "markdown",
  "includeMetadata": true,
  "flattenStructure": false,
  "maxRecords": 1000
}
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| timeRange | object | Yes | Time range for the query |
| timeRange.start | string | Yes | ISO 8601 start timestamp |
| timeRange.end | string | Yes | ISO 8601 end timestamp |
| dimensions | array[string] | No | Dimensions to group by |
| metrics | array[string] | No | Metrics to calculate |
| format | string | No | Output format (markdown, json, toon, json-single) |
| includeMetadata | boolean | No | Include metadata in output |
| flattenStructure | boolean | No | Flatten nested structure |
| maxRecords | number | No | Maximum records to return |

#### Response

The response format varies based on the requested `format` parameter:

**Markdown Format** (`format: "markdown"`)
```
Content-Type: text/markdown

# Analytics Data Summary

## Summary Statistics

- **Total Records:** 150
- **Time Range:** 2024-01-01T00:00:00Z to 2024-01-31T23:59:59Z

## Metric Summaries

### tokens
- **Average:** 1250.50
- **Min:** 100
- **Max:** 5000
- **Total:** 187575.00

### cost
- **Average:** 0.02
- **Min:** 0.001
- **Max:** 0.10
- **Total:** 3.15

## Sample Data (First 10 Records)

| timestamp | dimensions.model | dimensions.provider | metrics.tokens | metrics.cost |
|-----------|------------------|---------------------|---------------|-------------|
| 2024-01-01T00:00:00Z | gpt-4 | openai | 1500 | 0.03 |
| 2024-01-01T01:00:00Z | gpt-4 | openai | 1200 | 0.024 |
...
```

**JSON Format** (`format: "json"`)
```json
{
  "metadata": {
    "exportedAt": "2024-01-15T10:30:00Z",
    "recordCount": 150,
    "queryId": "query_abc123",
    "executionTimeMs": 125
  },
  "data": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "dimensions": {
        "model": "gpt-4",
        "provider": "openai"
      },
      "metrics": {
        "tokens": 1500,
        "cost": 0.03
      }
    }
  ]
}
```

**ToonFormat** (`format: "toon"`)
```
Content-Type: text/plain

data[150]{timestamp,dimensions.model,dimensions.provider,metrics.tokens,metrics.cost}:
  2024-01-01T00:00:00Z,gpt-4,openai,1500,0.03
  2024-01-01T01:00:00Z,gpt-4,openai,1200,0.024
  ...
```

**JSON-Single Format** (`format: "json-single"`)
```
Content-Type: application/jsonl

{"timestamp":"2024-01-01T00:00:00Z","dimensions":{"model":"gpt-4"},"metrics":{"tokens":1500}}
{"timestamp":"2024-01-01T01:00:00Z","dimensions":{"model":"gpt-4"},"metrics":{"tokens":1200}}
...
```

#### Response Headers

| Header | Description |
|--------|-------------|
| Content-Type | The content type of the response |
| Content-Disposition | Suggested filename for download |
| X-Record-Count | Number of records in response |
| X-Query-Time | Query execution time in milliseconds |
| X-Format | The format used for the response |

### 2. Quick Summary Endpoint

**GET** `/api/ai-agents/quick-summary`

Get a quick summary of analytics data for rapid AI agent consumption.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| start | string | No | Start timestamp (default: 7 days ago) |
| end | string | No | End timestamp (default: now) |

#### Response

```
Content-Type: text/markdown

# Quick Analytics Summary

**Time Range:** 2024-01-08T00:00:00Z to 2024-01-15T00:00:00Z
**Total Records:** 1,234

## Totals
- **Total Tokens:** 1,543,200
- **Total Cost:** $30.86

## Top Models by Usage
- **gpt-4:** 500,000 tokens ($10.00)
- **gpt-3.5-turbo:** 400,000 tokens ($4.00)
- **claude-3-opus:** 300,000 tokens ($12.00)
- **claude-3-sonnet:** 200,000 tokens ($3.00)
- **gemini-pro:** 143,200 tokens ($1.86)
```

### 3. Format Discovery Endpoint

**GET** `/api/ai-agents/query?path=formats`

Get information about available output formats.

#### Response

```json
{
  "formats": [
    {
      "name": "markdown",
      "description": "Human-readable markdown with summary statistics",
      "contentType": "text/markdown",
      "tokenEfficiency": "medium",
      "humanReadable": true
    },
    {
      "name": "json",
      "description": "Standard JSON format with metadata",
      "contentType": "application/json",
      "tokenEfficiency": "low",
      "humanReadable": true
    },
    {
      "name": "toon",
      "description": "ToonFormat for token-efficient AI consumption",
      "contentType": "text/plain",
      "tokenEfficiency": "high",
      "humanReadable": true
    },
    {
      "name": "json-single",
      "description": "JSONL format for single record exchanges",
      "contentType": "application/jsonl",
      "tokenEfficiency": "medium",
      "humanReadable": false
    }
  ]
}
```

### 4. Example Query Endpoint

**GET** `/api/ai-agents/query?path=example`

Get an example query request for AI agent consumption.

#### Response

```json
{
  "example": {
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "dimensions": ["model", "provider"],
    "metrics": ["tokens", "cost"],
    "format": "markdown"
  },
  "description": "Example query request for AI agent consumption"
}
```

### 5. Health Check Endpoint

**GET** `/api/ai-agents/query?path=health`

Health check for the AI agent API service.

#### Response

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "service": "ai-agent-api"
}
```

## Output Formats

### Markdown Format

**Best for:** Human-readable documentation, AI agents that need structured text

**Token Efficiency:** Medium

**Features:**
- Includes summary statistics
- Formatted tables for easy reading
- Supports markdown rendering
- Includes metadata and context

**Example Usage:**
```bash
curl -X POST http://your-dashboard.com/api/ai-agents/query \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "dimensions": ["model"],
    "metrics": ["tokens"],
    "format": "markdown"
  }'
```

### JSON Format

**Best for:** Programmatic access, data processing, standard API consumption

**Token Efficiency:** Low

**Features:**
- Standard JSON structure
- Includes metadata
- Easy to parse programmatically
- Widely supported

**Example Usage:**
```bash
curl -X POST http://your-dashboard.com/api/ai-agents/query \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "format": "json"
  }'
```

### ToonFormat

**Best for:** AI agents, LLM consumption, token-optimized data transfer

**Token Efficiency:** High (30-50% token savings vs JSON)

**Features:**
- Token-efficient format
- Human-readable
- Compact representation
- Optimized for LLM prompts

**Example Usage:**
```bash
curl -X POST http://your-dashboard.com/api/ai-agents/query \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "format": "toon"
  }'
```

### JSON-Single (JSONL)

**Best for:** Streaming, single-record exchanges, bulk processing

**Token Efficiency:** Medium

**Features:**
- JSON Lines format
- One record per line
- Streaming-friendly
- Easy to process incrementally

**Example Usage:**
```bash
curl -X POST http://your-dashboard.com/api/ai-agents/query \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "format": "json-single"
  }'
```

## Content Negotiation

The API supports automatic content negotiation based on:

### Accept Header

```
Accept: text/markdown,application/json,text/plain
```

The API will select the best format based on the Accept header quality values.

### Query Parameter Override

You can override content negotiation using the `format` query parameter:

```
GET /api/ai-agents/query?format=toon
```

### AI Agent Detection

The API automatically detects AI agents from:

- **User-Agent patterns**: Claude, GPT, OpenAI, etc.
- **Request headers**: AI-specific headers like `x-api-key`, `x-ai-model`
- **Request behavior**: Bulk data requests, format preferences

When an AI agent is detected, the API automatically prioritizes token-efficient formats like ToonFormat.

## AI Agent Optimization

### Automatic Format Selection

When an AI agent is detected, the API:

1. **Prioritizes ToonFormat** for maximum token efficiency
2. **Applies compression** to reduce response size
3. **Includes relevant metadata** for context
4. **Optimizes field names** for brevity

### Custom Headers

The API includes AI agent detection information in response headers:

```
X-AI-Agent-Detected: true
X-Agent-Type: claude
X-Detection-Confidence: high
X-Token-Efficiency: high
```

### Token Savings

ToonFormat typically provides:

- **30-50% token savings** compared to JSON
- **20-30% token savings** compared to Markdown
- **Human-readable** output for verification
- **Lossless compression** for data fidelity

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_TIME_RANGE",
    "message": "End time must be after start time",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Common Error Codes

| Code | Description |
|------|-------------|
| INVALID_TIME_RANGE | End time must be after start time |
| UNAUTHORIZED | Invalid or missing API key |
| RATE_LIMIT_EXCEEDED | Too many requests |
| INVALID_FORMAT | Unsupported output format |
| QUERY_TIMEOUT | Query execution took too long |

## Rate Limiting

- **Standard rate limit:** 100 requests per minute
- **AI agent rate limit:** 200 requests per minute (optimized for AI consumption)
- **Burst allowance:** 10 requests above the limit

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1705316400
```

## Best Practices

### For AI Agents

1. **Use ToonFormat** for maximum token efficiency
2. **Set appropriate time ranges** to limit data volume
3. **Use `maxRecords`** to control response size
4. **Leverage quick summary** for rapid insights
5. **Implement caching** for repeated queries

### For Human Users

1. **Use Markdown format** for readability
2. **Request summary statistics** for quick insights
3. **Use appropriate time ranges** for relevant data
4. **Leverage the dashboard UI** for interactive exploration

## Examples

### Example 1: Basic Query with ToonFormat

```bash
curl -X POST http://your-dashboard.com/api/ai-agents/query \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "dimensions": ["model", "provider"],
    "metrics": ["tokens", "cost"],
    "format": "toon"
  }'
```

### Example 2: Quick Summary

```bash
curl -X GET "http://your-dashboard.com/api/ai-agents/quick-summary?start=2024-01-01T00:00:00Z&end=2024-01-31T23:59:59Z" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### Example 3: Large Dataset with Pagination

```bash
curl -X POST http://your-dashboard.com/api/ai-agents/query \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-12-31T23:59:59Z"
    },
    "format": "json-single",
    "maxRecords": 10000
  }'
```

### Example 4: Content Negotiation

```bash
curl -X POST http://your-dashboard.com/api/ai-agents/query \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Accept: text/toon,text/markdown,application/json" \
  -H "Content-Type: application/json" \
  -d '{
    "timeRange": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    }
  }'
```

## SDK Integration

### Python Example

```python
import requests

headers = {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json'
}

data = {
    'timeRange': {
        'start': '2024-01-01T00:00:00Z',
        'end': '2024-01-31T23:59:59Z'
    },
    'dimensions': ['model', 'provider'],
    'metrics': ['tokens', 'cost'],
    'format': 'toon'
}

response = requests.post(
    'http://your-dashboard.com/api/ai-agents/query',
    headers=headers,
    json=data
)

print(response.text)
```

### JavaScript Example

```javascript
const headers = {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json'
};

const data = {
    timeRange: {
        start: '2024-01-01T00:00:00Z',
        end: '2024-01-31T23:59:59Z'
    },
    dimensions: ['model', 'provider'],
    metrics: ['tokens', 'cost'],
    format: 'toon'
};

fetch('http://your-dashboard.com/api/ai-agents/query', {
    method: 'POST',
    headers: headers,
    body: JSON.stringify(data)
})
.then(response => response.text())
.then(data => console.log(data));
```

## Support

For issues or questions about the AI Agent API:

- **Documentation:** https://docs.your-dashboard.com
- **Support Email:** support@your-dashboard.com
- **GitHub Issues:** https://github.com/your-org/ai-dashboard/issues
