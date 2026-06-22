/**
 * Time-Series Data Processing
 * 
 * Specialized processing for time-series data in dashboards
 * Handles resampling, interpolation, smoothing, and trend analysis
 */

import { QueryResultRow } from '../../api/analytics/types';

export interface TimeSeriesConfig {
  granularity: 'minute' | 'hour' | 'day' | 'week' | 'month';
  fillMissing: boolean;
  interpolation: 'linear' | 'previous' | 'none';
  smoothing?: {
    method: 'moving_average' | 'exponential';
    window: number;
    factor?: number; // for exponential smoothing
  };
}

export interface TimeSeriesData {
  timestamps: string[];
  values: number[];
  metadata?: {
    start: string;
    end: string;
    granularity: string;
    points: number;
  };
}

export interface TrendAnalysis {
  trend: 'increasing' | 'decreasing' | 'stable' | 'volatile';
  slope: number;
  correlation: number;
  confidence: number;
  seasonality?: {
    detected: boolean;
    period?: number;
  };
}

/**
 * Time-series processor for dashboard data
 */
export class TimeSeriesProcessor {
  /**
   * Process raw query results into time-series format
   */
  static processToTimeSeries(
    results: QueryResultRow[],
    metric: string,
    config: TimeSeriesConfig
  ): TimeSeriesData {
    // Extract and sort by timestamp
    const sorted = [...results]
      .filter(row => row.metrics?.[metric] !== undefined)
      .sort((a, b) => (a.timestamp || '').localeCompare(b.timestamp || ''));

    const timestamps = sorted.map(row => row.timestamp || '');
    const values = sorted.map(row => row.metrics?.[metric] || 0);

    // Apply resampling if needed
    const resampled = this.resample(timestamps, values, config);

    // Fill missing values if requested
    const filled = config.fillMissing ? 
      this.fillMissingValues(resampled.timestamps, resampled.values, config.interpolation) :
      resampled;

    // Apply smoothing if configured
    const smoothed = config.smoothing ?
      this.applySmoothing(filled.timestamps, filled.values, config.smoothing) :
      filled;

    return {
      timestamps: smoothed.timestamps,
      values: smoothed.values,
      metadata: {
        start: smoothed.timestamps[0] || '',
        end: smoothed.timestamps[smoothed.timestamps.length - 1] || '',
        granularity: config.granularity,
        points: smoothed.timestamps.length
      }
    };
  }

  /**
   * Resample time-series to target granularity
   */
  static resample(
    timestamps: string[],
    values: number[],
    config: TimeSeriesConfig
  ): { timestamps: string[]; values: number[] } {
    if (timestamps.length === 0) {
      return { timestamps: [], values: [] };
    }

    const targetGranularity = this.getGranularityMs(config.granularity);
    const buckets = new Map<string, number[]>();

    // Group values into time buckets
    for (let i = 0; i < timestamps.length; i++) {
      const timestamp = new Date(timestamps[i]);
      const bucketTime = this.roundToGranularity(timestamp, config.granularity);
      const bucketKey = bucketTime.toISOString();

      if (!buckets.has(bucketKey)) {
        buckets.set(bucketKey, []);
      }

      buckets.get(bucketKey)!.push(values[i]);
    }

    // Aggregate each bucket (sum for count metrics, avg for others)
    const resultTimestamps: string[] = [];
    const resultValues: number[] = [];

    for (const [key, bucketValues] of buckets.entries()) {
      resultTimestamps.push(key);
      resultValues.push(this.aggregateBucket(bucketValues));
    }

    return {
      timestamps: resultTimestamps.sort(),
      values: resultTimestamps
        .sort()
        .map(ts => buckets.get(ts)!)
        .map(v => this.aggregateBucket(v))
    };
  }

  /**
   * Fill missing values in time-series
   */
  static fillMissingValues(
    timestamps: string[],
    values: number[],
    method: 'linear' | 'previous' | 'none' = 'linear'
  ): { timestamps: string[]; values: number[] } {
    if (method === 'none' || timestamps.length === 0) {
      return { timestamps, values };
    }

    // Find gaps and fill them
    const resultTimestamps: string[] = [...timestamps];
    const resultValues: number[] = [...values];

    for (let i = 1; i < timestamps.length; i++) {
      const prevTime = new Date(timestamps[i - 1]);
      const currTime = new Date(timestamps[i]);
      const gap = currTime.getTime() - prevTime.getTime();

      // If gap is larger than expected (more than 2x the minimum interval)
      const minInterval = this.getMinInterval(timestamps);
      if (gap > minInterval * 2) {
        const filled = this.interpolateGap(
          prevTime,
          currTime,
          values[i - 1],
          values[i],
          method,
          Math.floor(gap / minInterval) - 1
        );

        // Insert filled values
        resultTimestamps.splice(i, 0, ...filled.timestamps);
        resultValues.splice(i, 0, ...filled.values);
      }
    }

    return { timestamps: resultTimestamps, values: resultValues };
  }

