/**
 * Dashboard Data Transformer
 * 
 * Transforms raw analytics data into dashboard-ready formats
 * Handles data normalization, formatting, and structure conversion
 */

import { QueryResultRow, AnalyticsQueryResponse } from '../../api/analytics/types';

export interface DashboardDataConfig {
  timeRange: {
    start: string;
    end: string;
  };
  granularity: 'hour' | 'day' | 'week' | 'month';
  includeEmptyBuckets: boolean;
}

export interface DashboardChartData {
  labels: string[];
  datasets: {
    label: string;
    data: number[];
    color?: string;
    metadata?: Record<string, any>;
  }[];
}

export interface DashboardMetricCard {
  title: string;
  value: number | string;
  change?: number;
  changeType?: 'increase' | 'decrease';
  unit?: string;
  metadata?: Record<string, any>;
}

export interface DashboardTableData {
  columns: {
    key: string;
    label: string;
    type: 'text' | 'number' | 'date' | 'percentage';
  }[];
  rows: Record<string, any>[];
  pagination?: {
    total: number;
    page: number;
    pageSize: number;
  };
}

/**
 * Main transformer class for dashboard data processing
 */
export class DashboardTransformer {
  /**
   * Transform raw query results into chart-ready format
   */
  static toChartData(
    results: QueryResultRow[],
    config: DashboardDataConfig
  ): DashboardChartData {
    const labels = this.extractTimeLabels(results, config);
    const datasets = this.extractDatasets(results);

    return {
      labels,
      datasets
    };
  }

  /**
   * Transform query results into metric card format
   */
  static toMetricCards(
    results: QueryResultRow[],
    previousPeriodResults?: QueryResultRow[]
  ): DashboardMetricCard[] {
    const cards: DashboardMetricCard[] = [];

    // Extract metrics from results
    if (results.length > 0) {
      const metrics = results[0].metrics;
      
      for (const [key, value] of Object.entries(metrics)) {
        const card: DashboardMetricCard = {
          title: this.formatMetricName(key),
          value: this.formatMetricValue(value, key)
        };

        // Calculate change if previous period data available
        if (previousPeriodResults && previousPeriodResults.length > 0) {
          const previousValue = previousPeriodResults[0].metrics[key];
          const change = this.calculateChange(value, previousValue);
          
          if (change !== null) {
            card.change = Math.abs(change);
            card.changeType = change >= 0 ? 'increase' : 'decrease';
          }
        }

        cards.push(card);
      }
    }

    return cards;
  }

  /**
   * Transform query results into table format
   */
  static toTableData(
    results: QueryResultRow[],
    columnConfig?: Record<string, { label: string; type: string }>
  ): DashboardTableData {
    if (results.length === 0) {
      return {
        columns: [],
        rows: []
      };
    }

    // Extract columns from first result
    const columns = this.extractColumns(results[0], columnConfig);
    
    // Transform rows
    const rows = results.map(row => this.flattenRow(row));

    return {
      columns,
      rows
    };
  }

  /**
   * Transform for time-series specific processing
   */
  static toTimeSeries(
    results: QueryResultRow[],
    config: DashboardDataConfig
  ): DashboardChartData {
    const timeSeriesData = this.groupByTime(results, config);
    
    return {
      labels: timeSeriesData.labels,
      datasets: timeSeriesData.datasets
    };
  }

  /**
   * Normalize data across different time ranges
   */
  static normalizeData(
    currentData: QueryResultRow[],
    previousData: QueryResultRow[],
    config: DashboardDataConfig
  ): { current: QueryResultRow[]; previous: QueryResultRow[] } {
    // Align time buckets between current and previous periods
    const currentBuckets = this.extractTimeLabels(currentData, config);
    const previousBuckets = this.extractTimeLabels(previousData, config);

    // Fill missing buckets if requested
    if (config.includeEmptyBuckets) {
      return {
        current: this.fillMissingBuckets(currentData, currentBuckets, config),
        previous: this.fillMissingBuckets(previousData, previousBuckets, config)
      };
    }

    return { current: currentData, previous: previousData };
  }

  // Helper methods

  private static extractTimeLabels(
    results: QueryResultRow[],
    config: DashboardDataConfig
  ): string[] {
    return results
      .map(row => row.timestamp || '')
      .filter(Boolean)
      .sort();
  }

  private static extractDatasets(results: QueryResultRow[]): DashboardChartData['datasets'] {
    if (results.length === 0) return [];

    const metricKeys = Object.keys(results[0].metrics || {});
    
    return metricKeys.map((key, index) => ({
      label: this.formatMetricName(key),
      data: results.map(row => row.metrics?.[key] || 0),
      color: this.getColorForIndex(index)
    }));
  }

  private static formatMetricName(key: string): string {
    return key
      .replace(/([A-Z])/g, ' $1')
      .replace(/^./, str => str.toUpperCase())
      .trim();
  }

