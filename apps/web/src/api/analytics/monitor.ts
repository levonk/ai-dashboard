/**
 * Query Performance Monitor and Optimizer
 * 
 * Monitors query performance and provides optimization recommendations
 */

import {
  QueryPerformanceMetrics,
  AnalyticsQueryRequest
} from './types';

export class QueryMonitor {
  private metrics: Map<string, QueryPerformanceMetrics[]>;
  private slowQueryThreshold: number; // milliseconds

  constructor(slowQueryThreshold: number = 5000) {
    this.metrics = new Map();
    this.slowQueryThreshold = slowQueryThreshold;
  }

  /**
   * Record query execution metrics
   */
  recordMetrics(metrics: QueryPerformanceMetrics): void {
    const queryHash = this.hashQuery(metrics.queryId);
    
    if (!this.metrics.has(queryHash)) {
      this.metrics.set(queryHash, []);
    }

    const queryMetrics = this.metrics.get(queryHash)!;
    queryMetrics.push(metrics);

    // Keep only last 100 metrics per query
    if (queryMetrics.length > 100) {
      queryMetrics.shift();
    }

    // Alert if query is slow
    if (metrics.executionTimeMs > this.slowQueryThreshold) {
      this.alertSlowQuery(metrics);
    }
  }

  /**
   * Get performance statistics for a query
   */
  getQueryStats(queryId: string): {
    avgExecutionTime: number;
    minExecutionTime: number;
    maxExecutionTime: number;
    cacheHitRate: number;
    totalExecutions: number;
    avgResultCount: number;
  } | null {
    const queryHash = this.hashQuery(queryId);
    const queryMetrics = this.metrics.get(queryHash);

    if (!queryMetrics || queryMetrics.length === 0) {
      return null;
    }

    const executionTimes = queryMetrics.map(m => m.executionTimeMs);
    const cacheHits = queryMetrics.filter(m => m.cacheHit).length;
    const resultCounts = queryMetrics.map(m => m.resultCount);

    return {
      avgExecutionTime: this.average(executionTimes),
      minExecutionTime: Math.min(...executionTimes),
      maxExecutionTime: Math.max(...executionTimes),
      cacheHitRate: cacheHits / queryMetrics.length,
      totalExecutions: queryMetrics.length,
      avgResultCount: this.average(resultCounts)
    };
  }

  /**
   * Get optimization recommendations for a query
   */
  getOptimizationRecommendations(queryId: string): string[] {
    const stats = this.getQueryStats(queryId);
    if (!stats) {
      return [];
    }

    const recommendations: string[] = [];

    // Check execution time
    if (stats.avgExecutionTime > this.slowQueryThreshold) {
      recommendations.push('Query execution time is slow. Consider adding caching or optimizing filters.');
    }

    // Check cache hit rate
    if (stats.cacheHitRate < 0.5 && stats.totalExecutions > 10) {
      recommendations.push('Low cache hit rate. Consider increasing cache TTL or adjusting cache key strategy.');
    }

    // Check result count
    if (stats.avgResultCount > 1000) {
      recommendations.push('Large result sets. Consider adding pagination or more specific filters.');
    }

    // Check execution count
    if (stats.totalExecutions > 100 && stats.cacheHitRate < 0.8) {
      recommendations.push('Frequently executed query with low cache hit rate. Review cache configuration.');
    }

    return recommendations;
  }

  /**
   * Get all slow queries
   */
  getSlowQueries(): QueryPerformanceMetrics[] {
    const slowQueries: QueryPerformanceMetrics[] = [];

    for (const metrics of this.metrics.values()) {
      for (const metric of metrics) {
        if (metric.executionTimeMs > this.slowQueryThreshold) {
          slowQueries.push(metric);
        }
      }
    }

    return slowQueries.sort((a, b) => b.executionTimeMs - a.executionTimeMs);
  }

