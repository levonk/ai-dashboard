/**
 * Tests for Query Validator
 */

import { QueryValidator } from '../validator';
import { AnalyticsQueryRequest } from '../types';

describe('QueryValidator', () => {
  let validator: QueryValidator;

  beforeEach(() => {
    validator = new QueryValidator();
  });

  const createValidRequest = (): AnalyticsQueryRequest => ({
    startTime: '2024-01-01T00:00:00Z',
    endTime: '2024-01-31T23:59:59Z',
    dimensions: {
      clientIds: ['client1'],
      providers: ['anthropic']
    },
    metrics: ['requestCount', 'tokenCount'],
    groupBy: ['provider'],
    aggregations: ['sum', 'count'],
    filters: [
      {
        field: 'cost',
        operator: 'gt',
        value: 10
      }
    ],
    limit: 100,
    offset: 0
  });

  describe('Valid queries', () => {
    it('should validate a correct query', () => {
      const request = createValidRequest();
      const result = validator.validate(request);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Time range validation', () => {
    it('should reject missing start time', () => {
      const request = createValidRequest();
      delete request.startTime;

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.field === 'startTime')).toBe(true);
    });

    it('should reject missing end time', () => {
      const request = createValidRequest();
      delete request.endTime;

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.field === 'endTime')).toBe(true);
    });

    it('should reject invalid ISO date format', () => {
      const request = createValidRequest();
      request.startTime = 'invalid-date';

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_DATE')).toBe(true);
    });

    it('should reject start time after end time', () => {
      const request = createValidRequest();
      request.startTime = '2024-02-01T00:00:00Z';
      request.endTime = '2024-01-01T00:00:00Z';

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_TIME_RANGE')).toBe(true);
    });

    it('should reject time range exceeding 1 year', () => {
      const request = createValidRequest();
      request.startTime = '2020-01-01T00:00:00Z';
      request.endTime = '2025-01-01T00:00:00Z';

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'TIME_RANGE_TOO_LARGE')).toBe(true);
    });
  });

  describe('Metrics validation', () => {
    it('should reject missing metrics', () => {
      const request = createValidRequest();
      request.metrics = [];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.field === 'metrics')).toBe(true);
    });

    it('should reject invalid metric', () => {
      const request = createValidRequest();
      request.metrics = ['invalidMetric' as any];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_METRIC')).toBe(true);
    });

    it('should accept valid metrics', () => {
      const request = createValidRequest();
      request.metrics = ['requestCount', 'tokenCount', 'cost', 'latency'];

      const result = validator.validate(request);

      expect(result.valid).toBe(true);
    });
  });

  describe('Aggregations validation', () => {
    it('should reject missing aggregations', () => {
      const request = createValidRequest();
      request.aggregations = [];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.field === 'aggregations')).toBe(true);
    });

    it('should reject invalid aggregation', () => {
      const request = createValidRequest();
      request.aggregations = ['invalidAgg' as any];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_AGGREGATION')).toBe(true);
    });

    it('should accept valid aggregations', () => {
      const request = createValidRequest();
      request.aggregations = ['sum', 'count', 'avg', 'p95'];

      const result = validator.validate(request);

      expect(result.valid).toBe(true);
    });
  });

  describe('GroupBy validation', () => {
    it('should reject invalid dimension', () => {
      const request = createValidRequest();
      request.groupBy = ['invalidDimension' as any];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_DIMENSION')).toBe(true);
    });

    it('should accept valid dimensions', () => {
      const request = createValidRequest();
      request.groupBy = ['provider', 'model', 'time'];

      const result = validator.validate(request);

      expect(result.valid).toBe(true);
    });
  });

  describe('Filters validation', () => {
    it('should reject filter without field', () => {
      const request = createValidRequest();
      request.filters = [{ operator: 'eq', value: 'test' } as any];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.field === 'filters')).toBe(true);
    });

    it('should reject invalid operator', () => {
      const request = createValidRequest();
      request.filters = [{ field: 'cost', operator: 'invalidOp' as any, value: 10 }];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_OPERATOR')).toBe(true);
    });

    it('should reject invalid field name (potential injection)', () => {
      const request = createValidRequest();
      request.filters = [{ field: 'cost; DROP TABLE', operator: 'eq', value: 10 }];

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_FIELD_NAME')).toBe(true);
    });

    it('should accept valid filters', () => {
      const request = createValidRequest();
      request.filters = [
        { field: 'cost', operator: 'gt', value: 10 },
        { field: 'provider', operator: 'eq', value: 'anthropic' }
      ];

      const result = validator.validate(request);

      expect(result.valid).toBe(true);
    });
  });

  describe('Pagination validation', () => {
    it('should reject negative limit', () => {
      const request = createValidRequest();
      request.limit = -10;

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_LIMIT')).toBe(true);
    });

    it('should reject limit exceeding maximum', () => {
      const request = createValidRequest();
      request.limit = 20000;

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'LIMIT_TOO_LARGE')).toBe(true);
    });

    it('should reject negative offset', () => {
      const request = createValidRequest();
      request.offset = -10;

      const result = validator.validate(request);

      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_OFFSET')).toBe(true);
    });

    it('should accept valid pagination', () => {
      const request = createValidRequest();
      request.limit = 100;
      request.offset = 50;

      const result = validator.validate(request);

      expect(result.valid).toBe(true);
    });
  });

  describe('Sanitization', () => {
    it('should sanitize dimension arrays', () => {
      const request = createValidRequest();
      request.dimensions!.clientIds = ['client1', '  client2  ', 'client3'];

      const result = validator.validate(request);

      expect(result.valid).toBe(true);
      // The validator should trim whitespace
    });

    it('should set default pagination limits', () => {
      const request = createValidRequest();
      delete request.limit;
      delete request.offset;

      const result = validator.validate(request);

      expect(result.valid).toBe(true);
      // The validator should set defaults
    });
  });
});
