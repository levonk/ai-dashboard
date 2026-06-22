/**
 * Cost Analysis API Types
 * 
 * Defines the request/response schemas for cost analysis features
 */

export interface CostBreakdownRequest {
  // Time range
  startTime: string; // ISO 8601 timestamp
  endTime: string;   // ISO 8601 timestamp
  
  // Breakdown dimensions
  dimensions: CostDimension[];
  
  // Filtering
  filters?: CostFilter[];
  
  // Granularity
  granularity?: 'hour' | 'day' | 'week' | 'month';
}

export type CostDimension =
  | 'client'
  | 'aiClient'
  | 'team'
  | 'provider'
  | 'model'
  | 'inputType'
  | 'pipelineStage';

export interface CostFilter {
  field: string;
  operator: FilterOperator;
  value: string | number | boolean | string[] | number[];
}

export type FilterOperator =
  | 'eq'
  | 'ne'
  | 'gt'
  | 'gte'
  | 'lt'
  | 'lte'
  | 'in'
  | 'notIn'
  | 'contains'
  | 'startsWith'
  | 'endsWith';

export interface CostBreakdownResponse {
  // Query metadata
  queryId: string;
  executionTimeMs: number;
  
  // Cost breakdown results
  breakdown: CostBreakdownItem[];
  
  // Summary statistics
  summary: {
    totalCost: number;
    totalTokens: number;
    totalRequests: number;
    averageCostPerRequest: number;
    averageCostPerToken: number;
  };
  
  // Time range
  timeRange: {
    start: string;
    end: string;
  };
}

export interface CostBreakdownItem {
  // Dimension values
  dimensions: Record<string, string | number>;
  
  // Cost metrics
  cost: number;
  costPercentage: number;
  
  // Usage metrics
  tokenCount: number;
  requestCount: number;
  
  // Derived metrics
  averageCostPerToken: number;
  averageCostPerRequest: number;
}

export interface CostTrendRequest {
  // Time range
  startTime: string;
  endTime: string;
  
  // Trend dimensions
  dimensions?: CostDimension[];
  
  // Filtering
  filters?: CostFilter[];
  
  // Granularity
  granularity: 'hour' | 'day' | 'week' | 'month';
  
  // Trend analysis options
  includeForecast?: boolean;
  forecastPeriod?: number; // number of periods to forecast
}

export interface CostTrendResponse {
  // Query metadata
  queryId: string;
  executionTimeMs: number;
  
  // Trend data
  trends: CostTrendData[];
  
  // Trend analysis
  analysis: {
    trend: 'increasing' | 'decreasing' | 'stable';
    growthRate: number;
    seasonality?: string;
    anomalies: CostAnomaly[];
  };
  
  // Forecast (if requested)
  forecast?: CostForecast;
}

export interface CostTrendData {
  timestamp: string;
  cost: number;
  tokenCount: number;
  requestCount: number;
  
  // Dimension values (if grouped)
  dimensions?: Record<string, string | number>;
}

export interface CostAnomaly {
  timestamp: string;
  expectedCost: number;
  actualCost: number;
  deviation: number;
  severity: 'low' | 'medium' | 'high';
  possibleCauses?: string[];
}

export interface CostForecast {
  forecastData: CostTrendData[];
  confidence: number;
  methodology: string;
  assumptions: string[];
}

export interface CostForecastRequest {
  // Historical data range
  historicalStartTime: string;
  historicalEndTime: string;
  
  // Forecast period
  forecastPeriod: number; // number of periods
  granularity: 'hour' | 'day' | 'week' | 'month';
  
  // Forecast dimensions
  dimensions?: CostDimension[];
  
  // Filtering
  filters?: CostFilter[];
  
  // Forecast method
  method?: 'linear' | 'exponential' | 'moving_average' | 'arima';
}

export interface CostForecastResponse {
  // Query metadata
  queryId: string;
  executionTimeMs: number;
  
  // Forecast results
  forecast: CostForecast;
  
  // Historical data used
  historicalData: CostTrendData[];
  
  // Model performance
  modelMetrics?: {
    accuracy: number;
    meanAbsoluteError: number;
    meanSquaredError: number;
  };
}

export interface CostOptimizationRequest {
  // Time range for analysis
  startTime: string;
  endTime: string;
  
  // Optimization scope
  scope: {
    dimensions?: CostDimension[];
    filters?: CostFilter[];
  };
  
  // Optimization options
  options: {
    includeModelRecommendations?: boolean;
    includeProviderRecommendations?: boolean;
    includeUsageOptimization?: boolean;
    targetSavingsPercentage?: number;
  };
}

