/**
 * Tests for Query Cache
 */

import { QueryCache } from '../cache';
import { AnalyticsQueryRequest, AnalyticsQueryResponse } from '../types';

describe('QueryCache', () => {
  let cache: QueryCache;

  beforeEach(() => {
    cache = new QueryCache(1000); // 1 second TTL for testing
  });

  const createMockRequest = (): AnalyticsQueryRequest => ({
    startTime: '2024-01-01T00:00:00Z',
    endTime: '2024-01-31T23:59:59Z',
    dimensions: {
      providers: ['anthropic']
    },
    metrics: ['requestCount'],
    aggregations: ['sum'],
    limit: 100,
    offset: 0
  });

  const createMockResponse = (): AnalyticsQueryResponse => ({
    queryId: 'test-query-id',
    executionTimeMs: 100,
    cached: false,
    data: [
      {
        metrics: { requestCount: 100 }
      }
    ],
    pagination: {
      total: 1,
      limit: 100,
      offset: 0,
      hasMore: false
    },
    metadata: {
      timeRange: {
        start: '2024-01-01T00:00:00Z',
        end: '2024-01-31T23:59:59Z'
      },
      dimensions: [],
      metrics: ['requestCount']
    }
  });

  describe('Basic caching', () => {
    it('should cache a response', () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response);

      const cached = cache.get(request);
      expect(cached).not.toBeNull();
      expect(cached?.queryId).toBe(response.queryId);
    });

    it('should return null for non-cached request', () => {
      const request = createMockRequest();

      const cached = cache.get(request);
      expect(cached).toBeNull();
    });

    it('should return null for expired cache entry', async () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response, 100); // 100ms TTL

      await new Promise(resolve => setTimeout(resolve, 150));

      const cached = cache.get(request);
      expect(cached).toBeNull();
    });

    it('should not return expired entry', async () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response, 100); // 100ms TTL

      await new Promise(resolve => setTimeout(resolve, 50));

      const cached = cache.get(request);
      expect(cached).not.toBeNull();

      await new Promise(resolve => setTimeout(resolve, 100));

      const cachedAfterExpiry = cache.get(request);
      expect(cachedAfterExpiry).toBeNull();
    });
  });

  describe('Cache key generation', () => {
    it('should generate same key for identical requests', () => {
      const request1 = createMockRequest();
      const request2 = createMockRequest();

      const response = createMockResponse();
      cache.set(request1, response);

      const cached = cache.get(request2);
      expect(cached).not.toBeNull();
    });

    it('should generate different keys for different requests', () => {
      const request1 = createMockRequest();
      const request2 = createMockRequest();
      request2.dimensions!.providers = ['openai'];

      const response = createMockResponse();
      cache.set(request1, response);

      const cached = cache.get(request2);
      expect(cached).toBeNull();
    });
  });

  describe('Cache statistics', () => {
    it('should track cache hits', () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response);

      cache.get(request);
      cache.get(request);
      cache.get(request);

      const stats = cache.getStats();
      expect(stats.totalHits).toBe(3);
    });

    it('should calculate hit rate correctly', () => {
      const request1 = createMockRequest();
      const request2 = createMockRequest();
      request2.dimensions!.providers = ['openai'];

      const response = createMockResponse();

      cache.set(request1, response);
      cache.set(request2, response);

      cache.get(request1); // hit
      cache.get(request1); // hit
      cache.get(request2); // hit
      cache.get(createMockRequest()); // miss (different request)

      const stats = cache.getStats();
      expect(stats.hitRate).toBe(0.75);
    });

    it('should count expired entries', async () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response, 100); // 100ms TTL

      await new Promise(resolve => setTimeout(resolve, 150));

      const stats = cache.getStats();
      expect(stats.expiredCount).toBe(1);
    });
  });

  describe('Cache invalidation', () => {
    it('should invalidate by dimension', () => {
      const request1 = createMockRequest();
      const request2 = createMockRequest();
      request2.dimensions!.providers = ['openai'];

      const response = createMockResponse();

      cache.set(request1, response);
      cache.set(request2, response);

      cache.invalidateByDimension('provider', 'anthropic');

      expect(cache.get(request1)).toBeNull();
      expect(cache.get(request2)).not.toBeNull();
    });

    it('should clear all cache entries', () => {
      const request1 = createMockRequest();
      const request2 = createMockRequest();
      request2.dimensions!.providers = ['openai'];

      const response = createMockResponse();

      cache.set(request1, response);
      cache.set(request2, response);

      cache.clear();

      expect(cache.get(request1)).toBeNull();
      expect(cache.get(request2)).toBeNull();
      expect(cache.getStats().size).toBe(0);
    });

    it('should clear expired entries', async () => {
      const request1 = createMockRequest();
      const request2 = createMockRequest();
      request2.dimensions!.providers = ['openai'];

      const response = createMockResponse();

      cache.set(request1, response, 100); // 100ms TTL
      cache.set(request2, response, 5000); // 5s TTL

      await new Promise(resolve => setTimeout(resolve, 150));

      cache.clearExpired();

      expect(cache.get(request1)).toBeNull();
      expect(cache.get(request2)).not.toBeNull();
    });
  });

  describe('Cache entry metadata', () => {
    it('should track creation time', () => {
      const request = createMockRequest();
      const response = createMockResponse();

      const beforeSet = new Date();
      cache.set(request, response);
      const afterSet = new Date();

      const entries = cache.getEntries();
      expect(entries.length).toBe(1);

      const createdAt = new Date(entries[0].createdAt);
      expect(createdAt.getTime()).toBeGreaterThanOrEqual(beforeSet.getTime());
      expect(createdAt.getTime()).toBeLessThanOrEqual(afterSet.getTime());
    });

    it('should track expiration time', () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response, 2000); // 2s TTL

      const entries = cache.getEntries();
      const expiresAt = new Date(entries[0].expiresAt);
      const createdAt = new Date(entries[0].createdAt);

      const timeDiff = expiresAt.getTime() - createdAt.getTime();
      expect(timeDiff).toBe(2000);
    });
  });

  describe('TTL configuration', () => {
    it('should use custom TTL when provided', () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response, 500); // 500ms TTL

      const entries = cache.getEntries();
      const expiresAt = new Date(entries[0].expiresAt);
      const createdAt = new Date(entries[0].createdAt);

      const timeDiff = expiresAt.getTime() - createdAt.getTime();
      expect(timeDiff).toBe(500);
    });

    it('should use default TTL when not provided', () => {
      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response);

      const entries = cache.getEntries();
      const expiresAt = new Date(entries[0].expiresAt);
      const createdAt = new Date(entries[0].createdAt);

      const timeDiff = expiresAt.getTime() - createdAt.getTime();
      expect(timeDiff).toBe(1000); // default TTL
    });

    it('should allow changing default TTL', () => {
      cache.setDefaultTTL(2000);

      const request = createMockRequest();
      const response = createMockResponse();

      cache.set(request, response);

      const entries = cache.getEntries();
      const expiresAt = new Date(entries[0].expiresAt);
      const createdAt = new Date(entries[0].createdAt);

      const timeDiff = expiresAt.getTime() - createdAt.getTime();
      expect(timeDiff).toBe(2000);
    });
  });
});
