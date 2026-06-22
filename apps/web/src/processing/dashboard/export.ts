/**
 * Dashboard Data Export
 * 
 * Handles data export formatting for dashboard data
 * Supports multiple formats: CSV, JSON, Excel, PDF
 */

import { QueryResultRow, AnalyticsQueryResponse } from '../../api/analytics/types';

export interface ExportConfig {
  format: 'csv' | 'json' | 'xlsx' | 'pdf';
  includeMetadata: boolean;
  dateFormat: string;
  numberFormat: string;
  compression: boolean;
}

export interface ExportResult {
  data: string | Buffer;
  format: string;
  size: number;
  filename: string;
  mimeType: string;
  metadata?: {
    exportedAt: string;
    recordCount: number;
    compressed: boolean;
  };
}

export interface ExportTemplate {
  name: string;
  format: ExportConfig['format'];
  columns: Array<{
    key: string;
    label: string;
    formatter?: (value: any) => string;
  }>;
  filters?: Record<string, any>;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

/**
 * Dashboard data exporter
 */
export class DashboardExporter {
  /**
   * Export data in specified format
   */
  async export(
    data: QueryResultRow[] | AnalyticsQueryResponse,
    config: ExportConfig
  ): Promise<ExportResult> {
    const isArray = Array.isArray(data);
    const records = isArray ? data : data.data;

    switch (config.format) {
      case 'csv':
        return this.exportCSV(records, config);
      case 'json':
        return this.exportJSON(data, config);
      case 'xlsx':
        return this.exportExcel(records, config);
      case 'pdf':
        return this.exportPDF(records, config);
      default:
        throw new Error(`Unsupported export format: ${config.format}`);
    }
  }

  /**
   * Export to CSV format
   */
  private exportCSV(data: QueryResultRow[], config: ExportConfig): ExportResult {
    if (data.length === 0) {
      return {
        data: '',
        format: 'csv',
        size: 0,
        filename: `dashboard-export-${Date.now()}.csv`,
        mimeType: 'text/csv',
        metadata: {
          exportedAt: new Date().toISOString(),
          recordCount: 0,
          compressed: config.compression
        }
      };
    }

    // Extract headers
    const headers = this.extractHeaders(data);
    
    // Build CSV content
    const csvRows: string[] = [];
    
    // Add header row
    csvRows.push(headers.map(h => this.escapeCSV(h)).join(','));

    // Add data rows
    for (const row of data) {
      const values = headers.map(header => {
        const value = this.getValue(row, header);
        return this.formatValue(value, config);
      });
      csvRows.push(values.map(v => this.escapeCSV(v)).join(','));
    }

    const csvContent = csvRows.join('\n');
    const filename = `dashboard-export-${Date.now()}.csv`;

    return {
      data: csvContent,
      format: 'csv',
      size: csvContent.length,
      filename,
      mimeType: 'text/csv',
      metadata: {
        exportedAt: new Date().toISOString(),
        recordCount: data.length,
        compressed: config.compression
      }
    };
  }

  /**
   * Export to JSON format
   */
  private exportJSON(
    data: QueryResultRow[] | AnalyticsQueryResponse,
    config: ExportConfig
  ): ExportResult {
    const exportData = config.includeMetadata ? {
      metadata: {
        exportedAt: new Date().toISOString(),
        recordCount: Array.isArray(data) ? data.length : data.data.length,
        format: 'json'
      },
      data: data
    } : data;

    const jsonContent = JSON.stringify(exportData, null, 2);
    const filename = `dashboard-export-${Date.now()}.json`;

    return {
      data: jsonContent,
      format: 'json',
      size: jsonContent.length,
      filename,
      mimeType: 'application/json',
      metadata: {
        exportedAt: new Date().toISOString(),
        recordCount: Array.isArray(data) ? data.length : data.data.length,
        compressed: config.compression
      }
    };
  }

  /**
   * Export to Excel format (simplified - would need xlsx library)
   */
  private exportExcel(data: QueryResultRow[], config: ExportConfig): ExportResult {
    // For now, return CSV as a simple Excel-compatible format
    // In production, this would use a library like xlsx or exceljs
    const csvResult = this.exportCSV(data, config);
    
    return {
      ...csvResult,
      format: 'xlsx',
      filename: csvResult.filename.replace('.csv', '.xlsx'),
      mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
    };
  }

