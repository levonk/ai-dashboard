/**
 * Analytics Query API Types
 * 
 * Defines the request/response schemas for analytics queries
 */

export interface AnalyticsQueryRequest {
  // Time range
  startTime: string; // ISO 8601 timestamp
  endTime: string;   // ISO 8601 timestamp
  
  // Dimensions for filtering
  dimensions: {
    clientIds?: string[];
    aiClients?: string[];      // claude-code, codex, pi, devin, etc.
    teams?: string[];
    pipelineStages?: string[]; // pre-optimization, post-optimization, etc.
    providers?: string[];      // anthropic, openai, google, etc.
    models?: string[];         // gpt-4, claude-3.5-opus, etc.
    inputTypes?: string[];     // text, image, audio, etc.
  };
  
  // Metrics to aggregate
  metrics: QueryMetric[];
  
  // Grouping dimensions
  groupBy?: QueryDimension[];
  
  // Aggregation functions
  aggregations: AggregationFunction[];
  
  // Filtering conditions
  filters?: QueryFilter[];
  
  // Sorting
  sortBy?: SortField;
  sortOrder?: 'asc' | 'desc';
  
  // Pagination
  limit?: number;
  offset?: number;
}

export type QueryMetric = 
  | 'requestCount'
  | 'tokenCount' 
  | 'cost'
  | 'latency'
  | 'errorRate'
  | 'throughput'
  | 'cacheHitRate';

export type QueryDimension =
  | 'clientId'
  | 'aiClient'
  | 'team'
  | 'pipelineStage'
  | 'provider'
  | 'model'
  | 'inputType'
  | 'time';

export type AggregationFunction =
  | 'sum'
  | 'count'
  | 'avg'
  | 'min'
  | 'max'
  | 'p50'
  | 'p90'
  | 'p95'
  | 'p99';

export interface QueryFilter {
  field: string;
  operator: FilterOperator;
  value: string | number | boolean | string[] | number[];
}

export type FilterOperator =
  | 'eq'        // equals
  | 'ne'        // not equals
  | 'gt'        // greater than
  | 'gte'       // greater than or equal
  | 'lt'        // less than
  | 'lte'       // less than or equal
  | 'in'        // in array
  | 'notIn'     // not in array
  | 'contains'  // string contains
  | 'startsWith'
  | 'endsWith';

export interface SortField {
  field: string;
  direction: 'asc' | 'desc';
}

export interface AnalyticsQueryResponse {
  // Query metadata
  queryId: string;
  executionTimeMs: number;
  cached: boolean;
  
  // Results
  data: QueryResultRow[];
  
  // Pagination info
  pagination: {
    total: number;
    limit: number;
    offset: number;
    hasMore: boolean;
  };
  
  // Metadata about results
  metadata: {
    timeRange: {
      start: string;
      end: string;
    };
    dimensions: string[];
    metrics: string[];
  };
}

export interface QueryResultRow {
  // Dimension values (if grouped)
  dimensions?: Record<string, string | number>;
  
  // Metric values
  metrics: Record<string, number>;
  
  // Additional metadata
  timestamp?: string;
}

export interface QueryValidationError {
  field: string;
  message: string;
  code: string;
}

export interface QueryError {
  code: string;
  message: string;
  details?: any;
  timestamp: string;
}

export interface QueryCacheEntry {
  queryId: string;
  request: AnalyticsQueryRequest;
  response: AnalyticsQueryResponse;
  createdAt: string;
  expiresAt: string;
  hitCount: number;
}

export interface QueryPerformanceMetrics {
  queryId: string;
  executionTimeMs: number;
  cacheHit: boolean;
  resultCount: number;
  timestamp: string;
}
