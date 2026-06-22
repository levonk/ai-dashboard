/**
 * Query Result Cache
 * 
 * Provides caching for analytics query results to improve performance
 * and reduce load on the analytics engine
 */

import {
  AnalyticsQueryRequest,
  AnalyticsQueryResponse,
  QueryCacheEntry
} from './types';

export class QueryCache {
  private cache: Map<string, QueryCacheEntry>;
  private defaultTTL: number; // Time to live in milliseconds

  constructor(ttl: number = 5 * 60 * 1000) { // 5 minutes default
    this.cache = new Map();
    this.defaultTTL = ttl;
  }

  /**
   * Generate a cache key from a request
   */
  private generateKey(request: AnalyticsQueryRequest): string {
    const str = JSON.stringify(request);
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    return `query_${Math.abs(hash).toString(16)}`;
  }

  /**
   * Get a cached response for a request
   */
  get(request: AnalyticsQueryRequest): AnalyticsQueryResponse | null {
    const key = this.generateKey(request);
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    // Check if entry has expired
    if (new Date() > new Date(entry.expiresAt)) {
      this.cache.delete(key);
      return null;
    }

    // Update hit count
    entry.hitCount++;

    return entry.response;
  }

  /**
   * Set a cached response for a request
   */
  set(request: AnalyticsQueryRequest, response: AnalyticsQueryResponse, ttl?: number): void {
    const key = this.generateKey(request);
    const now = new Date();
    const expiresAt = new Date(now.getTime() + (ttl || this.defaultTTL));

    const entry: QueryCacheEntry = {
      queryId: response.queryId,
      request,
      response,
      createdAt: now.toISOString(),
      expiresAt: expiresAt.toISOString(),
      hitCount: 0
    };

    this.cache.set(key, entry);
  }

  /**
   * Invalidate cache entries for a specific dimension
   */
  invalidateByDimension(dimension: string, value: string): void {
    const keysToDelete: string[] = [];

    for (const [key, entry] of this.cache.entries()) {
      const request = entry.request;
      const dimensions = request.dimensions;

      // Check if the dimension matches
      if (this.dimensionMatches(dimensions, dimension, value)) {
        keysToDelete.push(key);
      }
    }

    keysToDelete.forEach(key => this.cache.delete(key));
  }

  /**
   * Check if a dimension value matches in the request
   */
  private dimensionMatches(dimensions: any, dimension: string, value: string): boolean {
    const dimensionMap: Record<string, string[]> = {
      clientId: dimensions.clientIds || [],
      aiClient: dimensions.aiClients || [],
      team: dimensions.teams || [],
      pipelineStage: dimensions.pipelineStages || [],
      provider: dimensions.providers || [],
      model: dimensions.models || [],
      inputType: dimensions.inputTypes || []
    };

    const values = dimensionMap[dimension];
    return values && values.includes(value);
  }

  /**
   * Clear all cache entries
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Clear expired cache entries
   */
  clearExpired(): void {
    const now = new Date();
    const keysToDelete: string[] = [];

    for (const [key, entry] of this.cache.entries()) {
      if (now > new Date(entry.expiresAt)) {
        keysToDelete.push(key);
      }
    }

    keysToDelete.forEach(key => this.cache.delete(key));
  }

  /**
   * Get cache statistics
   */
  getStats(): {
    size: number;
    hitRate: number;
    totalHits: number;
    expiredCount: number;
  } {
    let totalHits = 0;
    let expiredCount = 0;
    const now = new Date();

    for (const entry of this.cache.values()) {
      totalHits += entry.hitCount;
      if (now > new Date(entry.expiresAt)) {
        expiredCount++;
      }
    }

    const size = this.cache.size;
    const hitRate = size > 0 ? totalHits / size : 0;

    return {
      size,
      hitRate,
      totalHits,
      expiredCount
    };
  }

  /**
   * Set a new default TTL
   */
  setDefaultTTL(ttl: number): void {
    this.defaultTTL = ttl;
  }

  /**
   * Get cache entries for debugging
   */
  getEntries(): QueryCacheEntry[] {
    return Array.from(this.cache.values());
  }
}

/**
 * Singleton instance of the query cache
 */
export const queryCache = new QueryCache();