  /**
   * Export to PDF format (simplified - would need PDF library)
   */
  private exportPDF(data: QueryResultRow[], config: ExportConfig): ExportResult {
    // For now, return a simple text-based format
    // In production, this would use a library like pdfkit or jsPDF
    const headers = this.extractHeaders(data);
    const textRows: string[] = [];

    textRows.push('Dashboard Export Report');
    textRows.push(`Exported: ${new Date().toISOString()}`);
    textRows.push(`Records: ${data.length}`);
    textRows.push('');

    // Add header row
    textRows.push(headers.join(' | '));

    // Add separator
    textRows.push(headers.map(() => '-'.repeat(20)).join(' | '));

    // Add data rows
    for (const row of data) {
      const values = headers.map(header => {
        const value = this.getValue(row, header);
        return String(this.formatValue(value, config)).padEnd(20);
      });
      textRows.push(values.join(' | '));
    }

    const textContent = textRows.join('\n');
    const filename = `dashboard-export-${Date.now()}.txt`;

    return {
      data: textContent,
      format: 'pdf',
      size: textContent.length,
      filename,
      mimeType: 'text/plain',
      metadata: {
        exportedAt: new Date().toISOString(),
        recordCount: data.length,
        compressed: config.compression
      }
    };
  }

  /**
   * Export using a template
   */
  async exportWithTemplate(
    data: QueryResultRow[],
    template: ExportTemplate,
    config: ExportConfig
  ): Promise<ExportResult> {
    // Apply filters
    let filteredData = data;
    if (template.filters) {
      filteredData = this.applyFilters(data, template.filters);
    }

    // Apply sorting
    if (template.sortBy) {
      filteredData = this.applySort(filteredData, template.sortBy, template.sortOrder);
    }

    // Select and format columns
    const formattedData = filteredData.map(row => {
      const formatted: QueryResultRow = {
        timestamp: row.timestamp,
        dimensions: {},
        metrics: {}
      };

      for (const column of template.columns) {
        const value = this.getValue(row, column.key);
        const formattedValue = column.formatter ? column.formatter(value) : value;
        
        // Store in appropriate bucket
        if (row.dimensions && column.key in row.dimensions) {
          formatted.dimensions![column.key] = formattedValue;
        } else if (row.metrics && column.key in row.metrics) {
          formatted.metrics![column.key] = formattedValue;
        } else {
          formatted.metrics![column.key] = formattedValue;
        }
      }

      return formatted;
    });

    // Export with the formatted data
    return this.export(formattedData, {
      ...config,
      format: template.format
    });
  }

  /**
   * Create export template from current view
   */
  createTemplate(
    name: string,
    data: QueryResultRow[],
    format: ExportConfig['format']
  ): ExportTemplate {
    const headers = this.extractHeaders(data);

    return {
      name,
      format,
      columns: headers.map(header => ({
        key: header,
        label: this.formatLabel(header),
        formatter: (value: any) => this.formatValue(value, {
          dateFormat: 'YYYY-MM-DD',
          numberFormat: '0,0.00'
        })
      }))
    };
  }