  /**
   * Get overall system performance statistics
   */
  getSystemStats(): {
    totalQueries: number;
    avgExecutionTime: number;
    slowQueryCount: number;
    cacheHitRate: number;
    topSlowQueries: QueryPerformanceMetrics[];
  } {
    let totalQueries = 0;
    let totalExecutionTime = 0;
    let slowQueryCount = 0;
    let cacheHits = 0;

    for (const metrics of this.metrics.values()) {
      for (const metric of metrics) {
        totalQueries++;
        totalExecutionTime += metric.executionTimeMs;
        
        if (metric.executionTimeMs > this.slowQueryThreshold) {
          slowQueryCount++;
        }
        
        if (metric.cacheHit) {
          cacheHits++;
        }
      }
    }

    const avgExecutionTime = totalQueries > 0 ? totalExecutionTime / totalQueries : 0;
    const cacheHitRate = totalQueries > 0 ? cacheHits / totalQueries : 0;

    return {
      totalQueries,
      avgExecutionTime,
      slowQueryCount,
      cacheHitRate,
      topSlowQueries: this.getSlowQueries().slice(0, 10)
    };
  }

  /**
   * Clear metrics for a specific query
   */
  clearQueryMetrics(queryId: string): void {
    const queryHash = this.hashQuery(queryId);
    this.metrics.delete(queryHash);
  }

  /**
   * Clear all metrics
   */
  clearAllMetrics(): void {
    this.metrics.clear();
  }

  /**
   * Set slow query threshold
   */
  setSlowQueryThreshold(threshold: number): void {
    this.slowQueryThreshold = threshold;
  }

  /**
   * Alert on slow query
   */
  private alertSlowQuery(metrics: QueryPerformanceMetrics): void {
    // TODO: Integrate with alerting system
    console.warn(`Slow query detected: ${metrics.queryId} took ${metrics.executionTimeMs}ms`);
  }

  /**
   * Hash query ID for storage
   */
  private hashQuery(queryId: string): string {
    let hash = 0;
    for (let i = 0; i < queryId.length; i++) {
      const char = queryId.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash).toString(16);
  }

  /**
   * Calculate average of array
   */
  private average(arr: number[]): number {
    if (arr.length === 0) return 0;
    return arr.reduce((sum, val) => sum + val, 0) / arr.length;
  }
}

/**
 * Query optimizer that suggests query improvements
 */
export class QueryOptimizer {
  /**
   * Analyze a query and suggest optimizations
   */
  static optimizeQuery(request: AnalyticsQueryRequest): {
    optimized: AnalyticsQueryRequest;
    suggestions: string[];
  } {
    const suggestions: string[] = [];
    const optimized = JSON.parse(JSON.stringify(request));

    // Suggest adding time limit if not present
    if (!optimized.limit || optimized.limit > 1000) {
      suggestions.push('Consider adding a limit to reduce result set size');
      optimized.limit = Math.min(optimized.limit || 100, 1000);
    }

    // Suggest adding filters if none present
    if (!optimized.filters || optimized.filters.length === 0) {
      suggestions.push('Consider adding filters to reduce data scanned');
    }

    // Suggest grouping if many dimensions selected
    const dimensionCount = Object.values(optimized.dimensions || {}).filter(arr => arr && arr.length > 0).length;
    if (dimensionCount > 3 && !optimized.groupBy) {
      suggestions.push('Consider adding groupBy to aggregate results by key dimensions');
    }

    // Suggest time range reduction if very large
    if (optimized.startTime && optimized.endTime) {
      const start = new Date(optimized.startTime);
      const end = new Date(optimized.endTime);
      const daysDiff = (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24);
      
      if (daysDiff > 30) {
        suggestions.push('Consider reducing time range to improve query performance');
      }
    }

    return {
      optimized,
      suggestions
    };
  }
}

/**
 * Singleton instance of the query monitor
 */
export const queryMonitor = new QueryMonitor();
