/**
 * AI Agent Interface Endpoints
 * 
 * API endpoints designed for AI agent consumption with markdown output
 * Supports multiple output formats (Markdown, JSON, ToonFormat) for optimal token usage
 */

import { NextRequest, NextResponse } from 'next/server';
import { QueryExecutor } from '../analytics/executor';
import { AnalyticsQueryBuilder } from '../analytics/query-builder';
import { ToonFormatExporter } from '../../export/formats/toonformat';
import { JSONSingleExporter } from '../../export/formats/json-single';
import { AnalyticsQueryRequest, AnalyticsQueryResponse, QueryResultRow } from '../analytics/types';

// Initialize components
const executor = new QueryExecutor();
const toonExporter = new ToonFormatExporter();
const jsonSingleExporter = new JSONSingleExporter();

/**
 * Supported output formats for AI agents
 */
type OutputFormat = 'markdown' | 'json' | 'toon' | 'json-single';

/**
 * AI Agent Query Request
 */
interface AIAgentQueryRequest extends AnalyticsQueryRequest {
  format?: OutputFormat;
  includeMetadata?: boolean;
  flattenStructure?: boolean;
  maxRecords?: number;
}

/**
 * Convert analytics data to markdown format
 */
function convertToMarkdown(data: QueryResultRow[], includeMetadata: boolean = false): string {
  let markdown = '# Analytics Data\n\n';

  if (includeMetadata) {
    markdown += `**Exported:** ${new Date().toISOString()}\n`;
    markdown += `**Record Count:** ${data.length}\n\n`;
  }

  if (data.length === 0) {
    markdown += 'No data available.\n';
    return markdown;
  }

  // Extract headers from first record
  const headers = extractHeaders(data[0]);

  // Create table header
  markdown += '| ' + headers.join(' | ') + ' |\n';
  markdown += '| ' + headers.map(() => '---').join(' | ') + ' |\n';

  // Create table rows
  for (const record of data) {
    const values = headers.map(header => {
      const value = getNestedValue(record, header);
      return formatCellValue(value);
    });
    markdown += '| ' + values.join(' | ') + ' |\n';
  }

  return markdown;
}

/**
 * Extract all possible headers from data
 */
function extractHeaders(record: QueryResultRow): string[] {
  const headers: string[] = ['timestamp'];

  if (record.dimensions) {
    Object.keys(record.dimensions).forEach(key => {
      headers.push(`dimensions.${key}`);
    });
  }

  if (record.metrics) {
    Object.keys(record.metrics).forEach(key => {
      headers.push(`metrics.${key}`);
    });
  }

  return headers;
}

/**
 * Get nested value from record using dot notation
 */
function getNestedValue(obj: any, path: string): any {
  const keys = path.split('.');
  let current = obj;

  for (const key of keys) {
    if (current === null || current === undefined) {
      return null;
    }
    current = current[key];
  }

  return current;
}

/**
 * Format cell value for markdown table
 */
function formatCellValue(value: any): string {
  if (value === null || value === undefined) {
    return 'N/A';
  }

  if (typeof value === 'object') {
    return JSON.stringify(value);
  }

  return String(value);
}

/**
 * Convert analytics data to markdown with summary statistics
 */
