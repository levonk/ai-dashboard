/**
 * Dashboard Processing Tests
 * 
 * Comprehensive tests for dashboard data processing modules
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { DashboardTransformer } from '../transformer';
import { DashboardAggregator } from '../aggregator';
import { TimeSeriesProcessor } from '../timeseries';
import { DashboardCache } from '../cache';
import { DashboardValidator } from '../validator';
import { RealtimeDashboardProcessor } from '../realtime';
import { DashboardExporter } from '../export';
import { QueryResultRow, AnalyticsQueryResponse } from '../../../api/analytics/types';

// Mock data for testing
const mockQueryResults: QueryResultRow[] = [
  {
    timestamp: '2024-01-01T00:00:00Z',
    dimensions: {
      aiClient: 'claude-code',
      provider: 'anthropic',
      model: 'claude-3.5-opus'
    },
    metrics: {
      requestCount: 100,
      tokenCount: 50000,
      cost: 0.50,
      latency: 250
    }
  },
  {
    timestamp: '2024-01-01T01:00:00Z',
    dimensions: {
      aiClient: 'claude-code',
      provider: 'anthropic',
      model: 'claude-3.5-opus'
    },
    metrics: {
      requestCount: 120,
      tokenCount: 60000,
      cost: 0.60,
      latency: 280
    }
  },
  {
    timestamp: '2024-01-01T02:00:00Z',
    dimensions: {
      aiClient: 'codex',
      provider: 'openai',
      model: 'gpt-4'
    },
    metrics: {
      requestCount: 80,
      tokenCount: 40000,
      cost: 0.40,
      latency: 300
    }
  }
];

const mockAnalyticsResponse: AnalyticsQueryResponse = {
  queryId: 'test-query-123',
  executionTimeMs: 150,
  cached: false,
  data: mockQueryResults,
  pagination: {
    total: 3,
    limit: 100,
    offset: 0,
    hasMore: false
  },
  metadata: {
    timeRange: {
      start: '2024-01-01T00:00:00Z',
      end: '2024-01-01T23:59:59Z'
    },
    dimensions: ['aiClient', 'provider', 'model'],
    metrics: ['requestCount', 'tokenCount', 'cost', 'latency']
  }
};

describe('DashboardTransformer', () => {
  describe('toChartData', () => {
    it('should transform query results to chart data format', () => {
      const config = {
        timeRange: {
          start: '2024-01-01T00:00:00Z',
          end: '2024-01-01T23:59:59Z'
        },
        granularity: 'hour' as const,
        includeEmptyBuckets: false
      };

      const chartData = DashboardTransformer.toChartData(mockQueryResults, config);

      expect(chartData).toHaveProperty('labels');
      expect(chartData).toHaveProperty('datasets');
      expect(Array.isArray(chartData.labels)).toBe(true);
      expect(Array.isArray(chartData.datasets)).toBe(true);
      expect(chartData.labels.length).toBeGreaterThan(0);
    });

    it('should handle empty results', () => {
      const config = {
        timeRange: {
          start: '2024-01-01T00:00:00Z',
          end: '2024-01-01T23:59:59Z'
        },
        granularity: 'hour' as const,
        includeEmptyBuckets: false
      };

      const chartData = DashboardTransformer.toChartData([], config);

      expect(chartData.labels).toEqual([]);
      expect(chartData.datasets).toEqual([]);
    });
  });

  describe('toMetricCards', () => {
    it('should transform results to metric cards', () => {
      const metricCards = DashboardTransformer.toMetricCards(mockQueryResults);

      expect(Array.isArray(metricCards)).toBe(true);
      expect(metricCards.length).toBeGreaterThan(0);
      expect(metricCards[0]).toHaveProperty('title');
      expect(metricCards[0]).toHaveProperty('value');
    });

    it('should calculate change with previous period data', () => {
      const previousResults = [
        {
          timestamp: '2023-12-31T00:00:00Z',
          dimensions: {},
          metrics: {
            requestCount: 90,
            tokenCount: 45000,
            cost: 0.45,
            latency: 240
          }
        }
      ];

      const metricCards = DashboardTransformer.toMetricCards(
        mockQueryResults,
        previousResults
      );

      expect(metricCards[0]).toHaveProperty('change');
      expect(metricCards[0]).toHaveProperty('changeType');
    });
  });

  describe('toTableData', () => {
    it('should transform results to table format', () => {
      const tableData = DashboardTransformer.toTableData(mockQueryResults);

      expect(tableData).toHaveProperty('columns');
      expect(tableData).toHaveProperty('rows');
      expect(Array.isArray(tableData.columns)).toBe(true);
      expect(Array.isArray(tableData.rows)).toBe(true);
    });

    it('should handle empty results', () => {
      const tableData = DashboardTransformer.toTableData([]);

      expect(tableData.columns).toEqual([]);
      expect(tableData.rows).toEqual([]);
    });
  });

  describe('normalizeData', () => {
    it('should normalize data across time ranges', () => {
      const config = {
        timeRange: {
          start: '2024-01-01T00:00:00Z',
          end: '2024-01-01T23:59:59Z'
        },
        granularity: 'hour' as const,
        includeEmptyBuckets: false
      };

      const normalized = DashboardTransformer.normalizeData(
        mockQueryResults,
        mockQueryResults,
        config
      );

      expect(normalized).toHaveProperty('current');
      expect(normalized).toHaveProperty('previous');
    });
  });
});

describe('DashboardAggregator', () => {
  describe('aggregateOverview', () => {
    it('should aggregate results for overview', () => {
      const config = {};

      const aggregation = DashboardAggregator.aggregateOverview(
        mockQueryResults,
        config
      );

      expect(aggregation).toHaveProperty('metrics');
      expect(aggregation).toHaveProperty('timeSeries');
      expect(aggregation).toHaveProperty('summary');
      expect(Array.isArray(aggregation.metrics)).toBe(true);
    });

    it('should calculate correct summary statistics', () => {
      const aggregation = DashboardAggregator.aggregateOverview(mockQueryResults);

      expect(aggregation.summary.totalRecords).toBe(mockQueryResults.length);
      expect(aggregation.summary).toHaveProperty('timeRange');
      expect(aggregation.summary).toHaveProperty('aggregatedAt');
    });
  });

  describe('aggregateByDimension', () => {
    it('should aggregate by specific dimension', () => {
      const aggregation = DashboardAggregator.aggregateByDimension(
        mockQueryResults,
        'aiClient'
      );

      expect(typeof aggregation).toBe('object');
      expect(Object.keys(aggregation).length).toBeGreaterThan(0);
    });
  });

  describe('getTopN', () => {
    it('should return top N items by metric', () => {
      const topN = DashboardAggregator.getTopN(mockQueryResults, 'requestCount', 2);

      expect(Array.isArray(topN)).toBe(true);
      expect(topN.length).toBeLessThanOrEqual(2);
      expect(topN[0]).toHaveProperty('key');
      expect(topN[0]).toHaveProperty('value');
    });

    it('should sort results by value descending', () => {
      const topN = DashboardAggregator.getTopN(mockQueryResults, 'requestCount', 10);

      for (let i = 1; i < topN.length; i++) {
        expect(topN[i].value).toBeLessThanOrEqual(topN[i - 1].value);
      }
    });
  });

  describe('calculatePercentiles', () => {
    it('should calculate percentiles for metric', () => {
      const percentiles = DashboardAggregator.calculatePercentiles(
        mockQueryResults,
        'requestCount',
        [50, 90, 95]
      );

      expect(percentiles).toHaveProperty(50);
      expect(percentiles).toHaveProperty(90);
      expect(percentiles).toHaveProperty(95);
      expect(typeof percentiles[50]).toBe('number');
    });
  });

  describe('calculateMovingAverage', () => {
    it('should calculate moving average', () => {
      const movingAvg = DashboardAggregator.calculateMovingAverage(
        mockQueryResults,
        'requestCount',
        2
      );

      expect(Array.isArray(movingAvg)).toBe(true);
      expect(movingAvg.length).toBe(mockQueryResults.length);
      expect(movingAvg[0].metrics).toHaveProperty('requestCount_moving_avg');
    });
  });

  describe('detectAnomalies', () => {
    it('should detect anomalies in data', () => {
      const anomalies = DashboardAggregator.detectAnomalies(
        mockQueryResults,
        'requestCount',
        2
      );

      expect(Array.isArray(anomalies)).toBe(true);
    });
  });
});

describe('TimeSeriesProcessor', () => {
  describe('processToTimeSeries', () => {
    it('should process data into time-series format', () => {
      const config = {
        granularity: 'hour' as const,
        fillMissing: true,
        interpolation: 'linear' as const
      };

      const timeSeries = TimeSeriesProcessor.processToTimeSeries(
        mockQueryResults,
        'requestCount',
        config
      );

      expect(timeSeries).toHaveProperty('timestamps');
      expect(timeSeries).toHaveProperty('values');
      expect(timeSeries).toHaveProperty('metadata');
      expect(Array.isArray(timeSeries.timestamps)).toBe(true);
      expect(Array.isArray(timeSeries.values)).toBe(true);
    });
  });

  describe('resample', () => {
    it('should resample time-series to target granularity', () => {
      const config = {
        granularity: 'hour' as const,
        fillMissing: false,
        interpolation: 'none' as const
      };

      const timestamps = mockQueryResults.map(r => r.timestamp!);
      const values = mockQueryResults.map(r => r.metrics.requestCount);

      const resampled = TimeSeriesProcessor.resample(timestamps, values, config);

      expect(resampled).toHaveProperty('timestamps');
      expect(resampled).toHaveProperty('values');
      expect(resampled.timestamps.length).toBe(resampled.values.length);
    });
  });

  describe('analyzeTrend', () => {
    it('should analyze trend in time-series', () => {
      const config = {
        granularity: 'hour' as const,
        fillMissing: false,
        interpolation: 'none' as const
      };

      const timeSeries = TimeSeriesProcessor.processToTimeSeries(
        mockQueryResults,
        'requestCount',
        config
      );

      const trend = TimeSeriesProcessor.analyzeTrend(timeSeries);

      expect(trend).toHaveProperty('trend');
      expect(trend).toHaveProperty('slope');
      expect(trend).toHaveProperty('correlation');
      expect(['increasing', 'decreasing', 'stable', 'volatile']).toContain(trend.trend);
    });
  });
});

describe('DashboardCache', () => {
  let cache: DashboardCache<AnalyticsQueryResponse>;

  beforeEach(() => {
    cache = new DashboardCache({
      ttl: 1000,
      maxSize: 10,
      strategy: 'lru',
      compression: false
    });
  });

  afterEach(() => {
    cache.clear();
  });

  describe('basic operations', () => {
    it('should set and get data', () => {
      cache.set('test-key', mockAnalyticsResponse);
      const retrieved = cache.get('test-key');

      expect(retrieved).toEqual(mockAnalyticsResponse);
    });

    it('should return null for non-existent key', () => {
      const retrieved = cache.get('non-existent');
      expect(retrieved).toBeNull();
    });

    it('should delete entries', () => {
      cache.set('test-key', mockAnalyticsResponse);
      cache.delete('test-key');
      const retrieved = cache.get('test-key');

      expect(retrieved).toBeNull();
    });

    it('should clear all entries', () => {
      cache.set('key1', mockAnalyticsResponse);
      cache.set('key2', mockAnalyticsResponse);
      cache.clear();

      expect(cache.size()).toBe(0);
    });
  });

  describe('cache statistics', () => {
    it('should track cache hits and misses', () => {
      cache.set('test-key', mockAnalyticsResponse);
      cache.get('test-key'); // hit
      cache.get('non-existent'); // miss

      const stats = cache.getStats();
      expect(stats.hits).toBe(1);
      expect(stats.misses).toBe(1);
    });

    it('should calculate hit rate correctly', () => {
      cache.set('test-key', mockAnalyticsResponse);
      cache.get('test-key');
      cache.get('test-key');
      cache.get('non-existent');

      const stats = cache.getStats();
      expect(stats.hitRate).toBeCloseTo(0.666, 1);
    });
  });

  describe('TTL and expiration', () => {
    it('should expire entries after TTL', async () => {
      cache.set('test-key', mockAnalyticsResponse);
      
      // Wait for TTL to expire
      await new Promise(resolve => setTimeout(resolve, 1100));
      
      const retrieved = cache.get('test-key');
      expect(retrieved).toBeNull();
    });

    it('should clean expired entries', async () => {
      cache.set('key1', mockAnalyticsResponse);
      cache.set('key2', mockAnalyticsResponse);
      
      await new Promise(resolve => setTimeout(resolve, 1100));
      
      const cleaned = cache.cleanExpired();
      expect(cleaned).toBe(2);
    });
  });

  describe('eviction strategies', () => {
    it('should evict LRU entries when full', () => {
      const smallCache = new DashboardCache({
        ttl: 60000,
        maxSize: 2,
        strategy: 'lru',
        compression: false
      });

      smallCache.set('key1', mockAnalyticsResponse);
      smallCache.set('key2', mockAnalyticsResponse);
      smallCache.set('key3', mockAnalyticsResponse); // Should evict key1

      expect(smallCache.has('key1')).toBe(false);
      expect(smallCache.has('key2')).toBe(true);
      expect(smallCache.has('key3')).toBe(true);
    });
  });
});

describe('DashboardValidator', () => {
  let validator: DashboardValidator;

  beforeEach(() => {
    validator = new DashboardValidator();
  });

  describe('validateQueryResponse', () => {
    it('should validate correct query response', () => {
      const result = validator.validateQueryResponse(mockAnalyticsResponse);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should detect missing required fields', () => {
      const invalidResponse = {
        ...mockAnalyticsResponse,
        queryId: undefined
      } as any;

      const result = validator.validateQueryResponse(invalidResponse);

      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should detect invalid data types', () => {
      const invalidResponse = {
        ...mockAnalyticsResponse,
        executionTimeMs: 'not a number'
      } as any;

      const result = validator.validateQueryResponse(invalidResponse);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.field === 'executionTimeMs')).toBe(true);
    });
  });

  describe('validateQueryRow', () => {
    it('should validate correct query row', () => {
      const result = validator.validateQueryRow(mockQueryResults[0]);

      expect(result.valid).toBe(true);
    });

    it('should detect missing metrics', () => {
      const invalidRow = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: {}
      } as any;

      const result = validator.validateQueryRow(invalidRow);

      expect(result.valid).toBe(false);
    });
  });

  describe('schema validation', () => {
    it('should validate against registered schema', () => {
      validator.registerSchema('test-schema', {
        fields: {
          name: {
            type: 'string',
            required: true
          },
          value: {
            type: 'number',
            required: true
          }
        }
      });

      const validData = { name: 'test', value: 42 };
      const result = validator.validateSchema(validData, 'test-schema');

      expect(result.valid).toBe(true);
    });

    it('should fail schema validation for missing required fields', () => {
      validator.registerSchema('test-schema', {
        fields: {
          name: {
            type: 'string',
            required: true
          },
          value: {
            type: 'number',
            required: true
          }
        }
      });

      const invalidData = { name: 'test' };
      const result = validator.validateSchema(invalidData, 'test-schema');

      expect(result.valid).toBe(false);
    });
  });
});

describe('RealtimeDashboardProcessor', () => {
  let processor: RealtimeDashboardProcessor;

  beforeEach(() => {
    processor = new RealtimeDashboardProcessor({
      enabled: true,
      updateInterval: 100,
      debounceTime: 50,
      maxBufferSize: 100,
      deltaUpdates: true
    });
  });

  afterEach(() => {
    processor.shutdown();
  });

  describe('basic operations', () => {
    it('should add data to buffer', () => {
      processor.addData(mockQueryResults);
      const buffer = processor.getBuffer();

      expect(buffer.length).toBeGreaterThan(0);
    });

    it('should trim buffer when exceeding max size', () => {
      const smallProcessor = new RealtimeDashboardProcessor({
        enabled: true,
        updateInterval: 1000,
        debounceTime: 100,
        maxBufferSize: 5,
        deltaUpdates: true
      });

      // Add more data than maxBufferSize
      for (let i = 0; i < 10; i++) {
        smallProcessor.addData(mockQueryResults);
      }

      const buffer = smallProcessor.getBuffer();
      expect(buffer.length).toBeLessThanOrEqual(5);
      smallProcessor.shutdown();
    });

    it('should subscribe and unsubscribe', () => {
      const callback = jest.fn();
      const subscriptionId = processor.subscribe(callback);

      expect(typeof subscriptionId).toBe('string');

      const unsubscribed = processor.unsubscribe(subscriptionId);
      expect(unsubscribed).toBe(true);
    });

    it('should clear buffer', () => {
      processor.addData(mockQueryResults);
      processor.clearBuffer();

      expect(processor.getBuffer()).toHaveLength(0);
    });
  });

  describe('configuration', () => {
    it('should update configuration', () => {
      processor.updateConfig({ updateInterval: 200 });

      const stats = processor.getStats();
      expect(stats).toBeDefined();
    });

    it('should enable and disable processing', () => {
      processor.setEnabled(false);
      expect(processor.isEnabled()).toBe(false);

      processor.setEnabled(true);
      expect(processor.isEnabled()).toBe(true);
    });
  });

  describe('statistics', () => {
    it('should track statistics', () => {
      processor.addData(mockQueryResults);
      const stats = processor.getStats();

      expect(stats).toHaveProperty('totalUpdates');
      expect(stats).toHaveProperty('totalSubscribers');
      expect(stats).toHaveProperty('bufferUtilization');
    });
  });
});

describe('DashboardExporter', () => {
  let exporter: DashboardExporter;

  beforeEach(() => {
    exporter = new DashboardExporter();
  });

  describe('CSV export', () => {
    it('should export data to CSV format', async () => {
      const config = {
        format: 'csv' as const,
        includeMetadata: true,
        dateFormat: 'YYYY-MM-DD',
        numberFormat: '0,0.00',
        compression: false
      };

      const result = await exporter.export(mockQueryResults, config);

      expect(result.format).toBe('csv');
      expect(result.mimeType).toBe('text/csv');
      expect(result.size).toBeGreaterThan(0);
      expect(result.filename).toMatch(/\.csv$/);
    });

    it('should handle empty data for CSV export', async () => {
      const config = {
        format: 'csv' as const,
        includeMetadata: false,
        dateFormat: 'YYYY-MM-DD',
        numberFormat: '0,0.00',
        compression: false
      };

      const result = await exporter.export([], config);

      expect(result.data).toBe('');
      expect(result.metadata?.recordCount).toBe(0);
    });
  });

  describe('JSON export', () => {
    it('should export data to JSON format', async () => {
      const config = {
        format: 'json' as const,
        includeMetadata: true,
        dateFormat: 'YYYY-MM-DD',
        numberFormat: '0,0.00',
        compression: false
      };

      const result = await exporter.export(mockQueryResults, config);

      expect(result.format).toBe('json');
      expect(result.mimeType).toBe('application/json');
      expect(result.size).toBeGreaterThan(0);
      
      const parsed = JSON.parse(result.data as string);
      expect(parsed).toBeDefined();
    });
  });

  describe('template export', () => {
    it('should export using template', async () => {
      const template = {
        name: 'test-template',
        format: 'csv' as const,
        columns: [
          { key: 'requestCount', label: 'Requests' },
          { key: 'cost', label: 'Cost' }
        ]
      };

      const config = {
        format: 'csv' as const,
        includeMetadata: false,
        dateFormat: 'YYYY-MM-DD',
        numberFormat: '0,0.00',
        compression: false
      };

      const result = await exporter.exportWithTemplate(
        mockQueryResults,
        template,
        config
      );

      expect(result.format).toBe('csv');
      expect(result.size).toBeGreaterThan(0);
    });

    it('should create template from data', () => {
      const template = exporter.createTemplate(
        'auto-template',
        mockQueryResults,
        'csv'
      );

      expect(template.name).toBe('auto-template');
      expect(template.format).toBe('csv');
      expect(Array.isArray(template.columns)).toBe(true);
    });
  });

  describe('format support', () => {
    it('should return supported formats', () => {
      const formats = exporter.getSupportedFormats();

      expect(Array.isArray(formats)).toBe(true);
      expect(formats).toContain('csv');
      expect(formats).toContain('json');
    });
  });
});