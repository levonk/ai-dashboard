/**
 * Cost Optimization Recommendations
 * 
 * Analyzes cost data and provides optimization recommendations
 */

import {
  CostOptimizationRequest,
  CostOptimizationResponse,
  CostOptimizationRecommendation,
  CostBreakdownItem
} from './types';
import { CostBreakdownAnalyzer } from './breakdown';

export class CostOptimizer {
  private breakdownAnalyzer: CostBreakdownAnalyzer;

  constructor() {
    this.breakdownAnalyzer = new CostBreakdownAnalyzer();
  }

  /**
   * Generate cost optimization recommendations
   */
  async generateRecommendations(request: CostOptimizationRequest): Promise<CostOptimizationResponse> {
    const startTime = Date.now();
    const queryId = this.generateQueryId(request);

    try {
      // Get current cost baseline
      const baseline = await this.getBaseline(request);
      
      // Generate recommendations
      const recommendations = await this.generateRecommendationsInternal(request, baseline);
      
      // Calculate potential savings
      const potentialSavings = this.calculatePotentialSavings(baseline, recommendations);
      
      // Prioritize recommendations
      const priority = this.prioritizeRecommendations(recommendations);

      return {
        queryId,
        executionTimeMs: Date.now() - startTime,
        baseline,
        recommendations,
        potentialSavings,
        priority
      };
    } catch (error) {
      throw this.handleError(error, queryId);
    }
  }

  /**
   * Get current cost baseline
   */
  private async getBaseline(request: CostOptimizationRequest): Promise<{
    totalCost: number;
    breakdown: CostBreakdownItem[];
  }> {
    // Use the breakdown analyzer to get current costs
    const breakdownRequest = {
      startTime: request.startTime,
      endTime: request.endTime,
      dimensions: request.scope.dimensions || ['provider', 'model'],
      filters: request.scope.filters
    };

    const breakdownResponse = await this.breakdownAnalyzer.analyzeBreakdown(breakdownRequest);

    return {
      totalCost: breakdownResponse.summary.totalCost,
      breakdown: breakdownResponse.breakdown
    };
  }

  /**
   * Generate optimization recommendations
   */
  private async generateRecommendationsInternal(
    request: CostOptimizationRequest,
    baseline: { totalCost: number; breakdown: CostBreakdownItem[] }
  ): Promise<CostOptimizationRecommendation[]> {
    const recommendations: CostOptimizationRecommendation[] = [];

    // Model switch recommendations
    if (request.options.includeModelRecommendations !== false) {
      const modelRecs = this.generateModelSwitchRecommendations(baseline);
      recommendations.push(...modelRecs);
    }

    // Provider switch recommendations
    if (request.options.includeProviderRecommendations !== false) {
      const providerRecs = this.generateProviderSwitchRecommendations(baseline);
      recommendations.push(...providerRecs);
    }

    // Usage optimization recommendations
    if (request.options.includeUsageOptimization !== false) {
      const usageRecs = this.generateUsageOptimizationRecommendations(baseline);
      recommendations.push(...usageRecs);
    }

    // Caching recommendations
    const cachingRecs = this.generateCachingRecommendations(baseline);
    recommendations.push(...cachingRecs);

    // Batching recommendations
    const batchingRecs = this.generateBatchingRecommendations(baseline);
    recommendations.push(...batchingRecs);

    return recommendations;
  }