  /**
   * Apply smoothing to time-series
   */
  static applySmoothing(
    timestamps: string[],
    values: number[],
    smoothing: { method: 'moving_average' | 'exponential'; window: number; factor?: number }
  ): { timestamps: string[]; values: number[] } {
    if (smoothing.method === 'moving_average') {
      return this.movingAverage(timestamps, values, smoothing.window);
    } else if (smoothing.method === 'exponential') {
      return this.exponentialSmoothing(timestamps, values, smoothing.factor || 0.3);
    }

    return { timestamps, values };
  }

  /**
   * Calculate moving average
   */
  static movingAverage(
    timestamps: string[],
    values: number[],
    window: number
  ): { timestamps: string[]; values: number[] } {
    const smoothedValues: number[] = [];

    for (let i = 0; i < values.length; i++) {
      const start = Math.max(0, i - window + 1);
      const windowValues = values.slice(start, i + 1);
      const avg = windowValues.reduce((sum, v) => sum + v, 0) / windowValues.length;
      smoothedValues.push(avg);
    }

    return { timestamps, values: smoothedValues };
  }

  /**
   * Calculate exponential smoothing
   */
  static exponentialSmoothing(
    timestamps: string[],
    values: number[],
    alpha: number = 0.3
  ): { timestamps: string[]; values: number[] } {
    const smoothedValues: number[] = [];
    let previousSmoothed = values[0] || 0;

    for (let i = 0; i < values.length; i++) {
      const current = values[i] || 0;
      const smoothed = alpha * current + (1 - alpha) * previousSmoothed;
      smoothedValues.push(smoothed);
      previousSmoothed = smoothed;
    }

    return { timestamps, values: smoothedValues };
  }

  /**
   * Analyze trend in time-series
   */
  static analyzeTrend(data: TimeSeriesData): TrendAnalysis {
    if (data.values.length < 2) {
      return {
        trend: 'stable',
        slope: 0,
        correlation: 0,
        confidence: 0
      };
    }

    // Linear regression
    const n = data.values.length;
    const xValues = data.values.map((_, i) => i);
    const yValues = data.values;

    const sumX = xValues.reduce((sum, x) => sum + x, 0);
    const sumY = yValues.reduce((sum, y) => sum + y, 0);
    const sumXY = xValues.reduce((sum, x, i) => sum + x * yValues[i], 0);
    const sumX2 = xValues.reduce((sum, x) => sum + x * x, 0);
    const sumY2 = yValues.reduce((sum, y) => sum + y * y, 0);

    const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;

    // Calculate correlation coefficient
    const correlation = (n * sumXY - sumX * sumY) / 
      Math.sqrt((n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY));

    // Determine trend
    let trend: TrendAnalysis['trend'] = 'stable';
    if (Math.abs(slope) < 0.01) {
      trend = 'stable';
    } else if (slope > 0) {
      trend = 'increasing';
    } else {
      trend = 'decreasing';
    }

    // Check for volatility
    const variance = yValues.reduce((sum, y) => sum + Math.pow(y - (slope * xValues[yValues.indexOf(y)] + intercept), 2), 0) / n;
    const stdDev = Math.sqrt(variance);
    const mean = sumY / n;
    const coefficientOfVariation = stdDev / mean;

    if (coefficientOfVariation > 0.5) {
      trend = 'volatile';
    }

    // Confidence based on correlation strength
    const confidence = Math.abs(correlation);

    return {
      trend,
      slope,
      correlation,
      confidence
    };
  }

  /**
   * Detect seasonality in time-series
   */
  static detectSeasonality(data: TimeSeriesData): {
    detected: boolean;
    period?: number;
    strength?: number;
  } {
    if (data.values.length < 10) {
      return { detected: false };
    }

    // Simple autocorrelation-based seasonality detection
    const maxLag = Math.floor(data.values.length / 2);
    const autocorrelations: number[] = [];

    const mean = data.values.reduce((sum, v) => sum + v, 0) / data.values.length;
    const variance = data.values.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / data.values.length;

    for (let lag = 1; lag <= maxLag; lag++) {
      let sum = 0;
      for (let i = lag; i < data.values.length; i++) {
        sum += (data.values[i] - mean) * (data.values[i - lag] - mean);
      }
      const autocorr = sum / ((data.values.length - lag) * variance);
      autocorrelations.push(autocorr);
    }

    // Find significant peaks in autocorrelation
    const threshold = 0.3;
    const peaks: number[] = [];

    for (let i = 1; i < autocorrelations.length - 1; i++) {
      if (autocorrelations[i] > threshold &&
          autocorrelations[i] > autocorrelations[i - 1] &&
          autocorrelations[i] > autocorrelations[i + 1]) {
        peaks.push(i + 1); // +1 because lag is 1-indexed
      }
    }

    if (peaks.length > 0) {
      // Find the most common period among peaks
      const periodCounts = new Map<number, number>();
      for (const peak of peaks) {
        periodCounts.set(peak, (periodCounts.get(peak) || 0) + 1);
      }

      const mostCommonPeriod = Array.from(periodCounts.entries())
        .sort((a, b) => b[1] - a[1])[0][0];

      return {
        detected: true,
        period: mostCommonPeriod,
        strength: autocorrelations[mostCommonPeriod - 1]
      };
    }

    return { detected: false };
  }

