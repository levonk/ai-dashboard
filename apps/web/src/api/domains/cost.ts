/**
 * Cost Domain Routes
 * 
 * Route handlers for cost analysis operations
 */

import { NextRequest, NextResponse } from 'next/server';
import { CostBreakdownAnalyzer } from '../cost/breakdown';
import { CostTrendAnalyzer } from '../cost/trends';
import { CostForecaster } from '../cost/forecast';
import { CostOptimizer } from '../cost/optimization';
import { CostComparator } from '../cost/comparison';
import { CostAnalysisCache } from '../cost/cache';

const breakdownAnalyzer = new CostBreakdownAnalyzer();
const trendAnalyzer = new CostTrendAnalyzer();
const costForecaster = new CostForecaster();
const costOptimizer = new CostOptimizer();
const costComparator = new CostComparator();

const costCache = new CostAnalysisCache({
  enabled: true,
  ttl: 300,
  maxSize: 1000
});

/**
 * Cost breakdown analysis
 */
export async function breakdown(request: NextRequest): Promise<NextResponse> {
  try {
    const body = await request.json();
    const cacheKey = CostAnalysisCache.generateKey('breakdown', body);
    
    const result = await costCache.getOrSet(cacheKey, async () => {
      return await breakdownAnalyzer.analyzeBreakdown(body);
    });

    return NextResponse.json(result);
  } catch (error) {
    console.error('Cost breakdown error:', error);
    return NextResponse.json(
      { error: 'Cost breakdown failed', message: error.message },
      { status: 500 }
    );
  }
}

/**
 * Cost trends analysis
 */
export async function trends(request: NextRequest): Promise<NextResponse> {
  try {
    const body = await request.json();
    const cacheKey = CostAnalysisCache.generateKey('trends', body);
    
    const result = await costCache.getOrSet(cacheKey, async () => {
      return await trendAnalyzer.analyzeTrends(body);
    });

    return NextResponse.json(result);
  } catch (error) {
    console.error('Cost trends error:', error);
    return NextResponse.json(
      { error: 'Cost trends analysis failed', message: error.message },
      { status: 500 }
    );
  }
}

/**
 * Cost forecast
 */
export async function forecast(request: NextRequest): Promise<NextResponse> {
  try {
    const body = await request.json();
    const cacheKey = CostAnalysisCache.generateKey('forecast', body);
    
    const result = await costCache.getOrSet(cacheKey, async () => {
      return await costForecaster.generateForecast(body);
    });

    return NextResponse.json(result);
  } catch (error) {
    console.error('Cost forecast error:', error);
    return NextResponse.json(
      { error: 'Cost forecast failed', message: error.message },
      { status: 500 }
    );
  }
}

/**
 * Cost optimization recommendations
 */
export async function optimization(request: NextRequest): Promise<NextResponse> {
  try {
    const body = await request.json();
    const cacheKey = CostAnalysisCache.generateKey('optimization', body);
    
    const result = await costCache.getOrSet(cacheKey, async () => {
      return await costOptimizer.generateRecommendations(body);
    });

    return NextResponse.json(result);
  } catch (error) {
    console.error('Cost optimization error:', error);
    return NextResponse.json(
      { error: 'Cost optimization failed', message: error.message },
      { status: 500 }
    );
  }
}

/**
 * Cost comparison
 */
export async function comparison(request: NextRequest): Promise<NextResponse> {
  try {
    const body = await request.json();
    const cacheKey = CostAnalysisCache.generateKey('comparison', body);
    
    const result = await costCache.getOrSet(cacheKey, async () => {
      return await costComparator.compareCosts(body);
    });

    return NextResponse.json(result);
  } catch (error) {
    console.error('Cost comparison error:', error);
    return NextResponse.json(
      { error: 'Cost comparison failed', message: error.message },
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
    service: 'cost-analysis-api'
  });
}
