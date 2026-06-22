/**
 * Analytics Domain Routes
 * 
 * Route handlers for analytics query operations
 */

import { NextRequest, NextResponse } from 'next/server';
import { QueryExecutor } from '../analytics/executor';
import { queryMonitor } from '../analytics/monitor';

const executor = new QueryExecutor();

/**
 * Execute analytics query
 */
export async function executeQuery(request: NextRequest): Promise<NextResponse> {
  try {
    const body = await request.json();
    const result = await executor.execute(body);
    
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
    return NextResponse.json(
      { error: 'Query execution failed', message: error.message },
      { status: 500 }
    );
  }
}

/**
 * Health check
 */
export async function healthCheck(request: NextRequest): Promise<NextResponse> {
  return NextResponse.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    service: 'analytics-query-api'
  });
}

/**
 * Get system statistics
 */
export async function getStats(request: NextRequest): Promise<NextResponse> {
  const stats = queryMonitor.getSystemStats();
  return NextResponse.json(stats);
}