  /**
   * Get supported export formats
   */
  static getSupportedFormats(): Array<{ format: string; mimeType: string; extension: string }> {
    return [
      { format: 'csv', mimeType: 'text/csv', extension: 'csv' },
      { format: 'json', mimeType: 'application/json', extension: 'json' },
      { format: 'xlsx', mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', extension: 'xlsx' },
      { format: 'pdf', mimeType: 'application/pdf', extension: 'pdf' }
    ];
  }

  // Helper methods

  private extractHeaders(data: QueryResultRow[]): string[] {
    if (data.length === 0) return [];

    const headers = new Set<string>();

    for (const row of data) {
      if (row.dimensions) {
        for (const key of Object.keys(row.dimensions)) {
          headers.add(key);
        }
      }
      if (row.metrics) {
        for (const key of Object.keys(row.metrics)) {
          headers.add(key);
        }
      }
      if (row.timestamp) {
        headers.add('timestamp');
      }
    }

    return Array.from(headers);
  }

  private getValue(row: QueryResultRow, key: string): any {
    if (key === 'timestamp') return row.timestamp;
    if (row.dimensions && key in row.dimensions) return row.dimensions[key];
    if (row.metrics && key in row.metrics) return row.metrics[key];
    return null;
  }

  private formatValue(value: any, config: ExportConfig): string {
    if (value === null || value === undefined) return '';

    if (typeof value === 'number') {
      return this.formatNumber(value, config.numberFormat);
    }

    if (typeof value === 'string') {
      // Check if it's a date
      if (this.isDateString(value)) {
        return this.formatDate(value, config.dateFormat);
      }
      return value;
    }

    return String(value);
  }

  private formatNumber(value: number, format: string): string {
    // Simple number formatting
    if (format.includes(',')) {
      return value.toLocaleString();
    }
    if (format.includes('.') && format.split('.').length > 1) {
      const decimals = format.split('.')[1].length;
      return value.toFixed(decimals);
    }
    return String(value);
  }

  private formatDate(value: string, format: string): string {
    try {
      const date = new Date(value);
      
      // Simple date formatting
      switch (format) {
        case 'YYYY-MM-DD':
          return date.toISOString().split('T')[0];
        case 'YYYY-MM-DD HH:mm:ss':
          return date.toISOString().replace('T', ' ').split('.')[0];
        default:
          return date.toISOString();
      }
    } catch {
      return value;
    }
  }

  private isDateString(value: string): boolean {
    return !isNaN(Date.parse(value));
  }

  private escapeCSV(value: string): string {
    if (value.includes(',') || value.includes('"') || value.includes('\n')) {
      return `"${value.replace(/"/g, '""')}"`;
    }
    return value;
  }

  private formatLabel(key: string): string {
    return key
      .replace(/([A-Z])/g, ' $1')
      .replace(/^./, str => str.toUpperCase())
      .trim();
  }

  private applyFilters(data: QueryResultRow[], filters: Record<string, any>): QueryResultRow[] {
    return data.filter(row => {
      for (const [key, filterValue] of Object.entries(filters)) {
        const rowValue = this.getValue(row, key);
        
        if (Array.isArray(filterValue)) {
          if (!filterValue.includes(rowValue)) return false;
        } else if (typeof filterValue === 'object') {
          // Handle range filters
          if (filterValue.min !== undefined && rowValue < filterValue.min) return false;
          if (filterValue.max !== undefined && rowValue > filterValue.max) return false;
        } else {
          if (rowValue !== filterValue) return false;
        }
      }
      return true;
    });
  }

  private applySort(
    data: QueryResultRow[],
    sortBy: string,
    sortOrder: 'asc' | 'desc' = 'asc'
  ): QueryResultRow[] {
    return [...data].sort((a, b) => {
      const aValue = this.getValue(a, sortBy);
      const bValue = this.getValue(b, sortBy);

      let comparison = 0;
      if (aValue < bValue) comparison = -1;
      if (aValue > bValue) comparison = 1;

      return sortOrder === 'desc' ? -comparison : comparison;
    });
  }
}

/**
 * Pre-built export templates
 */
export const ExportTemplates = {
  /**
   * Standard dashboard export template
   */
  standard: {
    name: 'standard',
    format: 'csv' as const,
    columns: [
      { key: 'timestamp', label: 'Timestamp' },
      { key: 'aiClient', label: 'AI Client' },
      { key: 'provider', label: 'Provider' },
      { key: 'model', label: 'Model' },
      { key: 'requestCount', label: 'Request Count' },
      { key: 'tokenCount', label: 'Token Count' },
      { key: 'cost', label: 'Cost' }
    ]
  } as ExportTemplate,

  /**
   * Cost analysis export template
   */
  costAnalysis: {
    name: 'cost-analysis',
    format: 'xlsx' as const,
    columns: [
      { key: 'timestamp', label: 'Date' },
      { key: 'provider', label: 'Provider' },
      { key: 'model', label: 'Model' },
      { key: 'cost', label: 'Cost ($)' },
      { key: 'tokenCount', label: 'Tokens' },
      { key: 'requestCount', label: 'Requests' }
    ],
    sortBy: 'cost',
    sortOrder: 'desc' as const
  } as ExportTemplate,

  /**
   * Performance metrics export template
   */
  performance: {
    name: 'performance',
    format: 'csv' as const,
    columns: [
      { key: 'timestamp', label: 'Timestamp' },
      { key: 'aiClient', label: 'AI Client' },
      { key: 'latency', label: 'Latency (ms)' },
      { key: 'throughput', label: 'Throughput' },
      { key: 'errorRate', label: 'Error Rate (%)' }
    ]
  } as ExportTemplate
};

/**
 * Global exporter instance
 */
export const dashboardExporter = new DashboardExporter();