  /**
   * Calculate rate of change
   */
  static calculateRateOfChange(
    data: TimeSeriesData,
    window: number = 1
  ): { timestamps: string[]; rates: number[] } {
    const rates: number[] = [];

    for (let i = window; i < data.values.length; i++) {
      const current = data.values[i];
      const previous = data.values[i - window];
      const rate = previous !== 0 ? ((current - previous) / previous) * 100 : 0;
      rates.push(rate);
    }

    return {
      timestamps: data.timestamps.slice(window),
      rates
    };
  }

  /**
   * Calculate cumulative sum
   */
  static calculateCumulative(data: TimeSeriesData): TimeSeriesData {
    const cumulative: number[] = [];
    let sum = 0;

    for (const value of data.values) {
      sum += value;
      cumulative.push(sum);
    }

    return {
      ...data,
      values: cumulative
    };
  }

  // Helper methods

  private static getGranularityMs(granularity: string): number {
    switch (granularity) {
      case 'minute': return 60 * 1000;
      case 'hour': return 60 * 60 * 1000;
      case 'day': return 24 * 60 * 60 * 1000;
      case 'week': return 7 * 24 * 60 * 60 * 1000;
      case 'month': return 30 * 24 * 60 * 60 * 1000;
      default: return 60 * 60 * 1000;
    }
  }

  private static roundToGranularity(date: Date, granularity: string): Date {
    const rounded = new Date(date);

    switch (granularity) {
      case 'minute':
        rounded.setSeconds(0, 0);
        break;
      case 'hour':
        rounded.setMinutes(0, 0, 0);
        break;
      case 'day':
        rounded.setHours(0, 0, 0, 0);
        break;
      case 'week':
        const dayOfWeek = rounded.getDay();
        rounded.setDate(rounded.getDate() - dayOfWeek);
        rounded.setHours(0, 0, 0, 0);
        break;
      case 'month':
        rounded.setDate(1);
        rounded.setHours(0, 0, 0, 0);
        break;
    }

    return rounded;
  }

  private static aggregateBucket(values: number[]): number {
    // Use sum for count-like metrics, could be configurable
    return values.reduce((sum, v) => sum + v, 0);
  }

  private static getMinInterval(timestamps: string[]): number {
    if (timestamps.length < 2) return 60 * 1000; // default to 1 minute

    let minInterval = Infinity;
    for (let i = 1; i < timestamps.length; i++) {
      const interval = new Date(timestamps[i]).getTime() - new Date(timestamps[i - 1]).getTime();
      minInterval = Math.min(minInterval, interval);
    }

    return minInterval;
  }

  private static interpolateGap(
    startTime: Date,
    endTime: Date,
    startValue: number,
    endValue: number,
    method: 'linear' | 'previous' | 'none',
    points: number
  ): { timestamps: string[]; values: number[] } {
    const timestamps: string[] = [];
    const values: number[] = [];

    const interval = (endTime.getTime() - startTime.getTime()) / (points + 1);

    for (let i = 1; i <= points; i++) {
      const timestamp = new Date(startTime.getTime() + i * interval);
      timestamps.push(timestamp.toISOString());

      if (method === 'linear') {
        const t = i / (points + 1);
        values.push(startValue + t * (endValue - startValue));
      } else if (method === 'previous') {
        values.push(startValue);
      } else {
        values.push(0);
      }
    }

    return { timestamps, values };
  }
}

/**
 * Pre-built time-series processing patterns
 */
export const TimeSeriesPatterns = {
  /**
   * Daily time-series with linear interpolation
   */
  dailyWithInterpolation(results: QueryResultRow[], metric: string): TimeSeriesData {
    return TimeSeriesProcessor.processToTimeSeries(results, metric, {
      granularity: 'day',
      fillMissing: true,
      interpolation: 'linear'
    });
  },

  /**
   * Hourly time-series with exponential smoothing
   */
  hourlySmoothed(results: QueryResultRow[], metric: string): TimeSeriesData {
    return TimeSeriesProcessor.processToTimeSeries(results, metric, {
      granularity: 'hour',
      fillMissing: true,
      interpolation: 'previous',
      smoothing: {
        method: 'exponential',
        window: 12,
        factor: 0.3
      }
    });
  },

  /**
   * Weekly time-series with moving average
   */
  weeklySmoothed(results: QueryResultRow[], metric: string): TimeSeriesData {
    return TimeSeriesProcessor.processToTimeSeries(results, metric, {
      granularity: 'week',
      fillMissing: true,
      interpolation: 'linear',
      smoothing: {
        method: 'moving_average',
        window: 4
      }
    });
  }
};
