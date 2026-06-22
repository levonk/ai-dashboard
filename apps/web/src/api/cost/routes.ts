/**
 * Cost Analysis API Routes
 * 
 * Express/Next.js API routes for cost analysis features
 */

import { NextRequest, NextResponse } from 'next/server';
import { CostBreakdownAnalyzer } from './breakdown';
import { CostTrendAnalyzer } from './trends';
import { CostForecaster } from './forecast';
import { CostOptimizer } from './optimization';
import { CostComparator } from './comparison';
import { CostAlertManager } from './alerts';
import { CostAnalysisCache, AggregatedCostCache } from './cache';

// Initialize analyzers
const breakdownAnalyzer = new CostBreakdownAnalyzer();
const trendAnalyzer = new CostTrendAnalyzer();
const costForecaster = new CostForecaster();
const costOptimizer = new CostOptimizer();
const costComparator = new CostComparator();
const costAlertManager = new CostAlertManager();

// Initialize cache
const costCache = new CostAnalysisCache({
  enabled: true,
  ttl: 300, // 5 minutes
  maxSize: 1000
});
const aggregatedCache = new AggregatedCostCache();

/**
 * POST /api/cost/breakdown
 * Analyze cost breakdown by dimensions
 */
export async function POST(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const action = searchParams.get('action');

  try {
    const body = await request.json();

    switch (action) {
      case 'breakdown':
        return await handleBreakdown(body);
      case 'trends':
        return await handleTrends(body);
      case 'forecast':
        return await handleForecast(body);
      case 'optimization':
        return await handleOptimization(body);
      case 'comparison':
        return await handleComparison(body);
      case 'alert-create':
        return await handleAlertCreate(body);
      case 'alert-status':
        return await handleAlertStatus(body);
      case 'cache-stats':
        return await handleCacheStats();
      default:
        return NextResponse.json(
          { error: 'Invalid action. Use: breakdown, trends, forecast, optimization, comparison, alert-create, alert-status, cache-stats' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Cost analysis API error:', error);
    return NextResponse.json(
      { error: 'An error occurred during cost analysis', message: error.message },
      { status: 500 }
    );
  }
}

/**
 * GET /api/cost
 * Get cost analysis metadata and cache stats
 */
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const path = searchParams.get('path');

  try {
    if (path === 'health') {
      return NextResponse.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        service: 'cost-analysis-api'
      });
    }

    if (path === 'cache-stats') {
      return await handleCacheStats();
    }

    if (path === 'alerts') {
      const alerts = await costAlertManager.listAlerts();
      return NextResponse.json({ alerts });
    }

    if (path === 'aggregation-stats') {
      const stats = aggregatedCache.getAggregationStats();
      return NextResponse.json(stats);
    }

    return NextResponse.json({
      message: 'Cost Analysis API',
      version: '1.0.0',
      endpoints: {
        'POST /api/cost?action=breakdown': 'Analyze cost breakdown',
        'POST /api/cost?action=trends': 'Analyze cost trends',
        'POST /api/cost?action=forecast': 'Generate cost forecast',
        'POST /api/cost?action=optimization': 'Get optimization recommendations',
        'POST /api/cost?action=comparison': 'Compare costs across dimensions',
        'POST /api/cost?action=alert-create': 'Create cost alert',
        'POST /api/cost?action=alert-status': 'Get alert status',
        'GET /api/cost?path=health': 'Health check',
        'GET /api/cost?path=cache-stats': 'Cache statistics',
        'GET /api/cost?path=alerts': 'List all alerts',
        'GET /api/cost?path=aggregation-stats': 'Aggregation statistics'
      }
    });
  } catch (error) {
    console.error('Cost analysis API error:', error);
    return NextResponse.json(
      { error: 'An error occurred', message: error.message },
      { status: 500 }
    );
  }
}

/**
 * Handle cost breakdown request
 */
async function handleBreakdown(body: any): Promise<NextResponse> {
  const cacheKey = CostAnalysisCache.generateKey('breakdown', body);
  
  const result = await costCache.getOrSet(cacheKey, async () => {
    return await breakdownAnalyzer.analyzeBreakdown(body);
  });

  return NextResponse.json(result);
}