  /**
   * Generate model switch recommendations
   */
  private generateModelSwitchRecommendations(baseline: { totalCost: number; breakdown: CostBreakdownItem[] }): CostOptimizationRecommendation[] {
    const recommendations: CostOptimizationRecommendation[] = [];
    
    // Model cost per token comparison (mock data)
    const modelCosts: Record<string, number> = {
      'claude-3.5-opus': 0.000278,
      'gpt-4': 0.000278,
      'gemini-pro': 0.000250,
      'claude-3-haiku': 0.000201,
      'gpt-3.5-turbo': 0.000150
    };

    baseline.breakdown.forEach(item => {
      const currentModel = item.dimensions.model as string;
      const currentCostPerToken = item.averageCostPerToken;
      
      // Find cheaper alternatives
      const alternatives = Object.entries(modelCosts)
        .filter(([model, cost]) => cost < currentCostPerToken * 0.8) // At least 20% cheaper
        .sort((a, b) => a[1] - b[1]);

      if (alternatives.length > 0) {
        const [bestModel, bestCost] = alternatives[0];
        const savingsPercentage = ((currentCostPerToken - bestCost) / currentCostPerToken) * 100;
        const estimatedSavings = item.cost * (savingsPercentage / 100);

        recommendations.push({
          id: `model_switch_${this.generateId()}`,
          type: 'model_switch',
          title: `Switch from ${currentModel} to ${bestModel}`,
          description: `Switching to ${bestModel} could reduce costs by ${savingsPercentage.toFixed(1)}% while maintaining similar functionality for many use cases.`,
          estimatedSavings,
          savingsPercentage,
          implementationEffort: this.assessImplementationEffort('model_switch'),
          details: {
            current: {
              model: currentModel,
              costPerToken: currentCostPerToken,
              monthlyCost: item.cost
            },
            recommended: {
              model: bestModel,
              costPerToken: bestCost,
              projectedMonthlyCost: item.cost - estimatedSavings
            },
            reasoning: `${bestModel} offers ${savingsPercentage.toFixed(1)}% cost reduction per token with comparable performance for most tasks.`
          },
          priority: this.assessPriority(estimatedSavings, baseline.totalCost)
        });
      }
    });

    return recommendations;
  }

  /**
   * Generate provider switch recommendations
   */
  private generateProviderSwitchRecommendations(baseline: { totalCost: number; breakdown: CostBreakdownItem[] }): CostOptimizationRecommendation[] {
    const recommendations: CostOptimizationRecommendation[] = [];
    
    // Provider cost comparison (mock data)
    const providerCosts: Record<string, number> = {
      'anthropic': 1.0,
      'openai': 1.0,
      'google': 0.85,
      'aws': 0.90,
      'azure': 0.95
    };

    // Group by provider
    const providerGroups = new Map<string, CostBreakdownItem[]>();
    baseline.breakdown.forEach(item => {
      const provider = item.dimensions.provider as string;
      if (!providerGroups.has(provider)) {
        providerGroups.set(provider, []);
      }
      providerGroups.get(provider)!.push(item);
    });

    providerGroups.forEach((items, currentProvider) => {
      const currentCost = items.reduce((sum, item) => sum + item.cost, 0);
      const currentMultiplier = providerCosts[currentProvider] || 1.0;
      
      // Find cheaper alternatives
      const alternatives = Object.entries(providerCosts)
        .filter(([provider, multiplier]) => multiplier < currentMultiplier * 0.9)
        .sort((a, b) => a[1] - b[1]);

      if (alternatives.length > 0) {
        const [bestProvider, bestMultiplier] = alternatives[0];
        const savingsPercentage = ((currentMultiplier - bestMultiplier) / currentMultiplier) * 100;
        const estimatedSavings = currentCost * (savingsPercentage / 100);

        recommendations.push({
          id: `provider_switch_${this.generateId()}`,
          type: 'provider_switch',
          title: `Switch from ${currentProvider} to ${bestProvider}`,
          description: `Switching to ${bestProvider} could reduce costs by ${savingsPercentage.toFixed(1)}% for equivalent models.`,
          estimatedSavings,
          savingsPercentage,
          implementationEffort: 'high', // Provider switches are complex
          details: {
            current: {
              provider: currentProvider,
              monthlyCost: currentCost
            },
            recommended: {
              provider: bestProvider,
              projectedMonthlyCost: currentCost - estimatedSavings
            },
            reasoning: `${bestProvider} offers ${savingsPercentage.toFixed(1)}% cost reduction with comparable model availability and performance.`
          },
          priority: this.assessPriority(estimatedSavings, baseline.totalCost)
        });
      }
    });

    return recommendations;
  }

