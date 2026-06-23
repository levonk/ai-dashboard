/**
 * ToonFormat Compression Utilities
 * 
 * Advanced optimization utilities for ToonFormat to maximize token efficiency
 * Includes field analysis, compression strategies, and token estimation
 */

import { ToonFormatEncoder, ToonFormatDecoder } from './formats/toonformat';

/**
 * Compression strategy options
 */
export interface CompressionOptions {
  aggressiveFolding: boolean;
  removeRedundantFields: boolean;
  compressNumbers: boolean;
  shortenTimestamps: boolean;
  useAbbreviations: boolean;
  maxFieldLength: number;
}

/**
 * Compression analysis result
 */
export interface CompressionAnalysis {
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  tokenSavings: number;
  fieldsRemoved: string[];
  fieldsCompressed: string[];
  recommendations: string[];
}

/**
 * Field analysis result
 */
export interface FieldAnalysis {
  fieldName: string;
  uniqueness: number; // 0-1, higher = more unique values
  redundancy: number; // 0-1, higher = more redundant
  compressibility: 'high' | 'medium' | 'low';
  recommendation: 'keep' | 'compress' | 'remove';
}

/**
 * Common field abbreviations
 */
const FIELD_ABBREVIATIONS: Record<string, string> = {
  'timestamp': 'ts',
  'dimensions': 'dims',
  'metrics': 'metrics',
  'provider': 'prov',
  'model': 'model',
  'tokens': 'tok',
  'cost': 'cost',
  'duration': 'dur',
  'latency': 'lat',
  'request_id': 'req_id',
  'session_id': 'sess_id',
  'user_id': 'user_id',
  'organization': 'org',
  'project': 'proj',
  'environment': 'env',
  'region': 'reg',
  'status': 'status',
  'error': 'err',
  'message': 'msg',
  'response': 'resp',
  'request': 'req'
};

/**
 * ToonFormat optimizer for advanced compression
 */
export class ToonFormatOptimizer {
  private defaultOptions: CompressionOptions = {
    aggressiveFolding: true,
    removeRedundantFields: false,
    compressNumbers: true,
    shortenTimestamps: true,
    useAbbreviations: true,
    maxFieldLength: 50
  };

  /**
   * Analyze data for compression opportunities
   */
  analyzeData(data: any[]): FieldAnalysis[] {
    if (data.length === 0) return [];

    const fields = this.extractAllFields(data);
    const analyses: FieldAnalysis[] = [];

    for (const field of fields) {
      const analysis = this.analyzeField(data, field);
      analyses.push(analysis);
    }

    return analyses.sort((a, b) => b.redundancy - a.redundancy);
  }

  /**
   * Extract all unique fields from data
   */
  private extractAllFields(data: any[]): string[] {
    const fieldSet = new Set<string>();

    for (const record of data) {
      this.extractFieldsFromObject(record, '', fieldSet);
    }

    return Array.from(fieldSet);
  }

