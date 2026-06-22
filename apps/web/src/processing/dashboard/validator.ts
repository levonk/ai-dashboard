/**
 * Dashboard Data Validator
 * 
 * Validates and sanitizes dashboard data
 * Ensures data integrity and prevents malformed data from reaching the UI
 */

import { QueryResultRow, AnalyticsQueryResponse } from '../../api/analytics/types';

export interface ValidationRule {
  name: string;
  validate: (data: any) => boolean;
  message: string;
  severity: 'error' | 'warning' | 'info';
}

export interface ValidationResult {
  valid: boolean;
  errors: Array<{
    field: string;
    message: string;
    severity: 'error' | 'warning' | 'info';
  }>;
  sanitized: any;
  stats: {
    totalChecked: number;
    errorsFound: number;
    warningsFound: number;
  };
}

export interface SchemaDefinition {
  fields: Record<string, {
    type: 'string' | 'number' | 'boolean' | 'date' | 'array' | 'object';
    required: boolean;
    validate?: (value: any) => boolean;
    sanitize?: (value: any) => any;
    defaultValue?: any;
  }>;
}

/**
 * Dashboard data validator
 */
export class DashboardValidator {
  private rules: ValidationRule[] = [];
  private schemas: Map<string, SchemaDefinition> = new Map();

  /**
   * Add a validation rule
   */
  addRule(rule: ValidationRule): void {
    this.rules.push(rule);
  }

  /**
   * Register a schema
   */
  registerSchema(name: string, schema: SchemaDefinition): void {
    this.schemas.set(name, schema);
  }

  /**
   * Validate data against all rules
   */
  validate(data: any): ValidationResult {
    const errors: ValidationResult['errors'] = [];
    let totalChecked = 0;

    for (const rule of this.rules) {
      totalChecked++;
      if (!rule.validate(data)) {
        errors.push({
          field: rule.name,
          message: rule.message,
          severity: rule.severity
        });
      }
    }

    return {
      valid: errors.filter(e => e.severity === 'error').length === 0,
      errors,
      sanitized: data,
      stats: {
        totalChecked,
        errorsFound: errors.filter(e => e.severity === 'error').length,
        warningsFound: errors.filter(e => e.severity === 'warning').length
      }
    };
  }

  /**
   * Validate data against a schema
   */
  validateSchema(data: any, schemaName: string): ValidationResult {
    const schema = this.schemas.get(schemaName);
    
    if (!schema) {
      return {
        valid: false,
        errors: [{
          field: 'schema',
          message: `Schema ${schemaName} not found`,
          severity: 'error'
        }],
        sanitized: data,
        stats: {
          totalChecked: 0,
          errorsFound: 1,
          warningsFound: 0
        }
      };
    }

    const errors: ValidationResult['errors'] = [];
    const sanitized = { ...data };
    let totalChecked = 0;

    for (const [fieldName, fieldDef] of Object.entries(schema.fields)) {
      totalChecked++;
      const value = data[fieldName];

      // Check required fields
      if (fieldDef.required && (value === undefined || value === null)) {
        errors.push({
          field: fieldName,
          message: `Field ${fieldName} is required`,
          severity: 'error'
        });
        continue;
      }

      // Skip validation if field is not required and not present
      if (!fieldDef.required && (value === undefined || value === null)) {
        if (fieldDef.defaultValue !== undefined) {
          sanitized[fieldName] = fieldDef.defaultValue;
        }
        continue;
      }

      // Type validation
      if (!this.validateType(value, fieldDef.type)) {
        errors.push({
          field: fieldName,
          message: `Field ${fieldName} must be of type ${fieldDef.type}`,
          severity: 'error'
        });
        continue;
      }

      // Custom validation
      if (fieldDef.validate && !fieldDef.validate(value)) {
        errors.push({
          field: fieldName,
          message: `Field ${fieldName} failed custom validation`,
          severity: 'error'
        });
        continue;
      }

      // Sanitization
      if (fieldDef.sanitize) {
        sanitized[fieldName] = fieldDef.sanitize(value);
      }
    }

    return {
      valid: errors.filter(e => e.severity === 'error').length === 0,
      errors,
      sanitized,
      stats: {
        totalChecked,
        errorsFound: errors.filter(e => e.severity === 'error').length,
        warningsFound: errors.filter(e => e.severity === 'warning').length
      }
    };
  }

