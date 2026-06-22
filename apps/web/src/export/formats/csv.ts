/**
 * CSV Export Format
 * 
 * Handles CSV format data export with proper escaping and formatting
 */

import { QueryResultRow } from '../../api/analytics/types';

export interface CSVExportOptions {
  includeHeaders: boolean;
  delimiter: string;
  includeBOM: boolean; // Byte Order Mark for Excel compatibility
  nullValue: string;
  dateFormat: string;
  numberFormat: string;
}

export interface CSVExportResult {
  content: string;
  filename: string;
  size: number;
  recordCount: number;
  metadata: {
    exportedAt: string;
    format: 'csv';
    delimiter: string;
    hasHeaders: boolean;
  };
}

/**
 * CSV exporter
 */
export class CSVExporter {
  private defaultOptions: CSVExportOptions = {
    includeHeaders: true,
    delimiter: ',',
    includeBOM: true,
    nullValue: '',
    dateFormat: 'YYYY-MM-DD HH:mm:ss',
    numberFormat: '0,0.00'
  };

  /**
   * Export data to CSV format
   */
  async export(
    data: QueryResultRow[],
    options: Partial<CSVExportOptions> = {}
  ): Promise<CSVExportResult> {
    const config = { ...this.defaultOptions, ...options };
    
    if (data.length === 0) {
      return this.createEmptyResult(config);
    }

    const headers = this.extractHeaders(data);
    const csvRows: string[] = [];

    // Add BOM for Excel compatibility if requested
    if (config.includeBOM) {
      csvRows.push('\uFEFF');
    }

    // Add header row
    if (config.includeHeaders) {
      csvRows.push(headers.map(h => this.escapeCSV(h, config.delimiter)).join(config.delimiter));
    }

    // Add data rows
    for (const row of data) {
      const values = headers.map(header => {
        const value = this.getValue(row, header);
        return this.formatValue(value, config);
      });
      csvRows.push(values.map(v => this.escapeCSV(v, config.delimiter)).join(config.delimiter));
    }

    const csvContent = csvRows.join('\n');
    const filename = this.generateFilename();

    return {
      content: csvContent,
      filename,
      size: csvContent.length,
      recordCount: data.length,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'csv',
        delimiter: config.delimiter,
        hasHeaders: config.includeHeaders
      }
    };
  }

  /**
   * Export data with custom column selection
   */
  async exportWithColumns(
    data: QueryResultRow[],
    columns: string[],
    options: Partial<CSVExportOptions> = {}
  ): Promise<CSVExportResult> {
    const config = { ...this.defaultOptions, ...options };
    
    if (data.length === 0) {
      return this.createEmptyResult(config);
    }

    const csvRows: string[] = [];

    // Add BOM for Excel compatibility if requested
    if (config.includeBOM) {
      csvRows.push('\uFEFF');
    }

    // Add header row
    if (config.includeHeaders) {
      csvRows.push(columns.map(h => this.escapeCSV(h, config.delimiter)).join(config.delimiter));
    }

    // Add data rows
    for (const row of data) {
      const values = columns.map(column => {
        const value = this.getValue(row, column);
        return this.formatValue(value, config);
      });
      csvRows.push(values.map(v => this.escapeCSV(v, config.delimiter)).join(config.delimiter));
    }

    const csvContent = csvRows.join('\n');
    const filename = this.generateFilename();

    return {
      content: csvContent,
      filename,
      size: csvContent.length,
      recordCount: data.length,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'csv',
        delimiter: config.delimiter,
        hasHeaders: config.includeHeaders
      }
    };
  }

  /**
   * Extract headers from data
   */
  private extractHeaders(data: QueryResultRow[]): string[] {
    const headers = new Set<string>();

    for (const row of data) {
      // Add dimension keys
      if (row.dimensions) {
        Object.keys(row.dimensions).forEach(key => headers.add(key));
      }
      
      // Add metric keys
      if (row.metrics) {
        Object.keys(row.metrics).forEach(key => headers.add(key));
      }
      
      // Add timestamp
      headers.add('timestamp');
    }

    return Array.from(headers).sort();
  }

  /**
   * Get value from row by key
   */
  private getValue(row: QueryResultRow, key: string): any {
    if (key === 'timestamp') {
      return row.timestamp;
    }
    
    if (row.dimensions && key in row.dimensions) {
      return row.dimensions[key];
    }
    
    if (row.metrics && key in row.metrics) {
      return row.metrics[key];
    }

    return null;
  }

  /**
   * Format value for CSV
   */
  private formatValue(value: any, config: CSVExportOptions): string {
    if (value === null || value === undefined) {
      return config.nullValue;
    }

    if (typeof value === 'number') {
      return this.formatNumber(value, config.numberFormat);
    }

    if (value instanceof Date) {
      return this.formatDate(value, config.dateFormat);
    }

    if (typeof value === 'object') {
      return JSON.stringify(value);
    }

    return String(value);
  }

  /**
   * Format number
   */
  private formatNumber(value: number, format: string): string {
    // Simple number formatting - in production would use a library like numeral.js
    return value.toString();
  }

  /**
   * Format date
   */
  private formatDate(date: Date, format: string): string {
    // Simple date formatting - in production would use a library like date-fns
    return date.toISOString();
  }

  /**
   * Escape CSV value
   */
  private escapeCSV(value: string, delimiter: string): string {
    const stringValue = String(value);
    
    // If value contains delimiter, quotes, or newlines, wrap in quotes and escape quotes
    if (stringValue.includes(delimiter) || 
        stringValue.includes('"') || 
        stringValue.includes('\n') ||
        stringValue.includes('\r')) {
      return `"${stringValue.replace(/"/g, '""')}"`;
    }

    return stringValue;
  }

  /**
   * Create empty result
   */
  private createEmptyResult(config: CSVExportOptions): CSVExportResult {
    const csvContent = config.includeBOM ? '\uFEFF' : '';
    
    return {
      content: csvContent,
      filename: this.generateFilename(),
      size: csvContent.length,
      recordCount: 0,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'csv',
        delimiter: config.delimiter,
        hasHeaders: config.includeHeaders
      }
    };
  }

  /**
   * Generate filename
   */
  private generateFilename(): string {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
    return `export-${timestamp}.csv`;
  }
}

// Export singleton instance
export const csvExporter = new CSVExporter();