/**
 * Real-time Dashboard Data Processing
 * 
 * Handles real-time data updates for dashboard
 * Implements efficient streaming, debouncing, and delta updates
 */

import { QueryResultRow, AnalyticsQueryResponse } from '../../api/analytics/types';

export interface RealtimeConfig {
  enabled: boolean;
  updateInterval: number; // milliseconds
  debounceTime: number; // milliseconds
  maxBufferSize: number;
  deltaUpdates: boolean;
}

export interface RealtimeUpdate {
  timestamp: string;
  data: QueryResultRow[];
  type: 'full' | 'delta' | 'partial';
  metadata: {
    sequence: number;
    previousSequence?: number;
    updateId: string;
  };
}

export interface RealtimeSubscription {
  id: string;
  callback: (update: RealtimeUpdate) => void;
  filter?: (update: RealtimeUpdate) => boolean;
  lastSequence: number;
  active: boolean;
}

export interface RealtimeStats {
  totalUpdates: number;
  totalSubscribers: number;
  averageUpdateLatency: number;
  bufferUtilization: number;
  deltaUpdateRate: number;
}

/**
 * Real-time data processor for dashboard updates
 */
export class RealtimeDashboardProcessor {
  private config: RealtimeConfig;
  private buffer: QueryResultRow[] = [];
  private subscribers: Map<string, RealtimeSubscription> = new Map();
  private sequence: number = 0;
  private updateTimer: NodeJS.Timeout | null = null;
  private debounceTimer: NodeJS.Timeout | null = null;
  private stats: RealtimeStats = {
    totalUpdates: 0,
    totalSubscribers: 0,
    averageUpdateLatency: 0,
    bufferUtilization: 0,
    deltaUpdateRate: 0
  };
  private latencyHistory: number[] = [];

  constructor(config: Partial<RealtimeConfig> = {}) {
    this.config = {
      enabled: true,
      updateInterval: 5000, // 5 seconds default
      debounceTime: 1000, // 1 second default
      maxBufferSize: 1000,
      deltaUpdates: true,
      ...config
    };
  }

  /**
   * Add data to the real-time buffer
   */
  addData(data: QueryResultRow[]): void {
    if (!this.config.enabled) return;

    this.buffer.push(...data);

    // Trim buffer if it exceeds max size
    if (this.buffer.length > this.config.maxBufferSize) {
      this.buffer = this.buffer.slice(-this.config.maxBufferSize);
    }

    this.updateBufferUtilization();

    // Trigger debounced update
    this.scheduleUpdate();
  }

  /**
   * Subscribe to real-time updates
   */
  subscribe(
    callback: (update: RealtimeUpdate) => void,
    filter?: (update: RealtimeUpdate) => boolean
  ): string {
    const subscriptionId = this.generateId();
    
    const subscription: RealtimeSubscription = {
      id: subscriptionId,
      callback,
      filter,
      lastSequence: this.sequence,
      active: true
    };

    this.subscribers.set(subscriptionId, subscription);
    this.stats.totalSubscribers = this.subscribers.size;

    return subscriptionId;
  }

  /**
   * Unsubscribe from real-time updates
   */
  unsubscribe(subscriptionId: string): boolean {
    const deleted = this.subscribers.delete(subscriptionId);
    this.stats.totalSubscribers = this.subscribers.size;
    return deleted;
  }

  /**
   * Force immediate update (bypasses debounce)
   */
  forceUpdate(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
    this.processUpdate();
  }

