# Export Guide

This guide explains how to use the AI Dashboard export functionality to export analytics data in various formats.

## Overview

The export system supports multiple formats and delivery methods with built-in job management, privacy controls, and performance optimization.

## Supported Formats

### CSV Export
- **Description**: CSV format with Excel compatibility
- **MIME Type**: `text/csv`
- **Extension**: `.csv`
- **Features**:
  - Excel-compatible (includes BOM)
  - Custom delimiters
  - Header options
  - Proper escaping for special characters

### JSON Export
- **Description**: JSON format with schema validation
- **MIME Type**: `application/json`
- **Extension**: `.json`
- **Features**:
  - Schema validation
  - Optional metadata
  - Pretty print support
  - Null handling options

### PDF Export
- **Description**: PDF report with summary statistics
- **MIME Type**: `application/pdf`
- **Extension**: `.pdf`
- **Features**:
  - Summary statistics
  - Data tables
  - Formatted report layout
  - Custom page sizes

## API Usage

### Create Export Job

```bash
POST /api/export/execute
Content-Type: application/json

{
  "format": "csv",
  "filters": {
    "client": "acme-corp",
    "dateRange": "2024-01-01:2024-12-31"
  },
  "data": [...], // Array of QueryResultRow
  "privacy": {
    "enabled": true,
    "rules": [
      {
        "field": "email",
        "action": "mask",
        "options": {
          "maskChar": "*",
          "preservePrefix": 2
        }
      }
    ]
  }
}
```

**Response**:
```json
{
  "exportId": "export-1234567890-abc123",
  "status": "processing",
  "format": "csv",
  "createdAt": "2024-01-15T10:30:00Z",
  "estimatedCompletion": "2024-01-15T10:30:30Z"
}
```

### Check Export Status

```bash
GET /api/export/status/{exportId}
```

**Response**:
```json
{
  "exportId": "export-1234567890-abc123",
  "status": "completed",
  "progress": 100,
  "totalRecords": 1000,
  "processedRecords": 1000,
  "createdAt": "2024-01-15T10:30:00Z",
  "startedAt": "2024-01-15T10:30:01Z",
  "completedAt": "2024-01-15T10:30:25Z",
  "result": {
    "filename": "export-2024-01-15-10-30-25.csv",
    "size": 102400,
    "downloadUrl": "/api/export/download/export-1234567890-abc123",
    "expiresAt": "2024-01-16T10:30:25Z"
  },
  "isActive": false,
  "canCancel": false
}
```

### Download Export

```bash
GET /api/export/download/{exportId}
```

### Cancel Export

```bash
POST /api/export/cancel/{exportId}
```

**Response**:
```json
{
  "exportId": "export-1234567890-abc123",
  "status": "cancelled",
  "message": "Export job cancelled successfully"
}
```

### Get User Jobs

```bash
GET /api/export/jobs?userId={userId}
```

### Get Queue Statistics

```bash
GET /api/export/stats
```

**Response**:
```json
{
  "total": 5,
  "pending": 2,
  "processing": 1,
  "completed": 1,
  "failed": 1,
  "cancelled": 0,
  "activeJobs": 1,
  "maxConcurrentJobs": 1
}
```

## Privacy and Data Sanitization

The export system includes built-in privacy controls to protect sensitive data.

### Privacy Actions

- **remove**: Completely remove the field
- **mask**: Replace with asterisks (configurable)
- **hash**: Replace with cryptographic hash
- **truncate**: Truncate to specified length

### Example Privacy Configuration

```json
{
  "enabled": true,
  "rules": [
    {
      "field": "email",
      "action": "mask",
      "options": {
        "maskChar": "*",
        "preservePrefix": 2
      }
    },
    {
      "field": "apiKey",
      "action": "remove"
    },
    {
      "field": "userId",
      "action": "hash",
      "options": {
        "hashAlgorithm": "sha256"
      }
    }
  ],
  "defaultAction": "mask"
}
```

