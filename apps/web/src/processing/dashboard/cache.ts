/**
 * Dashboard Data Cache
 * 
 * High-performance caching layer for dashboard data
 * Reduces load times and improves dashboard responsiveness
 */

import { AnalyticsQueryResponse, QueryResultRow } from '../../api/analytics/types';

export interface CacheConfig {
  ttl: number; // time to live in milliseconds
  maxSize: number; // maximum number of entries
  strategy: 'lru' | 'fifo' | 'lfu';
  compression: boolean;
}

export interface CacheEntry<T> {
  key: string;
  data: T;
  metadata: {
    createdAt: number;
    lastAccessed: number;
    accessCount: number;
    size: number;
    compressed: boolean;
  };
}

export interface CacheStats {
  hits: number;
  misses: number;
  hitRate: number;
  totalEntries: number;
  totalSize: number;
  evictions: number;
}

/**
 * Dashboard data cache with multiple eviction strategies
 */
export class DashboardCache<T = any> {
  private cache: Map<string, CacheEntry<T>> = new Map();
  private config: CacheConfig;
  private stats: CacheStats = {
    hits: 0,
    misses: 0,
    hitRate: 0,
    totalEntries: 0,
    totalSize: 0,
    evictions: 0
  };

  constructor(config: Partial<CacheConfig> = {}) {
    this.config = {
      ttl: 5 * 60 * 1000, // 5 minutes default
      maxSize: 1000,
      strategy: 'lru',
      compression: false,
      ...config
    };
  }

  /**
   * Get data from cache
   */
  get(key: string): T | null {
    const entry = this.cache.get(key);

    if (!entry) {
      this.stats.misses++;
      this.updateHitRate();
      return null;
    }

    // Check if entry has expired
    if (this.isExpired(entry)) {
      this.cache.delete(key);
      this.stats.misses++;
      this.updateHitRate();
      return null;
    }

    // Update access metadata
    entry.metadata.lastAccessed = Date.now();
    entry.metadata.accessCount++;

    this.stats.hits++;
    this.updateHitRate();

    return entry.data;
  }

  /**
   * Set data in cache
   */
  set(key: string, data: T, customTtl?: number): void {
    // Check if we need to evict entries
    this.evictIfNeeded();

    const size = this.calculateSize(data);
    const entry: CacheEntry<T> = {
      key,
      data,
      metadata: {
        createdAt: Date.now(),
        lastAccessed: Date.now(),
        accessCount: 0,
        size,
        compressed: this.config.compression
      }
    };

    this.cache.set(key, entry);
    this.stats.totalEntries = this.cache.size;
    this.stats.totalSize += size;
  }

  /**
   * Check if key exists in cache
   */
  has(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;

    if (this.isExpired(entry)) {
      this.cache.delete(key);
      return false;
    }

    return true;
  }

  /**
   * Delete entry from cache
   */
  delete(key: string): boolean {
    const entry = this.cache.get(key);
    if (entry) {
      this.stats.totalSize -= entry.metadata.size;
    }
    return this.cache.delete(key);
  }

  /**
   * Clear all cache entries
   */
  clear(): void {
    this.cache.clear();
    this.stats.totalEntries = 0;
    this.stats.totalSize = 0;
  }

  /**
   * Get cache statistics
   */
  getStats(): CacheStats {
    return {
      ...this.stats,
      totalEntries: this.cache.size
    };
  }

  /**
   * Get cache keys
   */
  keys(): string[] {
    return Array.from(this.cache.keys());
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.cache.size;
  }

  /**
   * Clean expired entries
   */
  cleanExpired(): number {
    let cleaned = 0;

    for (const [key, entry] of this.cache.entries()) {
      if (this.isExpired(entry)) {
        this.delete(key);
        cleaned++;
      }
    }

    return cleaned;
  }