  /**
   * Get current statistics
   */
  getStats(): RealtimeStats {
    return { ...this.stats };
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<RealtimeConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Clear the data buffer
   */
  clearBuffer(): void {
    this.buffer = [];
    this.updateBufferUtilization();
  }

  /**
   * Get current buffer contents
   */
  getBuffer(): QueryResultRow[] {
    return [...this.buffer];
  }

  /**
   * Check if real-time processing is enabled
   */
  isEnabled(): boolean {
    return this.config.enabled;
  }

  /**
   * Enable/disable real-time processing
   */
  setEnabled(enabled: boolean): void {
    this.config.enabled = enabled;
    
    if (!enabled) {
      if (this.updateTimer) {
        clearTimeout(this.updateTimer);
        this.updateTimer = null;
      }
      if (this.debounceTimer) {
        clearTimeout(this.debounceTimer);
        this.debounceTimer = null;
      }
    }
  }

  /**
   * Shut down the processor
   */
  shutdown(): void {
    this.setEnabled(false);
    this.subscribers.clear();
    this.clearBuffer();
  }

  // Private methods

  private scheduleUpdate(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    this.debounceTimer = setTimeout(() => {
      this.processUpdate();
      this.debounceTimer = null;
    }, this.config.debounceTime);
  }

  private processUpdate(): void {
    if (this.buffer.length === 0 || this.subscribers.size === 0) {
      return;
    }

    const startTime = Date.now();
    this.sequence++;

    const update = this.createUpdate();
    
    // Broadcast to subscribers
    let deltaCount = 0;
    for (const subscription of this.subscribers.values()) {
      if (!subscription.active) continue;

      // Apply filter if provided
      if (subscription.filter && !subscription.filter(update)) {
        continue;
      }

      // Send update
      try {
        subscription.callback(update);
        subscription.lastSequence = this.sequence;
        
        if (update.type === 'delta') {
          deltaCount++;
        }
      } catch (error) {
        console.error(`Error in subscription ${subscription.id}:`, error);
      }
    }

    // Update stats
    const latency = Date.now() - startTime;
    this.latencyHistory.push(latency);
    if (this.latencyHistory.length > 100) {
      this.latencyHistory.shift();
    }
    this.stats.averageUpdateLatency = 
      this.latencyHistory.reduce((sum, l) => sum + l, 0) / this.latencyHistory.length;
    
    this.stats.totalUpdates++;
    this.stats.deltaUpdateRate = deltaCount / this.subscribers.size;

    // Clear processed data
    if (update.type === 'full') {
      this.clearBuffer();
    } else {
      // Keep some data for delta calculations
      this.buffer = this.buffer.slice(-100);
    }
  }

  private createUpdate(): RealtimeUpdate {
    const updateId = this.generateId();
    
    // Determine update type
    let updateType: RealtimeUpdate['type'] = 'full';
    if (this.config.deltaUpdates && this.sequence > 0) {
      updateType = 'delta';
    }

    const update: RealtimeUpdate = {
      timestamp: new Date().toISOString(),
      data: [...this.buffer],
      type: updateType,
      metadata: {
        sequence: this.sequence,
        previousSequence: this.sequence - 1,
        updateId
      }
    };

    return update;
  }

  private generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private updateBufferUtilization(): void {
    this.stats.bufferUtilization = this.buffer.length / this.config.maxBufferSize;
  }
}

/**
 * Delta update calculator for efficient real-time updates
 */
export class DeltaUpdateCalculator {
  /**
   * Calculate delta between two datasets
   */
  static calculateDelta(
    previous: QueryResultRow[],
    current: QueryResultRow[]
  ): { added: QueryResultRow[]; modified: QueryResultRow[]; removed: string[] } {
    const previousMap = new Map<string, QueryResultRow>();
    const currentMap = new Map<string, QueryResultRow>();

    // Create maps for efficient lookup
    for (const row of previous) {
      const key = this.getRowKey(row);
      previousMap.set(key, row);
    }

    for (const row of current) {
      const key = this.getRowKey(row);
      currentMap.set(key, row);
    }

    // Find added and modified
    const added: QueryResultRow[] = [];
    const modified: QueryResultRow[] = [];

    for (const [key, currentRow] of currentMap.entries()) {
      const previousRow = previousMap.get(key);
      
      if (!previousRow) {
        added.push(currentRow);
      } else if (this.hasRowChanged(previousRow, currentRow)) {
        modified.push(currentRow);
      }
    }

    // Find removed
    const removed: string[] = [];
    for (const [key] of previousMap.entries()) {
      if (!currentMap.has(key)) {
        removed.push(key);
      }
    }

    return { added, modified, removed };
  }