  /**
   * Generate usage optimization recommendations
   */
  private generateUsageOptimizationRecommendations(baseline: { totalCost: number; breakdown: CostBreakdownItem[] }): CostOptimizationRecommendation[] {
    const recommendations: CostOptimizationRecommendation[] = [];

    // Analyze usage patterns
    baseline.breakdown.forEach(item => {
      const requestCount = item.requestCount;
      const averageCostPerRequest = item.averageCostPerRequest;
      
      // High request count with high cost per request
      if (requestCount > 10000 && averageCostPerRequest > 0.15) {
        const estimatedSavings = item.cost * 0.15; // Assume 15% savings through optimization
        
        recommendations.push({
          id: `usage_optimization_${this.generateId()}`,
          type: 'usage_optimization',
          title: 'Optimize high-volume request patterns',
          description: `High request volume (${requestCount.toLocaleString()} requests) with elevated cost per request ($${averageCostPerRequest.toFixed(3)}) indicates optimization opportunities.`,
          estimatedSavings,
          savingsPercentage: 15,
          implementationEffort: 'medium',
          details: {
            current: {
              requestCount,
              averageCostPerRequest,
              monthlyCost: item.cost
            },
            recommended: {
              targetCostPerRequest: averageCostPerRequest * 0.85,
              projectedMonthlyCost: item.cost - estimatedSavings
            },
            reasoning: 'Implement request batching, prompt optimization, and result caching to reduce per-request costs.'
          },
          priority: this.assessPriority(estimatedSavings, baseline.totalCost)
        });
      }
    });

    return recommendations;
  }

  /**
   * Generate caching recommendations
   */
  private generateCachingRecommendations(baseline: { totalCost: number; breakdown: CostBreakdownItem[] }): CostOptimizationRecommendation[] {
    const recommendations: CostOptimizationRecommendation[] = [];

    // Analyze potential for caching
    const totalRequests = baseline.breakdown.reduce((sum, item) => sum + item.requestCount, 0);
    const estimatedCacheHitRate = 0.25; // Assume 25% cache hit rate
    const estimatedSavings = baseline.totalCost * estimatedCacheHitRate * 0.8; // 80% cost reduction for cached requests

    if (totalRequests > 5000) {
      recommendations.push({
        id: `caching_${this.generateId()}`,
        type: 'caching',
        title: 'Implement response caching',
        description: `With ${totalRequests.toLocaleString()} total requests, implementing intelligent caching could significantly reduce costs.`,
        estimatedSavings,
        savingsPercentage: estimatedCacheHitRate * 80,
        implementationEffort: 'medium',
        details: {
          current: {
            totalRequests,
            cacheHitRate: 0,
            monthlyCost: baseline.totalCost
          },
          recommended: {
            targetCacheHitRate: estimatedCacheHitRate,
            projectedMonthlyCost: baseline.totalCost - estimatedSavings
          },
          reasoning: `Estimated ${estimatedCacheHitRate * 100}% cache hit rate could reduce API calls by ${(totalRequests * estimatedCacheHitRate).toLocaleString()} requests per month.`
        },
        priority: this.assessPriority(estimatedSavings, baseline.totalCost)
      });
    }

    return recommendations;
  }

