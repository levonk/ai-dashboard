/**
 * Cost Analysis Cache Tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { CostAnalysisCache, AggregatedCostCache } from '../cache';

describe('CostAnalysisCache', () => {
  let cache: CostAnalysisCache;

  beforeEach(() => {
    cache = new CostAnalysisCache({
      enabled: true,
      ttl: 300,
      maxSize: 1000
    });
  });

  describe('basic operations', () => {
    it('should set and get values', () => {
      const key = 'test-key';
      const data = { cost: 100, tokens: 1000 };

      cache.set(key, data);
      const retrieved = cache.get(key);

      expect(retrieved).toEqual(data);
    });

    it('should return null for non-existent keys', () => {
      const result = cache.get('non-existent-key');
      expect(result).toBeNull();
    });

    it('should delete values', () => {
      const key = 'test-key';
      const data = { cost: 100 };

      cache.set(key, data);
      cache.delete(key);
      
      const result = cache.get(key);
      expect(result).toBeNull();
    });

    it('should clear all values', () => {
      cache.set('key1', { data: 1 });
      cache.set('key2', { data: 2 });
      
      cache.clear();
      
      expect(cache.get('key1')).toBeNull();
      expect(cache.get('key2')).toBeNull();
    });
  });

  describe('getOrSet', () => {
    it('should return cached value if exists', async () => {
      const key = 'test-key';
      const data = { cost: 100 };
      
      cache.set(key, data);
      
      const factory = vi.fn().mockResolvedValue({ cost: 200 });
      const result = await cache.getOrSet(key, factory);
      
      expect(result).toEqual(data);
      expect(factory).not.toHaveBeenCalled();
    });

    it('should call factory and cache result if not exists', async () => {
      const key = 'test-key';
      const data = { cost: 100 };
      
      const factory = vi.fn().mockResolvedValue(data);
      const result = await cache.getOrSet(key, factory);
      
      expect(result).toEqual(data);
      expect(factory).toHaveBeenCalledOnce();
      
      const cached = cache.get(key);
      expect(cached).toEqual(data);
    });
  });

  describe('cache statistics', () => {
    it('should return cache statistics', () => {
      cache.set('key1', { data: 1 });
      cache.set('key2', { data: 2 });
      
      const stats = cache.getStats();
      
      expect(stats.size).toBe(2);
      expect(stats.maxSize).toBe(1000);
      expect(stats.enabled).toBe(true);
      expect(stats.ttl).toBe(300);
    });

    it('should track hit rate', () => {
      const key = 'test-key';
      const data = { cost: 100 };
      
      cache.set(key, data);
      cache.get(key);
      cache.get(key);
      
      const stats = cache.getStats();
      expect(stats.hitRate).toBeGreaterThan(0);
    });
  });

  describe('cache expiration', () => {
    it('should expire entries after TTL', () => {
      const shortTTLCache = new CostAnalysisCache({
        enabled: true,
        ttl: 0.1, // 100ms
        maxSize: 1000
      });

      const key = 'test-key';
      const data = { cost: 100 };

      shortTTLCache.set(key, data);
      
      // Wait for expiration
      return new Promise<void>((resolve) => {
        setTimeout(() => {
          const result = shortTTLCache.get(key);
          expect(result).toBeNull();
          resolve();
        }, 150);
      });
    });

    it('should invalidate expired entries', () => {
      const shortTTLCache = new CostAnalysisCache({
        enabled: true,
        ttl: 0.1,
        maxSize: 1000
      });

      shortTTLCache.set('key1', { data: 1 });
      shortTTLCache.set('key2', { data: 2 });

      return new Promise<void>((resolve) => {
        setTimeout(() => {
          const count = shortTTLCache.invalidateExpired();
          expect(count).toBeGreaterThan(0);
          resolve();
        }, 150);
      });
    });
  });

  describe('cache size management', () => {
    it('should evict oldest entries when max size reached', () => {
      const smallCache = new CostAnalysisCache({
        enabled: true,
        ttl: 300,
        maxSize: 3
      });

      smallCache.set('key1', { data: 1 });
      smallCache.set('key2', { data: 2 });
      smallCache.set('key3', { data: 3 });
      smallCache.set('key4', { data: 4 }); // Should evict key1

      expect(smallCache.get('key1')).toBeNull();
      expect(smallCache.get('key2')).not.toBeNull();
      expect(smallCache.get('key3')).not.toBeNull();
      expect(smallCache.get('key4')).not.toBeNull();
    });
  });

  describe('pattern invalidation', () => {
    it('should invalidate entries matching pattern', () => {
      cache.set('cost_breakdown_abc', { data: 1 });
      cache.set('cost_trend_def', { data: 2 });
      cache.set('other_key', { data: 3 });

      const count = cache.invalidatePattern('^cost_');
      
      expect(count).toBe(2);
      expect(cache.get('cost_breakdown_abc')).toBeNull();
      expect(cache.get('cost_trend_def')).toBeNull();
      expect(cache.get('other_key')).not.toBeNull();
    });
  });

  describe('configuration', () => {
    it('should update configuration', () => {
      cache.updateConfig({ ttl: 600 });
      
      const stats = cache.getStats();
      expect(stats.ttl).toBe(600);
    });

    it('should disable cache and clear data', () => {
      cache.set('key1', { data: 1 });
      
      cache.updateConfig({ enabled: false });
      
      expect(cache.get('key1')).toBeNull();
      const stats = cache.getStats();
      expect(stats.enabled).toBe(false);
    });
  });
});

describe('AggregatedCostCache', () => {
  let aggregatedCache: AggregatedCostCache;

  beforeEach(() => {
    aggregatedCache = new AggregatedCostCache();
  });

  describe('aggregation operations', () => {
    it('should set and get aggregated cost data', () => {
      const key = 'test-key';
      const interval = 'daily';
      const data = { totalCost: 1000 };

      aggregatedCache.setAggregatedCost(key, interval, data);
      const result = aggregatedCache.getAggregatedCost(key, interval);

      expect(result).toEqual(data);
    });

    it('should invalidate by interval', () => {
      aggregatedCache.setAggregatedCost('key1', 'daily', { data: 1 });
      aggregatedCache.setAggregatedCost('key2', 'daily', { data: 2 });
      aggregatedCache.setAggregatedCost('key3', 'weekly', { data: 3 });

      const count = aggregatedCache.invalidateInterval('daily');

      expect(count).toBe(2);
      expect(aggregatedCache.getAggregatedCost('key1', 'daily')).toBeNull();
      expect(aggregatedCache.getAggregatedCost('key2', 'daily')).toBeNull();
      expect(aggregatedCache.getAggregatedCost('key3', 'weekly')).not.toBeNull();
    });

    it('should return aggregation statistics', () => {
      aggregatedCache.setAggregatedCost('key1', 'daily', { data: 1 });
      aggregatedCache.setAggregatedCost('key2', 'daily', { data: 2 });
      aggregatedCache.setAggregatedCost('key3', 'weekly', { data: 3 });

      const stats = aggregatedCache.getAggregationStats();

      expect(stats.totalAggregations).toBe(3);
      expect(stats.byInterval.daily).toBe(2);
      expect(stats.byInterval.weekly).toBe(1);
    });
  });
});