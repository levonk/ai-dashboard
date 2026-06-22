/**
 * Cost Forecaster Tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { CostForecaster } from '../forecast';

describe('CostForecaster', () => {
  let forecaster: CostForecaster;

  beforeEach(() => {
    forecaster = new CostForecaster();
  });

  describe('generateForecast', () => {
    it('should generate forecast successfully', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'linear' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result).toHaveProperty('queryId');
      expect(result).toHaveProperty('executionTimeMs');
      expect(result).toHaveProperty('forecast');
      expect(result).toHaveProperty('historicalData');
      expect(result).toHaveProperty('modelMetrics');
      
      expect(result.forecast.forecastData).toBeInstanceOf(Array);
      expect(result.forecast.forecastData.length).toBe(7);
      
      expect(result.historicalData).toBeInstanceOf(Array);
      expect(result.historicalData.length).toBeGreaterThan(0);
    });

    it('should use linear regression method', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'linear' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result.forecast.methodology).toBe('linear_regression');
    });

    it('should use exponential growth method', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'exponential' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result.forecast.methodology).toBe('exponential_growth');
    });

    it('should use moving average method', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'moving_average' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result.forecast.methodology).toBe('moving_average');
    });

    it('should use ARIMA method', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'arima' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result.forecast.methodology).toBe('arima');
    });

    it('should calculate model metrics', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'linear' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result.modelMetrics).toBeDefined();
      expect(result.modelMetrics).toHaveProperty('accuracy');
      expect(result.modelMetrics).toHaveProperty('meanAbsoluteError');
      expect(result.modelMetrics).toHaveProperty('meanSquaredError');
      
      expect(result.modelMetrics.accuracy).toBeGreaterThanOrEqual(0);
      expect(result.modelMetrics.accuracy).toBeLessThanOrEqual(1);
    });
  });

  describe('forecast quality', () => {
    it('should provide confidence score', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'linear' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result.forecast.confidence).toBeGreaterThanOrEqual(0.5);
      expect(result.forecast.confidence).toBeLessThanOrEqual(0.95);
    });

    it('should include assumptions', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'linear' as const
      };

      const result = await forecaster.generateForecast(request);

      expect(result.forecast.assumptions).toBeInstanceOf(Array);
      expect(result.forecast.assumptions.length).toBeGreaterThan(0);
    });
  });

  describe('error handling', () => {
    it('should handle invalid time ranges', async () => {
      const request = {
        historicalStartTime: 'invalid-date',
        historicalEndTime: '2024-01-31T23:59:59Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'linear' as const
      };

      await expect(forecaster.generateForecast(request)).rejects.toThrow();
    });

    it('should handle insufficient historical data', async () => {
      const request = {
        historicalStartTime: '2024-01-01T00:00:00Z',
        historicalEndTime: '2024-01-02T00:00:00Z',
        forecastPeriod: 7,
        granularity: 'day',
        method: 'linear' as const
      };

      const result = await forecaster.generateForecast(request);
      expect(result).toBeDefined(); // Should handle gracefully
    });
  });
});