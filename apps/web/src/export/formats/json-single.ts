/**
 * JSON Single Record Export Format
 * 
 * Optimized JSON format for single record plaintext protocol exchanges
 * Designed for AI agent interactions where individual records are exchanged
 */

import { QueryResultRow } from '../../api/analytics/types';
import { z } from 'zod';

export interface JSONSingleExportOptions {
  includeMetadata: boolean;
  prettyPrint: boolean;
  flattenStructure: boolean;
  timestampFormat: 'iso' | 'unix' | 'readable';
}

export interface JSONSingleExportResult {
  content: string;
  filename: string;
  size: number;
  metadata: {
    exportedAt: string;
    format: 'json-single';
    hasMetadata: boolean;
    isFlattened: boolean;
  };
}

// Schema for single record export
const SingleRecordSchema = z.object({
  timestamp: z.string(),
  dimensions: z.record(z.any()).optional(),
  metrics: z.record(z.any()).optional()
});

// Schema for export with metadata
const SingleExportSchema = z.object({
  metadata: z.object({
    exportedAt: z.string(),
    format: z.literal('json-single'),
    version: z.string().default('1.0'),
    recordId: z.string()
  }).optional(),
  record: SingleRecordSchema
});

/**
 * JSON Single Record exporter for plaintext protocol exchanges
 */
export class JSONSingleExporter {
  private defaultOptions: JSONSingleExportOptions = {
    includeMetadata: false,
    prettyPrint: false,
    flattenStructure: false,
    timestampFormat: 'iso'
  };

  /**
   * Export a single record to JSON format
   */
  async export(
    record: QueryResultRow,
    options: Partial<JSONSingleExportOptions> = {}
  ): Promise<JSONSingleExportResult> {
    const config = { ...this.defaultOptions, ...options };
    const recordId = this.generateRecordId();

    // Process the record
    const processedRecord = this.processRecord(record, config);

    // Build export object
    const exportObj: any = {
      record: processedRecord
    };

    // Add metadata if requested
    if (config.includeMetadata) {
      exportObj.metadata = {
        exportedAt: new Date().toISOString(),
        format: 'json-single',
        version: '1.0',
        recordId
      };
    }

    // Validate export structure
    const validationResult = SingleExportSchema.safeParse(exportObj);
    if (!validationResult.success) {
      throw new Error(`JSON single export validation failed: ${validationResult.error.message}`);
    }

    // Serialize to JSON
    const jsonContent = config.prettyPrint 
      ? JSON.stringify(exportObj, null, 2)
      : JSON.stringify(exportObj);

    const filename = this.generateFilename(recordId);

    return {
      content: jsonContent,
      filename,
      size: jsonContent.length,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'json-single',
        hasMetadata: config.includeMetadata,
        isFlattened: config.flattenStructure
      }
    };
  }

  /**
   * Export multiple records as individual JSON objects (one per line)
   */
  async exportMultiple(
    records: QueryResultRow[],
    options: Partial<JSONSingleExportOptions> = {}
  ): Promise<JSONSingleExportResult> {
    const config = { ...this.defaultOptions, ...options };
    const recordId = this.generateRecordId();

    // Process each record and create JSON lines
    const jsonLines = records.map(record => {
      const processedRecord = this.processRecord(record, config);
      return JSON.stringify(processedRecord);
    });

    // Join with newlines for JSONL format
    const jsonContent = jsonLines.join('\n');

    const filename = this.generateFilename(recordId);

    return {
      content: jsonContent,
      filename,
      size: jsonContent.length,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'json-single',
        hasMetadata: false,
        isFlattened: config.flattenStructure
      }
    };
  }

  /**
   * Process record according to options
   */
  private processRecord(record: QueryResultRow, config: JSONSingleExportOptions): any {
    let processed: any = {
      timestamp: this.formatTimestamp(record.timestamp, config.timestampFormat)
    };

    if (record.dimensions) {
      if (config.flattenStructure) {
        // Flatten dimensions into top-level properties
        Object.entries(record.dimensions).forEach(([key, value]) => {
          processed[`dim_${key}`] = value;
        });
      } else {
        processed.dimensions = record.dimensions;
      }
    }

    if (record.metrics) {
      if (config.flattenStructure) {
        // Flatten metrics into top-level properties
        Object.entries(record.metrics).forEach(([key, value]) => {
          processed[`metric_${key}`] = value;
        });
      } else {
        processed.metrics = record.metrics;
      }
    }

    return processed;
  }

  /**
   * Format timestamp according to specified format
   */
  private formatTimestamp(timestamp: string, format: string): string {
    const date = new Date(timestamp);

    switch (format) {
      case 'unix':
        return Math.floor(date.getTime() / 1000).toString();
      case 'readable':
        return date.toLocaleString();
      case 'iso':
      default:
        return date.toISOString();
    }
  }

  /**
   * Generate record ID
   */
  private generateRecordId(): string {
    return `record_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate filename
   */
  private generateFilename(recordId: string): string {
    const date = new Date().toISOString().split('T')[0];
    return `record_${date}_${recordId}.json`;
  }

  /**
   * Validate JSON single export
   */
  static validate(jsonContent: string): { valid: boolean; error?: string } {
    try {
      const parsed = JSON.parse(jsonContent);
      const result = SingleExportSchema.safeParse(parsed);
      
      if (!result.success) {
        return { valid: false, error: result.error.message };
      }
      
      return { valid: true };
    } catch (error) {
      return { valid: false, error: error instanceof Error ? error.message : 'Invalid JSON' };
    }
  }
}

/**
 * JSON Single Record decoder for parsing exported data
 */
export class JSONSingleDecoder {
  /**
   * Decode a single JSON record
   */
  async decode(jsonContent: string): Promise<QueryResultRow> {
    try {
      const parsed = JSON.parse(jsonContent);
      
      // Handle both wrapped and unwrapped formats
      const record = parsed.record || parsed;
      
      return this.parseRecord(record);
    } catch (error) {
      throw new Error(`Failed to decode JSON single record: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Decode multiple JSON lines (JSONL format)
   */
  async decodeMultiple(jsonContent: string): Promise<QueryResultRow[]> {
    try {
      const lines = jsonContent.trim().split('\n');
      const records: QueryResultRow[] = [];

      for (const line of lines) {
        if (line.trim()) {
          const record = await this.decode(line);
          records.push(record);
        }
      }

      return records;
    } catch (error) {
      throw new Error(`Failed to decode JSON multiple records: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Parse a record object
   */
  private parseRecord(obj: any): QueryResultRow {
    const record: QueryResultRow = {
      timestamp: obj.timestamp || new Date().toISOString()
    };

    // Handle flattened structure
    const dimensions: any = {};
    const metrics: any = {};

    for (const [key, value] of Object.entries(obj)) {
      if (key.startsWith('dim_')) {
        dimensions[key.substring(4)] = value;
      } else if (key.startsWith('metric_')) {
        metrics[key.substring(7)] = value;
      } else if (key === 'dimensions' && typeof value === 'object') {
        Object.assign(dimensions, value);
      } else if (key === 'metrics' && typeof value === 'object') {
        Object.assign(metrics, value);
      }
    }

    if (Object.keys(dimensions).length > 0) {
      record.dimensions = dimensions;
    }

    if (Object.keys(metrics).length > 0) {
      record.metrics = metrics;
    }

    return record;
  }
}

// Export singleton instances
export const jsonSingleExporter = new JSONSingleExporter();
export const jsonSingleDecoder = new JSONSingleDecoder();