  /**
   * Validate analytics query response
   */
  validateQueryResponse(response: AnalyticsQueryResponse): ValidationResult {
    const errors: ValidationResult['errors'] = [];
    const sanitized = { ...response };
    let totalChecked = 0;

    // Validate response structure
    totalChecked++;
    if (!response.queryId) {
      errors.push({
        field: 'queryId',
        message: 'queryId is required',
        severity: 'error'
      });
    }

    totalChecked++;
    if (typeof response.executionTimeMs !== 'number') {
      errors.push({
        field: 'executionTimeMs',
        message: 'executionTimeMs must be a number',
        severity: 'error'
      });
    }

    totalChecked++;
    if (!Array.isArray(response.data)) {
      errors.push({
        field: 'data',
        message: 'data must be an array',
        severity: 'error'
      });
    } else {
      // Validate each row
      for (let i = 0; i < response.data.length; i++) {
        const rowValidation = this.validateQueryRow(response.data[i]);
        errors.push(...rowValidation.errors.map(e => ({
          ...e,
          field: `data[${i}].${e.field}`
        })));
        totalChecked += rowValidation.stats.totalChecked;
      }
    }

    // Validate pagination
    totalChecked++;
    if (!response.pagination) {
      errors.push({
        field: 'pagination',
        message: 'pagination is required',
        severity: 'error'
      });
    } else {
      if (typeof response.pagination.total !== 'number') {
        errors.push({
          field: 'pagination.total',
          message: 'pagination.total must be a number',
          severity: 'error'
        });
      }
      if (typeof response.pagination.limit !== 'number') {
        errors.push({
          field: 'pagination.limit',
          message: 'pagination.limit must be a number',
          severity: 'error'
        });
      }
    }

    return {
      valid: errors.filter(e => e.severity === 'error').length === 0,
      errors,
      sanitized,
      stats: {
        totalChecked,
        errorsFound: errors.filter(e => e.severity === 'error').length,
        warningsFound: errors.filter(e => e.severity === 'warning').length
      }
    };
  }

  /**
   * Validate a single query result row
   */
  validateQueryRow(row: QueryResultRow): ValidationResult {
    const errors: ValidationResult['errors'] = [];
    const sanitized = { ...row };
    let totalChecked = 0;

    // Validate dimensions if present
    if (row.dimensions) {
      totalChecked++;
      if (typeof row.dimensions !== 'object') {
        errors.push({
          field: 'dimensions',
          message: 'dimensions must be an object',
          severity: 'error'
        });
      } else {
        // Sanitize dimension values
        for (const [key, value] of Object.entries(row.dimensions)) {
          sanitized.dimensions![key] = this.sanitizeString(value);
        }
      }
    }

    // Validate metrics
    totalChecked++;
    if (!row.metrics) {
      errors.push({
        field: 'metrics',
        message: 'metrics is required',
        severity: 'error'
      });
    } else if (typeof row.metrics !== 'object') {
      errors.push({
        field: 'metrics',
        message: 'metrics must be an object',
        severity: 'error'
      });
    } else {
      // Validate metric values are numbers
      for (const [key, value] of Object.entries(row.metrics)) {
        totalChecked++;
        if (typeof value !== 'number' || isNaN(value)) {
          errors.push({
            field: `metrics.${key}`,
            message: `Metric ${key} must be a valid number`,
            severity: 'error'
          });
          sanitized.metrics![key] = 0; // Default to 0
        }
        
        // Check for infinite values
        if (!isFinite(value)) {
          errors.push({
            field: `metrics.${key}`,
            message: `Metric ${key} has infinite value`,
            severity: 'error'
          });
          sanitized.metrics![key] = 0;
        }
      }
    }

    // Validate timestamp if present
    if (row.timestamp) {
      totalChecked++;
      if (!this.isValidDate(row.timestamp)) {
        errors.push({
          field: 'timestamp',
          message: 'timestamp must be a valid ISO 8601 date',
          severity: 'error'
        });
      }
    }

    return {
      valid: errors.filter(e => e.severity === 'error').length === 0,
      errors,
      sanitized,
      stats: {
        totalChecked,
        errorsFound: errors.filter(e => e.severity === 'error').length,
        warningsFound: errors.filter(e => e.severity === 'warning').length
      }
    };
  }

  /**
   * Sanitize data for dashboard display
   */
  sanitizeForDashboard(data: any): any {
    if (Array.isArray(data)) {
      return data.map(item => this.sanitizeForDashboard(item));
    }

    if (typeof data === 'object' && data !== null) {
      const sanitized: any = {};
      for (const [key, value] of Object.entries(data)) {
        sanitized[key] = this.sanitizeForDashboard(value);
      }
      return sanitized;
    }

    if (typeof data === 'string') {
      return this.sanitizeString(data);
    }

    return data;
  }

  // Helper methods