### Automatic PII Detection

The system automatically detects potentially sensitive fields:
- Email addresses
- Phone numbers
- Social Security Numbers
- Credit card numbers
- API keys and tokens
- Passwords and secrets

## Export Filtering

You can filter and customize export data using the filter configuration.

### Filter Operators

- `eq`: Equal to
- `ne`: Not equal to
- `gt`: Greater than
- `gte`: Greater than or equal to
- `lt`: Less than
- `lte`: Less than or equal to
- `contains`: Contains string
- `startsWith`: Starts with string
- `endsWith`: Ends with string
- `in`: In array
- `notIn`: Not in array

### Example Filter Configuration

```json
{
  "filters": [
    {
      "field": "client",
      "operator": "eq",
      "value": "acme-corp"
    },
    {
      "field": "timestamp",
      "operator": "gte",
      "value": "2024-01-01T00:00:00Z"
    }
  ],
  "sorts": [
    {
      "field": "timestamp",
      "order": "desc"
    }
  ],
  "columns": [
    {
      "key": "timestamp",
      "label": "Date",
      "visible": true
    },
    {
      "key": "client",
      "label": "Client",
      "visible": true
    }
  ],
  "limit": 1000,
  "offset": 0
}
```

## Delivery Methods

### Download (Default)
- Generates a download URL
- Files expire after 24 hours
- No additional configuration required

### Email (Requires Configuration)
- Sends export as email attachment
- Requires email service configuration
- Supports multiple recipients

### S3 (Requires Configuration)
- Uploads directly to S3
- Requires AWS credentials
- Supports custom bucket and key

## Performance Considerations

- Large exports are processed asynchronously
- Job queue prevents system overload
- Configurable timeout (default: 30 minutes)
- Automatic cleanup of old jobs (default: 24 hours)

## Error Handling

The export system provides detailed error messages:

- **Invalid format**: Format must be csv, json, or pdf
- **Missing data**: Data array is required
- **Job not found**: Export ID does not exist
- **Not ready**: Export is not ready for download
- **Timeout**: Export exceeded time limit

## Best Practices

1. **Use filters**: Reduce dataset size before export
2. **Enable privacy**: Always enable privacy for sensitive data
3. **Monitor jobs**: Check job status for large exports
4. **Clean up**: Old jobs are automatically cleaned up
5. **Choose appropriate format**: CSV for data analysis, JSON for applications, PDF for reports

## Troubleshooting

### Export stuck in "processing" state
- Check queue statistics: `GET /api/export/stats`
- Cancel and retry if needed: `POST /api/export/cancel/{id}`

### Download URL expired
- Exports expire after 24 hours
- Create a new export job

### Large export timeout
- Reduce dataset size with filters
- Increase timeout in export manager configuration

### Privacy not working
- Verify privacy configuration is enabled
- Check field names match your data structure
- Review privacy report for applied rules

## Examples

### Simple CSV Export

```bash
curl -X POST http://localhost:3000/api/export/execute \
  -H "Content-Type: application/json" \
  -d '{
    "format": "csv",
    "data": [
      {
        "timestamp": "2024-01-15T10:00:00Z",
        "dimensions": {"client": "acme-corp"},
        "metrics": {"requests": 100, "tokens": 5000}
      }
    ]
  }'
```

### Export with Privacy

```bash
curl -X POST http://localhost:3000/api/export/execute \
  -H "Content-Type: application/json" \
  -d '{
    "format": "json",
    "data": [...],
    "privacy": {
      "enabled": true,
      "rules": [
        {"field": "email", "action": "mask"}
      ]
    }
  }'
```

### Export with Filters

```bash
curl -X POST http://localhost:3000/api/export/execute \
  -H "Content-Type: application/json" \
  -d '{
    "format": "pdf",
    "filters": {
      "client": "acme-corp"
    },
    "data": [...]
  }'
```