  private static formatMetricValue(value: number, metric: string): string {
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

  private static calculateChange(current: number, previous: number): number | null {
    if (previous === 0) return null;
    return ((current - previous) / previous) * 100;
  }

  private static extractColumns(
    row: QueryResultRow,
    columnConfig?: Record<string, { label: string; type: string }>
  ): DashboardTableData['columns'] {
    const columns: DashboardTableData['columns'] = [];
    
    // Add dimension columns
    if (row.dimensions) {
      for (const [key, value] of Object.entries(row.dimensions)) {
        columns.push({
          key,
          label: columnConfig?.[key]?.label || this.formatMetricName(key),
          type: (columnConfig?.[key]?.type as any) || 'text'
        });
      }
    }

    // Add metric columns
    if (row.metrics) {
      for (const [key] of Object.entries(row.metrics)) {
        columns.push({
          key,
          label: columnConfig?.[key]?.label || this.formatMetricName(key),
          type: (columnConfig?.[key]?.type as any) || 'number'
        });
      }
    }

    return columns;
  }

  private static flattenRow(row: QueryResultRow): Record<string, any> {
    return {
      ...row.dimensions,
      ...row.metrics,
      timestamp: row.timestamp
    };
  }

  private static groupByTime(
    results: QueryResultRow[],
    config: DashboardDataConfig
  ): DashboardChartData {
    const grouped = new Map<string, Record<string, number>>();

    for (const row of results) {
      const timeKey = this.formatTimeKey(row.timestamp || '', config.granularity);
      
      if (!grouped.has(timeKey)) {
        grouped.set(timeKey, {});
      }

      const bucket = grouped.get(timeKey)!;
      for (const [key, value] of Object.entries(row.metrics || {})) {
        bucket[key] = (bucket[key] || 0) + value;
      }
    }

    const labels = Array.from(grouped.keys()).sort();
    const allMetrics = new Set<string>();
    
    for (const bucket of grouped.values()) {
      for (const key of Object.keys(bucket)) {
        allMetrics.add(key);
      }
    }

    const datasets = Array.from(allMetrics).map((metric, index) => ({
      label: this.formatMetricName(metric),
      data: labels.map(label => grouped.get(label)?.[metric] || 0),
      color: this.getColorForIndex(index)
    }));

    return { labels, datasets };
  }

  private static formatTimeKey(timestamp: string, granularity: string): string {
    const date = new Date(timestamp);
    
    switch (granularity) {
      case 'hour':
        return date.toISOString().slice(0, 13) + ':00';
      case 'day':
        return date.toISOString().slice(0, 10);
      case 'week':
        const weekStart = new Date(date);
        weekStart.setDate(date.getDate() - date.getDay());
        return weekStart.toISOString().slice(0, 10);
      case 'month':
        return date.toISOString().slice(0, 7);
      default:
        return timestamp;
    }
  }

  private static fillMissingBuckets(
    results: QueryResultRow[],
    labels: string[],
    config: DashboardDataConfig
  ): QueryResultRow[] {
    const filled: QueryResultRow[] = [...results];
    const labelSet = new Set(labels);

    // Generate expected time range
    const expectedLabels = this.generateTimeLabels(config.timeRange.start, config.timeRange.end, config.granularity);

    for (const expectedLabel of expectedLabels) {
      if (!labelSet.has(expectedLabel)) {
        filled.push({
          timestamp: expectedLabel,
          metrics: {}
        });
      }
    }

    return filled.sort((a, b) => (a.timestamp || '').localeCompare(b.timestamp || ''));
  }

  private static generateTimeLabels(start: string, end: string, granularity: string): string[] {
    const labels: string[] = [];
    const current = new Date(start);
    const endDate = new Date(end);

    while (current <= endDate) {
      labels.push(this.formatTimeKey(current.toISOString(), granularity));
      
      switch (granularity) {
        case 'hour':
          current.setHours(current.getHours() + 1);
          break;
        case 'day':
          current.setDate(current.getDate() + 1);
          break;
        case 'week':
          current.setDate(current.getDate() + 7);
          break;
        case 'month':
          current.setMonth(current.getMonth() + 1);
          break;
      }
    }

    return labels;
  }

  private static getColorForIndex(index: number): string {
    const colors = [
      '#3b82f6', // blue
      '#10b981', // green
      '#f59e0b', // amber
      '#ef4444', // red
      '#8b5cf6', // violet
      '#ec4899', // pink
      '#06b6d4', // cyan
      '#84cc16'  // lime
    ];
    return colors[index % colors.length];
  }
}

/**
 * Utility functions for common transformations
 */
export const TransformerUtils = {
  /**
   * Convert query response to multiple dashboard formats
   */
  transformQueryResponse(
    response: AnalyticsQueryResponse,
    formats: Array<'chart' | 'metric' | 'table' | 'timeseries'>,
    config?: DashboardDataConfig
  ) {
    const results: Record<string, any> = {};

    if (formats.includes('chart')) {
      results.chart = DashboardTransformer.toChartData(response.data, config || {
        timeRange: response.metadata.timeRange,
        granularity: 'day',
        includeEmptyBuckets: false
      });
    }

    if (formats.includes('timeseries')) {
      results.timeseries = DashboardTransformer.toTimeSeries(response.data, config || {
        timeRange: response.metadata.timeRange,
        granularity: 'day',
        includeEmptyBuckets: false
      });
    }

    if (formats.includes('metric')) {
      results.metrics = DashboardTransformer.toMetricCards(response.data);
    }

    if (formats.includes('table')) {
      results.table = DashboardTransformer.toTableData(response.data);
    }

    return results;
  },

  /**
   * Merge multiple query responses for combined dashboard
   */
  mergeQueryResponses(responses: AnalyticsQueryResponse[]): AnalyticsQueryResponse {
    if (responses.length === 0) {
      throw new Error('No responses to merge');
    }

    const merged = {
      ...responses[0],
      data: responses.flatMap(r => r.data),
      executionTimeMs: responses.reduce((sum, r) => sum + r.executionTimeMs, 0),
      pagination: {
        total: responses.reduce((sum, r) => sum + r.pagination.total, 0),
        limit: responses[0].pagination.limit,
        offset: responses[0].pagination.offset,
        hasMore: responses.some(r => r.pagination.hasMore)
      }
    };

    return merged;
  }
};
