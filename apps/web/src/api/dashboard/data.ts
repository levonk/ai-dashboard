/**
 * Dashboard Data API Routes
 * 
 * API endpoints for dashboard-specific data processing
 * Integrates with the dashboard processing modules for transformation, aggregation, and caching
 */

import { NextRequest, NextResponse } from 'next/server';
import { DashboardTransformer } from '../../processing/dashboard/transformer';
import { DashboardAggregator } from '../../processing/dashboard/aggregator';
import { TimeSeriesProcessor } from '../../processing/dashboard/timeseries';
import { DashboardCache, DashboardQueryCache } from '../../processing/dashboard/cache';
import { DashboardValidator } from '../../processing/dashboard/validator';
import { RealtimeDashboardProcessor } from '../../processing/dashboard/realtime';
import { DashboardExporter } from '../../processing/dashboard/export';
import { AnalyticsQueryResponse, QueryResultRow } from '../analytics/types';

// Initialize cache
const dashboardCache = new DashboardQueryCache({
  ttl: 5 * 60 * 1000, // 5 minutes
  maxSize: 100,
  strategy: 'lru',
  compression: false
});

// Initialize real-time processor
const realtimeProcessor = new RealtimeDashboardProcessor({
  enabled: true,
  updateInterval: 5000,
  debounceTime: 1000,
  maxBufferSize: 1000,
  deltaUpdates: true
});

// Initialize validator
const validator = new DashboardValidator();

/**
 * POST /api/dashboard/data
 * Process raw analytics data for dashboard consumption
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { operation, data, config } = body;

    // Validate input data
    const validationResult = validator.validateQueryResponse(data);
    if (!validationResult.valid) {
      return NextResponse.json(
        {
          error: 'Validation failed',
          details: validationResult.errors
        },
        { status: 400 }
      );
    }

    // Generate cache key
    const cacheKey = `dashboard:${operation}:${JSON.stringify(config)}`;

    // Check cache first
    const cached = dashboardCache.get(cacheKey);
    if (cached) {
      return NextResponse.json({
        data: cached,
        cached: true,
        timestamp: new Date().toISOString()
      });
    }

    // Process based on operation type
    let result;
    switch (operation) {
      case 'transform-chart':
        result = await transformToChartData(data, config);
        break;
      case 'transform-metrics':
        result = await transformToMetricCards(data, config);
        break;
      case 'transform-table':
        result = await transformToTableData(data, config);
        break;
      case 'aggregate-overview':
        result = await aggregateOverview(data, config);
        break;
      case 'aggregate-dimension':
        result = await aggregateByDimension(data, config);
        break;
      case 'time-series':
        result = await processTimeSeries(data, config);
        break;
      case 'top-n':
        result = await getTopN(data, config);
        break;
      case 'percentiles':
        result = await calculatePercentiles(data, config);
        break;
      default:
        return NextResponse.json(
          {
            error: 'Invalid operation',
            validOperations: [
              'transform-chart',
              'transform-metrics',
              'transform-table',
              'aggregate-overview',
              'aggregate-dimension',
              'time-series',
              'top-n',
              'percentiles'
            ]
          },
          { status: 400 }
        );
    }

    // Cache the result
    dashboardCache.set(cacheKey, result);

    return NextResponse.json({
      data: result,
      cached: false,
      timestamp: new Date().toISOString()
    });

  } catch (error) {
    console.error('Dashboard data processing error:', error);
    return NextResponse.json(
      {
        error: 'Processing failed',
        message: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}

/**
 * GET /api/dashboard/data
 * Get dashboard data processing status and statistics
 */
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const path = searchParams.get('path');

  if (path === 'health') {
    return NextResponse.json({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      service: 'dashboard-data-api'
    });
  }

  if (path === 'cache-stats') {
    const stats = dashboardCache.getStats();
    return NextResponse.json(stats);
  }

  if (path === 'realtime-stats') {
    const stats = realtimeProcessor.getStats();
    return NextResponse.json(stats);
  }

  if (path === 'clear-cache') {
    dashboardCache.clear();
    return NextResponse.json({
      message: 'Cache cleared',
      timestamp: new Date().toISOString()
    });
  }

  return NextResponse.json({
    message: 'Dashboard Data API',
    version: '1.0.0',
    endpoints: {
      'POST /api/dashboard/data': 'Process data for dashboard',
      'GET /api/dashboard/data?path=health': 'Health check',
      'GET /api/dashboard/data?path=cache-stats': 'Cache statistics',
      'GET /api/dashboard/data?path=realtime-stats': 'Real-time statistics',
      'GET /api/dashboard/data?path=clear-cache': 'Clear cache'
    }
  });
}

