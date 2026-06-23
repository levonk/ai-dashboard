/**
 * Tests for ToonFormat Compression Utilities
 */

import { ToonFormatOptimizer, ToonFormatTokenEstimator, CompressionOptions, FieldAnalysis } from '../toonformat-optimizer';

describe('ToonFormatOptimizer', () => {
  let optimizer: ToonFormatOptimizer;

  beforeEach(() => {
    optimizer = new ToonFormatOptimizer();
  });

  describe('analyzeData', () => {
    it('should analyze empty data', () => {
      const result = optimizer.analyzeData([]);
      expect(result).toHaveLength(0);
    });

    it('should analyze simple data', () => {
      const data = [
        { timestamp: '2024-01-01', value: 100 },
        { timestamp: '2024-01-02', value: 200 }
      ];

      const result = optimizer.analyzeData(data);

      expect(result.length).toBeGreaterThan(0);
      expect(result.every(r => r.fieldName)).toBe(true);
      expect(result.every(r => typeof r.uniqueness === 'number')).toBe(true);
      expect(result.every(r => typeof r.redundancy === 'number')).toBe(true);
    });

    it('should detect redundant fields', () => {
      const data = [
        { constant: 'same', value: 1 },
        { constant: 'same', value: 2 },
        { constant: 'same', value: 3 }
      ];

      const result = optimizer.analyzeData(data);
      const constantField = result.find(r => r.fieldName === 'constant');

      expect(constantField).toBeDefined();
      expect(constantField?.redundancy).toBeCloseTo(1, 1);
      expect(constantField?.recommendation).toBe('remove');
    });

    it('should detect unique fields', () => {
      const data = [
        { unique: 'a', value: 1 },
        { unique: 'b', value: 2 },
        { unique: 'c', value: 3 }
      ];

      const result = optimizer.analyzeData(data);
      const uniqueField = result.find(r => r.fieldName === 'unique');

      expect(uniqueField).toBeDefined();
      expect(uniqueField?.uniqueness).toBeCloseTo(1, 1);
      expect(uniqueField?.recommendation).toBe('keep');
    });

    it('should analyze nested fields', () => {
      const data = [
        { nested: { field: 'value' } },
        { nested: { field: 'value2' } }
      ];

      const result = optimizer.analyzeData(data);
      const nestedField = result.find(r => r.fieldName === 'nested.field');

      expect(nestedField).toBeDefined();
    });
  });

  describe('optimizeData', () => {
    it('should optimize data with default options', () => {
      const data = [
        { timestamp: '2024-01-01T00:00:00Z', value: 100.123456 },
        { timestamp: '2024-01-02T00:00:00Z', value: 200.789012 }
      ];

      const result = optimizer.optimizeData(data);

      expect(result).toHaveLength(data.length);
      expect(result[0]).toHaveProperty('timestamp');
      expect(result[0]).toHaveProperty('value');
    });

    it('should remove redundant fields when enabled', () => {
      const data = [
        { constant: 'same', value: 1 },
        { constant: 'same', value: 2 }
      ];

      const result = optimizer.optimizeData(data, { removeRedundantFields: true });

      expect(result[0]).not.toHaveProperty('constant');
    });

    it('should compress numbers when enabled', () => {
      const data = [
        { value: 100.123456789 },
        { value: 200.987654321 }
      ];

      const result = optimizer.optimizeData(data, { compressNumbers: true });

      expect(result[0].value).toBe(100.12);
      expect(result[1].value).toBe(200.99);
    });

    it('should shorten timestamps when enabled', () => {
      const data = [
        { timestamp: '2024-01-01T00:00:00Z' }
      ];

      const result = optimizer.optimizeData(data, { shortenTimestamps: true });

      expect(result[0].timestamp).not.toBe('2024-01-01T00:00:00Z');
      expect(typeof result[0].timestamp).toBe('string');
    });

    it('should truncate long strings when max length is set', () => {
      const data = [
        { longField: 'a'.repeat(100) }
      ];

      const result = optimizer.optimizeData(data, { maxFieldLength: 50 });

      expect(result[0].longField.length).toBeLessThanOrEqual(53); // 50 + '...'
      expect(result[0].longField).toContain('...');
    });
  });

  describe('compressWithAnalysis', () => {
    it('should compress data and provide analysis', async () => {
      const data = [
        { timestamp: '2024-01-01T00:00:00Z', value: 100 },
        { timestamp: '2024-01-02T00:00:00Z', value: 200 }
      ];

      const result = await optimizer.compressWithAnalysis(data);

      expect(result.toonContent).toBeDefined();
      expect(result.analysis).toBeDefined();
      expect(result.analysis.originalSize).toBeGreaterThan(0);
      expect(result.analysis.compressedSize).toBeGreaterThan(0);
      expect(result.analysis.compressionRatio).toBeGreaterThanOrEqual(0);
      expect(result.analysis.tokenSavings).toBeGreaterThanOrEqual(0);
    });

    it('should provide field analysis in results', async () => {
      const data = [
        { constant: 'same', value: 1 },
        { constant: 'same', value: 2 }
      ];

      const result = await optimizer.compressWithAnalysis(data, { removeRedundantFields: true });

      expect(result.analysis.fieldsRemoved).toContain('constant');
    });

    it('should provide recommendations', async () => {
      const data = [
        { timestamp: '2024-01-01T00:00:00Z', value: 100 }
      ];

      const result = await optimizer.compressWithAnalysis(data);

      expect(result.analysis.recommendations).toBeInstanceOf(Array);
    });
  });

  describe('getFieldAbbreviation', () => {
    it('should return abbreviation for known fields', () => {
      expect(optimizer.getFieldAbbreviation('timestamp')).toBe('ts');
      expect(optimizer.getFieldAbbreviation('dimensions')).toBe('dims');
      expect(optimizer.getFieldAbbreviation('tokens')).toBe('tok');
    });

    it('should return original name for unknown fields', () => {
      expect(optimizer.getFieldAbbreviation('unknown_field')).toBe('unknown_field');
    });

    it('should be case insensitive', () => {
      expect(optimizer.getFieldAbbreviation('Timestamp')).toBe('ts');
      expect(optimizer.getFieldAbbreviation('TIMESTAMP')).toBe('ts');
    });
  });

  describe('applyFieldAbbreviations', () => {
    it('should apply abbreviations to data', () => {
      const data = [
        { timestamp: '2024-01-01', dimensions: { model: 'gpt-4' } }
      ];

      const result = optimizer.applyFieldAbbreviations(data);

      expect(result[0]).toHaveProperty('ts');
      expect(result[0]).toHaveProperty('dims');
      expect(result[0]).not.toHaveProperty('timestamp');
      expect(result[0]).not.toHaveProperty('dimensions');
    });
  });
});