  /**
   * Generate batching recommendations
   */
  private generateBatchingRecommendations(baseline: { totalCost: number; breakdown: CostBreakdownItem[] }): CostOptimizationRecommendation[] {
    const recommendations: CostOptimizationRecommendation[] = [];

    // Analyze potential for batching
    const totalRequests = baseline.breakdown.reduce((sum, item) => sum + item.requestCount, 0);
    const estimatedBatchingEfficiency = 0.20; // Assume 20% efficiency gain
    const estimatedSavings = baseline.totalCost * estimatedBatchingEfficiency;

    if (totalRequests > 10000) {
      recommendations.push({
        id: `batching_${this.generateId()}`,
        type: 'batching',
        title: 'Implement request batching',
        description: `High request volume (${totalRequests.toLocaleString()}) suggests opportunities for batching similar requests.`,
        estimatedSavings,
        savingsPercentage: estimatedBatchingEfficiency * 100,
        implementationEffort: 'medium',
        details: {
          current: {
            totalRequests,
            batchingEfficiency: 0,
            monthlyCost: baseline.totalCost
          },
          recommended: {
            targetBatchingEfficiency: estimatedBatchingEfficiency,
            projectedMonthlyCost: baseline.totalCost - estimatedSavings
          },
            reasoning: `Batching similar requests could reduce API overhead and improve throughput by ${estimatedBatchingEfficiency * 100}%.`
          },
          priority: this.assessPriority(estimatedSavings, baseline.totalCost)
        });
    }

    return recommendations;
  }

  /**
   * Calculate potential savings
   */
  private calculatePotentialSavings(
    baseline: { totalCost: number; breakdown: CostBreakdownItem[] },
    recommendations: CostOptimizationRecommendation[]
  ): {
    totalSavings: number;
    savingsPercentage: number;
    byCategory: Record<string, number>;
  } {
    const totalSavings = recommendations.reduce((sum, rec) => sum + rec.estimatedSavings, 0);
    const savingsPercentage = baseline.totalCost > 0 ? (totalSavings / baseline.totalCost) * 100 : 0;
    
    const byCategory: Record<string, number> = {};
    recommendations.forEach(rec => {
      if (!byCategory[rec.type]) {
        byCategory[rec.type] = 0;
      }
      byCategory[rec.type] += rec.estimatedSavings;
    });

    return {
      totalSavings,
      savingsPercentage,
      byCategory
    };
  }

  /**
   * Prioritize recommendations
   */
  private prioritizeRecommendations(recommendations: CostOptimizationRecommendation[]): CostOptimizationRecommendation[] {
    return recommendations.sort((a, b) => {
      // Sort by priority (high > medium > low), then by savings
      const priorityOrder = { high: 0, medium: 1, low: 2 };
      const priorityDiff = priorityOrder[a.priority] - priorityOrder[b.priority];
      
      if (priorityDiff !== 0) return priorityDiff;
      
      // If same priority, sort by savings (higher first)
      return b.estimatedSavings - a.estimatedSavings;
    });
  }

  /**
   * Assess implementation effort
   */
  private assessImplementationEffort(type: string): 'low' | 'medium' | 'high' {
    const efforts: Record<string, 'low' | 'medium' | 'high'> = {
      'model_switch': 'medium',
      'provider_switch': 'high',
      'usage_optimization': 'medium',
      'caching': 'medium',
      'batching': 'medium'
    };
    
    return efforts[type] || 'medium';
  }

  /**
   * Assess priority based on savings
   */
  private assessPriority(savings: number, totalCost: number): 'high' | 'medium' | 'low' {
    const savingsPercentage = totalCost > 0 ? (savings / totalCost) * 100 : 0;
    
    if (savingsPercentage > 10) return 'high';
    if (savingsPercentage > 5) return 'medium';
    return 'low';
  }

  /**
   * Generate unique ID
   */
  private generateId(): string {
    return Math.random().toString(36).substring(2, 9);
  }

  /**
   * Handle errors
   */
  private handleError(error: any, queryId: string): Error {
    console.error(`Cost optimization error (${queryId}):`, error);
    return new Error(`Cost optimization failed: ${error.message}`);
  }

  /**
   * Generate query ID
   */
  private generateQueryId(request: CostOptimizationRequest): string {
    const hash = this.hashRequest(request);
    return `cost_optimization_${Date.now()}_${hash.substring(0, 8)}`;
  }

  /**
   * Hash request for identification
   */
  private hashRequest(request: CostOptimizationRequest): string {
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