export interface CostOptimizationResponse {
  // Query metadata
  queryId: string;
  executionTimeMs: number;
  
  // Current cost baseline
  baseline: {
    totalCost: number;
    breakdown: CostBreakdownItem[];
  };
  
  // Optimization recommendations
  recommendations: CostOptimizationRecommendation[];
  
  // Potential savings
  potentialSavings: {
    totalSavings: number;
    savingsPercentage: number;
    byCategory: Record<string, number>;
  };
  
  // Implementation priority
  priority: CostOptimizationRecommendation[];
}

export interface CostOptimizationRecommendation {
  id: string;
  type: 'model_switch' | 'provider_switch' | 'usage_optimization' | 'caching' | 'batching';
  title: string;
  description: string;
  
  // Impact
  estimatedSavings: number;
  savingsPercentage: number;
  implementationEffort: 'low' | 'medium' | 'high';
  
  // Details
  details: {
    current: any;
    recommended: any;
    reasoning: string;
  };
  
  // Priority
  priority: 'high' | 'medium' | 'low';
}

export interface CostComparisonRequest {
  // Time range
  startTime: string;
  endTime: string;
  
  // Comparison dimensions
  compareBy: CostDimension[];
  
  // Filtering
  filters?: CostFilter[];
  
  // Comparison options
  options: {
    includeEfficiencyMetrics?: boolean;
    includePerformanceMetrics?: boolean;
  };
}

export interface CostComparisonResponse {
  // Query metadata
  queryId: string;
  executionTimeMs: number;
  
  // Comparison results
  comparisons: CostComparisonItem[];
  
  // Rankings
  rankings: {
    byCost: CostRanking[];
    byEfficiency: CostRanking[];
    byPerformance: CostRanking[];
  };
  
  // Insights
  insights: string[];
}

export interface CostComparisonItem {
  // Dimension values
  dimensions: Record<string, string | number>;
  
  // Cost metrics
  cost: number;
  costRank: number;
  
  // Efficiency metrics
  costPerToken: number;
  costPerRequest: number;
  efficiencyRank: number;
  
  // Performance metrics
  averageLatency?: number;
  errorRate?: number;
  performanceRank?: number;
}

export interface CostRanking {
  dimensions: Record<string, string | number>;
  value: number;
  rank: number;
  percentile: number;
}

export interface CostAlertRequest {
  // Alert configuration
  alert: {
    name: string;
    description?: string;
    
    // Alert conditions
    conditions: CostAlertCondition[];
    
    // Alert thresholds
    thresholds: {
      costThreshold?: number;
      percentageChange?: number;
      anomalySeverity?: 'low' | 'medium' | 'high';
    };
    
    // Notification settings
    notifications: {
      channels: ('email' | 'slack' | 'webhook')[];
      recipients?: string[];
      webhookUrl?: string;
    };
    
    // Schedule
    schedule?: {
      frequency: 'immediate' | 'hourly' | 'daily' | 'weekly';
      timezone?: string;
    };
  };
}

export interface CostAlertCondition {
  type: 'total_cost' | 'cost_increase' | 'anomaly' | 'budget_exceeded';
  dimension?: CostDimension;
  operator: 'gt' | 'gte' | 'lt' | 'lte' | 'eq';
  value: number;
  timeWindow?: string; // e.g., '1h', '1d', '1w'
}

export interface CostAlertResponse {
  // Alert metadata
  alertId: string;
  status: 'created' | 'updated';
  
  // Alert configuration
  alert: CostAlertRequest['alert'];
  
  // Validation
  validation: {
    valid: boolean;
    errors?: string[];
  };
}

export interface CostAlertStatus {
  // Alert metadata
  alertId: string;
  name: string;
  
  // Current status
  status: 'active' | 'triggered' | 'paused' | 'disabled';
  
  // Trigger history
  triggerHistory: CostAlertTrigger[];
  
  // Current state
  currentState: {
    lastTriggered?: string;
    triggerCount: number;
    currentValue?: number;
    threshold?: number;
  };
}

export interface CostAlertTrigger {
  triggerId: string;
  timestamp: string;
  condition: CostAlertCondition;
  value: number;
  threshold: number;
  severity: 'low' | 'medium' | 'high';
  acknowledged: boolean;
}

export interface CostCacheConfig {
  enabled: boolean;
  ttl: number; // time to live in seconds
  maxSize: number; // maximum number of entries
}

export interface CostAnalysisError {
  code: string;
  message: string;
  details?: any;
  timestamp: string;
}

export interface CostAnalysisMetrics {
  queryId: string;
  executionTimeMs: number;
  cacheHit: boolean;
  resultCount: number;
  timestamp: string;
}