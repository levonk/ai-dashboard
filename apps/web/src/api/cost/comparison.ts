/**
 * Cost Comparison Analysis
 * 
 * Compares costs across providers, models, and other dimensions
 */

import {
  CostComparisonRequest,
  CostComparisonResponse,
  CostComparisonItem,
  CostRanking,
  CostDimension
} from './types';
import { CostBreakdownAnalyzer } from './breakdown';

export class CostComparator {
  private breakdownAnalyzer: CostBreakdownAnalyzer;

  constructor() {
    this.breakdownAnalyzer = new CostBreakdownAnalyzer();
  }

  /**
   * Compare costs across dimensions
   */
  async compareCosts(request: CostComparisonRequest): Promise<CostComparisonResponse> {
    const startTime = Date.now();
    const queryId = this.generateQueryId(request);

    try {
      // Get breakdown data for comparison
      const breakdownData = await this.getBreakdownData(request);
      
      // Generate comparison items
      const comparisons = this.generateComparisons(breakdownData, request);
      
      // Generate rankings
      const rankings = this.generateRankings(comparisons, request);
      
      // Generate insights
      const insights = this.generateInsights(comparisons, rankings);

      return {
        queryId,
        executionTimeMs: Date.now() - startTime,
        comparisons,
        rankings,
        insights
      };
    } catch (error) {
      throw this.handleError(error, queryId);
    }
  }

  /**
   * Get breakdown data for comparison
   */
  private async getBreakdownData(request: CostComparisonRequest): Promise<any[]> {
    const breakdownRequest = {
      startTime: request.startTime,
      endTime: request.endTime,
      dimensions: request.compareBy,
      filters: request.filters
    };

    const breakdownResponse = await this.breakdownAnalyzer.analyzeBreakdown(breakdownRequest);
    return breakdownResponse.breakdown;
  }

  /**
   * Generate comparison items
   */
  private generateComparisons(breakdownData: any[], request: CostComparisonRequest): CostComparisonItem[] {
    return breakdownData.map((item, index) => {
      const comparisonItem: CostComparisonItem = {
        dimensions: item.dimensions,
        cost: item.cost,
        costRank: 0, // Will be calculated
        costPerToken: item.averageCostPerToken,
        costPerRequest: item.averageCostPerRequest,
        efficiencyRank: 0 // Will be calculated
      };

      // Add performance metrics if requested
      if (request.options.includePerformanceMetrics) {
        comparisonItem.averageLatency = this.generateMockLatency();
        comparisonItem.errorRate = this.generateMockErrorRate();
        comparisonItem.performanceRank = 0; // Will be calculated
      }

      return comparisonItem;
    });
  }

  /**
   * Generate rankings
   */
  private generateRankings(comparisons: CostComparisonItem[], request: CostComparisonRequest): {
    byCost: CostRanking[];
    byEfficiency: CostRanking[];
    byPerformance: CostRanking[];
  } {
    // Sort by cost (lower is better)
    const byCost = this.sortByMetric(comparisons, 'cost', 'asc');
    
    // Sort by efficiency (cost per token, lower is better)
    const byEfficiency = this.sortByMetric(comparisons, 'costPerToken', 'asc');
    
    // Sort by performance (if available, lower latency and error rate is better)
    const byPerformance = request.options.includePerformanceMetrics
      ? this.sortByPerformance(comparisons)
      : [];

    return {
      byCost,
      byEfficiency,
      byPerformance
    };
  }

  /**
   * Sort by metric and create rankings
   */
  private sortByMetric(comparisons: CostComparisonItem[], metric: keyof CostComparisonItem, order: 'asc' | 'desc'): CostRanking[] {
    const sorted = [...comparisons].sort((a, b) => {
      const aVal = a[metric] as number;
      const bVal = b[metric] as number;
      return order === 'asc' ? aVal - bVal : bVal - aVal;
    });

    return sorted.map((item, index) => ({
      dimensions: item.dimensions,
      value: item[metric] as number,
      rank: index + 1,
      percentile: ((sorted.length - index) / sorted.length) * 100
    }));
  }

  /**
   * Sort by performance metrics
   */
  private sortByPerformance(comparisons: CostComparisonItem[]): CostRanking[] {
    const withPerformance = comparisons.filter(c => c.averageLatency !== undefined && c.errorRate !== undefined);
    
    const sorted = [...withPerformance].sort((a, b) => {
      // Combine latency and error rate into a single score
      const aScore = (a.averageLatency || 0) * 0.7 + (a.errorRate || 0) * 30;
      const bScore = (b.averageLatency || 0) * 0.7 + (b.errorRate || 0) * 30;
      return aScore - bScore;
    });

    return sorted.map((item, index) => ({
      dimensions: item.dimensions,
      value: (item.averageLatency || 0) * 0.7 + (item.errorRate || 0) * 30,
      rank: index + 1,
      percentile: ((sorted.length - index) / sorted.length) * 100
    }));
  }

