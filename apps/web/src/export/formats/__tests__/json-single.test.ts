/**
 * Tests for JSON Single Record export format
 */

import { JSONSingleExporter, JSONSingleDecoder } from '../json-single';

describe('JSONSingleExporter', () => {
  let exporter: JSONSingleExporter;

  beforeEach(() => {
    exporter = new JSONSingleExporter();
  });

  describe('export', () => {
    it('should export a single record to JSON', async () => {
      const record = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4', provider: 'openai' },
        metrics: { tokens: 100, cost: 0.01 }
      };

      const result = await exporter.export(record);

      expect(result.content).toContain('"timestamp"');
      expect(result.content).toContain('"record"');
      expect(result.metadata.format).toBe('json-single');
      expect(result.metadata.hasMetadata).toBe(false);
    });

    it('should include metadata when requested', async () => {
      const record = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      };

      const result = await exporter.export(record, { includeMetadata: true });

      expect(result.content).toContain('"metadata"');
      expect(result.content).toContain('"format"');
      expect(result.content).toContain('"json-single"');
      expect(result.metadata.hasMetadata).toBe(true);
    });

    it('should flatten structure when requested', async () => {
      const record = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4', provider: 'openai' },
        metrics: { tokens: 100, cost: 0.01 }
      };

      const result = await exporter.export(record, { flattenStructure: true });

      expect(result.content).toContain('"dim_model"');
      expect(result.content).toContain('"dim_provider"');
      expect(result.content).toContain('"metric_tokens"');
      expect(result.content).toContain('"metric_cost"');
      expect(result.metadata.isFlattened).toBe(true);
    });

    it('should format timestamp as ISO by default', async () => {
      const record = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      };

      const result = await exporter.export(record);

      expect(result.content).toContain('2024-01-01T00:00:00');
    });

    it('should format timestamp as unix when requested', async () => {
      const record = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      };

      const result = await exporter.export(record, { timestampFormat: 'unix' });

      expect(result.content).toMatch(/\d{10}/); // Unix timestamp
    });

    it('should format timestamp as readable when requested', async () => {
      const record = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      };

      const result = await exporter.export(record, { timestampFormat: 'readable' });

      expect(result.content).toMatch(/\d{1,2}\/\d{1,2}\/\d{4}/); // Date format
    });

    it('should pretty print when requested', async () => {
      const record = {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      };

      const result = await exporter.export(record, { prettyPrint: true });

      expect(result.content).toContain('\n');
      expect(result.content).toContain('  ');
    });
  });

  describe('exportMultiple', () => {
    it('should export multiple records as JSONL', async () => {
      const records = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 100 }
        },
        {
          timestamp: '2024-01-01T01:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 150 }
        }
      ];

      const result = await exporter.exportMultiple(records);

      const lines = result.content.split('\n');
      expect(lines).toHaveLength(2);
      expect(result.metadata.format).toBe('json-single');
    });

    it('should handle empty array', async () => {
      const result = await exporter.exportMultiple([]);

      expect(result.content).toBe('');
      expect(result.size).toBe(0);
    });
  });
});

