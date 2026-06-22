/**
 * Dashboard Aggregation Functions
 * 
 * Provides dashboard-specific aggregation operations
 * Optimized for common dashboard metrics and KPIs
 */

import { QueryResultRow, AnalyticsQueryResponse } from '../../api/analytics/types';

export interface AggregationConfig {
  groupBy?: string[];
  timeWindow?: {
    size: number;
    unit: 'minutes' | 'hours' | 'days';
  };
  filters?: Array<{
    field: string;
    operator: string;
    value: any;
  }>;
}

export interface AggregatedMetric {
  name: string;
  value: number;
  formatted: string;
  trend?: {
    value: number;
    period: string;
  };
  breakdown?: Record<string, number>;
}

export interface DashboardAggregationResult {
  metrics: AggregatedMetric[];
  timeSeries: {
    timestamp: string;
    values: Record<string, number>;
  }[];
  summary: {
    totalRecords: number;
    timeRange: { start: string; end: string };
    aggregatedAt: string;
  };
}

/**
 * Dashboard-specific aggregator
 */
export class DashboardAggregator {
  /**
   * Aggregate query results for dashboard overview
   */
  static aggregateOverview(
    results: QueryResultRow[],
    config: AggregationConfig = {}
  ): DashboardAggregationResult {
    const metrics = this.calculateOverviewMetrics(results);
    const timeSeries = this.calculateTimeSeries(results, config);
    
    return {
      metrics,
      timeSeries,
      summary: {
        totalRecords: results.length,
        timeRange: this.extractTimeRange(results),
        aggregatedAt: new Date().toISOString()
      }
    };
  }

  /**
   * Aggregate by specific dimension (e.g., by AI client, by provider)
   */
  static aggregateByDimension(
    results: QueryResultRow[],
    dimension: string,
    config: AggregationConfig = {}
  ): Record<string, AggregatedMetric[]> {
    const grouped = this.groupBy(results, dimension);
    const aggregated: Record<string, AggregatedMetric[]> = {};

    for (const [key, group] of Object.entries(grouped)) {
      aggregated[key] = this.calculateOverviewMetrics(group);
    }

    return aggregated;
  }

  /**
   * Calculate top N items by metric
   */
  static getTopN(
    results: QueryResultRow[],
    metric: string,
    n: number = 10
  ): Array<{ key: string; value: number; label: string }> {
    const dimensionValues = this.extractDimensionValues(results);
    
    return dimensionValues
      .map(item => ({
        key: item.key,
        value: item.values[metric] || 0,
        label: item.key
      }))
      .sort((a, b) => b.value - a.value)
      .slice(0, n);
  }

  /**
   * Calculate percentile distributions
   */
  static calculatePercentiles(
    results: QueryResultRow[],
    metric: string,
    percentiles: number[] = [50, 90, 95, 99]
  ): Record<number, number> {
    const values = results
      .map(row => row.metrics?.[metric] || 0)
      .filter(v => v > 0)
      .sort((a, b) => a - b);

    const result: Record<number, number> = {};
    
    for (const p of percentiles) {
      const index = Math.ceil((p / 100) * values.length) - 1;
      result[p] = values[index] || 0;
    }

    return result;
  }

  /**
   * Calculate moving average
   */
  static calculateMovingAverage(
    results: QueryResultRow[],
    metric: string,
    windowSize: number = 7
  ): QueryResultRow[] {
    const sorted = [...results].sort((a, b) => 
      (a.timestamp || '').localeCompare(b.timestamp || '')
    );

    return sorted.map((row, index) => {
      const start = Math.max(0, index - windowSize + 1);
      const window = sorted.slice(start, index + 1);
      
      const avg = window.reduce((sum, r) => sum + (r.metrics?.[metric] || 0), 0) / window.length;

      return {
        ...row,
        metrics: {
          ...row.metrics,
          [`${metric}_moving_avg`]: avg
        }
      };
    });
  }

