/**
 * ToonFormat Export Format
 * 
 * Handles ToonFormat data export for token-efficient AI agent consumption
 * ToonFormat is a compact, human-readable format designed for LLM prompts
 * with significantly reduced token usage compared to JSON
 */

import { QueryResultRow, AnalyticsQueryResponse } from '../../api/analytics/types';
import { z } from 'zod';

export interface ToonFormatExportOptions {
  includeMetadata: boolean;
  delimiter: 'comma' | 'tab' | 'pipe';
  indentation: number;
  keyFolding: boolean;
  flattenDepth: number;
}

export interface ToonFormatExportResult {
  content: string;
  filename: string;
  size: number;
  recordCount: number;
  compressionRatio: number;
  metadata: {
    exportedAt: string;
    format: 'toon';
    hasMetadata: boolean;
    compressionRatio: number;
    tokenSavings: number;
  };
}

// Schema for export metadata
const ExportMetadataSchema = z.object({
  exportedAt: z.string(),
  recordCount: z.number(),
  format: z.literal('toon'),
  version: z.string().default('3.0'),
  exportId: z.string(),
  compressionRatio: z.number()
});

// Schema for a single record
const RecordSchema = z.object({
  timestamp: z.string(),
  dimensions: z.record(z.any()).optional(),
  metrics: z.record(z.any()).optional()
});

// Schema for full export
const ExportDataSchema = z.object({
  metadata: ExportMetadataSchema.optional(),
  data: z.array(RecordSchema)
});

/**
 * ToonFormat exporter for token-efficient data export
 */
export class ToonFormatExporter {
  private defaultOptions: ToonFormatExportOptions = {
    includeMetadata: true,
    delimiter: 'comma',
    indentation: 2,
    keyFolding: true,
    flattenDepth: 3
  };

