/**
 * Cost Trend Analyzer Tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { CostTrendAnalyzer } from '../trends';

describe('CostTrendAnalyzer', () => {
  let analyzer: CostTrendAnalyzer;

  beforeEach(() => {
    analyzer = new CostTrendAnalyzer();
  });

  describe('analyzeTrends', () => {
    it('should analyze cost trends successfully', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        granularity: 'day',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);

      expect(result).toHaveProperty('queryId');
      expect(result).toHaveProperty('executionTimeMs');
      expect(result).toHaveProperty('trends');
      expect(result).toHaveProperty('analysis');
      
      expect(result.trends).toBeInstanceOf(Array);
      expect(result.trends.length).toBeGreaterThan(0);
      
      expect(result.analysis).toHaveProperty('trend');
      expect(result.analysis).toHaveProperty('growthRate');
      expect(result.analysis).toHaveProperty('anomalies');
    });

    it('should include forecast when requested', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        granularity: 'day',
        includeForecast: true,
        forecastPeriod: 7
      };

      const result = await analyzer.analyzeTrends(request);

      expect(result.forecast).toBeDefined();
      expect(result.forecast).toHaveProperty('forecastData');
      expect(result.forecast).toHaveProperty('confidence');
      expect(result.forecast).toHaveProperty('methodology');
    });

    it('should detect anomalies in trend data', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        granularity: 'day',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);

      expect(result.analysis.anomalies).toBeInstanceOf(Array);
    });

    it('should analyze trend direction correctly', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        granularity: 'day',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);

      expect(['increasing', 'decreasing', 'stable']).toContain(result.analysis.trend);
      expect(typeof result.analysis.growthRate).toBe('number');
    });
  });

  describe('granularity handling', () => {
    it('should handle hourly granularity', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-02T00:00:00Z',
        granularity: 'hour',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);
      expect(result.trends).toBeDefined();
    });

    it('should handle daily granularity', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        granularity: 'day',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);
      expect(result.trends).toBeDefined();
    });

    it('should handle weekly granularity', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-03-31T23:59:59Z',
        granularity: 'week',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);
      expect(result.trends).toBeDefined();
    });

    it('should handle monthly granularity', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-12-31T23:59:59Z',
        granularity: 'month',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);
      expect(result.trends).toBeDefined();
    });
  });

  describe('error handling', () => {
    it('should handle invalid time ranges', async () => {
      const request = {
        startTime: 'invalid-date',
        endTime: '2024-01-31T23:59:59Z',
        granularity: 'day',
        includeForecast: false
      };

      await expect(analyzer.analyzeTrends(request)).rejects.toThrow();
    });

    it('should handle end time before start time', async () => {
      const request = {
        startTime: '2024-01-31T23:59:59Z',
        endTime: '2024-01-01T00:00:00Z',
        granularity: 'day',
        includeForecast: false
      };

      const result = await analyzer.analyzeTrends(request);
      expect(result.trends).toBeDefined(); // Should handle gracefully
    });
  });
});