  /**
   * Calculate growth rates
   */
  static calculateGrowthRates(
    results: QueryResultRow[],
    metric: string,
    period: 'day' | 'week' | 'month' = 'day'
  ): QueryResultRow[] {
    const sorted = [...results].sort((a, b) => 
      (a.timestamp || '').localeCompare(b.timestamp || '')
    );

    const periodMs = period === 'day' ? 24 * 60 * 60 * 1000 :
                    period === 'week' ? 7 * 24 * 60 * 60 * 1000 :
                    30 * 24 * 60 * 60 * 1000;

    return sorted.map((row, index) => {
      if (index === 0) {
        return row;
      }

      const current = row.metrics?.[metric] || 0;
      const previous = sorted[index - 1].metrics?.[metric] || 0;
      
      const growth = previous > 0 ? ((current - previous) / previous) * 100 : 0;

      return {
        ...row,
        metrics: {
          ...row.metrics,
          [`${metric}_growth_rate`]: growth
        }
      };
    });
  }

  /**
   * Calculate anomaly detection
   */
  static detectAnomalies(
    results: QueryResultRow[],
    metric: string,
    threshold: number = 2 // standard deviations
  ): QueryResultRow[] {
    const values = results.map(row => row.metrics?.[metric] || 0);
    const mean = values.reduce((sum, v) => sum + v, 0) / values.length;
    const variance = values.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);

