/**
 * Export Data Sanitization for Privacy
 * 
 * Handles data sanitization to remove or mask sensitive information in exports
 */

import { QueryResultRow } from '../../api/analytics/types';

export interface PrivacyRule {
  field: string;
  action: 'remove' | 'mask' | 'hash' | 'truncate';
  options?: {
    maskChar?: string;
    maskLength?: number;
    hashAlgorithm?: 'sha256' | 'md5';
    truncateLength?: number;
    preservePrefix?: number;
  };
}

export interface PrivacyConfig {
  enabled: boolean;
  rules: PrivacyRule[];
  defaultAction: 'remove' | 'mask';
  piiFields: string[];
  sensitivePatterns: RegExp[];
}

/**
 * Data sanitization for privacy
 */
export class DataSanitizer {
  private defaultConfig: PrivacyConfig = {
    enabled: true,
    rules: [],
    defaultAction: 'mask',
    piiFields: [
      'email',
      'phone',
      'ssn',
      'creditCard',
      'apiKey',
      'token',
      'password',
      'secret',
      'address',
      'fullName'
    ],
    sensitivePatterns: [
      /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g, // Email
      /\b\d{3}-\d{2}-\d{4}\b/g, // SSN
      /\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b/g, // Credit card
      /\b[A-Z]{2}\d{2}[A-Z0-9]{4}\d{10}\b/g // IBAN
    ]
  };

  /**
   * Sanitize data according to privacy rules
   */
  sanitize(data: QueryResultRow[], config: Partial<PrivacyConfig> = {}): QueryResultRow[] {
    const privacyConfig = { ...this.defaultConfig, ...config };

    if (!privacyConfig.enabled) {
      return data;
    }

    return data.map(row => this.sanitizeRow(row, privacyConfig));
  }

  /**
   * Sanitize a single row
   */
  private sanitizeRow(row: QueryResultRow, config: PrivacyConfig): QueryResultRow {
    const sanitized: QueryResultRow = {
      timestamp: row.timestamp,
      dimensions: {},
      metrics: {}
    };

    // Sanitize dimensions
    if (row.dimensions) {
      for (const [key, value] of Object.entries(row.dimensions)) {
        sanitized.dimensions![key] = this.sanitizeField(key, value, config);
      }
    }

    // Sanitize metrics (metrics are usually numbers, but we check anyway)
    if (row.metrics) {
      for (const [key, value] of Object.entries(row.metrics)) {
        sanitized.metrics![key] = this.sanitizeField(key, value, config);
      }
    }

    return sanitized;
  }

  /**
   * Sanitize a single field
   */
  private sanitizeField(fieldName: string, value: any, config: PrivacyConfig): any {
    // Check if field has a specific rule
    const specificRule = config.rules.find(rule => rule.field === fieldName);
    if (specificRule) {
      return this.applyAction(value, specificRule.action, specificRule.options);
    }

    // Check if field is a PII field
    if (config.piiFields.includes(fieldName)) {
      return this.applyAction(value, config.defaultAction);
    }

    // Check if value matches sensitive patterns
    if (typeof value === 'string') {
      for (const pattern of config.sensitivePatterns) {
        if (pattern.test(value)) {
          return this.applyAction(value, config.defaultAction);
        }
      }
    }

    // Return value as-is if no rules apply
    return value;
  }

  /**
   * Apply sanitization action
   */
  private applyAction(value: any, action: PrivacyRule['action'], options?: PrivacyRule['options']): any {
    if (value === null || value === undefined) {
      return value;
    }

    switch (action) {
      case 'remove':
        return null;
      
      case 'mask':
        return this.maskValue(value, options);
      
      case 'hash':
        return this.hashValue(value, options);
      
      case 'truncate':
        return this.truncateValue(value, options);
      
      default:
        return value;
    }
  }

