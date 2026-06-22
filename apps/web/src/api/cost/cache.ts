/**
 * Cost Analysis Cache
 * 
 * Provides caching and aggregation for cost analysis results
 */

import {
  CostCacheConfig,
  CostAnalysisMetrics
} from './types';

export class CostAnalysisCache {
  private cache: Map<string, CacheEntry>;
  private config: CostCacheConfig;
  private metrics: Map<string, CostAnalysisMetrics>;

  constructor(config?: Partial<CostCacheConfig>) {
    this.cache = new Map();
    this.metrics = new Map();
    this.config = {
      enabled: config?.enabled ?? true,
      ttl: config?.ttl ?? 300, // 5 minutes default
      maxSize: config?.maxSize ?? 1000
    };
  }

  /**
   * Get cached result
   */
  get<T>(key: string): T | null {
    if (!this.config.enabled) return null;

    const entry = this.cache.get(key);
    if (!entry) return null;

    // Check if entry has expired
    if (this.isExpired(entry)) {
      this.cache.delete(key);
      return null;
    }

    // Update hit count and last accessed
    entry.hitCount++;
    entry.lastAccessed = Date.now();

    // Record cache hit metric
    this.recordMetric(key, true, entry.dataSize);

    return entry.data as T;
  }

  /**
   * Set cached result
   */
  set<T>(key: string, data: T): void {
    if (!this.config.enabled) return;

    // Check cache size limit
    if (this.cache.size >= this.config.maxSize) {
      this.evictOldest();
    }

    const entry: CacheEntry = {
      data,
      createdAt: Date.now(),
      lastAccessed: Date.now(),
      hitCount: 0,
      dataSize: this.estimateSize(data)
    };

    this.cache.set(key, entry);

    // Record cache miss metric
    this.recordMetric(key, false, entry.dataSize);
  }

  /**
   * Delete cached result
   */
  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Clear all cache entries
   */
  clear(): void {
    this.cache.clear();
    this.metrics.clear();
  }

  /**
   * Get cache statistics
   */
  getStats(): CacheStats {
    const entries = Array.from(this.cache.values());
    const totalSize = entries.reduce((sum, entry) => sum + entry.dataSize, 0);
    const totalHits = entries.reduce((sum, entry) => sum + entry.hitCount, 0);
    
    // Calculate hit rate from metrics
    let totalHitsMetric = 0;
    let totalMissesMetric = 0;
    
    for (const metric of this.metrics.values()) {
      if (metric.cacheHit) {
        totalHitsMetric++;
      } else {
        totalMissesMetric++;
      }
    }
    
    const hitRate = totalHitsMetric + totalMissesMetric > 0 
      ? totalHitsMetric / (totalHitsMetric + totalMissesMetric) 
      : 0;

    return {
      size: this.cache.size,
      maxSize: this.config.maxSize,
      hitRate,
      totalHits,
      totalSize,
      enabled: this.config.enabled,
      ttl: this.config.ttl
    };
  }

  /**
   * Get or set with factory function
   */
  async getOrSet<T>(key: string, factory: () => Promise<T>): Promise<T> {
    const cached = this.get<T>(key);
    if (cached !== null) {
      return cached;
    }

    const data = await factory();
    this.set(key, data);
    return data;
  }

  /**
   * Invalidate cache entries matching a pattern
   */
  invalidatePattern(pattern: string): number {
    let count = 0;
    const regex = new RegExp(pattern);
    
    for (const key of this.cache.keys()) {
      if (regex.test(key)) {
        this.cache.delete(key);
        count++;
      }
    }
    
    return count;
  }

  /**
   * Invalidate expired entries
   */
  invalidateExpired(): number {
    let count = 0;
    
    for (const [key, entry] of this.cache.entries()) {
      if (this.isExpired(entry)) {
        this.cache.delete(key);
        count++;
      }
    }
    
    return count;
  }

  /**
   * Get metrics for a specific key
   */
  getMetrics(key: string): CostAnalysisMetrics | null {
    return this.metrics.get(key) || null;
  }