    return results.filter(row => {
      const value = row.metrics?.[metric] || 0;
      const zScore = Math.abs((value - mean) / stdDev);
      return zScore > threshold;
    }).map(row => ({
      ...row,
      metrics: {
        ...row.metrics,
        [`${metric}_anomaly_score`]: Math.abs((row.metrics?.[metric] || 0 - mean) / stdDev)
      }
    }));
  }

  // Helper methods

  private static calculateOverviewMetrics(results: QueryResultRow[]): AggregatedMetric[] {
    if (results.length === 0) return [];

    const metrics: AggregatedMetric[] = [];
    const allMetrics = new Set<string>();

    // Collect all metric names
    for (const row of results) {
      if (row.metrics) {
        for (const key of Object.keys(row.metrics)) {
          allMetrics.add(key);
        }
      }
    }

    // Calculate aggregates for each metric
    for (const metricName of allMetrics) {
      const values = results.map(row => row.metrics?.[metricName] || 0);
      const sum = values.reduce((a, b) => a + b, 0);
      const avg = sum / values.length;
      const max = Math.max(...values);
      const min = Math.min(...values);

      metrics.push({
        name: metricName,
        value: sum,
        formatted: this.formatValue(sum, metricName),
        breakdown: {
          sum,
          average: avg,
          max,
          min,
          count: values.length
        }
      });
    }

    return metrics;
  }

  private static calculateTimeSeries(
    results: QueryResultRow[],
    config: AggregationConfig
  ): Array<{ timestamp: string; values: Record<string, number> }> {
    const sorted = [...results].sort((a, b) => 
      (a.timestamp || '').localeCompare(b.timestamp || '')
    );

    // Apply time window if specified
    if (config.timeWindow) {
      return this.applyTimeWindow(sorted, config.timeWindow);
    }

    return sorted.map(row => ({
      timestamp: row.timestamp || '',
      values: row.metrics || {}
    }));
  }

  private static applyTimeWindow(
    results: QueryResultRow[],
    window: { size: number; unit: string }
  ): Array<{ timestamp: string; values: Record<string, number> }> {
    const windowMs = window.size * (
      window.unit === 'minutes' ? 60 * 1000 :
      window.unit === 'hours' ? 60 * 60 * 1000 :
      24 * 60 * 60 * 1000
    );

    const windows: Array<{ timestamp: string; values: Record<string, number> }> = [];
    let currentWindow: QueryResultRow[] = [];
    let windowStart: Date | null = null;

    for (const row of results) {
      const rowTime = new Date(row.timestamp || '');

      if (!windowStart) {
        windowStart = rowTime;
      }

      if (rowTime.getTime() - windowStart.getTime() <= windowMs) {
        currentWindow.push(row);
      } else {
        // Aggregate current window
        if (currentWindow.length > 0) {
          windows.push(this.aggregateWindow(currentWindow, windowStart.toISOString()));
        }
        
        // Start new window
        currentWindow = [row];
        windowStart = rowTime;
      }
    }

    // Add last window
    if (currentWindow.length > 0) {
      windows.push(this.aggregateWindow(currentWindow, windowStart?.toISOString() || ''));
    }

    return windows;
  }

  private static aggregateWindow(
    results: QueryResultRow[],
    timestamp: string
  ): { timestamp: string; values: Record<string, number> } {
    const values: Record<string, number> = {};

    for (const row of results) {
      if (row.metrics) {
        for (const [key, value] of Object.entries(row.metrics)) {
          values[key] = (values[key] || 0) + value;
        }
      }
    }

    return { timestamp, values };
  }

  private static groupBy(
    results: QueryResultRow[],
    dimension: string
  ): Record<string, QueryResultRow[]> {
    const grouped: Record<string, QueryResultRow[]> = {};

    for (const row of results) {
      const key = row.dimensions?.[dimension] || 'unknown';
      
      if (!grouped[key]) {
        grouped[key] = [];
      }
      
      grouped[key].push(row);
    }

    return grouped;
  }

  private static extractDimensionValues(results: QueryResultRow[]): Array<{
    key: string;
    values: Record<string, number>;
  }> {
    const dimensionMap = new Map<string, Record<string, number>>();

    for (const row of results) {
      if (row.dimensions) {
        for (const [key, value] of Object.entries(row.dimensions)) {
          if (!dimensionMap.has(String(value))) {
            dimensionMap.set(String(value), {});
          }

          const metricValues = dimensionMap.get(String(value))!;
          if (row.metrics) {
            for (const [metric, val] of Object.entries(row.metrics)) {
              metricValues[metric] = (metricValues[metric] || 0) + val;
            }
          }
        }
      }
    }

    return Array.from(dimensionMap.entries()).map(([key, values]) => ({
      key,
      values
    }));
  }

  private static extractTimeRange(results: QueryResultRow[]): { start: string; end: string } {
    if (results.length === 0) {
      return {
        start: new Date().toISOString(),
        end: new Date().toISOString()
      };
    }

    const timestamps = results
      .map(row => row.timestamp)
      .filter(Boolean) as string[];

    return {
      start: timestamps.sort()[0] || new Date().toISOString(),
      end: timestamps.sort().reverse()[0] || new Date().toISOString()
    };
  }

  private static formatValue(value: number, metric: string): string {
    if (metric.toLowerCase().includes('cost')) {
      return `$${value.toFixed(2)}`;
    }
    if (metric.toLowerCase().includes('rate') || metric.toLowerCase().includes('percentage')) {
      return `${(value * 100).toFixed(1)}%`;
    }
    if (metric.toLowerCase().includes('time') || metric.toLowerCase().includes('latency')) {
      return `${value.toFixed(0)}ms`;
    }
    if (value >= 1000000) {
      return `${(value / 1000000).toFixed(1)}M`;
    }
    if (value >= 1000) {
      return `${(value / 1000).toFixed(1)}K`;
    }
    return value.toFixed(0);
  }
}

/**
 * Pre-built aggregation patterns for common dashboard use cases
 */
export const AggregationPatterns = {
  /**
   * Daily usage aggregation
   */
  dailyUsage(results: QueryResultRow[]): DashboardAggregationResult {
    return DashboardAggregator.aggregateOverview(results, {
      timeWindow: { size: 1, unit: 'days' }
    });
  },

  /**
   * Cost breakdown by provider
   */
  costByProvider(results: QueryResultRow[]): Record<string, AggregatedMetric[]> {
    return DashboardAggregator.aggregateByDimension(results, 'provider');
  },

  /**
   * Performance metrics by AI client
   */
  performanceByClient(results: QueryResultRow[]): Record<string, AggregatedMetric[]> {
    return DashboardAggregator.aggregateByDimension(results, 'aiClient');
  },

  /**
   * Top models by usage
   */
  topModels(results: QueryResultRow[], n: number = 10) {
    return DashboardAggregator.getTopN(results, 'requestCount', n);
  },

  /**
   * Latency percentiles
   */
  latencyPercentiles(results: QueryResultRow[]) {
    return DashboardAggregator.calculatePercentiles(results, 'latency', [50, 90, 95, 99]);
  }
};
