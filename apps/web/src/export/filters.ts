/**
 * Export Filtering and Customization
 * 
 * Handles data filtering, transformation, and customization for exports
 */

import { QueryResultRow } from '../api/analytics/types';

export interface ExportFilter {
  field: string;
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'contains' | 'startsWith' | 'endsWith' | 'in' | 'notIn';
  value: any;
}

export interface ExportSort {
  field: string;
  order: 'asc' | 'desc';
}

export interface ExportColumnConfig {
  key: string;
  label: string;
  formatter?: (value: any) => string;
  visible: boolean;
  width?: number;
}

export interface ExportFilterConfig {
  filters: ExportFilter[];
  sorts: ExportSort[];
  columns: ExportColumnConfig[];
  limit?: number;
  offset?: number;
}

/**
 * Export filter processor
 */
export class ExportFilterProcessor {
  /**
   * Apply filters to data
   */
  applyFilters(data: QueryResultRow[], filters: ExportFilter[]): QueryResultRow[] {
    if (filters.length === 0) {
      return data;
    }

    return data.filter(row => {
      return filters.every(filter => this.matchesFilter(row, filter));
    });
  }

  /**
   * Check if row matches a single filter
   */
  private matchesFilter(row: QueryResultRow, filter: ExportFilter): boolean {
    const value = this.getFieldValue(row, filter.field);

    switch (filter.operator) {
      case 'eq':
        return value === filter.value;
      case 'ne':
        return value !== filter.value;
      case 'gt':
        return typeof value === 'number' && value > filter.value;
      case 'gte':
        return typeof value === 'number' && value >= filter.value;
      case 'lt':
        return typeof value === 'number' && value < filter.value;
      case 'lte':
        return typeof value === 'number' && value <= filter.value;
      case 'contains':
        return typeof value === 'string' && value.includes(filter.value);
      case 'startsWith':
        return typeof value === 'string' && value.startsWith(filter.value);
      case 'endsWith':
        return typeof value === 'string' && value.endsWith(filter.value);
      case 'in':
        return Array.isArray(filter.value) && filter.value.includes(value);
      case 'notIn':
        return Array.isArray(filter.value) && !filter.value.includes(value);
      default:
        return false;
    }
  }

  /**
   * Apply sorting to data
   */
  applySorts(data: QueryResultRow[], sorts: ExportSort[]): QueryResultRow[] {
    if (sorts.length === 0) {
      return data;
    }

    return [...data].sort((a, b) => {
      for (const sort of sorts) {
        const aValue = this.getFieldValue(a, sort.field);
        const bValue = this.getFieldValue(b, sort.field);

        const comparison = this.compareValues(aValue, bValue);
        
        if (comparison !== 0) {
          return sort.order === 'asc' ? comparison : -comparison;
        }
      }
      return 0;
    });
  }

  /**
   * Compare two values for sorting
   */
  private compareValues(a: any, b: any): number {
    if (a === null || a === undefined) return -1;
    if (b === null || b === undefined) return 1;

    if (typeof a === 'number' && typeof b === 'number') {
      return a - b;
    }

    if (a instanceof Date && b instanceof Date) {
      return a.getTime() - b.getTime();
    }

    const aStr = String(a);
    const bStr = String(b);
    return aStr.localeCompare(bStr);
  }

  /**
   * Apply column selection and transformation
   */
  applyColumns(data: QueryResultRow[], columns: ExportColumnConfig[]): QueryResultRow[] {
    const visibleColumns = columns.filter(col => col.visible);

    return data.map(row => {
      const transformed: QueryResultRow = {
        timestamp: row.timestamp,
        dimensions: {},
        metrics: {}
      };

      for (const column of visibleColumns) {
        const value = this.getFieldValue(row, column.key);
        const formattedValue = column.formatter ? column.formatter(value) : value;

        // Store in appropriate bucket
        if (row.dimensions && column.key in row.dimensions) {
          transformed.dimensions![column.key] = formattedValue;
        } else if (row.metrics && column.key in row.metrics) {
          transformed.metrics![column.key] = formattedValue;
        } else if (column.key === 'timestamp') {
          transformed.timestamp = formattedValue;
        } else {
          transformed.metrics![column.key] = formattedValue;
        }
      }

      return transformed;
    });
  }