/**
 * Transform data to chart format
 */
async function transformToChartData(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const chartData = DashboardTransformer.toChartData(
    data.data,
    config
  );
  return chartData;
}

/**
 * Transform data to metric cards
 */
async function transformToMetricCards(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const previousData = config.previousPeriodData;
  const metricCards = DashboardTransformer.toMetricCards(
    data.data,
    previousData
  );
  return metricCards;
}

/**
 * Transform data to table format
 */
async function transformToTableData(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const tableData = DashboardTransformer.toTableData(
    data.data,
    config.columnConfig
  );
  return tableData;
}

/**
 * Aggregate overview data
 */
async function aggregateOverview(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const aggregation = DashboardAggregator.aggregateOverview(
    data.data,
    config
  );
  return aggregation;
}

/**
 * Aggregate by dimension
 */
async function aggregateByDimension(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const dimension = config.dimension;
  const aggregation = DashboardAggregator.aggregateByDimension(
    data.data,
    dimension,
    config
  );
  return aggregation;
}

/**
 * Process time-series data
 */
async function processTimeSeries(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const metric = config.metric;
  const timeSeriesConfig = {
    granularity: config.granularity || 'hour',
    fillMissing: config.fillMissing ?? true,
    interpolation: config.interpolation || 'linear',
    smoothing: config.smoothing
  };
  
  const timeSeriesData = TimeSeriesProcessor.processToTimeSeries(
    data.data,
    metric,
    timeSeriesConfig
  );
  return timeSeriesData;
}

/**
 * Get top N items
 */
async function getTopN(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const metric = config.metric;
  const n = config.n || 10;
  const topN = DashboardAggregator.getTopN(data.data, metric, n);
  return topN;
}

/**
 * Calculate percentiles
 */
async function calculatePercentiles(
  data: AnalyticsQueryResponse,
  config: any
): Promise<any> {
  const metric = config.metric;
  const percentiles = config.percentiles || [50, 90, 95, 99];
  const percentileData = DashboardAggregator.calculatePercentiles(
    data.data,
    metric,
    percentiles
  );
  return percentileData;
}

/**
 * Real-time update subscription helper
 */
export function subscribeToRealtimeUpdates(
  callback: (update: any) => void,
  filter?: (update: any) => boolean
): string {
  return realtimeProcessor.subscribe(callback, filter);
}

/**
 * Unsubscribe from real-time updates
 */
export function unsubscribeFromRealtimeUpdates(subscriptionId: string): boolean {
  return realtimeProcessor.unsubscribe(subscriptionId);
}

/**
 * Add data to real-time processor
 */
export function addRealtimeData(data: QueryResultRow[]): void {
  realtimeProcessor.addData(data);
}

/**
 * Export dashboard data
 */
export async function exportDashboardData(
  data: QueryResultRow[] | AnalyticsQueryResponse,
  format: 'csv' | 'json' | 'xlsx' | 'pdf',
  config?: any
): Promise<any> {
  const exporter = new DashboardExporter();
  const exportConfig = {
    format,
    includeMetadata: config?.includeMetadata ?? true,
    dateFormat: config?.dateFormat || 'YYYY-MM-DD',
    numberFormat: config?.numberFormat || '0,0.00',
    compression: config?.compression || false
  };
  
  return exporter.export(data, exportConfig);
}