describe('JSONSingleDecoder', () => {
  let decoder: JSONSingleDecoder;

  beforeEach(() => {
    decoder = new JSONSingleDecoder();
  });

  describe('decode', () => {
    it('should decode a wrapped JSON record', async () => {
      const jsonContent = JSON.stringify({
        record: {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 100 }
        }
      });

      const result = await decoder.decode(jsonContent);

      expect(result.timestamp).toBe('2024-01-01T00:00:00Z');
      expect(result.dimensions?.model).toBe('gpt-4');
      expect(result.metrics?.tokens).toBe(100);
    });

    it('should decode an unwrapped JSON record', async () => {
      const jsonContent = JSON.stringify({
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      });

      const result = await decoder.decode(jsonContent);

      expect(result.timestamp).toBe('2024-01-01T00:00:00Z');
      expect(result.dimensions?.model).toBe('gpt-4');
      expect(result.metrics?.tokens).toBe(100);
    });

    it('should decode flattened structure', async () => {
      const jsonContent = JSON.stringify({
        timestamp: '2024-01-01T00:00:00Z',
        dim_model: 'gpt-4',
        dim_provider: 'openai',
        metric_tokens: 100,
        metric_cost: 0.01
      });

      const result = await decoder.decode(jsonContent);

      expect(result.dimensions?.model).toBe('gpt-4');
      expect(result.dimensions?.provider).toBe('openai');
      expect(result.metrics?.tokens).toBe(100);
      expect(result.metrics?.cost).toBe(0.01);
    });

    it('should handle missing dimensions and metrics', async () => {
      const jsonContent = JSON.stringify({
        timestamp: '2024-01-01T00:00:00Z'
      });

      const result = await decoder.decode(jsonContent);

      expect(result.timestamp).toBe('2024-01-01T00:00:00Z');
      expect(result.dimensions).toBeUndefined();
      expect(result.metrics).toBeUndefined();
    });
  });

  describe('decodeMultiple', () => {
    it('should decode JSONL format', async () => {
      const jsonContent = `{"timestamp":"2024-01-01T00:00:00Z","dimensions":{"model":"gpt-4"},"metrics":{"tokens":100}}
{"timestamp":"2024-01-01T01:00:00Z","dimensions":{"model":"gpt-4"},"metrics":{"tokens":150}}`;

      const result = await decoder.decodeMultiple(jsonContent);

      expect(result).toHaveLength(2);
      expect(result[0].metrics?.tokens).toBe(100);
      expect(result[1].metrics?.tokens).toBe(150);
    });

    it('should handle empty content', async () => {
      const result = await decoder.decodeMultiple('');

      expect(result).toHaveLength(0);
    });

    it('should skip empty lines', async () => {
      const jsonContent = `{"timestamp":"2024-01-01T00:00:00Z","dimensions":{"model":"gpt-4"},"metrics":{"tokens":100}}

{"timestamp":"2024-01-01T01:00:00Z","dimensions":{"model":"gpt-4"},"metrics":{"tokens":150}}`;

      const result = await decoder.decodeMultiple(jsonContent);

      expect(result).toHaveLength(2);
    });
  });
});

describe('Round-trip encoding/decoding', () => {
  it('should maintain data integrity through encode/decode cycle', async () => {
    const originalRecord = {
      timestamp: '2024-01-01T00:00:00Z',
      dimensions: { model: 'gpt-4', provider: 'openai' },
      metrics: { tokens: 100, cost: 0.01 }
    };

    const exporter = new JSONSingleExporter();
    const decoder = new JSONSingleDecoder();

    const exported = await exporter.export(originalRecord);
    const decoded = await decoder.decode(exported.content);

    expect(decoded.timestamp).toBe(originalRecord.timestamp);
    expect(decoded.dimensions?.model).toBe(originalRecord.dimensions.model);
    expect(decoded.metrics?.tokens).toBe(originalRecord.metrics.tokens);
  });

  it('should maintain data integrity with flattened structure', async () => {
    const originalRecord = {
      timestamp: '2024-01-01T00:00:00Z',
      dimensions: { model: 'gpt-4', provider: 'openai' },
      metrics: { tokens: 100, cost: 0.01 }
    };

    const exporter = new JSONSingleExporter();
    const decoder = new JSONSingleDecoder();

    const exported = await exporter.export(originalRecord, { flattenStructure: true });
    const decoded = await decoder.decode(exported.content);

    expect(decoded.dimensions?.model).toBe(originalRecord.dimensions.model);
    expect(decoded.metrics?.tokens).toBe(originalRecord.metrics.tokens);
  });

  it('should maintain data integrity for multiple records', async () => {
    const originalRecords = [
      {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      },
      {
        timestamp: '2024-01-01T01:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 150 }
      }
    ];

    const exporter = new JSONSingleExporter();
    const decoder = new JSONSingleDecoder();

    const exported = await exporter.exportMultiple(originalRecords);
    const decoded = await decoder.decodeMultiple(exported.content);

    expect(decoded).toHaveLength(originalRecords.length);
    expect(decoded[0].metrics?.tokens).toBe(originalRecords[0].metrics?.tokens);
    expect(decoded[1].metrics?.tokens).toBe(originalRecords[1].metrics?.tokens);
  });
});

describe('Validation', () => {
  it('should validate correct JSON single export', () => {
    const validJson = JSON.stringify({
      record: {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4' },
        metrics: { tokens: 100 }
      }
    });

    const result = JSONSingleExporter.validate(validJson);

    expect(result.valid).toBe(true);
    expect(result.error).toBeUndefined();
  });

  it('should reject invalid JSON', () => {
    const invalidJson = '{invalid json}';

    const result = JSONSingleExporter.validate(invalidJson);

    expect(result.valid).toBe(false);
    expect(result.error).toBeDefined();
  });

  it('should reject JSON with wrong structure', () => {
    const wrongStructure = JSON.stringify({
      wrongField: 'value'
    });

    const result = JSONSingleExporter.validate(wrongStructure);

    expect(result.valid).toBe(false);
    expect(result.error).toBeDefined();
  });
});
