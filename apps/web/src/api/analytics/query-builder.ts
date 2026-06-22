/**
 * Query Builder for Multi-Dimensional Analytics Queries
 * 
 * Provides a fluent interface for building complex analytics queries
 * with proper validation and optimization
 */

import {
  AnalyticsQueryRequest,
  QueryDimension,
  QueryMetric,
  AggregationFunction,
  QueryFilter,
  SortField,
  FilterOperator
} from './types';

export class AnalyticsQueryBuilder {
  private query: Partial<AnalyticsQueryRequest> = {};

  constructor() {
    this.query = {
      dimensions: {},
      metrics: [],
      aggregations: [],
      filters: [],
      limit: 100,
      offset: 0
    };
  }

  /**
   * Set time range for the query
   */
  setTimeRange(startTime: string, endTime: string): this {
    this.query.startTime = startTime;
    this.query.endTime = endTime;
    return this;
  }

  /**
   * Filter by specific client IDs
   */
  filterByClientIds(clientIds: string[]): this {
    this.query.dimensions!.clientIds = clientIds;
    return this;
  }

  /**
   * Filter by AI clients (claude-code, codex, etc.)
   */
  filterByAiClients(aiClients: string[]): this {
    this.query.dimensions!.aiClients = aiClients;
    return this;
  }

  /**
   * Filter by teams
   */
  filterByTeams(teams: string[]): this {
    this.query.dimensions!.teams = teams;
    return this;
  }

  /**
   * Filter by pipeline stages
   */
  filterByPipelineStages(stages: string[]): this {
    this.query.dimensions!.pipelineStages = stages;
    return this;
  }

  /**
   * Filter by providers (anthropic, openai, etc.)
   */
  filterByProviders(providers: string[]): this {
    this.query.dimensions!.providers = providers;
    return this;
  }

  /**
   * Filter by models
   */
  filterByModels(models: string[]): this {
    this.query.dimensions!.models = models;
    return this;
  }

  /**
   * Filter by input types
   */
  filterByInputTypes(inputTypes: string[]): this {
    this.query.dimensions!.inputTypes = inputTypes;
    return this;
  }

  /**
   * Add metrics to query
   */
  addMetrics(metrics: QueryMetric[]): this {
    this.query.metrics = [...(this.query.metrics || []), ...metrics];
    return this;
  }

  /**
   * Set grouping dimensions
   */
  groupBy(dimensions: QueryDimension[]): this {
    this.query.groupBy = dimensions;
    return this;
  }

  /**
   * Add aggregation functions
   */
  addAggregations(aggregations: AggregationFunction[]): this {
    this.query.aggregations = [...(this.query.aggregations || []), ...aggregations];
    return this;
  }

  /**
   * Add custom filter
   */
  addFilter(field: string, operator: FilterOperator, value: any): this {
    this.query.filters = this.query.filters || [];
    this.query.filters.push({ field, operator, value });
    return this;
  }

  /**
   * Set sorting
   */
  sortBy(field: string, order: 'asc' | 'desc' = 'asc'): this {
    this.query.sortBy = { field, direction: order };
    this.query.sortOrder = order;
    return this;
  }

  /**
   * Set pagination limit
   */
  setLimit(limit: number): this {
    this.query.limit = limit;
    return this;
  }

  /**
   * Set pagination offset
   */
  setOffset(offset: number): this {
    this.query.offset = offset;
    return this;
  }

  /**
   * Build and validate the query
   */
  build(): AnalyticsQueryRequest {
    this.validate();
    return this.query as AnalyticsQueryRequest;
  }

  /**
   * Validate the query before building
   */
  private validate(): void {
    if (!this.query.startTime || !this.query.endTime) {
      throw new Error('Time range (startTime and endTime) is required');
    }

    if (!this.query.metrics || this.query.metrics.length === 0) {
      throw new Error('At least one metric is required');
    }

    if (!this.query.aggregations || this.query.aggregations.length === 0) {
      throw new Error('At least one aggregation function is required');
    }

    // Validate time range
    const start = new Date(this.query.startTime);
    const end = new Date(this.query.endTime);
    if (start >= end) {
      throw new Error('startTime must be before endTime');
    }

    // Validate limit
    if (this.query.limit && this.query.limit > 10000) {
      throw new Error('Limit cannot exceed 10000');
    }

    // Validate offset
    if (this.query.offset && this.query.offset < 0) {
      throw new Error('Offset cannot be negative');
    }
  }

  /**
   * Create a builder from an existing request
   */
  static fromRequest(request: AnalyticsQueryRequest): AnalyticsQueryBuilder {
    const builder = new AnalyticsQueryBuilder();
    builder.query = { ...request };
    return builder;
  }
}

/**
 * Helper function to create common query patterns
 */
export class QueryPatterns {
  /**
   * Create a basic usage query
   */
  static basicUsage(startTime: string, endTime: string): AnalyticsQueryBuilder {
    return new AnalyticsQueryBuilder()
      .setTimeRange(startTime, endTime)
      .addMetrics(['requestCount', 'tokenCount', 'cost'])
      .addAggregations(['sum', 'count'])
      .groupBy(['time']);
  }

  /**
   * Create a cost breakdown query
   */
  static costBreakdown(startTime: string, endTime: string): AnalyticsQueryBuilder {
    return new AnalyticsQueryBuilder()
      .setTimeRange(startTime, endTime)
      .addMetrics(['cost', 'tokenCount'])
      .addAggregations(['sum', 'avg'])
      .groupBy(['provider', 'model'])
      .sortBy('cost', 'desc');
  }

  /**
   * Create a performance analysis query
   */
  static performanceAnalysis(startTime: string, endTime: string): AnalyticsQueryBuilder {
    return new AnalyticsQueryBuilder()
      .setTimeRange(startTime, endTime)
      .addMetrics(['latency', 'errorRate', 'throughput'])
      .addAggregations(['avg', 'p95', 'p99'])
      .groupBy(['aiClient', 'pipelineStage']);
  }

  /**
   * Create a trend analysis query
   */
  static trendAnalysis(startTime: string, endTime: string): AnalyticsQueryBuilder {
    return new AnalyticsQueryBuilder()
      .setTimeRange(startTime, endTime)
      .addMetrics(['requestCount', 'tokenCount', 'cost'])
      .addAggregations(['sum'])
      .groupBy(['time'])
      .sortBy('time', 'asc');
  }
}
