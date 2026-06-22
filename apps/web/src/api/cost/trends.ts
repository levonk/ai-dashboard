/**
 * Cost Trend Analysis
 * 
 * Provides cost trend analysis and time-series data
 */

import {
  CostTrendRequest,
  CostTrendResponse,
  CostTrendData,
  CostAnomaly,
  CostForecast,
  CostDimension
} from './types';

export class CostTrendAnalyzer {
  /**
   * Analyze cost trends over time
   */
  async analyzeTrends(request: CostTrendRequest): Promise<CostTrendResponse> {
    const startTime = Date.now();
    const queryId = this.generateQueryId(request);

    try {
      // Execute trend analysis
      const trends = await this.executeTrendAnalysis(request);
      
      // Analyze trends for patterns
      const analysis = this.analyzeTrendPatterns(trends);
      
      // Generate forecast if requested
      let forecast: CostForecast | undefined;
      if (request.includeForecast) {
        forecast = this.generateForecast(trends, request.forecastPeriod || 7);
      }

      return {
        queryId,
        executionTimeMs: Date.now() - startTime,
        trends,
        analysis,
        forecast
      };
    } catch (error) {
      throw this.handleError(error, queryId);
    }
  }

  /**
   * Execute trend analysis
   */
  private async executeTrendAnalysis(request: CostTrendRequest): Promise<CostTrendData[]> {
    // TODO: Integrate with analytics-rs package for actual data
    // For now, generate mock trend data
    
    const trends: CostTrendData[] = [];
    const startDate = new Date(request.startTime);
    const endDate = new Date(request.endTime);
    
    // Generate data points based on granularity
    const granularityMs = this.getGranularityMs(request.granularity);
    let currentDate = startDate;
    
    while (currentDate <= endDate) {
      const baseCost = 100 + Math.random() * 50;
      const trendFactor = Math.sin((currentDate.getTime() - startDate.getTime()) / (7 * 24 * 60 * 60 * 1000) * Math.PI * 2) * 20;
      
      trends.push({
        timestamp: currentDate.toISOString(),
        cost: Math.max(0, baseCost + trendFactor),
        tokenCount: Math.floor((baseCost + trendFactor) * 1000),
        requestCount: Math.floor((baseCost + trendFactor) * 10)
      });
      
      currentDate = new Date(currentDate.getTime() + granularityMs);
    }

    // Apply dimension grouping if specified
    if (request.dimensions && request.dimensions.length > 0) {
      return this.groupByDimensions(trends, request.dimensions);
    }

    return trends;
  }

  /**
   * Analyze trend patterns
   */
  private analyzeTrendPatterns(trends: CostTrendData[]): {
    trend: 'increasing' | 'decreasing' | 'stable';
    growthRate: number;
    seasonality?: string;
    anomalies: CostAnomaly[];
  } {
    if (trends.length < 2) {
      return {
        trend: 'stable',
        growthRate: 0,
        anomalies: []
      };
    }

    // Calculate overall trend
    const firstCost = trends[0].cost;
    const lastCost = trends[trends.length - 1].cost;
    const growthRate = ((lastCost - firstCost) / firstCost) * 100;

    let trend: 'increasing' | 'decreasing' | 'stable';
    if (growthRate > 5) {
      trend = 'increasing';
    } else if (growthRate < -5) {
      trend = 'decreasing';
    } else {
      trend = 'stable';
    }

    // Detect anomalies
    const anomalies = this.detectAnomalies(trends);

    // Detect seasonality (simplified)
    let seasonality: string | undefined;
    if (trends.length >= 30) {
      seasonality = this.detectSeasonality(trends);
    }

    return {
      trend,
      growthRate,
      seasonality,
      anomalies
    };
  }