function convertToMarkdownWithSummary(data: QueryResultRow[]): string {
  let markdown = '# Analytics Data Summary\n\n';

  if (data.length === 0) {
    markdown += 'No data available.\n';
    return markdown;
  }

  // Add summary statistics
  markdown += '## Summary Statistics\n\n';
  markdown += `- **Total Records:** ${data.length}\n`;
  markdown += `- **Time Range:** ${data[0].timestamp} to ${data[data.length - 1].timestamp}\n\n`;

  // Calculate metric summaries if available
  if (data[0].metrics) {
    markdown += '## Metric Summaries\n\n';
    
    for (const metric of Object.keys(data[0].metrics)) {
      const values = data
        .map(r => r.metrics?.[metric])
        .filter(v => typeof v === 'number');

      if (values.length > 0) {
        const sum = values.reduce((a, b) => a + b, 0);
        const avg = sum / values.length;
        const min = Math.min(...values);
        const max = Math.max(...values);

        markdown += `### ${metric}\n`;
        markdown += `- **Average:** ${avg.toFixed(2)}\n`;
        markdown += `- **Min:** ${min}\n`;
        markdown += `- **Max:** ${max}\n`;
        markdown += `- **Total:** ${sum.toFixed(2)}\n\n`;
      }
    }
  }

  // Add dimension breakdown
  if (data[0].dimensions) {
    markdown += '## Dimension Breakdown\n\n';
    
    for (const dimension of Object.keys(data[0].dimensions)) {
      const uniqueValues = new Set(
        data.map(r => r.dimensions?.[dimension]).filter(v => v !== undefined)
      );

      markdown += `### ${dimension}\n`;
      markdown += `- **Unique Values:** ${uniqueValues.size}\n`;
      markdown += `- **Values:** ${Array.from(uniqueValues).slice(0, 10).join(', ')}${uniqueValues.size > 10 ? '...' : ''}\n\n`;
    }
  }

  // Add sample data table
  markdown += '## Sample Data (First 10 Records)\n\n';
  const sampleData = data.slice(0, 10);
  markdown += convertToMarkdown(sampleData, false);

  return markdown;
}

/**
 * POST /api/ai-agents/query
 * Execute analytics query with AI agent optimized output
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json() as AIAgentQueryRequest;
    
    // Set defaults for AI agent requests
    const format = body.format || 'markdown';
    const includeMetadata = body.includeMetadata !== false;
    const maxRecords = body.maxRecords || 1000;

    // Execute query
    const result = await executor.execute(body);
    
    // Limit records if specified
    const limitedData = result.data.slice(0, maxRecords);

    // Convert to requested format
    let content: string;
    let contentType: string;
    let filename: string;

    switch (format) {
      case 'markdown':
        content = convertToMarkdownWithSummary(limitedData);
        contentType = 'text/markdown';
        filename = `analytics_${Date.now()}.md`;
        break;

      case 'json':
        content = JSON.stringify({
          metadata: {
            exportedAt: new Date().toISOString(),
            recordCount: limitedData.length,
            queryId: result.queryId,
            executionTimeMs: result.executionTimeMs
          },
          data: limitedData
        }, null, 2);
        contentType = 'application/json';
        filename = `analytics_${Date.now()}.json`;
        break;

      case 'toon':
        const toonResult = await toonExporter.export(limitedData, {
          includeMetadata,
          delimiter: 'comma',
          indentation: 2,
          keyFolding: true,
          flattenDepth: 3
        });
        content = toonResult.content;
        contentType = 'text/plain';
        filename = toonResult.filename;
        break;

      case 'json-single':
        const jsonSingleResult = await jsonSingleExporter.exportMultiple(limitedData, {
          includeMetadata,
          flattenStructure: body.flattenStructure || false,
          timestampFormat: 'iso'
        });
        content = jsonSingleResult.content;
        contentType = 'application/jsonl';
        filename = jsonSingleResult.filename;
        break;

      default:
        content = convertToMarkdownWithSummary(limitedData);
        contentType = 'text/markdown';
        filename = `analytics_${Date.now()}.md`;
    }

    // Return response with appropriate content type
    return new NextResponse(content, {
      headers: {
        'Content-Type': contentType,
        'Content-Disposition': `attachment; filename="${filename}"`,
        'X-Record-Count': limitedData.length.toString(),
        'X-Query-Time': result.executionTimeMs.toString(),
        'X-Format': format
      }
    });

  } catch (error) {
    console.error('AI agent query error:', error);
    
    return NextResponse.json(
      {
        error: {
          code: 'INTERNAL_ERROR',
          message: error instanceof Error ? error.message : 'An error occurred during query execution',
          timestamp: new Date().toISOString()
        }
      },
      { status: 500 }
    );
  }
}

/**
 * GET /api/ai-agents/query
 * Get information about available AI agent endpoints
 */
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const path = searchParams.get('path');

  if (path === 'health') {
    return NextResponse.json({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      service: 'ai-agent-api'
    });
  }

  if (path === 'formats') {
    return NextResponse.json({
      formats: [
        {
          name: 'markdown',
          description: 'Human-readable markdown with summary statistics',
          contentType: 'text/markdown',
          tokenEfficiency: 'medium',
          humanReadable: true
        },
        {
          name: 'json',
          description: 'Standard JSON format with metadata',
          contentType: 'application/json',
          tokenEfficiency: 'low',
          humanReadable: true
        },
        {
          name: 'toon',
          description: 'ToonFormat for token-efficient AI consumption',
          contentType: 'text/plain',
          tokenEfficiency: 'high',
          humanReadable: true
        },
        {
          name: 'json-single',
          description: 'JSONL format for single record exchanges',
          contentType: 'application/jsonl',
          tokenEfficiency: 'medium',
          humanReadable: false
        }
      ]
    });
  }

  if (path === 'example') {
    const exampleQuery = {
      timeRange: {
        start: '2024-01-01T00:00:00Z',
        end: '2024-01-31T23:59:59Z'
      },
      dimensions: ['model', 'provider'],
      metrics: ['tokens', 'cost'],
      format: 'markdown' as OutputFormat
    };

    return NextResponse.json({
      example: exampleQuery,
      description: 'Example query request for AI agent consumption'
    });
  }

  return NextResponse.json({
    message: 'AI Agent Interface API',
    version: '1.0.0',
    description: 'Optimized API endpoints for AI agent consumption with multiple output formats',
    endpoints: {
      'POST /api/ai-agents/query': 'Execute analytics query with AI-optimized output',
      'GET /api/ai-agents/query?path=health': 'Health check',
      'GET /api/ai-agents/query?path=formats': 'Available output formats',
      'GET /api/ai-agents/query?path=example': 'Example query request'
    },
    supportedFormats: ['markdown', 'json', 'toon', 'json-single'],
    defaultFormat: 'markdown'
  });
}