  /**
   * Apply pagination
   */
  applyPagination(data: QueryResultRow[], limit?: number, offset?: number): QueryResultRow[] {
    let result = data;

    if (offset && offset > 0) {
      result = result.slice(offset);
    }

    if (limit && limit > 0) {
      result = result.slice(0, limit);
    }

    return result;
  }

  /**
   * Apply complete filter configuration
   */
  applyConfig(data: QueryResultRow[], config: ExportFilterConfig): QueryResultRow[] {
    let result = data;

    // Apply filters
    result = this.applyFilters(result, config.filters);

    // Apply sorts
    result = this.applySorts(result, config.sorts);

    // Apply column selection
    result = this.applyColumns(result, config.columns);

    // Apply pagination
    result = this.applyPagination(result, config.limit, config.offset);

    return result;
  }

  /**
   * Get field value from row
   */
  private getFieldValue(row: QueryResultRow, field: string): any {
    if (field === 'timestamp') {
      return row.timestamp;
    }

    if (row.dimensions && field in row.dimensions) {
      return row.dimensions[field];
    }

    if (row.metrics && field in row.metrics) {
      return row.metrics[field];
    }

    return null;
  }

  /**
   * Create default column config from data
   */
  createDefaultColumns(data: QueryResultRow[]): ExportColumnConfig[] {
    const fields = this.extractFields(data);

    return fields.map(field => ({
      key: field,
      label: this.formatLabel(field),
      visible: true,
      formatter: (value: any) => this.formatValue(value)
    }));
  }

  /**
   * Extract all unique fields from data
   */
  private extractFields(data: QueryResultRow[]): string[] {
    const fields = new Set<string>();

    for (const row of data) {
      fields.add('timestamp');
      
      if (row.dimensions) {
        Object.keys(row.dimensions).forEach(key => fields.add(key));
      }
      
      if (row.metrics) {
        Object.keys(row.metrics).forEach(key => fields.add(key));
      }
    }

    return Array.from(fields).sort();
  }

  /**
   * Format field label
   */
  private formatLabel(field: string): string {
    return field
      .replace(/([A-Z])/g, ' $1')
      .replace(/^./, str => str.toUpperCase())
      .trim();
  }

  /**
   * Format value for display
   */
  private formatValue(value: any): string {
    if (value === null || value === undefined) {
      return '';
    }

    if (typeof value === 'number') {
      return value.toLocaleString();
    }

    if (value instanceof Date) {
      return value.toISOString();
    }

    if (typeof value === 'object') {
      return JSON.stringify(value);
    }

    return String(value);
  }

  /**
   * Validate filter configuration
   */
  validateConfig(config: ExportFilterConfig, data: QueryResultRow[]): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    const fields = this.extractFields(data);

    // Validate filters
    for (const filter of config.filters) {
      if (!fields.includes(filter.field)) {
        errors.push(`Invalid filter field: ${filter.field}`);
      }

      if (filter.operator === 'in' || filter.operator === 'notIn') {
        if (!Array.isArray(filter.value)) {
          errors.push(`Filter ${filter.field} with ${filter.operator} requires array value`);
        }
      }
    }

    // Validate sorts
    for (const sort of config.sorts) {
      if (!fields.includes(sort.field)) {
        errors.push(`Invalid sort field: ${sort.field}`);
      }
    }

    // Validate columns
    for (const column of config.columns) {
      if (!fields.includes(column.key)) {
        errors.push(`Invalid column field: ${column.key}`);
      }
    }

    // Validate pagination
    if (config.limit !== undefined && config.limit < 0) {
      errors.push('Limit must be non-negative');
    }

    if (config.offset !== undefined && config.offset < 0) {
      errors.push('Offset must be non-negative');
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }
}

// Export singleton instance
export const exportFilterProcessor = new ExportFilterProcessor();