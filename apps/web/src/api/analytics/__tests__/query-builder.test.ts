/**
 * Tests for Analytics Query Builder
 */

import { AnalyticsQueryBuilder, QueryPatterns } from '../query-builder';

describe('AnalyticsQueryBuilder', () => {
  let builder: AnalyticsQueryBuilder;

  beforeEach(() => {
    builder = new AnalyticsQueryBuilder();
  });

  describe('Basic query building', () => {
    it('should build a basic query with time range', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .build();

      expect(query.startTime).toBe('2024-01-01T00:00:00Z');
      expect(query.endTime).toBe('2024-01-31T23:59:59Z');
    });

    it('should add metrics to query', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount', 'tokenCount'])
        .build();

      expect(query.metrics).toContain('requestCount');
      expect(query.metrics).toContain('tokenCount');
    });

    it('should add aggregations to query', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum', 'count'])
        .build();

      expect(query.aggregations).toContain('sum');
      expect(query.aggregations).toContain('count');
    });
  });

  describe('Dimension filtering', () => {
    it('should filter by client IDs', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum'])
        .filterByClientIds(['client1', 'client2'])
        .build();

      expect(query.dimensions.clientIds).toEqual(['client1', 'client2']);
    });

    it('should filter by AI clients', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum'])
        .filterByAiClients(['claude-code', 'codex'])
        .build();

      expect(query.dimensions.aiClients).toEqual(['claude-code', 'codex']);
    });

    it('should filter by providers', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum'])
        .filterByProviders(['anthropic', 'openai'])
        .build();

      expect(query.dimensions.providers).toEqual(['anthropic', 'openai']);
    });
  });

  describe('Grouping and sorting', () => {
    it('should set grouping dimensions', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum'])
        .groupBy(['provider', 'model'])
        .build();

      expect(query.groupBy).toEqual(['provider', 'model']);
    });

    it('should set sorting', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum'])
        .sortBy('cost', 'desc')
        .build();

      expect(query.sortBy).toEqual({ field: 'cost', direction: 'desc' });
      expect(query.sortOrder).toBe('desc');
    });
  });

  describe('Pagination', () => {
    it('should set pagination limit', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum'])
        .setLimit(50)
        .build();

      expect(query.limit).toBe(50);
    });

    it('should set pagination offset', () => {
      const query = builder
        .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
        .addMetrics(['requestCount'])
        .addAggregations(['sum'])
        .setOffset(100)
        .build();

      expect(query.offset).toBe(100);
    });
  });

  describe('Validation', () => {
    it('should validate required time range', () => {
      expect(() => {
        builder.addMetrics(['requestCount']).addAggregations(['sum']).build();
      }).toThrow('Time range (startTime and endTime) is required');
    });

    it('should validate required metrics', () => {
      expect(() => {
        builder
          .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
          .addAggregations(['sum'])
          .build();
      }).toThrow('At least one metric is required');
    });

    it('should validate required aggregations', () => {
      expect(() => {
        builder
          .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
          .addMetrics(['requestCount'])
          .build();
      }).toThrow('At least one aggregation function is required');
    });

    it('should validate time range order', () => {
      expect(() => {
        builder
          .setTimeRange('2024-01-31T23:59:59Z', '2024-01-01T00:00:00Z')
          .addMetrics(['requestCount'])
          .addAggregations(['sum'])
          .build();
      }).toThrow('startTime must be before endTime');
    });

    it('should validate limit maximum', () => {
      expect(() => {
        builder
          .setTimeRange('2024-01-01T00:00:00Z', '2024-01-31T23:59:59Z')
          .addMetrics(['requestCount'])
          .addAggregations(['sum'])
          .setLimit(20000)
          .build();
      }).toThrow('Limit cannot exceed 10000');
    });
  });
});

describe('QueryPatterns', () => {
  describe('Basic usage pattern', () => {
    it('should create basic usage query', () => {
      const builder = QueryPatterns.basicUsage(
        '2024-01-01T00:00:00Z',
        '2024-01-31T23:59:59Z'
      );
      const query = builder.build();

      expect(query.metrics).toContain('requestCount');
      expect(query.metrics).toContain('tokenCount');
      expect(query.metrics).toContain('cost');
      expect(query.aggregations).toContain('sum');
      expect(query.aggregations).toContain('count');
      expect(query.groupBy).toContain('time');
    });
  });

  describe('Cost breakdown pattern', () => {
    it('should create cost breakdown query', () => {
      const builder = QueryPatterns.costBreakdown(
        '2024-01-01T00:00:00Z',
        '2024-01-31T23:59:59Z'
      );
      const query = builder.build();

      expect(query.metrics).toContain('cost');
      expect(query.metrics).toContain('tokenCount');
      expect(query.aggregations).toContain('sum');
      expect(query.aggregations).toContain('avg');
      expect(query.groupBy).toContain('provider');
      expect(query.groupBy).toContain('model');
      expect(query.sortBy?.field).toBe('cost');
      expect(query.sortBy?.direction).toBe('desc');
    });
  });

  describe('Performance analysis pattern', () => {
    it('should create performance analysis query', () => {
      const builder = QueryPatterns.performanceAnalysis(
        '2024-01-01T00:00:00Z',
        '2024-01-31T23:59:59Z'
      );
      const query = builder.build();

      expect(query.metrics).toContain('latency');
      expect(query.metrics).toContain('errorRate');
      expect(query.metrics).toContain('throughput');
      expect(query.aggregations).toContain('avg');
      expect(query.aggregations).toContain('p95');
      expect(query.aggregations).toContain('p99');
      expect(query.groupBy).toContain('aiClient');
      expect(query.groupBy).toContain('pipelineStage');
    });
  });

  describe('Trend analysis pattern', () => {
    it('should create trend analysis query', () => {
      const builder = QueryPatterns.trendAnalysis(
        '2024-01-01T00:00:00Z',
        '2024-01-31T23:59:59Z'
      );
      const query = builder.build();

      expect(query.metrics).toContain('requestCount');
      expect(query.metrics).toContain('tokenCount');
      expect(query.metrics).toContain('cost');
      expect(query.aggregations).toContain('sum');
      expect(query.groupBy).toContain('time');
      expect(query.sortBy?.field).toBe('time');
      expect(query.sortBy?.direction).toBe('asc');
    });
  });
});