  private validateType(value: any, type: string): boolean {
    switch (type) {
      case 'string':
        return typeof value === 'string';
      case 'number':
        return typeof value === 'number' && !isNaN(value);
      case 'boolean':
        return typeof value === 'boolean';
      case 'date':
        return this.isValidDate(value);
      case 'array':
        return Array.isArray(value);
      case 'object':
        return typeof value === 'object' && value !== null && !Array.isArray(value);
      default:
        return true;
    }
  }

  private isValidDate(value: any): boolean {
    if (typeof value !== 'string') return false;
    const date = new Date(value);
    return !isNaN(date.getTime());
  }

  private sanitizeString(value: any): string {
    if (typeof value !== 'string') return String(value);
    
    // Remove potentially dangerous characters
    return value
      .replace(/[\x00-\x1F\x7F]/g, '') // Remove control characters
      .trim();
  }
}

/**
 * Pre-built validation rules for dashboard data
 */
export const DashboardValidationRules = {
  /**
   * Rule to ensure no negative costs
   */
  noNegativeCosts: {
    name: 'noNegativeCosts',
    validate: (data: QueryResultRow[]) => {
      return !data.some(row => 
        Object.entries(row.metrics || {}).some(([key, value]) => 
          key.toLowerCase().includes('cost') && value < 0
        )
      );
    },
    message: 'Cost metrics cannot be negative',
    severity: 'error' as const
  },

  /**
   * Rule to ensure reasonable latency values
   */
  reasonableLatency: {
    name: 'reasonableLatency',
    validate: (data: QueryResultRow[]) => {
      return !data.some(row => 
        Object.entries(row.metrics || {}).some(([key, value]) => 
          key.toLowerCase().includes('latency') && (value < 0 || value > 60000) // Max 60 seconds
        )
      );
    },
    message: 'Latency values must be between 0 and 60000ms',
    severity: 'warning' as const
  },

  /**
   * Rule to ensure timestamps are in valid range
   */
  validTimestampRange: {
    name: 'validTimestampRange',
    validate: (data: QueryResultRow[]) => {
      const now = Date.now();
      const oneYearAgo = now - (365 * 24 * 60 * 60 * 1000);
      
      return !data.some(row => {
        if (!row.timestamp) return false;
        const timestamp = new Date(row.timestamp).getTime();
        return timestamp < oneYearAgo || timestamp > now;
      });
    },
    message: 'Timestamps must be within the last year',
    severity: 'warning' as const
  },

  /**
   * Rule to ensure data is not empty
   */
  nonEmptyData: {
    name: 'nonEmptyData',
    validate: (data: any) => {
      if (Array.isArray(data)) return data.length > 0;
      if (typeof data === 'object') return Object.keys(data).length > 0;
      return data !== null && data !== undefined;
    },
    message: 'Data cannot be empty',
    severity: 'error' as const
  }
};

/**
 * Pre-built schemas for dashboard data
 */
export const DashboardSchemas = {
  /**
   * Schema for metric card data
   */
  metricCard: {
    fields: {
      title: {
        type: 'string',
        required: true,
        sanitize: (value: string) => value.trim()
      },
      value: {
        type: 'number',
        required: true,
        validate: (value: number) => !isNaN(value) && isFinite(value)
      },
      change: {
        type: 'number',
        required: false,
        defaultValue: 0
      },
      changeType: {
        type: 'string',
        required: false,
        validate: (value: string) => ['increase', 'decrease'].includes(value)
      },
      unit: {
        type: 'string',
        required: false,
        defaultValue: ''
      }
    }
  } as SchemaDefinition,

  /**
   * Schema for chart data
   */
  chartData: {
    fields: {
      labels: {
        type: 'array',
        required: true,
        validate: (value: any[]) => Array.isArray(value) && value.every(v => typeof v === 'string')
      },
      datasets: {
        type: 'array',
        required: true,
        validate: (value: any[]) => Array.isArray(value)
      }
    }
  } as SchemaDefinition,

  /**
   * Schema for table data
   */
  tableData: {
    fields: {
      columns: {
        type: 'array',
        required: true
      },
      rows: {
        type: 'array',
        required: true
      }
    }
  } as SchemaDefinition
};

/**
 * Global validator instance with pre-configured rules and schemas
 */
export const dashboardValidator = new DashboardValidator();

// Add pre-built rules
Object.values(DashboardValidationRules).forEach(rule => {
  dashboardValidator.addRule(rule);
});

// Register pre-built schemas
Object.entries(DashboardSchemas).forEach(([name, schema]) => {
  dashboardValidator.registerSchema(name, schema);
});