  /**
   * Detect anomalies in trend data
   */
  private detectAnomalies(trends: CostTrendData[]): CostAnomaly[] {
    const anomalies: CostAnomaly[] = [];
    
    if (trends.length < 3) return anomalies;

    // Calculate moving average and standard deviation
    const windowSize = Math.min(7, trends.length);
    const costs = trends.map(t => t.cost);
    
    for (let i = windowSize; i < trends.length; i++) {
      const window = costs.slice(i - windowSize, i);
      const mean = window.reduce((sum, val) => sum + val, 0) / windowSize;
      const variance = window.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / windowSize;
      const stdDev = Math.sqrt(variance);
      
      const currentCost = trends[i].cost;
      const threshold = 2 * stdDev; // 2 standard deviations
      
      if (Math.abs(currentCost - mean) > threshold) {
        const deviation = ((currentCost - mean) / mean) * 100;
        let severity: 'low' | 'medium' | 'high';
        
        if (Math.abs(deviation) < 20) {
          severity = 'low';
        } else if (Math.abs(deviation) < 50) {
          severity = 'medium';
        } else {
          severity = 'high';
        }

        anomalies.push({
          timestamp: trends[i].timestamp,
          expectedCost: mean,
          actualCost: currentCost,
          deviation,
          severity,
          possibleCauses: this.suggestAnomalyCauses(deviation)
        });
      }
    }

    return anomalies;
  }

  /**
   * Suggest possible causes for anomalies
   */
  private suggestAnomalyCauses(deviation: number): string[] {
    const causes: string[] = [];
    
    if (deviation > 0) {
      causes.push('Unexpected spike in usage');
      causes.push('New project or team onboarded');
      causes.push('Model or provider change');
    } else {
      causes.push('Usage reduction');
      causes.push('Optimization implemented');
      causes.push('Team or project completed');
    }
    
    return causes;
  }

  /**
   * Detect seasonality in trend data
   */
  private detectSeasonality(trends: CostTrendData[]): string | undefined {
    // Simplified seasonality detection
    // In production, this would use more sophisticated algorithms
    
    const costs = trends.map(t => t.cost);
    const weeklyPattern = this.checkWeeklyPattern(costs);
    const monthlyPattern = this.checkMonthlyPattern(costs);
    
    if (weeklyPattern > 0.7) {
      return 'weekly';
    }
    if (monthlyPattern > 0.7) {
      return 'monthly';
    }
    
    return undefined;
  }

  /**
   * Check for weekly pattern
   */
  private checkWeeklyPattern(costs: number[]): number {
    // Simplified pattern detection
    // In production, this would use autocorrelation or FFT
    return 0.5; // Placeholder
  }

  /**
   * Check for monthly pattern
   */
  private checkMonthlyPattern(costs: number[]): number {
    // Simplified pattern detection
    return 0.3; // Placeholder
  }

  /**
   * Generate cost forecast
   */
  private generateForecast(historicalData: CostTrendData[], forecastPeriod: number): CostForecast {
    const lastDataPoint = historicalData[historicalData.length - 1];
    const forecastData: CostTrendData[] = [];
    
    // Simple linear regression for forecasting
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
    const lastTimestamp = new Date(lastDataPoint.timestamp).getTime();
    const interval = this.getIntervalFromData(historicalData);
    
    for (let i = 1; i <= forecastPeriod; i++) {
      const forecastIndex = n + i - 1;
      const forecastCost = slope * forecastIndex + intercept;
      const forecastTimestamp = new Date(lastTimestamp + i * interval);
      
      forecastData.push({
        timestamp: forecastTimestamp.toISOString(),
        cost: Math.max(0, forecastCost),
        tokenCount: Math.floor(forecastCost * 1000),
        requestCount: Math.floor(forecastCost * 10)
      });
    }

    return {
      forecastData,
      confidence: 0.75, // Placeholder confidence score
      methodology: 'linear_regression',
      assumptions: [
        'Historical trends will continue',
        'No major changes in usage patterns',
        'Linear growth model'
      ]
    };
  }

  /**
   * Get interval between data points
   */
  private getIntervalFromData(data: CostTrendData[]): number {
    if (data.length < 2) return 24 * 60 * 60 * 1000; // Default to 1 day
    
    const first = new Date(data[0].timestamp).getTime();
    const second = new Date(data[1].timestamp).getTime();
    return second - first;
  }

  /**
   * Group trends by dimensions
   */
  private groupByDimensions(trends: CostTrendData[], dimensions: CostDimension[]): CostTrendData[] {
    // TODO: Implement dimension grouping for trends
    // For now, return ungrouped data
    return trends;
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
    console.error(`Cost trend analysis error (${queryId}):`, error);
    return new Error(`Cost trend analysis failed: ${error.message}`);
  }

  /**
   * Generate query ID
   */
  private generateQueryId(request: CostTrendRequest): string {
    const hash = this.hashRequest(request);
    return `cost_trend_${Date.now()}_${hash.substring(0, 8)}`;
  }

  /**
   * Hash request for identification
   */
  private hashRequest(request: CostTrendRequest): string {
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