  /**
   * Recursively extract fields from object
   */
  private extractFieldsFromObject(obj: any, prefix: string, fieldSet: Set<string>): void {
    if (typeof obj !== 'object' || obj === null) return;

    for (const [key, value] of Object.entries(obj)) {
      const fieldPath = prefix ? `${prefix}.${key}` : key;
      
      if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        this.extractFieldsFromObject(value, fieldPath, fieldSet);
      } else {
        fieldSet.add(fieldPath);
      }
    }
  }

  /**
   * Analyze a specific field for compression potential
   */
  private analyzeField(data: any[], field: string): FieldAnalysis {
    const values = data.map(record => this.getNestedValue(record, field)).filter(v => v !== null && v !== undefined);
    
    if (values.length === 0) {
      return {
        fieldName: field,
        uniqueness: 0,
        redundancy: 1,
        compressibility: 'high',
        recommendation: 'remove'
      };
    }

    // Calculate uniqueness (ratio of unique values to total values)
    const uniqueValues = new Set(values);
    const uniqueness = uniqueValues.size / values.length;

    // Calculate redundancy (inverse of uniqueness)
    const redundancy = 1 - uniqueness;

    // Determine compressibility
    let compressibility: 'high' | 'medium' | 'low';
    if (redundancy > 0.7) {
      compressibility = 'high';
    } else if (redundancy > 0.3) {
      compressibility = 'medium';
    } else {
      compressibility = 'low';
    }

    // Determine recommendation
    let recommendation: 'keep' | 'compress' | 'remove';
    if (redundancy > 0.9) {
      recommendation = 'remove';
    } else if (redundancy > 0.5) {
      recommendation = 'compress';
    } else {
      recommendation = 'keep';
    }

    return {
      fieldName: field,
      uniqueness,
      redundancy,
      compressibility,
      recommendation
    };
  }

  /**
   * Get nested value from object
   */
  private getNestedValue(obj: any, path: string): any {
    const keys = path.split('.');
    let current = obj;
    
    for (const key of keys) {
      if (current === null || current === undefined) return null;
      current = current[key];
    }
    
    return current;
  }

  /**
   * Optimize data using compression strategies
   */
  optimizeData(data: any[], options: Partial<CompressionOptions> = {}): any[] {
    const config = { ...this.defaultOptions, ...options };
    
    // Analyze fields
    const analyses = this.analyzeData(data);
    
    // Apply optimizations
    let optimizedData = data.map(record => this.optimizeRecord(record, analyses, config));

    return optimizedData;
  }

  /**
   * Optimize a single record
   */
  private optimizeRecord(record: any, analyses: FieldAnalysis[], options: CompressionOptions): any {
    const optimized: any = {};

    for (const analysis of analyses) {
      const value = this.getNestedValue(record, analysis.fieldName);

      // Skip if recommendation is to remove
      if (analysis.recommendation === 'remove' && options.removeRedundantFields) {
        continue;
      }

      // Apply compression if recommended
      if (analysis.recommendation === 'compress' || options.aggressiveFolding) {
        const compressed = this.compressValue(value, analysis.fieldName, options);
        this.setNestedValue(optimized, analysis.fieldName, compressed);
      } else {
        this.setNestedValue(optimized, analysis.fieldName, value);
      }
    }

    return optimized;
  }

  /**
   * Compress a value based on field name and options
   */
  private compressValue(value: any, fieldName: string, options: CompressionOptions): any {
    if (value === null || value === undefined) return value;

    // Apply field abbreviations
    if (options.useAbbreviations && typeof value === 'string') {
      // This would be applied to field names, not values
      // Kept for future expansion
    }

    // Compress numbers
    if (options.compressNumbers && typeof value === 'number') {
      return this.compressNumber(value);
    }

    // Shorten timestamps
    if (options.shortenTimestamps && this.isTimestampField(fieldName) && typeof value === 'string') {
      return this.shortenTimestamp(value);
    }

    // Truncate long strings
    if (options.maxFieldLength && typeof value === 'string' && value.length > options.maxFieldLength) {
      return value.substring(0, options.maxFieldLength) + '...';
    }

    return value;
  }

  /**
   * Compress a number (remove unnecessary precision)
   */
  private compressNumber(value: number): number {
    if (Number.isInteger(value)) {
      return value;
    }
    
    // Round to 2 decimal places for typical metrics
    return Math.round(value * 100) / 100;
  }

  /**
   * Check if field is a timestamp field
   */
  private isTimestampField(fieldName: string): boolean {
    return fieldName.toLowerCase().includes('timestamp') || 
           fieldName.toLowerCase().includes('time') ||
           fieldName.toLowerCase().includes('date');
  }

  /**
   * Shorten timestamp to more compact format
   */
  private shortenTimestamp(timestamp: string): string {
    try {
      const date = new Date(timestamp);
      // Return Unix timestamp for compactness
      return Math.floor(date.getTime() / 1000).toString();
    } catch {
      return timestamp;
    }
  }

  /**
   * Set nested value in object
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
   * Compress and encode data with analysis
   */
  async compressWithAnalysis(
    data: any[],
    options: Partial<CompressionOptions> = {}
  ): Promise<{
    toonContent: string;
    analysis: CompressionAnalysis;
  }> {
    const config = { ...this.defaultOptions, ...options };
    
    // Get original size
    const originalSize = JSON.stringify(data).length;

    // Optimize data
    const optimizedData = this.optimizeData(data, config);

    // Encode to ToonFormat
    const encoder = new ToonFormatEncoder();
    const toonContent = await encoder.encode(optimizedData);

    // Calculate compression metrics
    const compressedSize = toonContent.length;
    const compressionRatio = (originalSize - compressedSize) / originalSize;
    const tokenSavings = this.estimateTokenSavings(originalSize, compressedSize);

    // Analyze what was done
    const fieldAnalyses = this.analyzeData(data);
    const fieldsRemoved = fieldAnalyses
      .filter(a => a.recommendation === 'remove')
      .map(a => a.fieldName);
    const fieldsCompressed = fieldAnalyses
      .filter(a => a.recommendation === 'compress')
      .map(a => a.fieldName);

    // Generate recommendations
    const recommendations = this.generateRecommendations(fieldAnalyses);

    return {
      toonContent,
      analysis: {
        originalSize,
        compressedSize,
        compressionRatio,
        tokenSavings,
        fieldsRemoved,
        fieldsCompressed,
        recommendations
      }
    };
  }

  /**
   * Estimate token savings (rough approximation)
   */
  private estimateTokenSavings(originalSize: number, compressedSize: number): number {
    // Rough approximation: 4 characters per token on average
    const originalTokens = originalSize / 4;
    const compressedTokens = compressedSize / 4;
    return ((originalTokens - compressedTokens) / originalTokens) * 100;
  }

  /**
   * Generate optimization recommendations
   */
  private generateRecommendations(analyses: FieldAnalysis[]): string[] {
    const recommendations: string[] = [];

    const highRedundancy = analyses.filter(a => a.redundancy > 0.8);
    if (highRedundancy.length > 0) {
      recommendations.push(
        `Consider removing ${highRedundancy.length} highly redundant fields: ${highRedundancy.map(a => a.fieldName).slice(0, 3).join(', ')}`
      );
    }

    const lowUniqueness = analyses.filter(a => a.uniqueness < 0.1);
    if (lowUniqueness.length > 0) {
      recommendations.push(
        `${lowUniqueness.length} fields have very low uniqueness and could be compressed: ${lowUniqueness.map(a => a.fieldName).slice(0, 3).join(', ')}`
      );
    }

    if (analyses.some(a => a.fieldName.includes('timestamp'))) {
      recommendations.push('Consider using Unix timestamps for date fields to save space');
    }

    if (analyses.some(a => a.fieldName.includes('cost') || a.fieldName.includes('tokens'))) {
      recommendations.push('Numeric fields like cost and tokens can be rounded to reduce precision');
    }

    return recommendations;
  }

  /**
   * Get field abbreviation
   */
  getFieldAbbreviation(fieldName: string): string {
    return FIELD_ABBREVIATIONS[fieldName.toLowerCase()] || fieldName;
  }

  /**
   * Apply field abbreviations to data structure
   */
  applyFieldAbbreviations(data: any[]): any[] {
    return data.map(record => {
      const abbreviated: any = {};

      for (const [key, value] of Object.entries(record)) {
        const abbreviatedKey = this.getFieldAbbreviation(key);
        abbreviated[abbreviatedKey] = value;
      }

      return abbreviated;
    });
  }
}