  /**
   * Export data to ToonFormat for AI agent consumption
   */
  async export(
    data: QueryResultRow[] | AnalyticsQueryResponse,
    options: Partial<ToonFormatExportOptions> = {}
  ): Promise<ToonFormatExportResult> {
    const config = { ...this.defaultOptions, ...options };
    
    const isArray = Array.isArray(data);
    const records = isArray ? data : data.data;
    const exportId = this.generateExportId();

    // Calculate JSON size for compression ratio
    const jsonSize = JSON.stringify(records).length;

    // Build ToonFormat content
    let toonContent = this.buildToonFormat(records, config);

    // Add metadata if requested
    if (config.includeMetadata) {
      const metadata = this.buildMetadata(records.length, exportId, jsonSize, toonContent.length);
      toonContent = this.prependMetadata(toonContent, metadata);
    }

    const compressionRatio = this.calculateCompressionRatio(jsonSize, toonContent.length);
    const filename = this.generateFilename(exportId);

    return {
      content: toonContent,
      filename,
      size: toonContent.length,
      recordCount: records.length,
      compressionRatio,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'toon',
        hasMetadata: config.includeMetadata,
        compressionRatio,
        tokenSavings: this.estimateTokenSavings(jsonSize, toonContent.length)
      }
    };
  }

  /**
   * Export data with custom field selection
   */
  async exportWithFields(
    data: QueryResultRow[],
    fields: string[],
    options: Partial<ToonFormatExportOptions> = {}
  ): Promise<ToonFormatExportResult> {
    const config = { ...this.defaultOptions, ...options };
    const exportId = this.generateExportId();

    // Process records with field selection
    const processedRecords = data.map(row => {
      const selected: any = {};
      
      for (const field of fields) {
        if (field === 'timestamp') {
          selected.timestamp = row.timestamp;
        } else if (row.dimensions && field in row.dimensions) {
          selected[field] = row.dimensions[field];
        } else if (row.metrics && field in row.metrics) {
          selected[field] = row.metrics[field];
        }
      }

      return selected;
    });

    // Calculate JSON size for compression ratio
    const jsonSize = JSON.stringify(processedRecords).length;

    // Build ToonFormat content
    let toonContent = this.buildToonFormat(processedRecords, config);

    // Add metadata if requested
    if (config.includeMetadata) {
      const metadata = this.buildMetadata(processedRecords.length, exportId, jsonSize, toonContent.length);
      toonContent = this.prependMetadata(toonContent, metadata);
    }

    const compressionRatio = this.calculateCompressionRatio(jsonSize, toonContent.length);
    const filename = this.generateFilename(exportId);

    return {
      content: toonContent,
      filename,
      size: toonContent.length,
      recordCount: processedRecords.length,
      compressionRatio,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'toon',
        hasMetadata: config.includeMetadata,
        compressionRatio,
        tokenSavings: this.estimateTokenSavings(jsonSize, toonContent.length)
      }
    };
  }

  /**
   * Build ToonFormat content from records
   */
  private buildToonFormat(records: any[], config: ToonFormatExportOptions): string {
    if (records.length === 0) {
      return 'data[]';
    }

    const delimiter = this.getDelimiterChar(config.delimiter);
    const indent = ' '.repeat(config.indentation);

    // Collect all unique fields
    const fields = this.collectFields(records, config);
    
    // Build header with field names
    let toon = `data[${records.length}]{${fields.join(delimiter)}}:\n`;

    // Build data rows
    for (const record of records) {
      const values = fields.map(field => {
        const value = this.getNestedValue(record, field);
        return this.formatValue(value, config);
      });
      
      toon += `${indent}${values.join(delimiter)}\n`;
    }

    return toon;
  }

  /**
   * Collect all unique fields from records
   */
  private collectFields(records: any[], config: ToonFormatExportOptions): string[] {
    const fieldSet = new Set<string>();
    
    for (const record of records) {
      this.extractFields(record, '', fieldSet, config.keyFolding ? 3 : 0);
    }
    
    return Array.from(fieldSet).sort();
  }

  /**
   * Extract fields from a record recursively
   */
  private extractFields(obj: any, prefix: string, fieldSet: Set<string>, depth: number): void {
    if (depth <= 0 || typeof obj !== 'object' || obj === null) {
      return;
    }

    for (const [key, value] of Object.entries(obj)) {
      const fieldPath = prefix ? `${prefix}.${key}` : key;
      
      if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        this.extractFields(value, fieldPath, fieldSet, depth - 1);
      } else {
        fieldSet.add(fieldPath);
      }
    }
  }

  /**
   * Get nested value from object using dot notation
   */
  private getNestedValue(obj: any, path: string): any {
    const keys = path.split('.');
    let current = obj;
    
    for (const key of keys) {
      if (current === null || current === undefined) {
        return null;
      }
      current = current[key];
    }
    
    return current;
  }

  /**
   * Format a value for ToonFormat
   */
  private formatValue(value: any, config: ToonFormatExportOptions): string {
    if (value === null || value === undefined) {
      return '';
    }
    
    if (typeof value === 'string') {
      // Escape special characters if needed
      if (value.includes(',') || value.includes('\n') || value.includes('"')) {
        return `"${value.replace(/"/g, '\\"')}"`;
      }
      return value;
    }
    
    if (typeof value === 'number') {
      return value.toString();
    }
    
    if (typeof value === 'boolean') {
      return value ? 'true' : 'false';
    }
    
    if (Array.isArray(value)) {
      return `[${value.map(v => this.formatValue(v, config)).join(',')}]`;
    }
    
    if (typeof value === 'object') {
      return JSON.stringify(value);
    }
    
    return String(value);
  }

  /**
   * Get delimiter character
   */
  private getDelimiterChar(delimiter: string): string {
    switch (delimiter) {
      case 'tab': return '\t';
      case 'pipe': return '|';
      default: return ',';
    }
  }

  /**
   * Build metadata section
   */
  private buildMetadata(recordCount: number, exportId: string, jsonSize: number, toonSize: number): string {
    const compressionRatio = this.calculateCompressionRatio(jsonSize, toonSize);
    
    return `# ToonFormat Export
exportedAt: ${new Date().toISOString()}
recordCount: ${recordCount}
format: toon
version: 3.0
exportId: ${exportId}
compressionRatio: ${compressionRatio.toFixed(2)}

`;
  }

  /**
   * Prepend metadata to content
   */
  private prependMetadata(content: string, metadata: string): string {
    return metadata + content;
  }

  /**
   * Calculate compression ratio
   */
  private calculateCompressionRatio(jsonSize: number, toonSize: number): number {
    if (jsonSize === 0) return 0;
    return (jsonSize - toonSize) / jsonSize;
  }

  /**
   * Estimate token savings (rough approximation)
   */
  private estimateTokenSavings(jsonSize: number, toonSize: number): number {
    // Rough approximation: 4 characters per token on average
    const jsonTokens = jsonSize / 4;
    const toonTokens = toonSize / 4;
    return ((jsonTokens - toonTokens) / jsonTokens) * 100;
  }

  /**
   * Generate export ID
   */
  private generateExportId(): string {
    return `toon_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate filename
   */
  private generateFilename(exportId: string): string {
    const date = new Date().toISOString().split('T')[0];
    return `analytics_${date}_${exportId}.toon`;
  }
}

/**
 * ToonFormat decoder for parsing exported data
 */
export class ToonFormatDecoder {
  /**
   * Decode ToonFormat string to JSON
   */
  async decode(toonContent: string): Promise<any> {
    const lines = toonContent.split('\n');
    const data: any[] = [];
    let fields: string[] = [];
    let delimiter = ',';
    let inDataSection = false;

    for (const line of lines) {
      const trimmed = line.trim();

      // Skip comments and empty lines
      if (trimmed.startsWith('#') || trimmed === '') {
        continue;
      }

      // Parse metadata
      if (trimmed.includes(':') && !inDataSection) {
        const [key, value] = trimmed.split(':').map(s => s.trim());
        if (key === 'delimiter') {
          delimiter = this.getDelimiterChar(value);
        }
        continue;
      }

      // Parse data header
      if (trimmed.startsWith('data[')) {
        inDataSection = true;
        const match = trimmed.match(/data\[\d+\]\{([^}]+)\}:/);
        if (match) {
          fields = match[1].split(delimiter).map(f => f.trim());
        }
        continue;
      }

      // Parse data rows
      if (inDataSection && trimmed) {
        const values = trimmed.split(delimiter).map(v => v.trim().replace(/^"|"$/g, ''));
        const record: any = {};
        
        for (let i = 0; i < fields.length; i++) {
          if (i < values.length && values[i] !== '') {
            this.setNestedValue(record, fields[i], this.parseValue(values[i]));
          }
        }
        
        data.push(record);
      }
    }

    return { data };
  }

  /**
   * Set nested value using dot notation
   */
  private setNestedValue(obj: any, path: string, value: any): void {
    const keys = path.split('.');
    let current = obj;
    
    for (let i = 0; i < keys.length - 1; i++) {
      const key = keys[i];
      if (!(key in current)) {
        current[key] = {};
      }
      current = current[key];
    }
    
    current[keys[keys.length - 1]] = value;
  }

  /**
   * Parse a value from string
   */
  private parseValue(value: string): any {
    if (value === 'true') return true;
    if (value === 'false') return false;
    if (value === '') return null;
    
    const num = Number(value);
    if (!isNaN(num)) return num;
    
    if (value.startsWith('[') && value.endsWith(']')) {
      return value.slice(1, -1).split(',').map(v => this.parseValue(v.trim()));
    }
    
    if (value.startsWith('{') && value.endsWith('}')) {
      try {
        return JSON.parse(value);
      } catch {
        return value;
      }
    }
    
    return value;
  }

  /**
   * Get delimiter character from string
   */
  private getDelimiterChar(delimiter: string): string {
    switch (delimiter.toLowerCase()) {
      case 'tab': return '\t';
      case 'pipe': return '|';
      default: return ',';
    }
  }

  /**
   * Validate ToonFormat export
   */
  static validate(toonContent: string): { valid: boolean; error?: string } {
    try {
      const lines = toonContent.split('\n');
      let hasDataHeader = false;
      let hasDataRows = false;

      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed.startsWith('data[')) {
          hasDataHeader = true;
        }
        if (hasDataHeader && trimmed && !trimmed.startsWith('data[') && !trimmed.startsWith('#')) {
          hasDataRows = true;
        }
      }

      if (!hasDataHeader) {
        return { valid: false, error: 'Missing data header' };
      }

      return { valid: true };
    } catch (error) {
      return { valid: false, error: error instanceof Error ? error.message : 'Invalid ToonFormat' };
    }
  }
}

// Export singleton instance
export const toonFormatExporter = new ToonFormatExporter();
export const toonFormatDecoder = new ToonFormatDecoder();