describe('ToonFormatTokenEstimator', () => {
  let estimator: ToonFormatTokenEstimator;

  beforeEach(() => {
    estimator = new ToonFormatTokenEstimator();
  });

  describe('estimateTokens', () => {
    it('should estimate tokens for content', () => {
      const content = 'data[1]{timestamp,value}:\n  2024-01-01,100';
      const result = estimator.estimateTokens(content);

      expect(result).toBeGreaterThan(0);
      expect(typeof result).toBe('number');
    });

    it('should return 0 for empty content', () => {
      const result = estimator.estimateTokens('');
      expect(result).toBe(0);
    });

    it('should scale with content length', () => {
      const shortContent = 'data[1]{value}:\n  100';
      const longContent = 'data[10]{value}:\n  100\n  200\n  300\n  400\n  500\n  600\n  700\n  800\n  900\n  1000';

      const shortTokens = estimator.estimateTokens(shortContent);
      const longTokens = estimator.estimateTokens(longContent);

      expect(longTokens).toBeGreaterThan(shortTokens);
    });
  });

  describe('compareFormats', () => {
    it('should compare JSON and ToonFormat token usage', () => {
      const data = [
        { timestamp: '2024-01-01', value: 100 },
        { timestamp: '2024-01-02', value: 200 }
      ];

      const result = estimator.compareFormats(data);

      expect(result.jsonTokens).toBeGreaterThan(0);
      expect(result.toonTokens).toBeGreaterThan(0);
      expect(result.savings).toBeGreaterThan(0);
      expect(result.savingsPercent).toBeGreaterThan(0);
    });

    it('should show ToonFormat as more efficient', () => {
      const data = [
        { timestamp: '2024-01-01', value: 100 }
      ];

      const result = estimator.compareFormats(data);

      expect(result.toonTokens).toBeLessThan(result.jsonTokens);
    });

    it('should handle empty data', () => {
      const result = estimator.compareFormats([]);

      expect(result.jsonTokens).toBe(0);
      expect(result.toonTokens).toBe(0);
      expect(result.savings).toBe(0);
      expect(result.savingsPercent).toBe(0);
    });
  });
});