/**
 * Token estimator for ToonFormat content
 */
export class ToonFormatTokenEstimator {
  /**
   * Estimate token count for ToonFormat content
   */
  estimateTokens(toonContent: string): number {
    // Rough estimation based on character count
    // ToonFormat is typically 30-50% more token-efficient than JSON
    const charCount = toonContent.length;
    
    // Approximate: 4 characters per token for English text
    // ToonFormat gets better ratio due to compression
    return Math.ceil(charCount / 4);
  }

  /**
   * Compare token usage between formats
   */
  compareFormats(data: any[]): {
    jsonTokens: number;
    toonTokens: number;
    savings: number;
    savingsPercent: number;
  } {
    const jsonSize = JSON.stringify(data).length;
    const toonEncoder = new ToonFormatEncoder();
    
    // Estimate JSON tokens
    const jsonTokens = Math.ceil(jsonSize / 4);

    // Estimate ToonFormat tokens (would need actual encoding for accurate result)
    // Using 30% compression as estimate
    const toonSize = jsonSize * 0.7;
    const toonTokens = Math.ceil(toonSize / 4);

    const savings = jsonTokens - toonTokens;
    const savingsPercent = (savings / jsonTokens) * 100;

    return {
      jsonTokens,
      toonTokens,
      savings,
      savingsPercent
    };
  }
}

// Export singleton instances
export const toonFormatOptimizer = new ToonFormatOptimizer();
export const toonFormatTokenEstimator = new ToonFormatTokenEstimator();
