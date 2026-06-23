/**
 * Tests for Format Conversion Utilities
 */

import { FormatConverter, formatConverter } from '../converters';

describe('FormatConverter', () => {
  let converter: FormatConverter;

  beforeEach(() => {
    converter = new FormatConverter();
  });

  describe('htmlToMarkdown', () => {
    it('should convert basic HTML to markdown', async () => {
      const html = '<h1>Title</h1><p>Paragraph</p>';
      const result = await converter.htmlToMarkdown(html);

      expect(result.content).toContain('# Title');
      expect(result.content).toContain('Paragraph');
      expect(result.sourceFormat).toBe('html');
      expect(result.targetFormat).toBe('markdown');
    });

    it('should convert HTML tables to markdown', async () => {
      const html = '<table><tr><th>Header</th></tr><tr><td>Data</td></tr></table>';
      const result = await converter.htmlToMarkdown(html);

      expect(result.content).toContain('| Header |');
      expect(result.content).toContain('| Data |');
    });

    it('should convert HTML links to markdown', async () => {
      const html = '<a href="https://example.com">Link</a>';
      const result = await converter.htmlToMarkdown(html);

      expect(result.content).toContain('[Link](https://example.com)');
    });

    it('should convert HTML emphasis to markdown', async () => {
      const html = '<strong>Bold</strong> and <em>italic</em>';
      const result = await converter.htmlToMarkdown(html);

      expect(result.content).toContain('**Bold**');
      expect(result.content).toContain('*italic*');
    });

    it('should handle empty HTML', async () => {
      const result = await converter.htmlToMarkdown('');

      expect(result.content).toBe('');
    });
  });

  describe('markdownToHTML', () => {
    it('should convert basic markdown to HTML', async () => {
      const markdown = '# Title\n\nParagraph';
      const result = await converter.markdownToHTML(markdown);

      expect(result.content).toContain('<h1>Title</h1>');
      expect(result.content).toContain('<p>Paragraph</p>');
      expect(result.sourceFormat).toBe('markdown');
      expect(result.targetFormat).toBe('html');
    });

    it('should convert markdown tables to HTML', async () => {
      const markdown = '| Header |\n| --- |\n| Data |';
      const result = await converter.markdownToHTML(markdown);

      expect(result.content).toContain('<table>');
      expect(result.content).toContain('<th>Header</th>');
      expect(result.content).toContain('<td>Data</td>');
    });

    it('should convert markdown links to HTML', async () => {
      const markdown = '[Link](https://example.com)';
      const result = await converter.markdownToHTML(markdown);

      expect(result.content).toContain('<a href="https://example.com">Link</a>');
    });

    it('should convert markdown code to HTML', async () => {
      const markdown = '`code`';
      const result = await converter.markdownToHTML(markdown);

      expect(result.content).toContain('<code>code</code>');
    });

    it('should wrap in HTML structure', async () => {
      const markdown = '# Title';
      const result = await converter.markdownToHTML(markdown, { includeMetadata: true });

      expect(result.content).toContain('<!DOCTYPE html>');
      expect(result.content).toContain('<html>');
      expect(result.content).toContain('<body>');
    });
  });

  describe('markdownToToonFormat', () => {
    it('should convert markdown table to ToonFormat', async () => {
      const markdown = '| timestamp | value |\n| --- |\n| 2024-01-01 | 100 |';
      const result = await converter.markdownToToonFormat(markdown);

      expect(result.content).toContain('data[1]');
      expect(result.content).toContain('timestamp');
      expect(result.content).toContain('value');
      expect(result.sourceFormat).toBe('markdown');
      expect(result.targetFormat).toBe('toon');
    });

    it('should handle markdown without tables', async () => {
      const markdown = '# Title\n\nSome text';
      const result = await converter.markdownToToonFormat(markdown);

      expect(result.warnings.length).toBeGreaterThan(0);
      expect(result.warnings[0]).toContain('No parseable tables');
    });
  });

  describe('toonFormatToMarkdown', () => {
    it('should convert ToonFormat to markdown', async () => {
      const toon = 'data[1]{timestamp,value}:\n  2024-01-01,100';
      const result = await converter.toonFormatToMarkdown(toon);

      expect(result.content).toContain('# Analytics Data');
      expect(result.content).toContain('| timestamp |');
      expect(result.content).toContain('| value |');
      expect(result.sourceFormat).toBe('toon');
      expect(result.targetFormat).toBe('markdown');
    });

    it('should handle empty ToonFormat', async () => {
      const toon = 'data[]';
      const result = await converter.toonFormatToMarkdown(toon);

      expect(result.content).toContain('# No Data');
    });
  });

  describe('htmlToToonFormat', () => {
    it('should convert HTML to ToonFormat via markdown', async () => {
      const html = '<table><tr><th>value</th></tr><tr><td>100</td></tr></table>';
      const result = await converter.htmlToToonFormat(html);

      expect(result.content).toContain('data[1]');
      expect(result.sourceFormat).toBe('html');
      expect(result.targetFormat).toBe('toon');
    });
  });

  describe('toonFormatToHTML', () => {
    it('should convert ToonFormat to HTML via markdown', async () => {
      const toon = 'data[1]{value}:\n  100';
      const result = await converter.toonFormatToHTML(toon);

      expect(result.content).toContain('<table>');
      expect(result.content).toContain('<td>100</td>');
      expect(result.sourceFormat).toBe('toon');
      expect(result.targetFormat).toBe('html');
    });
  });

  describe('conversion time tracking', () => {
    it('should track conversion time', async () => {
      const html = '<h1>Title</h1>';
      const result = await converter.htmlToMarkdown(html);

      expect(result.conversionTimeMs).toBeGreaterThanOrEqual(0);
      expect(typeof result.conversionTimeMs).toBe('number');
    });
  });

  describe('error handling', () => {
    it('should handle conversion errors gracefully', async () => {
      const invalidHTML = '<div>Unclosed tag';
      const result = await converter.htmlToMarkdown(invalidHTML);

      expect(result.warnings.length).toBeGreaterThan(0);
    });

    it('should provide warnings for failed conversions', async () => {
      const result = await converter.markdownToToonFormat('invalid toon');

      expect(result.warnings.length).toBeGreaterThan(0);
      expect(result.warnings[0]).toContain('failed');
    });
  });

  describe('format conversion options', () => {
    it('should respect includeMetadata option', async () => {
      const markdown = '# Title';
      const result = await converter.markdownToHTML(markdown, { includeMetadata: true });

      expect(result.content).toContain('<meta');
    });

    it('should respect maxTableRows option', async () => {
      const toon = 'data[5]{value}:\n  1\n  2\n  3\n  4\n  5';
      const result = await converter.toonFormatToMarkdown(toon, { maxTableRows: 2 });

      expect(result.content).toContain('... and 3 more records');
    });
  });
});