/**
 * GET /api/ai-agents/quick-summary
 * Quick summary endpoint for rapid AI agent consumption
 */
export async function QUICK_SUMMARY(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const startTime = searchParams.get('start') || new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString();
    const endTime = searchParams.get('end') || new Date().toISOString();

    // Build a basic usage query
    const builder = new AnalyticsQueryBuilder()
      .timeRange(startTime, endTime)
      .dimensions(['model', 'provider'])
      .metrics(['tokens', 'cost'])
      .groupBy(['model', 'provider'])
      .aggregation({
        tokens: 'sum',
        cost: 'sum'
      });

    const query = builder.build();
    const result = await executor.execute(query);

    // Generate quick markdown summary
    let summary = '# Quick Analytics Summary\n\n';
    summary += `**Time Range:** ${startTime} to ${endTime}\n`;
    summary += `**Total Records:** ${result.data.length}\n\n`;

    if (result.data.length > 0) {
      const totalTokens = result.data.reduce((sum, r) => sum + (r.metrics?.tokens || 0), 0);
      const totalCost = result.data.reduce((sum, r) => sum + (r.metrics?.cost || 0), 0);

      summary += `## Totals\n`;
      summary += `- **Total Tokens:** ${totalTokens.toLocaleString()}\n`;
      summary += `- **Total Cost:** $${totalCost.toFixed(2)}\n\n`;

      summary += `## Top Models by Usage\n`;
      const sortedByTokens = [...result.data].sort((a, b) => 
        (b.metrics?.tokens || 0) - (a.metrics?.tokens || 0)
      ).slice(0, 5);

      for (const record of sortedByTokens) {
        const model = record.dimensions?.model || 'Unknown';
        const tokens = record.metrics?.tokens || 0;
        const cost = record.metrics?.cost || 0;
        summary += `- **${model}:** ${tokens.toLocaleString()} tokens ($${cost.toFixed(2)})\n`;
      }
    }

    return new NextResponse(summary, {
      headers: {
        'Content-Type': 'text/markdown',
        'X-Record-Count': result.data.length.toString()
      }
    });

  } catch (error) {
    console.error('Quick summary error:', error);
    
    return NextResponse.json(
      {
        error: {
          code: 'INTERNAL_ERROR',
          message: error instanceof Error ? error.message : 'Failed to generate quick summary',
          timestamp: new Date().toISOString()
        }
      },
      { status: 500 }
    );
  }
}