  /**
   * Mask a value
   */
  private maskValue(value: any, options?: PrivacyRule['options']): string {
    const strValue = String(value);
    const maskChar = options?.maskChar || '*';
    const maskLength = options?.maskLength || 4;
    const preservePrefix = options?.preservePrefix || 0;

    if (strValue.length <= preservePrefix) {
      return maskChar.repeat(maskLength);
    }

    const prefix = strValue.slice(0, preservePrefix);
    const masked = maskChar.repeat(Math.min(maskLength, strValue.length - preservePrefix));

    return prefix + masked;
  }

  /**
   * Hash a value
   */
  private hashValue(value: any, options?: PrivacyRule['options']): string {
    const strValue = String(value);
    const algorithm = options?.hashAlgorithm || 'sha256';

    // In production, use crypto.subtle or a crypto library
    // For now, return a simple hash placeholder
    const hash = this.simpleHash(strValue);
    return `${algorithm}:${hash}`;
  }

  /**
   * Simple hash function (placeholder - use crypto in production)
   */
  private simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash).toString(16);
  }

  /**
   * Truncate a value
   */
  private truncateValue(value: any, options?: PrivacyRule['options']): string {
    const strValue = String(value);
    const truncateLength = options?.truncateLength || 10;
    const preservePrefix = options?.preservePrefix || 0;

    if (strValue.length <= truncateLength) {
      return strValue;
    }

    if (preservePrefix > 0) {
      return strValue.slice(0, preservePrefix) + '...';
    }

    return strValue.slice(0, truncateLength) + '...';
  }

  /**
   * Add a privacy rule
   */
  addRule(rule: PrivacyRule, config: PrivacyConfig): PrivacyConfig {
    return {
      ...config,
      rules: [...config.rules, rule]
    };
  }

  /**
   * Remove a privacy rule
   */
  removeRule(fieldName: string, config: PrivacyConfig): PrivacyConfig {
    return {
      ...config,
      rules: config.rules.filter(rule => rule.field !== fieldName)
    };
  }

  /**
   * Detect potentially sensitive fields in data
   */
  detectSensitiveFields(data: QueryResultRow[]): string[] {
    const sensitiveFields = new Set<string>();

    for (const row of data) {
      if (row.dimensions) {
        for (const [key, value] of Object.entries(row.dimensions)) {
          if (this.isSensitiveField(key, value)) {
            sensitiveFields.add(key);
          }
        }
      }

      if (row.metrics) {
        for (const [key, value] of Object.entries(row.metrics)) {
          if (this.isSensitiveField(key, value)) {
            sensitiveFields.add(key);
          }
        }
      }
    }

    return Array.from(sensitiveFields);
  }

  /**
   * Check if a field is potentially sensitive
   */
  private isSensitiveField(fieldName: string, value: any): boolean {
    // Check field name
    const lowerFieldName = fieldName.toLowerCase();
    if (this.defaultConfig.piiFields.some(pii => lowerFieldName.includes(pii.toLowerCase()))) {
      return true;
    }

    // Check value patterns
    if (typeof value === 'string') {
      for (const pattern of this.defaultConfig.sensitivePatterns) {
        if (pattern.test(value)) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Generate privacy report
   */
  generateReport(data: QueryResultRow[], config: PrivacyConfig): {
    totalRecords: number;
    sensitiveFields: string[];
    appliedRules: string[];
    sampleSanitizations: Array<{ field: string; original: string; sanitized: string }>;
  } {
    const sensitiveFields = this.detectSensitiveFields(data);
    const sampleRow = data[0];

    const sampleSanitizations: Array<{ field: string; original: string; sanitized: string }> = [];

    if (sampleRow) {
      if (sampleRow.dimensions) {
        for (const [key, value] of Object.entries(sampleRow.dimensions)) {
          const sanitized = this.sanitizeField(key, value, config);
          if (sanitized !== value) {
            sampleSanitizations.push({
              field: key,
              original: String(value),
              sanitized: String(sanitized)
            });
          }
        }
      }
    }

    return {
      totalRecords: data.length,
      sensitiveFields,
      appliedRules: config.rules.map(rule => `${rule.field}: ${rule.action}`),
      sampleSanitizations: sampleSanitizations.slice(0, 10) // Limit to 10 samples
    };
  }
}

// Export singleton instance
export const dataSanitizer = new DataSanitizer();