/**
 * Cost Breakdown Analysis
 * 
 * Provides detailed cost breakdowns across multiple dimensions
 */

import {
  CostBreakdownRequest,
  CostBreakdownResponse,
  CostBreakdownItem,
  CostDimension,
  CostFilter
} from './types';

export class CostBreakdownAnalyzer {
  /**
   * Analyze cost breakdown by specified dimensions
   */
  async analyzeBreakdown(request: CostBreakdownRequest): Promise<CostBreakdownResponse> {
    const startTime = Date.now();
    const queryId = this.generateQueryId(request);

    try {
      // Execute breakdown analysis
      const breakdown = await this.executeBreakdown(request);
      
      // Calculate summary statistics
      const summary = this.calculateSummary(breakdown);

      return {
        queryId,
        executionTimeMs: Date.now() - startTime,
        breakdown,
        summary,
        timeRange: {
          start: request.startTime,
          end: request.endTime
        }
      };
    } catch (error) {
      throw this.handleError(error, queryId);
    }
  }

  /**
   * Execute cost breakdown analysis
   */
  private async executeBreakdown(request: CostBreakdownRequest): Promise<CostBreakdownItem[]> {
    // TODO: Integrate with analytics-rs package for actual data
    // For now, return mock data
    
    const mockBreakdown: CostBreakdownItem[] = [
      {
        dimensions: {
          provider: 'anthropic',
          model: 'claude-3.5-opus'
        },
        cost: 1250.50,
        costPercentage: 45.2,
        tokenCount: 4500000,
        requestCount: 12500,
        averageCostPerToken: 0.000278,
        averageCostPerRequest: 0.10
      },
      {
        dimensions: {
          provider: 'openai',
          model: 'gpt-4'
        },
        cost: 890.25,
        costPercentage: 32.1,
        tokenCount: 3200000,
        requestCount: 8900,
        averageCostPerToken: 0.000278,
        averageCostPerRequest: 0.10
      },
      {
        dimensions: {
          provider: 'google',
          model: 'gemini-pro'
        },
        cost: 450.75,
        costPercentage: 16.3,
        tokenCount: 1800000,
        requestCount: 5600,
        averageCostPerToken: 0.000250,
        averageCostPerRequest: 0.08
      },
      {
        dimensions: {
          provider: 'anthropic',
          model: 'claude-3-haiku'
        },
        cost: 180.50,
        costPercentage: 6.4,
        tokenCount: 900000,
        requestCount: 4500,
        averageCostPerToken: 0.000201,
        averageCostPerRequest: 0.04
      }
    ];

    // Apply filters if specified
    let filteredBreakdown = mockBreakdown;
    if (request.filters && request.filters.length > 0) {
      filteredBreakdown = this.applyFilters(mockBreakdown, request.filters);
    }

    // Group by specified dimensions
    if (request.dimensions && request.dimensions.length > 0) {
      return this.groupByDimensions(filteredBreakdown, request.dimensions);
    }

    return filteredBreakdown;
  }

  /**
   * Calculate summary statistics
   */
  private calculateSummary(breakdown: CostBreakdownItem[]): {
    totalCost: number;
    totalTokens: number;
    totalRequests: number;
    averageCostPerRequest: number;
    averageCostPerToken: number;
  } {
    const totalCost = breakdown.reduce((sum, item) => sum + item.cost, 0);
    const totalTokens = breakdown.reduce((sum, item) => sum + item.tokenCount, 0);
    const totalRequests = breakdown.reduce((sum, item) => sum + item.requestCount, 0);

    return {
      totalCost,
      totalTokens,
      totalRequests,
      averageCostPerRequest: totalRequests > 0 ? totalCost / totalRequests : 0,
      averageCostPerToken: totalTokens > 0 ? totalCost / totalTokens : 0
    };
  }

  /**
   * Apply filters to breakdown results
   */
  private applyFilters(breakdown: CostBreakdownItem[], filters: CostFilter[]): CostBreakdownItem[] {
    return breakdown.filter(item => {
      return filters.every(filter => {
        const value = this.getNestedValue(item.dimensions, filter.field);
        return this.applyFilterOperator(value, filter.operator, filter.value);
      });
    });
  }

  /**
   * Group breakdown by dimensions
   */
  private groupByDimensions(breakdown: CostBreakdownItem[], dimensions: CostDimension[]): CostBreakdownItem[] {
    // Group by the specified dimensions
    const groups = new Map<string, CostBreakdownItem>();

    breakdown.forEach(item => {
      const key = dimensions.map(dim => item.dimensions[dim]).join('|');
      
      if (!groups.has(key)) {
        groups.set(key, {
          dimensions: {},
          cost: 0,
          costPercentage: 0,
          tokenCount: 0,
          requestCount: 0,
          averageCostPerToken: 0,
          averageCostPerRequest: 0
        });
      }

      const group = groups.get(key)!;
      
      // Merge dimensions
      dimensions.forEach(dim => {
        group.dimensions[dim] = item.dimensions[dim];
      });

      // Aggregate metrics
      group.cost += item.cost;
      group.tokenCount += item.tokenCount;
      group.requestCount += item.requestCount;
    });

    // Calculate derived metrics and percentages
    const totalCost = Array.from(groups.values()).reduce((sum, item) => sum + item.cost, 0);
    
    Array.from(groups.values()).forEach(item => {
      item.costPercentage = totalCost > 0 ? (item.cost / totalCost) * 100 : 0;
      item.averageCostPerToken = item.tokenCount > 0 ? item.cost / item.tokenCount : 0;
      item.averageCostPerRequest = item.requestCount > 0 ? item.cost / item.requestCount : 0;
    });

    return Array.from(groups.values());
  }

  /**
   * Get nested value from object
   */
  private getNestedValue(obj: any, path: string): any {
    return path.split('.').reduce((current, key) => current?.[key], obj);
  }

  /**
   * Apply filter operator
   */
  private applyFilterOperator(value: any, operator: string, filterValue: any): boolean {
    switch (operator) {
      case 'eq':
        return value === filterValue;
      case 'ne':
        return value !== filterValue;
      case 'gt':
        return value > filterValue;
      case 'gte':
        return value >= filterValue;
      case 'lt':
        return value < filterValue;
      case 'lte':
        return value <= filterValue;
      case 'in':
        return Array.isArray(filterValue) && filterValue.includes(value);
      case 'notIn':
        return Array.isArray(filterValue) && !filterValue.includes(value);
      case 'contains':
        return typeof value === 'string' && value.includes(filterValue);
      case 'startsWith':
        return typeof value === 'string' && value.startsWith(filterValue);
      case 'endsWith':
        return typeof value === 'string' && value.endsWith(filterValue);
      default:
        return true;
    }
  }

  /**
   * Handle errors
   */
  private handleError(error: any, queryId: string): Error {
    console.error(`Cost breakdown analysis error (${queryId}):`, error);
    return new Error(`Cost breakdown analysis failed: ${error.message}`);
  }

  /**
   * Generate query ID
   */
  private generateQueryId(request: CostBreakdownRequest): string {
    const hash = this.hashRequest(request);
    return `cost_breakdown_${Date.now()}_${hash.substring(0, 8)}`;
  }

  /**
   * Hash request for identification
   */
  private hashRequest(request: CostBreakdownRequest): string {
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