/**
 * Handle cost trends request
 */
async function handleTrends(body: any): Promise<NextResponse> {
  const cacheKey = CostAnalysisCache.generateKey('trends', body);
  
  const result = await costCache.getOrSet(cacheKey, async () => {
    return await trendAnalyzer.analyzeTrends(body);
  });

  return NextResponse.json(result);
}

/**
 * Handle cost forecast request
 */
async function handleForecast(body: any): Promise<NextResponse> {
  const cacheKey = CostAnalysisCache.generateKey('forecast', body);
  
  const result = await costCache.getOrSet(cacheKey, async () => {
    return await costForecaster.generateForecast(body);
  });

  return NextResponse.json(result);
}

/**
 * Handle cost optimization request
 */
async function handleOptimization(body: any): Promise<NextResponse> {
  const cacheKey = CostAnalysisCache.generateKey('optimization', body);
  
  const result = await costCache.getOrSet(cacheKey, async () => {
    return await costOptimizer.generateRecommendations(body);
  });

  return NextResponse.json(result);
}

/**
 * Handle cost comparison request
 */
async function handleComparison(body: any): Promise<NextResponse> {
  const cacheKey = CostAnalysisCache.generateKey('comparison', body);
  
  const result = await costCache.getOrSet(cacheKey, async () => {
    return await costComparator.compareCosts(body);
  });

  return NextResponse.json(result);
}

/**
 * Handle alert creation request
 */
async function handleAlertCreate(body: any): Promise<NextResponse> {
  const result = await costAlertManager.createAlert(body);
  return NextResponse.json(result);
}

/**
 * Handle alert status request
 */
async function handleAlertStatus(body: any): Promise<NextResponse> {
  const { alertId } = body;
  const status = await costAlertManager.getAlertStatus(alertId);
  
  if (!status) {
    return NextResponse.json(
      { error: 'Alert not found' },
      { status: 404 }
    );
  }

  return NextResponse.json(status);
}

/**
 * Handle cache stats request
 */
async function handleCacheStats(): Promise<NextResponse> {
  const stats = costCache.getStats();
  return NextResponse.json(stats);
}

/**
 * Cost analysis helper functions
 */
export const CostAnalysisHelpers = {
  /**
   * Execute a complete cost analysis
   */
  async completeAnalysis(startTime: string, endTime: string) {
    const breakdown = await breakdownAnalyzer.analyzeBreakdown({
      startTime,
      endTime,
      dimensions: ['provider', 'model']
    });

    const trends = await trendAnalyzer.analyzeTrends({
      startTime,
      endTime,
      granularity: 'day',
      includeForecast: true,
      forecastPeriod: 7
    });

    const optimization = await costOptimizer.generateRecommendations({
      startTime,
      endTime,
      scope: {
        dimensions: ['provider', 'model']
      },
      options: {
        includeModelRecommendations: true,
        includeProviderRecommendations: true,
        includeUsageOptimization: true
      }
    });

    return {
      breakdown,
      trends,
      optimization
    };
  },

  /**
   * Get cost summary
   */
  async getCostSummary(startTime: string, endTime: string) {
    const breakdown = await breakdownAnalyzer.analyzeBreakdown({
      startTime,
      endTime,
      dimensions: ['provider']
    });

    return {
      totalCost: breakdown.summary.totalCost,
      totalTokens: breakdown.summary.totalTokens,
      totalRequests: breakdown.summary.totalRequests,
      averageCostPerRequest: breakdown.summary.averageCostPerRequest,
      averageCostPerToken: breakdown.summary.averageCostPerToken,
      breakdown: breakdown.breakdown
    };
  },

  /**
   * Clear cost analysis cache
   */
  clearCache() {
    costCache.clear();
    return { success: true, message: 'Cache cleared' };
  },

  /**
   * Invalidate cache by pattern
   */
  invalidateCachePattern(pattern: string) {
    const count = costCache.invalidatePattern(pattern);
    return { success: true, invalidated: count };
  }
};