  /**
   * Pre-warm cache with common queries
   */
  async preWarm(
    keys: string[],
    dataLoader: (key: string) => Promise<T>
  ): Promise<void> {
    const promises = keys.map(async (key) => {
      if (!this.has(key)) {
        try {
          const data = await dataLoader(key);
          this.set(key, data);
        } catch (error) {
          console.error(`Failed to pre-warm cache for key ${key}:`, error);
        }
      }
    });

    await Promise.all(promises);
  }

  /**
   * Get or set pattern - fetch from cache or load and cache
   */
  async getOrSet(
    key: string,
    dataLoader: () => Promise<T>,
    customTtl?: number
  ): Promise<T> {
    const cached = this.get(key);
    if (cached !== null) {
      return cached;
    }

    const data = await dataLoader();
    this.set(key, data, customTtl);
    return data;
  }

  /**
   * Invalidate entries matching a pattern
   */
  invalidatePattern(pattern: string): number {
    let invalidated = 0;
    const regex = new RegExp(pattern);

    for (const key of this.cache.keys()) {
      if (regex.test(key)) {
        this.delete(key);
        invalidated++;
      }
    }

    return invalidated;
  }

  /**
   * Get cache entries as array
   */
  entries(): Array<{ key: string; data: T; metadata: CacheEntry<T>['metadata'] }> {
    return Array.from(this.cache.entries()).map(([key, entry]) => ({
      key,
      data: entry.data,
      metadata: entry.metadata
    }));
  }

  // Private methods

  private isExpired(entry: CacheEntry<T>): boolean {
    const age = Date.now() - entry.metadata.createdAt;
    return age > this.config.ttl;
  }

  private evictIfNeeded(): void {
    while (this.cache.size >= this.config.maxSize) {
      this.evictOne();
    }
  }

  private evictOne(): void {
    let keyToEvict: string | null = null;

    switch (this.config.strategy) {
      case 'lru':
        keyToEvict = this.findLRU();
        break;
      case 'fifo':
        keyToEvict = this.findFIFO();
        break;
      case 'lfu':
        keyToEvict = this.findLFU();
        break;
    }

    if (keyToEvict) {
      this.delete(keyToEvict);
      this.stats.evictions++;
    }
  }

  private findLRU(): string | null {
    let lruKey: string | null = null;
    let lruTime = Infinity;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.metadata.lastAccessed < lruTime) {
        lruTime = entry.metadata.lastAccessed;
        lruKey = key;
      }
    }

    return lruKey;
  }

  private findFIFO(): string | null {
    let fifoKey: string | null = null;
    let fifoTime = Infinity;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.metadata.createdAt < fifoTime) {
        fifoTime = entry.metadata.createdAt;
        fifoKey = key;
      }
    }

    return fifoKey;
  }

  private findLFU(): string | null {
    let lfuKey: string | null = null;
    let lfuCount = Infinity;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.metadata.accessCount < lfuCount) {
        lfuCount = entry.metadata.accessCount;
        lfuKey = key;
      }
    }

    return lfuKey;
  }

  private calculateSize(data: T): number {
    // Rough estimation of size in bytes
    try {
      return JSON.stringify(data).length * 2; // 2 bytes per char (UTF-16)
    } catch {
      return 1024; // default to 1KB if serialization fails
    }
  }

  private updateHitRate(): void {
    const total = this.stats.hits + this.stats.misses;
    this.stats.hitRate = total > 0 ? this.stats.hits / total : 0;
  }
}

/**
 * Specialized cache for dashboard query results
 */
export class DashboardQueryCache extends DashboardCache<AnalyticsQueryResponse> {
  constructor(config?: Partial<CacheConfig>) {
    super({
      ttl: 5 * 60 * 1000, // 5 minutes for dashboard queries
      maxSize: 500,
      strategy: 'lru',
      compression: false,
      ...config
    });
  }

