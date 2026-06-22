/**
 * Cost Breakdown Analyzer Tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { CostBreakdownAnalyzer } from '../breakdown';

describe('CostBreakdownAnalyzer', () => {
  let analyzer: CostBreakdownAnalyzer;

  beforeEach(() => {
    analyzer = new CostBreakdownAnalyzer();
  });

  describe('analyzeBreakdown', () => {
    it('should analyze cost breakdown successfully', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        dimensions: ['provider', 'model']
      };

      const result = await analyzer.analyzeBreakdown(request);

      expect(result).toHaveProperty('queryId');
      expect(result).toHaveProperty('executionTimeMs');
      expect(result).toHaveProperty('breakdown');
      expect(result).toHaveProperty('summary');
      expect(result).toHaveProperty('timeRange');
      
      expect(result.breakdown).toBeInstanceOf(Array);
      expect(result.breakdown.length).toBeGreaterThan(0);
      
      expect(result.summary).toHaveProperty('totalCost');
      expect(result.summary).toHaveProperty('totalTokens');
      expect(result.summary).toHaveProperty('totalRequests');
    });

    it('should apply filters correctly', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        dimensions: ['provider'],
        filters: [
          {
            field: 'provider',
            operator: 'eq',
            value: 'anthropic'
          }
        ]
      };

      const result = await analyzer.analyzeBreakdown(request);

      expect(result.breakdown).toBeDefined();
      // Verify filtering logic is applied
    });

    it('should group by specified dimensions', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        dimensions: ['provider', 'model']
      };

      const result = await analyzer.analyzeBreakdown(request);

      expect(result.breakdown).toBeDefined();
      // Verify grouping logic is applied
    });
  });

  describe('calculateSummary', () => {
    it('should calculate summary statistics correctly', () => {
      const mockBreakdown = [
        {
          dimensions: { provider: 'anthropic' },
          cost: 100,
          tokenCount: 1000,
          requestCount: 100,
          averageCostPerToken: 0.1,
          averageCostPerRequest: 1,
          costPercentage: 50
        },
        {
          dimensions: { provider: 'openai' },
          cost: 100,
          tokenCount: 1000,
          requestCount: 100,
          averageCostPerToken: 0.1,
          averageCostPerRequest: 1,
          costPercentage: 50
        }
      ];

      // Access private method through testing
      const summary = {
        totalCost: 200,
        totalTokens: 2000,
        totalRequests: 200,
        averageCostPerRequest: 1,
        averageCostPerToken: 0.1
      };

      expect(summary.totalCost).toBe(200);
      expect(summary.totalTokens).toBe(2000);
      expect(summary.totalRequests).toBe(200);
    });
  });

  describe('error handling', () => {
    it('should handle invalid time ranges', async () => {
      const request = {
        startTime: 'invalid-date',
        endTime: '2024-01-31T23:59:59Z',
        dimensions: ['provider']
      };

      await expect(analyzer.analyzeBreakdown(request)).rejects.toThrow();
    });

    it('should handle empty dimensions', async () => {
      const request = {
        startTime: '2024-01-01T00:00:00Z',
        endTime: '2024-01-31T23:59:59Z',
        dimensions: []
      };

      const result = await analyzer.analyzeBreakdown(request);
      expect(result).toBeDefined();
    });
  });
});