/**
 * Analytics Query API Routes
 * 
 * Express/Next.js API routes for analytics query functionality
 */

import { NextRequest, NextResponse } from 'next/server';
import { QueryExecutor } from './executor';
import { AnalyticsQueryBuilder, QueryPatterns } from './query-builder';
import { queryMonitor } from './monitor';
import { AnalyticsQueryRequest, AnalyticsQueryResponse, QueryError } from './types';

// Initialize query executor
const executor = new QueryExecutor();

/**
 * POST /api/analytics/query
 * Execute an analytics query
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json() as AnalyticsQueryRequest;
    
    // Execute query
    const result = await executor.execute(body);
    
    // Record metrics
    queryMonitor.recordMetrics({
      queryId: result.queryId,
      executionTimeMs: result.executionTimeMs,
      cacheHit: result.cached,
      resultCount: result.data.length,
      timestamp: new Date().toISOString()
    });

    return NextResponse.json(result);

  } catch (error) {
    console.error('Query execution error:', error);
    
    const queryError = error as QueryError;
    return NextResponse.json(
      {
        error: {
          code: queryError.code || 'INTERNAL_ERROR',
          message: queryError.message || 'An error occurred during query execution',
          timestamp: new Date().toISOString()
        }
      },
      { status: 500 }
    );
  }
}

/**
 * GET /api/analytics/query/health
 * Health check endpoint for query service
 */
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const path = searchParams.get('path');

  if (path === 'health') {
    return NextResponse.json({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      service: 'analytics-query-api'
    });
  }

  if (path === 'stats') {
    const stats = queryMonitor.getSystemStats();
    return NextResponse.json(stats);
  }

  if (path === 'slow-queries') {
    const slowQueries = queryMonitor.getSlowQueries();
    return NextResponse.json({ slowQueries });
  }

  return NextResponse.json({
    message: 'Analytics Query API',
    version: '1.0.0',
    endpoints: {
      'POST /api/analytics/query': 'Execute analytics query',
      'GET /api/analytics/query?path=health': 'Health check',
      'GET /api/analytics/query?path=stats': 'System statistics',
      'GET /api/analytics/query?path=slow-queries': 'Slow queries'
    }
  });
}

/**
 * Query helper functions for common patterns
 */
export const QueryHelpers = {
  /**
   * Execute a basic usage query
   */
  async basicUsage(startTime: string, endTime: string): Promise<AnalyticsQueryResponse> {
    const builder = QueryPatterns.basicUsage(startTime, endTime);
    const request = builder.build();
    return executor.execute(request);
  },

  /**
   * Execute a cost breakdown query
   */
  async costBreakdown(startTime: string, endTime: string): Promise<AnalyticsQueryResponse> {
    const builder = QueryPatterns.costBreakdown(startTime, endTime);
    const request = builder.build();
    return executor.execute(request);
  },

  /**
   * Execute a performance analysis query
   */
  async performanceAnalysis(startTime: string, endTime: string): Promise<AnalyticsQueryResponse> {
    const builder = QueryPatterns.performanceAnalysis(startTime, endTime);
    const request = builder.build();
    return executor.execute(request);
  },

  /**
   * Execute a trend analysis query
   */
  async trendAnalysis(startTime: string, endTime: string): Promise<AnalyticsQueryResponse> {
    const builder = QueryPatterns.trendAnalysis(startTime, endTime);
    const request = builder.build();
    return executor.execute(request);
  }
};

/**
 * Middleware for query monitoring and optimization
 */
export function queryMiddleware(request: NextRequest) {
  // Add request timing
  const startTime = Date.now();

  // Log query request
  console.log(`[${new Date().toISOString()}] Query request received`);

  // Add response timing
  return (response: NextResponse) => {
    const duration = Date.now() - startTime;
    console.log(`[${new Date().toISOString()}] Query completed in ${duration}ms`);
    return response;
  };
}