  /**
   * Generate cache key from query parameters
   */
  static generateKey(params: {
    startTime: string;
    endTime: string;
    dimensions?: Record<string, string[]>;
    metrics?: string[];
    groupBy?: string[];
  }): string {
    const keyParts = [
      params.startTime,
      params.endTime,
      JSON.stringify(params.dimensions || {}),
      JSON.stringify(params.metrics || []),
      JSON.stringify(params.groupBy || [])
    ];

    // Create hash-like key
    return keyParts.join('|');
  }

  /**
   * Invalidate cache for time range
   */
  invalidateTimeRange(startTime: string, endTime: string): number {
    const start = new Date(startTime).getTime();
    const end = new Date(endTime).getTime();

    let invalidated = 0;

    for (const key of this.keys()) {
      const parts = key.split('|');
      if (parts.length >= 2) {
        const keyStart = new Date(parts[0]).getTime();
        const keyEnd = new Date(parts[1]).getTime();

        // Invalidate if time ranges overlap
        if (!(keyEnd < start || keyStart > end)) {
          this.delete(key);
          invalidated++;
        }
      }
    }

    return invalidated;
  }
}

/**
 * Specialized cache for dashboard metric cards
 */
export class DashboardMetricCache extends DashboardCache<QueryResultRow[]> {
  constructor(config?: Partial<CacheConfig>) {
    super({
      ttl: 2 * 60 * 1000, // 2 minutes for metrics
      maxSize: 200,
      strategy: 'lru',
      compression: false,
      ...config
    });
  }

  /**
   * Generate cache key for metrics
   */
  static generateKey(params: {
    timeRange: { start: string; end: string };
    metrics: string[];
    dimensions?: Record<string, string>;
  }): string {
    return JSON.stringify(params);
  }
}

/**
 * Cache manager for coordinating multiple caches
 */
export class DashboardCacheManager {
  private queryCache: DashboardQueryCache;
  private metricCache: DashboardMetricCache;
  private enabled: boolean = true;

  constructor(config?: {
    queryCache?: Partial<CacheConfig>;
    metricCache?: Partial<CacheConfig>;
    enabled?: boolean;
  }) {
    this.queryCache = new DashboardQueryCache(config?.queryCache);
    this.metricCache = new DashboardMetricCache(config?.metricCache);
    this.enabled = config?.enabled ?? true;
  }

  /**
   * Get query cache
   */
  getQueryCache(): DashboardQueryCache {
    return this.queryCache;
  }

  /**
   * Get metric cache
   */
  getMetricCache(): DashboardMetricCache {
    return this.metricCache;
  }

  /**
   * Enable/disable caching
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  /**
   * Check if caching is enabled
   */
  isEnabled(): boolean {
    return this.enabled;
  }

  /**
   * Get combined cache statistics
   */
  getCombinedStats(): {
    queryCache: CacheStats;
    metricCache: CacheStats;
    combined: CacheStats;
  } {
    const queryStats = this.queryCache.getStats();
    const metricStats = this.metricCache.getStats();

    return {
      queryCache: queryStats,
      metricCache: metricStats,
      combined: {
        hits: queryStats.hits + metricStats.hits,
        misses: queryStats.misses + metricStats.misses,
        hitRate: (queryStats.hits + metricStats.hits) / 
                 (queryStats.hits + queryStats.misses + metricStats.hits + metricStats.misses),
        totalEntries: queryStats.totalEntries + metricStats.totalEntries,
        totalSize: queryStats.totalSize + metricStats.totalSize,
        evictions: queryStats.evictions + metricStats.evictions
      }
    };
  }

  /**
   * Clear all caches
   */
  clearAll(): void {
    this.queryCache.clear();
    this.metricCache.clear();
  }

  /**
   * Clean expired entries from all caches
   */
  cleanAllExpired(): number {
    return this.queryCache.cleanExpired() + this.metricCache.cleanExpired();
  }
}

/**
 * Global cache manager instance
 */
export const dashboardCacheManager = new DashboardCacheManager();