  /**
   * Generate insights from comparisons
   */
  private generateInsights(comparisons: CostComparisonItem[], rankings: {
    byCost: CostRanking[];
    byEfficiency: CostRanking[];
    byPerformance: CostRanking[];
  }): string[] {
    const insights: string[] = [];

    if (comparisons.length === 0) {
      return ['No data available for comparison'];
    }

    // Cost insights
    const lowestCost = rankings.byCost[0];
    const highestCost = rankings.byCost[rankings.byCost.length - 1];
    const costDifference = highestCost.value - lowestCost.value;
    const costPercentage = (costDifference / highestCost.value) * 100;
    
    insights.push(
      `Lowest cost option is ${this.formatDimensions(lowestCost.dimensions)} at $${lowestCost.value.toFixed(2)}, ` +
      `which is ${costPercentage.toFixed(1)}% cheaper than the most expensive option.`
    );

    // Efficiency insights
    const mostEfficient = rankings.byEfficiency[0];
    const leastEfficient = rankings.byEfficiency[rankings.byEfficiency.length - 1];
    const efficiencyDifference = leastEfficient.value - mostEfficient.value;
    const efficiencyPercentage = (efficiencyDifference / leastEfficient.value) * 100;
    
    insights.push(
      `Most cost-efficient option is ${this.formatDimensions(mostEfficient.dimensions)} ` +
      `at $${(mostEfficient.value * 1000).toFixed(4)} per 1K tokens, ` +
      `${efficiencyPercentage.toFixed(1)}% more efficient than the least efficient option.`
    );

    // Performance insights (if available)
    if (rankings.byPerformance.length > 0) {
      const bestPerformance = rankings.byPerformance[0];
      insights.push(
        `Best performance is ${this.formatDimensions(bestPerformance.dimensions)} ` +
        `with combined latency/error score of ${bestPerformance.value.toFixed(2)}.`
      );
    }

    // Value for money insight
    const bestValue = this.findBestValue(comparisons);
    if (bestValue) {
      insights.push(
        `Best value for money is ${this.formatDimensions(bestValue.dimensions)} ` +
        `offering a good balance of cost and performance.`
      );
    }

    // Cost distribution insight
    const totalCost = comparisons.reduce((sum, c) => sum + c.cost, 0);
    const top3Cost = rankings.byCost.slice(0, 3).reduce((sum, r) => sum + r.value, 0);
    const concentration = (top3Cost / totalCost) * 100;
    
    insights.push(
      `Top 3 most expensive options account for ${concentration.toFixed(1)}% of total costs.`
    );

    return insights;
  }

  /**
   * Find best value option (balance of cost and performance)
   */
  private findBestValue(comparisons: CostComparisonItem[]): CostComparisonItem | null {
    const withPerformance = comparisons.filter(c => c.averageLatency !== undefined && c.errorRate !== undefined);
    
    if (withPerformance.length === 0) return null;

    // Calculate a value score (lower cost and better performance = higher value)
    const scored = withPerformance.map(c => {
      const costScore = c.cost / Math.max(...comparisons.map(comp => comp.cost));
      const performanceScore = ((c.averageLatency || 0) * 0.7 + (c.errorRate || 0) * 30) / 
        Math.max(...withPerformance.map(comp => (comp.averageLatency || 0) * 0.7 + (comp.errorRate || 0) * 30));
      const valueScore = 1 - (costScore * 0.6 + performanceScore * 0.4);
      return { item: c, valueScore };
    });

    scored.sort((a, b) => b.valueScore - a.valueScore);
    return scored[0].item;
  }

  /**
   * Format dimensions for display
   */
  private formatDimensions(dimensions: Record<string, string | number>): string {
    const entries = Object.entries(dimensions);
    if (entries.length === 0) return 'unknown';
    
    return entries.map(([key, value]) => `${value}`).join(' / ');
  }

  /**
   * Generate mock latency (for demo purposes)
   */
  private generateMockLatency(): number {
    return 500 + Math.random() * 2000; // 500-2500ms
  }

  /**
   * Generate mock error rate (for demo purposes)
   */
  private generateMockErrorRate(): number {
    return Math.random() * 0.05; // 0-5%
  }

  /**
   * Handle errors
   */
  private handleError(error: any, queryId: string): Error {
    console.error(`Cost comparison error (${queryId}):`, error);
    return new Error(`Cost comparison failed: ${error.message}`);
  }

  /**
   * Generate query ID
   */
  private generateQueryId(request: CostComparisonRequest): string {
    const hash = this.hashRequest(request);
    return `cost_comparison_${Date.now()}_${hash.substring(0, 8)}`;
  }

  /**
   * Hash request for identification
   */
  private hashRequest(request: CostComparisonRequest): string {
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