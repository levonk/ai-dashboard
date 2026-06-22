/**
 * Cost Forecasting and Projection
 * 
 * Provides cost forecasting capabilities using various prediction methods
 */

import {
  CostForecastRequest,
  CostForecastResponse,
  CostTrendData,
  CostForecast
} from './types';

export class CostForecaster {
  /**
   * Generate cost forecast
   */
  async generateForecast(request: CostForecastRequest): Promise<CostForecastResponse> {
    const startTime = Date.now();
    const queryId = this.generateQueryId(request);

    try {
      // Get historical data
      const historicalData = await this.getHistoricalData(request);
      
      // Generate forecast using specified method
      const forecast = this.executeForecast(historicalData, request);
      
      // Calculate model metrics
      const modelMetrics = this.calculateModelMetrics(historicalData, forecast);

      return {
        queryId,
        executionTimeMs: Date.now() - startTime,
        forecast,
        historicalData,
        modelMetrics
      };
    } catch (error) {
      throw this.handleError(error, queryId);
    }
  }

  /**
   * Get historical data for forecasting
   */
  private async getHistoricalData(request: CostForecastRequest): Promise<CostTrendData[]> {
    // TODO: Integrate with analytics-rs package for actual data
    // For now, generate mock historical data
    
    const historicalData: CostTrendData[] = [];
    const startDate = new Date(request.historicalStartTime);
    const endDate = new Date(request.historicalEndTime);
    
    const granularityMs = this.getGranularityMs(request.granularity);
    let currentDate = startDate;
    
    while (currentDate <= endDate) {
      const daysSinceStart = (currentDate.getTime() - startDate.getTime()) / (24 * 60 * 60 * 1000);
      const baseCost = 100 + daysSinceStart * 0.5 + Math.random() * 30;
      
      historicalData.push({
        timestamp: currentDate.toISOString(),
        cost: Math.max(0, baseCost),
        tokenCount: Math.floor(baseCost * 1000),
        requestCount: Math.floor(baseCost * 10)
      });
      
      currentDate = new Date(currentDate.getTime() + granularityMs);
    }

    return historicalData;
  }

  /**
   * Execute forecast using specified method
   */
  private executeForecast(historicalData: CostTrendData[], request: CostForecastRequest): CostForecast {
    const method = request.method || 'linear';
    
    switch (method) {
      case 'linear':
        return this.linearRegressionForecast(historicalData, request.forecastPeriod, request.granularity);
      case 'exponential':
        return this.exponentialForecast(historicalData, request.forecastPeriod, request.granularity);
      case 'moving_average':
        return this.movingAverageForecast(historicalData, request.forecastPeriod, request.granularity);
      case 'arima':
        return this.arimaForecast(historicalData, request.forecastPeriod, request.granularity);
      default:
        return this.linearRegressionForecast(historicalData, request.forecastPeriod, request.granularity);
    }
  }

  /**
   * Linear regression forecast
   */
  private linearRegressionForecast(
    historicalData: CostTrendData[],
    forecastPeriod: number,
    granularity: string
  ): CostForecast {
    const costs = historicalData.map(t => t.cost);
    const n = costs.length;
    
    // Calculate linear regression
    let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0;
    for (let i = 0; i < n; i++) {
      sumX += i;
      sumY += costs[i];
      sumXY += i * costs[i];
      sumX2 += i * i;
    }
    
    const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;
    
    // Generate forecast
    const forecastData = this.generateForecastData(
      historicalData,
      forecastPeriod,
      granularity,
      (index) => slope * index + intercept
    );

    return {
      forecastData,
      confidence: this.calculateConfidence(historicalData, slope, intercept),
      methodology: 'linear_regression',
      assumptions: [
        'Linear trend continues',
        'Constant growth rate',
        'No seasonal patterns'
      ]
    };
  }

  /**
   * Exponential growth forecast
   */
  private exponentialForecast(
    historicalData: CostTrendData[],
    forecastPeriod: number,
    granularity: string
  ): CostForecast {
    const costs = historicalData.map(t => t.cost);
    const n = costs.length;
    
    // Calculate exponential regression (log-linear)
    const logCosts = costs.map(c => Math.log(c));
    
    let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0;
    for (let i = 0; i < n; i++) {
      sumX += i;
      sumY += logCosts[i];
      sumXY += i * logCosts[i];
      sumX2 += i * i;
    }
    
    const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;
    
    // Generate forecast
    const forecastData = this.generateForecastData(
      historicalData,
      forecastPeriod,
      granularity,
      (index) => Math.exp(intercept + slope * index)
    );

    return {
      forecastData,
      confidence: this.calculateConfidence(historicalData, slope, intercept, true),
      methodology: 'exponential_growth',
      assumptions: [
        'Exponential growth continues',
        'Constant percentage growth rate',
        'Accelerating adoption'
      ]
    };
  }

