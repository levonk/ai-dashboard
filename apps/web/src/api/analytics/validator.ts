/**
 * Query Validator and Sanitizer
 * 
 * Validates analytics query requests to prevent injection attacks
 * and ensure data integrity
 */

import {
  AnalyticsQueryRequest,
  QueryValidationError,
  QueryMetric,
  QueryDimension,
  AggregationFunction,
  FilterOperator
} from './types';

export class QueryValidator {
  private validMetrics: Set<QueryMetric>;
  private validDimensions: Set<QueryDimension>;
  private validAggregations: Set<AggregationFunction>;
  private validOperators: Set<FilterOperator>;

  constructor() {
    this.validMetrics = new Set([
      'requestCount',
      'tokenCount',
      'cost',
      'latency',
      'errorRate',
      'throughput',
      'cacheHitRate'
    ]);

    this.validDimensions = new Set([
      'clientId',
      'aiClient',
      'team',
      'pipelineStage',
      'provider',
      'model',
      'inputType',
      'time'
    ]);

    this.validAggregations = new Set([
      'sum',
      'count',
      'avg',
      'min',
      'max',
      'p50',
      'p90',
      'p95',
      'p99'
    ]);

    this.validOperators = new Set([
      'eq',
      'ne',
      'gt',
      'gte',
      'lt',
      'lte',
      'in',
      'notIn',
      'contains',
      'startsWith',
      'endsWith'
    ]);
  }

