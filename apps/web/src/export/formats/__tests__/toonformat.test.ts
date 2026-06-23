/**
 * Tests for ToonFormat export format
 */

import { ToonFormatExporter, ToonFormatDecoder } from '../toonformat';

describe('ToonFormatExporter', () => {
  let exporter: ToonFormatExporter;

  beforeEach(() => {
    exporter = new ToonFormatExporter();
  });

  describe('export', () => {
    it('should export simple data to ToonFormat', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4', provider: 'openai' },
          metrics: { tokens: 100, cost: 0.01 }
        }
      ];

      const result = await exporter.export(data);

      expect(result.content).toContain('data[1]');
      expect(result.content).toContain('timestamp');
      expect(result.content).toContain('model');
      expect(result.content).toContain('tokens');
      expect(result.recordCount).toBe(1);
      expect(result.format).toBe('toon');
    });

    it('should export multiple records', async () => {
      const data = [
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

      const result = await exporter.export(data);

      expect(result.content).toContain('data[2]');
      expect(result.recordCount).toBe(2);
    });

    it('should calculate compression ratio', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4', provider: 'openai' },
          metrics: { tokens: 100, cost: 0.01 }
        }
      ];

      const result = await exporter.export(data);

      expect(result.compressionRatio).toBeGreaterThan(0);
      expect(result.metadata.compressionRatio).toBeGreaterThan(0);
      expect(result.metadata.tokenSavings).toBeGreaterThan(0);
    });

    it('should include metadata when requested', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 100 }
        }
      ];

      const result = await exporter.export(data, { includeMetadata: true });

      expect(result.content).toContain('# ToonFormat Export');
      expect(result.content).toContain('exportedAt:');
      expect(result.content).toContain('recordCount:');
      expect(result.metadata.hasMetadata).toBe(true);
    });

    it('should exclude metadata when requested', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 100 }
        }
      ];

      const result = await exporter.export(data, { includeMetadata: false });

      expect(result.content).not.toContain('# ToonFormat Export');
      expect(result.metadata.hasMetadata).toBe(false);
    });

    it('should handle empty data', async () => {
      const result = await exporter.export([]);

      expect(result.content).toContain('data[]');
      expect(result.recordCount).toBe(0);
    });
  });

  describe('exportWithFields', () => {
    it('should export only selected fields', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4', provider: 'openai' },
          metrics: { tokens: 100, cost: 0.01 }
        }
      ];

      const result = await exporter.exportWithFields(data, ['timestamp', 'model', 'tokens']);

      expect(result.content).toContain('timestamp');
      expect(result.content).toContain('model');
      expect(result.content).toContain('tokens');
      expect(result.content).not.toContain('provider');
      expect(result.content).not.toContain('cost');
    });
  });

  describe('delimiter options', () => {
    it('should use comma delimiter by default', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 100 }
        }
      ];

      const result = await exporter.export(data, { delimiter: 'comma' });

      expect(result.content).toContain(',');
    });

    it('should use tab delimiter when specified', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 100 }
        }
      ];

      const result = await exporter.export(data, { delimiter: 'tab' });

      expect(result.content).toContain('\t');
    });

    it('should use pipe delimiter when specified', async () => {
      const data = [
        {
          timestamp: '2024-01-01T00:00:00Z',
          dimensions: { model: 'gpt-4' },
          metrics: { tokens: 100 }
        }
      ];

      const result = await exporter.export(data, { delimiter: 'pipe' });

      expect(result.content).toContain('|');
    });
  });
});

describe('ToonFormatDecoder', () => {
  let decoder: ToonFormatDecoder;

  beforeEach(() => {
    decoder = new ToonFormatDecoder();
  });

  describe('decode', () => {
    it('should decode simple ToonFormat', async () => {
      const toonContent = `data[1]{timestamp,model,tokens}:
  2024-01-01T00:00:00Z,gpt-4,100`;

      const result = await decoder.decode(toonContent);

      expect(result.data).toHaveLength(1);
      expect(result.data[0].timestamp).toBe('2024-01-01T00:00:00Z');
      expect(result.data[0].model).toBe('gpt-4');
      expect(result.data[0].tokens).toBe(100);
    });

    it('should decode multiple records', async () => {
      const toonContent = `data[2]{timestamp,model,tokens}:
  2024-01-01T00:00:00Z,gpt-4,100
  2024-01-01T01:00:00Z,gpt-4,150`;

      const result = await decoder.decode(toonContent);

      expect(result.data).toHaveLength(2);
      expect(result.data[0].tokens).toBe(100);
      expect(result.data[1].tokens).toBe(150);
    });

    it('should handle nested fields', async () => {
      const toonContent = `data[1]{timestamp,dimensions.model,metrics.tokens}:
  2024-01-01T00:00:00Z,gpt-4,100`;

      const result = await decoder.decode(toonContent);

      expect(result.data[0].dimensions.model).toBe('gpt-4');
      expect(result.data[0].metrics.tokens).toBe(100);
    });

    it('should skip comments', async () => {
      const toonContent = `# This is a comment
data[1]{timestamp,tokens}:
  2024-01-01T00:00:00Z,100`;

      const result = await decoder.decode(toonContent);

      expect(result.data).toHaveLength(1);
    });

    it('should parse metadata', async () => {
      const toonContent = `# ToonFormat Export
exportedAt: 2024-01-01T00:00:00Z
recordCount: 1
format: toon

data[1]{timestamp,tokens}:
  2024-01-01T00:00:00Z,100`;

      const result = await decoder.decode(toonContent);

      expect(result.data).toHaveLength(1);
    });
  });
});

describe('Round-trip encoding/decoding', () => {
  it('should maintain data integrity through encode/decode cycle', async () => {
    const originalData = [
      {
        timestamp: '2024-01-01T00:00:00Z',
        dimensions: { model: 'gpt-4', provider: 'openai' },
        metrics: { tokens: 100, cost: 0.01 }
      },
      {
        timestamp: '2024-01-01T01:00:00Z',
        dimensions: { model: 'gpt-4', provider: 'openai' },
        metrics: { tokens: 150, cost: 0.015 }
      }
    ];

    const exporter = new ToonFormatExporter();
    const decoder = new ToonFormatDecoder();

    const exported = await exporter.export(originalData);
    const decoded = await decoder.decode(exported.content);

    expect(decoded.data).toHaveLength(originalData.length);
    expect(decoded.data[0].timestamp).toBe(originalData[0].timestamp);
    expect(decoded.data[0].dimensions.model).toBe(originalData[0].dimensions.model);
    expect(decoded.data[0].metrics.tokens).toBe(originalData[0].metrics.tokens);
  });
});