  /**
   * Get all metrics
   */
  getAllMetrics(): CostAnalysisMetrics[] {
    return Array.from(this.metrics.values());
  }

  /**
   * Update cache configuration
   */
  updateConfig(config: Partial<CostCacheConfig>): void {
    this.config = { ...this.config, ...config };
    
    // If cache is being disabled, clear it
    if (!this.config.enabled) {
      this.clear();
    }
  }

  /**
   * Check if entry has expired
   */
  private isExpired(entry: CacheEntry): boolean {
    const age = Date.now() - entry.createdAt;
    return age > this.config.ttl * 1000;
  }

  /**
   * Evict oldest entry (LRU)
   */
  private evictOldest(): void {
    let oldestKey: string | null = null;
    let oldestAccess = Date.now();

    for (const [key, entry] of this.cache.entries()) {
      if (entry.lastAccessed < oldestAccess) {
        oldestAccess = entry.lastAccessed;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      this.cache.delete(oldestKey);
      this.metrics.delete(oldestKey);
    }
  }

  /**
   * Estimate size of data in bytes
   */
  private estimateSize(data: any): number {
    try {
      return JSON.stringify(data).length * 2; // Approximate bytes (UTF-16)
    } catch {
      return 0;
    }
  }

  /**
   * Record cache metric
   */
  private recordMetric(key: string, cacheHit: boolean, dataSize: number): void {
    const metric: CostAnalysisMetrics = {
      queryId: key,
      executionTimeMs: 0, // Not tracked in cache
      cacheHit,
      resultCount: 1,
      timestamp: new Date().toISOString()
    };

    this.metrics.set(key, metric);
  }

  /**
   * Generate cache key from request
   */
  static generateKey(prefix: string, request: any): string {
    const hash = CostAnalysisCache.hashRequest(request);
    return `${prefix}_${hash}`;
  }

  /**
   * Hash request for cache key
   */
  private static hashRequest(request: any): string {
    const str = JSON.stringify(request);
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return Math.abs(hash).toString(16);
  }
}

/**
 * Cache entry structure
 */
interface CacheEntry {
  data: any;
  createdAt: number;
  lastAccessed: number;
  hitCount: number;
  dataSize: number;
}

/**
 * Cache statistics
 */
interface CacheStats {
  size: number;
  maxSize: number;
  hitRate: number;
  totalHits: number;
  totalSize: number;
  enabled: boolean;
  ttl: number;
}

/**
 * Aggregated cost data cache
 */
export class AggregatedCostCache {
  private cache: CostAnalysisCache;
  private aggregationIntervals: Map<string, number>;

  constructor(config?: Partial<CostCacheConfig>) {
    this.cache = new CostAnalysisCache(config);
    this.aggregationIntervals = new Map();
  }

  /**
   * Get aggregated cost data
   */
  getAggregatedCost(key: string, interval: string): any | null {
    const cacheKey = `aggregated_${interval}_${key}`;
    return this.cache.get(cacheKey);
  }

  /**
   * Set aggregated cost data
   */
  setAggregatedCost(key: string, interval: string, data: any): void {
    const cacheKey = `aggregated_${interval}_${key}`;
    this.cache.set(cacheKey, data);
    this.aggregationIntervals.set(cacheKey, Date.now());
  }

  /**
   * Invalidate aggregated data for a specific interval
   */
  invalidateInterval(interval: string): number {
    return this.cache.invalidatePattern(`^aggregated_${interval}_`);
  }

  /**
   * Get aggregation statistics
   */
  getAggregationStats(): {
    totalAggregations: number;
    byInterval: Record<string, number>;
  } {
    const byInterval: Record<string, number> = {};
    
    for (const [key, timestamp] of this.aggregationIntervals.entries()) {
      const interval = key.split('_')[1];
      byInterval[interval] = (byInterval[interval] || 0) + 1;
    }

    return {
      totalAggregations: this.aggregationIntervals.size,
      byInterval
    };
  }

  /**
   * Get underlying cache
   */
  getCache(): CostAnalysisCache {
    return this.cache;
  }
}