  /**
   * Moving average forecast
   */
  private movingAverageForecast(
    historicalData: CostTrendData[],
    forecastPeriod: number,
    granularity: string
  ): CostForecast {
    const costs = historicalData.map(t => t.cost);
    const windowSize = Math.min(7, Math.floor(costs.length / 3));
    
    // Calculate moving average
    const movingAverages: number[] = [];
    for (let i = windowSize; i < costs.length; i++) {
      const window = costs.slice(i - windowSize, i);
      const avg = window.reduce((sum, val) => sum + val, 0) / windowSize;
      movingAverages.push(avg);
    }
    
    // Use the last moving average as the forecast base
    const lastAverage = movingAverages[movingAverages.length - 1];
    
    // Generate forecast (flat projection with slight trend)
    const recentTrend = costs[costs.length - 1] - costs[costs.length - windowSize];
    const trendPerPeriod = recentTrend / windowSize;
    
    const forecastData = this.generateForecastData(
      historicalData,
      forecastPeriod,
      granularity,
      (index) => lastAverage + (index - costs.length + 1) * trendPerPeriod
    );

    return {
      forecastData,
      confidence: 0.65, // Moving average has lower confidence
      methodology: 'moving_average',
      assumptions: [
        'Recent trends continue',
        'Short-term patterns persist',
        'No major changes expected'
      ]
    };
  }

  /**
   * ARIMA forecast (simplified implementation)
   */
  private arimaForecast(
    historicalData: CostTrendData[],
    forecastPeriod: number,
    granularity: string
  ): CostForecast {
    // Simplified ARIMA implementation
    // In production, this would use a proper statistical library
    
    const costs = historicalData.map(t => t.cost);
    const n = costs.length;
    
    // Simple autoregressive model (AR(1))
    let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0;
    for (let i = 1; i < n; i++) {
      sumX += costs[i - 1];
      sumY += costs[i];
      sumXY += costs[i - 1] * costs[i];
      sumX2 += costs[i - 1] * costs[i - 1];
    }
    
    const phi = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    const mean = costs.reduce((sum, val) => sum + val, 0) / n;
    
    // Generate forecast using AR(1) model
    const forecastData: CostTrendData[] = [];
    const lastDataPoint = historicalData[historicalData.length - 1];
    const lastTimestamp = new Date(lastDataPoint.timestamp).getTime();
    const interval = this.getIntervalFromData(historicalData);
    
    let lastCost = costs[costs.length - 1];
    for (let i = 1; i <= forecastPeriod; i++) {
      const forecastCost = mean + phi * (lastCost - mean);
      const forecastTimestamp = new Date(lastTimestamp + i * interval);
      
      forecastData.push({
        timestamp: forecastTimestamp.toISOString(),
        cost: Math.max(0, forecastCost),
        tokenCount: Math.floor(forecastCost * 1000),
        requestCount: Math.floor(forecastCost * 10)
      });
      
      lastCost = forecastCost;
    }

    return {
      forecastData,
      confidence: 0.72,
      methodology: 'arima',
      assumptions: [
        'Time series is stationary',
        'Autoregressive patterns persist',
        'Linear dependencies between observations'
      ]
    };
  }

  /**
   * Generate forecast data using a prediction function
   */
  private generateForecastData(
    historicalData: CostTrendData[],
    forecastPeriod: number,
    granularity: string,
    predictFn: (index: number) => number
  ): CostTrendData[] {
    const forecastData: CostTrendData[] = [];
    const lastDataPoint = historicalData[historicalData.length - 1];
    const lastTimestamp = new Date(lastDataPoint.timestamp).getTime();
    const interval = this.getIntervalFromData(historicalData);
    
    for (let i = 1; i <= forecastPeriod; i++) {
      const forecastIndex = historicalData.length + i - 1;
      const forecastCost = Math.max(0, predictFn(forecastIndex));
      const forecastTimestamp = new Date(lastTimestamp + i * interval);
      
      forecastData.push({
        timestamp: forecastTimestamp.toISOString(),
        cost: forecastCost,
        tokenCount: Math.floor(forecastCost * 1000),
        requestCount: Math.floor(forecastCost * 10)
      });
    }
    
    return forecastData;
  }