  /**
   * Apply delta to a dataset
   */
  static applyDelta(
    base: QueryResultRow[],
    delta: { added: QueryResultRow[]; modified: QueryResultRow[]; removed: string[] }
  ): QueryResultRow[] {
    const result = [...base];
    const baseMap = new Map<string, number>();

    // Create index map
    for (let i = 0; i < result.length; i++) {
      const key = this.getRowKey(result[i]);
      baseMap.set(key, i);
    }

    // Apply additions
    for (const row of delta.added) {
      result.push(row);
    }

    // Apply modifications
    for (const row of delta.modified) {
      const key = this.getRowKey(row);
      const index = baseMap.get(key);
      if (index !== undefined) {
        result[index] = row;
      }
    }

    // Apply removals
    for (const key of delta.removed) {
      const index = baseMap.get(key);
      if (index !== undefined) {
        result.splice(index, 1);
        // Update indices
        for (const [k, i] of baseMap.entries()) {
          if (i > index) {
            baseMap.set(k, i - 1);
          }
        }
      }
    }

    return result;
  }

  /**
   * Compress delta to reduce payload size
   */
  static compressDelta(delta: {
    added: QueryResultRow[];
    modified: QueryResultRow[];
    removed: string[];
  }): { added: QueryResultRow[]; modified: QueryResultRow[]; removed: string[] } {
    // Only include changed fields in modified rows
    const compressedModified = delta.modified.map(row => {
      const compressed: QueryResultRow = {
        timestamp: row.timestamp,
        dimensions: {},
        metrics: {}
      };

      if (row.dimensions) {
        for (const [key, value] of Object.entries(row.dimensions)) {
          compressed.dimensions![key] = value;
        }
      }

      if (row.metrics) {
        for (const [key, value] of Object.entries(row.metrics)) {
          compressed.metrics![key] = value;
        }
      }

      return compressed;
    });

    return {
      added: delta.added,
      modified: compressedModified,
      removed: delta.removed
    };
  }

  // Helper methods

  private static getRowKey(row: QueryResultRow): string {
    const parts = [
      row.timestamp || '',
      JSON.stringify(row.dimensions || {}),
      JSON.stringify(row.metrics || {})
    ];
    return parts.join('|');
  }

  private static hasRowChanged(previous: QueryResultRow, current: QueryResultRow): boolean {
    // Compare timestamps
    if (previous.timestamp !== current.timestamp) return true;

    // Compare dimensions
    const prevDims = JSON.stringify(previous.dimensions || {});
    const currDims = JSON.stringify(current.dimensions || {});
    if (prevDims !== currDims) return true;

    // Compare metrics
    const prevMetrics = JSON.stringify(previous.metrics || {});
    const currMetrics = JSON.stringify(current.metrics || {});
    if (prevMetrics !== currMetrics) return true;

    return false;
  }
}

/**
 * Real-time update manager for coordinating multiple processors
 */
export class RealtimeUpdateManager {
  private processors: Map<string, RealtimeDashboardProcessor> = new Map();
  private globalEnabled: boolean = true;

  /**
   * Register a processor
   */
  registerProcessor(key: string, processor: RealtimeDashboardProcessor): void {
    this.processors.set(key, processor);
  }

  /**
   * Get a processor
   */
  getProcessor(key: string): RealtimeDashboardProcessor | undefined {
    return this.processors.get(key);
  }

  /**
   * Enable/disable all processors
   */
  setGlobalEnabled(enabled: boolean): void {
    this.globalEnabled = enabled;
    for (const processor of this.processors.values()) {
      processor.setEnabled(enabled);
    }
  }

  /**
   * Get combined statistics
   */
  getCombinedStats(): Record<string, RealtimeStats> {
    const stats: Record<string, RealtimeStats> = {};
    for (const [key, processor] of this.processors.entries()) {
      stats[key] = processor.getStats();
    }
    return stats;
  }

  /**
   * Shutdown all processors
   */
  shutdownAll(): void {
    for (const processor of this.processors.values()) {
      processor.shutdown();
    }
    this.processors.clear();
  }
}

/**
 * Global real-time processor instance
 */
export const realtimeProcessor = new RealtimeDashboardProcessor({
  enabled: true,
  updateInterval: 5000,
  debounceTime: 1000,
  maxBufferSize: 1000,
  deltaUpdates: true
});

/**
 * Global real-time manager instance
 */
export const realtimeManager = new RealtimeUpdateManager();
realtimeManager.registerProcessor('default', realtimeProcessor);
