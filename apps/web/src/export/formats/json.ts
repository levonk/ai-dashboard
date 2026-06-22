/**
 * JSON Export Format
 * 
 * Handles JSON format data export with schema validation
 */

import { QueryResultRow, AnalyticsQueryResponse } from '../../api/analytics/types';
import { z } from 'zod';

export interface JSONExportOptions {
  includeMetadata: boolean;
  prettyPrint: boolean;
  includeSchema: boolean;
  nullHandling: 'include' | 'omit' | 'string';
  dateFormat: string;
}

export interface JSONExportResult {
  content: string;
  filename: string;
  size: number;
  recordCount: number;
  metadata: {
    exportedAt: string;
    format: 'json';
    hasMetadata: boolean;
    hasSchema: boolean;
    schema?: any;
  };
}

// Schema for export metadata
const ExportMetadataSchema = z.object({
  exportedAt: z.string(),
  recordCount: z.number(),
  format: z.literal('json'),
  version: z.string().default('1.0'),
  exportId: z.string()
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
  schema: z.any().optional(),
  data: z.array(RecordSchema)
});

/**
 * JSON exporter
 */
export class JSONExporter {
  private defaultOptions: JSONExportOptions = {
    includeMetadata: true,
    prettyPrint: true,
    includeSchema: false,
    nullHandling: 'include',
    dateFormat: 'YYYY-MM-DD HH:mm:ss'
  };

  /**
   * Export data to JSON format
   */
  async export(
    data: QueryResultRow[] | AnalyticsQueryResponse,
    options: Partial<JSONExportOptions> = {}
  ): Promise<JSONExportResult> {
    const config = { ...this.defaultOptions, ...options };
    
    const isArray = Array.isArray(data);
    const records = isArray ? data : data.data;
    const exportId = this.generateExportId();

    // Build export object
    const exportObj: any = {
      data: this.processRecords(records, config)
    };

    // Add metadata if requested
    if (config.includeMetadata) {
      exportObj.metadata = {
        exportedAt: new Date().toISOString(),
        recordCount: records.length,
        format: 'json',
        version: '1.0',
        exportId
      };
    }

    // Add schema if requested
    if (config.includeSchema) {
      exportObj.schema = this.generateSchema(records);
    }

    // Validate export structure
    const validationResult = ExportDataSchema.safeParse(exportObj);
    if (!validationResult.success) {
      throw new Error(`JSON export validation failed: ${validationResult.error.message}`);
    }

    // Serialize to JSON
    const jsonContent = config.prettyPrint 
      ? JSON.stringify(exportObj, null, 2)
      : JSON.stringify(exportObj);

    const filename = this.generateFilename(exportId);

    return {
      content: jsonContent,
      filename,
      size: jsonContent.length,
      recordCount: records.length,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'json',
        hasMetadata: config.includeMetadata,
        hasSchema: config.includeSchema,
        schema: config.includeSchema ? exportObj.schema : undefined
      }
    };
  }

  /**
   * Export data with custom field selection
   */
  async exportWithFields(
    data: QueryResultRow[],
    fields: string[],
    options: Partial<JSONExportOptions> = {}
  ): Promise<JSONExportResult> {
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

      return this.handleNulls(selected, config);
    });

    // Build export object
    const exportObj: any = {
      data: processedRecords
    };

    // Add metadata if requested
    if (config.includeMetadata) {
      exportObj.metadata = {
        exportedAt: new Date().toISOString(),
        recordCount: processedRecords.length,
        format: 'json',
        version: '1.0',
        exportId,
        fields
      };
    }

    // Serialize to JSON
    const jsonContent = config.prettyPrint 
      ? JSON.stringify(exportObj, null, 2)
      : JSON.stringify(exportObj);

    const filename = this.generateFilename(exportId);

    return {
      content: jsonContent,
      filename,
      size: jsonContent.length,
      recordCount: processedRecords.length,
      metadata: {
        exportedAt: new Date().toISOString(),
        format: 'json',
        hasMetadata: config.includeMetadata,
        hasSchema: false
      }
    };
  }

  /**
   * Process records according to null handling policy
   */
  private processRecords(records: QueryResultRow[], config: JSONExportOptions): any[] {
    return records.map(row => {
      const processed: any = {
        timestamp: row.timestamp
      };

      if (row.dimensions) {
        processed.dimensions = this.handleNulls(row.dimensions, config);
      }

      if (row.metrics) {
        processed.metrics = this.handleNulls(row.metrics, config);
      }

      return processed;
    });
  }

  /**
   * Handle null values according to policy
   */
  private handleNulls(obj: any, config: JSONExportOptions): any {
    if (obj === null || obj === undefined) {
      return config.nullHandling === 'string' ? 'null' : obj;
    }

    if (Array.isArray(obj)) {
      return obj.map(item => this.handleNulls(item, config));
    }

    if (typeof obj === 'object') {
      const result: any = {};
      for (const [key, value] of Object.entries(obj)) {
        if (value === null || value === undefined) {
          if (config.nullHandling !== 'omit') {
            result[key] = config.nullHandling === 'string' ? 'null' : null;
          }
        } else {
          result[key] = this.handleNulls(value, config);
        }
      }
      return result;
    }

    return obj;
  }

  /**
   * Generate JSON schema from data
   */
  private generateSchema(records: QueryResultRow[]): any {
    if (records.length === 0) {
      return {
        type: 'object',
        properties: {
          timestamp: { type: 'string' },
          dimensions: { type: 'object' },
          metrics: { type: 'object' }
        }
      };
    }

    const properties: any = {
      timestamp: { type: 'string', format: 'date-time' }
    };

    const dimensionFields = new Set<string>();
    const metricFields = new Set<string>();

    for (const record of records) {
      if (record.dimensions) {
        Object.keys(record.dimensions).forEach(key => dimensionFields.add(key));
      }
      if (record.metrics) {
        Object.keys(record.metrics).forEach(key => metricFields.add(key));
      }
    }

    if (dimensionFields.size > 0) {
      properties.dimensions = {
        type: 'object',
        properties: {}
      };
      
      for (const field of dimensionFields) {
        properties.dimensions.properties[field] = { type: ['string', 'number', 'boolean'] };
      }
    }

    if (metricFields.size > 0) {
      properties.metrics = {
        type: 'object',
        properties: {}
      };
      
      for (const field of metricFields) {
        properties.metrics.properties[field] = { type: 'number' };
      }
    }

    return {
      type: 'object',
      properties,
      required: ['timestamp']
    };
  }

  /**
   * Generate export ID
   */
  private generateExportId(): string {
    return `export-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate filename
   */
  private generateFilename(exportId: string): string {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
    return `export-${timestamp}.json`;
  }

  /**
   * Validate JSON export
   */
  static validate(jsonContent: string): { valid: boolean; error?: string } {
    try {
      const parsed = JSON.parse(jsonContent);
      const result = ExportDataSchema.safeParse(parsed);
      
      if (!result.success) {
        return { valid: false, error: result.error.message };
      }
      
      return { valid: true };
    } catch (error) {
      return { valid: false, error: error instanceof Error ? error.message : 'Invalid JSON' };
    }
  }
}

// Export singleton instance
export const jsonExporter = new JSONExporter();