  /**
   * Calculate confidence score for forecast
   */
  private calculateConfidence(
    historicalData: CostTrendData[],
    slope: number,
    intercept: number,
    isLogarithmic: boolean = false
  ): number {
    const costs = historicalData.map(t => t.cost);
    const n = costs.length;
    
    // Calculate R-squared
    let sumY = 0, sumYhat = 0, sumYYhat = 0, sumY2 = 0, sumYhat2 = 0;
    
    for (let i = 0; i < n; i++) {
      const y = isLogarithmic ? Math.log(costs[i]) : costs[i];
      const yhat = intercept + slope * i;
      
      sumY += y;
      sumYhat += yhat;
      sumYYhat += y * yhat;
      sumY2 += y * y;
      sumYhat2 += yhat * yhat;
    }
    
    const numerator = n * sumYYhat - sumY * sumYhat;
    const denominator = Math.sqrt((n * sumY2 - sumY * sumY) * (n * sumYhat2 - sumYhat * sumYhat));
    
    const rSquared = denominator !== 0 ? (numerator / denominator) ** 2 : 0;
    
    // Convert R-squared to confidence score
    return Math.max(0.5, Math.min(0.95, rSquared));
  }

  /**
   * Calculate model performance metrics
   */
  private calculateModelMetrics(
    historicalData: CostTrendData[],
    forecast: CostForecast
  ): {
    accuracy: number;
    meanAbsoluteError: number;
    meanSquaredError: number;
  } {
    // Use last 20% of historical data for validation
    const validationSize = Math.max(1, Math.floor(historicalData.length * 0.2));
    const validationData = historicalData.slice(-validationSize);
    const trainingData = historicalData.slice(0, -validationSize);
    
    // Generate forecast for validation period
    const validationForecast = this.generateForecastData(
      trainingData,
      validationSize,
      'day',
      (index) => {
        // Simple linear projection for validation
        const costs = trainingData.map(t => t.cost);
        const avgCost = costs.reduce((sum, val) => sum + val, 0) / costs.length;
        return avgCost;
      }
    );
    
    // Calculate errors
    let absoluteErrorSum = 0;
    let squaredErrorSum = 0;
    
    for (let i = 0; i < validationData.length; i++) {
      const actual = validationData[i].cost;
      const predicted = validationForecast[i]?.cost || 0;
      
      absoluteErrorSum += Math.abs(actual - predicted);
      squaredErrorSum += Math.pow(actual - predicted, 2);
    }
    
    const meanAbsoluteError = absoluteErrorSum / validationData.length;
    const meanSquaredError = squaredErrorSum / validationData.length;
    
    // Calculate accuracy (inverse of normalized MAE)
    const avgCost = validationData.reduce((sum, t) => sum + t.cost, 0) / validationData.length;
    const normalizedMAE = meanAbsoluteError / avgCost;
    const accuracy = Math.max(0, 1 - normalizedMAE);
    
    return {
      accuracy,
      meanAbsoluteError,
      meanSquaredError
    };
  }

  /**
   * Get interval between data points
   */
  private getIntervalFromData(data: CostTrendData[]): number {
    if (data.length < 2) return 24 * 60 * 60 * 1000;
    
    const first = new Date(data[0].timestamp).getTime();
    const second = new Date(data[1].timestamp).getTime();
    return second - first;
  }

  /**
   * Get granularity in milliseconds
   */
  private getGranularityMs(granularity: string): number {
    switch (granularity) {
      case 'hour':
        return 60 * 60 * 1000;
      case 'day':
        return 24 * 60 * 60 * 1000;
      case 'week':
        return 7 * 24 * 60 * 60 * 1000;
      case 'month':
        return 30 * 24 * 60 * 60 * 1000;
      default:
        return 24 * 60 * 60 * 1000;
    }
  }

  /**
   * Handle errors
   */
  private handleError(error: any, queryId: string): Error {
    console.error(`Cost forecasting error (${queryId}):`, error);
    return new Error(`Cost forecasting failed: ${error.message}`);
  }

  /**
   * Generate query ID
   */
  private generateQueryId(request: CostForecastRequest): string {
    const hash = this.hashRequest(request);
    return `cost_forecast_${Date.now()}_${hash.substring(0, 8)}`;
  }

  /**
   * Hash request for identification
   */
  private hashRequest(request: CostForecastRequest): string {
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