  /**
   * Validate a query request
   */
  validate(request: AnalyticsQueryRequest): {
    valid: boolean;
    errors: QueryValidationError[];
  } {
    const errors: QueryValidationError[] = [];

    // Validate time range
    this.validateTimeRange(request, errors);

    // Validate dimensions
    this.validateDimensions(request, errors);

    // Validate metrics
    this.validateMetrics(request, errors);

    // Validate groupBy
    this.validateGroupBy(request, errors);

    // Validate aggregations
    this.validateAggregations(request, errors);

    // Validate filters
    this.validateFilters(request, errors);

    // Validate pagination
    this.validatePagination(request, errors);

    // Sanitize request
    const sanitized = this.sanitize(request);

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * Validate time range
   */
  private validateTimeRange(request: AnalyticsQueryRequest, errors: QueryValidationError[]): void {
    if (!request.startTime) {
      errors.push({
        field: 'startTime',
        message: 'startTime is required',
        code: 'REQUIRED_FIELD'
      });
    } else if (!this.isValidISODate(request.startTime)) {
      errors.push({
        field: 'startTime',
        message: 'startTime must be a valid ISO 8601 date',
        code: 'INVALID_DATE'
      });
    }

    if (!request.endTime) {
      errors.push({
        field: 'endTime',
        message: 'endTime is required',
        code: 'REQUIRED_FIELD'
      });
    } else if (!this.isValidISODate(request.endTime)) {
      errors.push({
        field: 'endTime',
        message: 'endTime must be a valid ISO 8601 date',
        code: 'INVALID_DATE'
      });
    }

    if (request.startTime && request.endTime) {
      const start = new Date(request.startTime);
      const end = new Date(request.endTime);
      if (start >= end) {
        errors.push({
          field: 'timeRange',
          message: 'startTime must be before endTime',
          code: 'INVALID_TIME_RANGE'
        });
      }

      // Check for reasonable time range (max 1 year)
      const maxRange = 365 * 24 * 60 * 60 * 1000; // 1 year in milliseconds
      if (end.getTime() - start.getTime() > maxRange) {
        errors.push({
          field: 'timeRange',
          message: 'Time range cannot exceed 1 year',
          code: 'TIME_RANGE_TOO_LARGE'
        });
      }
    }
  }

  /**
   * Validate dimensions
   */
  private validateDimensions(request: AnalyticsQueryRequest, errors: QueryValidationError[]): void {
    if (!request.dimensions) {
      return;
    }

    const { clientIds, aiClients, teams, pipelineStages, providers, models, inputTypes } = request.dimensions;

    if (clientIds && !this.validateStringArray(clientIds, errors, 'dimensions.clientIds')) {
      return;
    }

    if (aiClients && !this.validateStringArray(aiClients, errors, 'dimensions.aiClients')) {
      return;
    }

    if (teams && !this.validateStringArray(teams, errors, 'dimensions.teams')) {
      return;
    }

    if (pipelineStages && !this.validateStringArray(pipelineStages, errors, 'dimensions.pipelineStages')) {
      return;
    }

    if (providers && !this.validateStringArray(providers, errors, 'dimensions.providers')) {
      return;
    }

    if (models && !this.validateStringArray(models, errors, 'dimensions.models')) {
      return;
    }

    if (inputTypes && !this.validateStringArray(inputTypes, errors, 'dimensions.inputTypes')) {
      return;
    }
  }

  /**
   * Validate metrics
   */
  private validateMetrics(request: AnalyticsQueryRequest, errors: QueryValidationError[]): void {
    if (!request.metrics || request.metrics.length === 0) {
      errors.push({
        field: 'metrics',
        message: 'At least one metric is required',
        code: 'REQUIRED_FIELD'
      });
      return;
    }

    for (const metric of request.metrics) {
      if (!this.validMetrics.has(metric)) {
        errors.push({
          field: 'metrics',
          message: `Invalid metric: ${metric}`,
          code: 'INVALID_METRIC'
        });
      }
    }
  }

  /**
   * Validate groupBy dimensions
   */
  private validateGroupBy(request: AnalyticsQueryRequest, errors: QueryValidationError[]): void {
    if (!request.groupBy || request.groupBy.length === 0) {
      return;
    }

    for (const dimension of request.groupBy) {
      if (!this.validDimensions.has(dimension)) {
        errors.push({
          field: 'groupBy',
          message: `Invalid dimension: ${dimension}`,
          code: 'INVALID_DIMENSION'
        });
      }
    }
  }

  /**
   * Validate aggregation functions
   */
  private validateAggregations(request: AnalyticsQueryRequest, errors: QueryValidationError[]): void {
    if (!request.aggregations || request.aggregations.length === 0) {
      errors.push({
        field: 'aggregations',
        message: 'At least one aggregation function is required',
        code: 'REQUIRED_FIELD'
      });
      return;
    }

    for (const agg of request.aggregations) {
      if (!this.validAggregations.has(agg)) {
        errors.push({
          field: 'aggregations',
          message: `Invalid aggregation function: ${agg}`,
          code: 'INVALID_AGGREGATION'
        });
      }
    }
  }

  /**
   * Validate filters
   */
  private validateFilters(request: AnalyticsQueryRequest, errors: QueryValidationError[]): void {
    if (!request.filters || request.filters.length === 0) {
      return;
    }

    for (const filter of request.filters) {
      if (!filter.field) {
        errors.push({
          field: 'filters',
          message: 'Filter field is required',
          code: 'REQUIRED_FIELD'
        });
        continue;
      }

      if (!this.validOperators.has(filter.operator)) {
        errors.push({
          field: 'filters',
          message: `Invalid operator: ${filter.operator}`,
          code: 'INVALID_OPERATOR'
        });
      }

      // Sanitize field name to prevent injection
      if (!this.isValidFieldName(filter.field)) {
        errors.push({
          field: 'filters',
          message: `Invalid field name: ${filter.field}`,
          code: 'INVALID_FIELD_NAME'
        });
      }
    }
  }

  /**
   * Validate pagination parameters
   */
  private validatePagination(request: AnalyticsQueryRequest, errors: QueryValidationError[]): void {
    if (request.limit !== undefined) {
      if (typeof request.limit !== 'number' || request.limit < 1) {
        errors.push({
          field: 'limit',
          message: 'Limit must be a positive number',
          code: 'INVALID_LIMIT'
        });
      } else if (request.limit > 10000) {
        errors.push({
          field: 'limit',
          message: 'Limit cannot exceed 10000',
          code: 'LIMIT_TOO_LARGE'
        });
      }
    }

    if (request.offset !== undefined) {
      if (typeof request.offset !== 'number' || request.offset < 0) {
        errors.push({
          field: 'offset',
          message: 'Offset must be a non-negative number',
          code: 'INVALID_OFFSET'
        });
      }
    }
  }

  /**
   * Validate string array
   */
  private validateStringArray(arr: string[], errors: QueryValidationError[], field: string): boolean {
    if (!Array.isArray(arr)) {
      errors.push({
        field,
        message: `${field} must be an array`,
        code: 'INVALID_ARRAY'
      });
      return false;
    }

    for (const item of arr) {
      if (typeof item !== 'string') {
        errors.push({
          field,
          message: `${field} must contain only strings`,
          code: 'INVALID_ARRAY_ITEM'
        });
        return false;
      }
    }

    return true;
  }

  /**
   * Check if string is valid ISO date
   */
  private isValidISODate(str: string): boolean {
    return !isNaN(Date.parse(str));
  }

  /**
   * Check if field name is valid (prevent injection)
   */
  private isValidFieldName(fieldName: string): boolean {
    // Allow only alphanumeric characters, underscores, and dots
    return /^[a-zA-Z0-9_.]+$/.test(fieldName);
  }

  /**
   * Sanitize request by removing potentially harmful content
   */
  private sanitize(request: AnalyticsQueryRequest): AnalyticsQueryRequest {
    const sanitized = JSON.parse(JSON.stringify(request));

    // Sanitize dimension arrays
    if (sanitized.dimensions) {
      Object.keys(sanitized.dimensions).forEach(key => {
        if (Array.isArray(sanitized.dimensions[key])) {
          sanitized.dimensions[key] = sanitized.dimensions[key]
            .filter((item: any) => typeof item === 'string')
            .map((item: string) => item.trim());
        }
      });
    }

    // Sanitize filters
    if (sanitized.filters) {
      sanitized.filters = sanitized.filters
        .filter((filter: any) => this.isValidFieldName(filter.field))
        .map((filter: any) => ({
          ...filter,
          field: filter.field.replace(/[^a-zA-Z0-9_.]/g, '')
        }));
    }

    // Set reasonable defaults
    sanitized.limit = Math.min(sanitized.limit || 100, 10000);
    sanitized.offset = Math.max(sanitized.offset || 0, 0);

    return sanitized;
  }
}
