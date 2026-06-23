/**
 * Format Conversion Utilities
 * 
 * Bidirectional conversion between HTML, Markdown, and ToonFormat formats
 * Enables seamless data transformation for different consumption patterns
 */

import { ToonFormatEncoder, ToonFormatDecoder } from './formats/toonformat';
import { QueryResultRow } from '../api/analytics/types';

/**
 * Conversion options
 */
export interface ConversionOptions {
  includeMetadata: boolean;
  preserveFormatting: boolean;
  sanitizeHTML: boolean;
  maxTableRows: number;
}

/**
 * Conversion result
 */
export interface ConversionResult {
  content: string;
  sourceFormat: string;
  targetFormat: string;
  conversionTimeMs: number;
  warnings: string[];
}

/**
 * Format converter for bidirectional conversion
 */
export class FormatConverter {
  private defaultOptions: ConversionOptions = {
    includeMetadata: true,
    preserveFormatting: true,
    sanitizeHTML: true,
    maxTableRows: 1000
  };

  /**
   * Convert HTML to Markdown
   */
  async htmlToMarkdown(html: string, options: Partial<ConversionOptions> = {}): Promise<ConversionResult> {
    const startTime = Date.now();
    const config = { ...this.defaultOptions, ...options };
    const warnings: string[] = [];

    try {
      let markdown = html;

      // Remove HTML tags and convert to markdown
      markdown = this.stripHTMLTags(markdown, config);
      
      // Convert tables to markdown
      markdown = this.convertHTMLTablesToMarkdown(markdown);
      
      // Convert headers
      markdown = this.convertHTMLHeaders(markdown);
      
      // Convert lists
      markdown = this.convertHTMLLists(markdown);
      
      // Convert links
      markdown = this.convertHTMLLinks(markdown);
      
      // Clean up
      markdown = this.cleanMarkdown(markdown);

      const conversionTime = Date.now() - startTime;

      return {
        content: markdown,
        sourceFormat: 'html',
        targetFormat: 'markdown',
        conversionTimeMs: conversionTime,
        warnings
      };
    } catch (error) {
      warnings.push(`HTML to Markdown conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      
      return {
        content: '',
        sourceFormat: 'html',
        targetFormat: 'markdown',
        conversionTimeMs: Date.now() - startTime,
        warnings
      };
    }
  }

  /**
   * Convert Markdown to HTML
   */
  async markdownToHTML(markdown: string, options: Partial<ConversionOptions> = {}): Promise<ConversionResult> {
    const startTime = Date.now();
    const config = { ...this.defaultOptions, ...options };
    const warnings: string[] = [];

    try {
      let html = markdown;

      // Convert headers
      html = this.convertMarkdownHeaders(html);
      
      // Convert tables
      html = this.convertMarkdownTables(html);
      
      // Convert lists
      html = this.convertMarkdownLists(html);
      
      // Convert links
      html = this.convertMarkdownLinks(html);
      
      // Convert code blocks
      html = this.convertMarkdownCode(html);
      
      // Convert bold/italic
      html = this.convertMarkdownEmphasis(html);
      
      // Wrap in HTML structure
      html = this.wrapInHTMLStructure(html, config);

      const conversionTime = Date.now() - startTime;

      return {
        content: html,
        sourceFormat: 'markdown',
        targetFormat: 'html',
        conversionTimeMs: conversionTime,
        warnings
      };
    } catch (error) {
      warnings.push(`Markdown to HTML conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      
      return {
        content: '',
        sourceFormat: 'markdown',
        targetFormat: 'html',
        conversionTimeMs: Date.now() - startTime,
        warnings
      };
    }
  }

  /**
   * Convert Markdown to ToonFormat
   */
  async markdownToToonFormat(markdown: string, options: Partial<ConversionOptions> = {}): Promise<ConversionResult> {
    const startTime = Date.now();
    const config = { ...this.defaultOptions, ...options };
    const warnings: string[] = [];

    try {
      // Parse markdown tables to structured data
      const data = this.parseMarkdownTables(markdown);
      
      if (data.length === 0) {
        warnings.push('No parseable tables found in markdown');
        return {
          content: markdown,
          sourceFormat: 'markdown',
          targetFormat: 'toon',
          conversionTimeMs: Date.now() - startTime,
          warnings
        };
      }

      // Convert to ToonFormat
      const encoder = new ToonFormatEncoder();
      const toonContent = await encoder.encode(data);

      const conversionTime = Date.now() - startTime;

      return {
        content: toonContent,
        sourceFormat: 'markdown',
        targetFormat: 'toon',
        conversionTimeMs: conversionTime,
        warnings
      };
    } catch (error) {
      warnings.push(`Markdown to ToonFormat conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      
      return {
        content: '',
        sourceFormat: 'markdown',
        targetFormat: 'toon',
        conversionTimeMs: Date.now() - startTime,
        warnings
      };
    }
  }

  /**
   * Convert ToonFormat to Markdown
   */
  async toonFormatToMarkdown(toonContent: string, options: Partial<ConversionOptions> = {}): Promise<ConversionResult> {
    const startTime = Date.now();
    const config = { ...this.defaultOptions, ...options };
    const warnings: string[] = [];

    try {
      // Decode ToonFormat to structured data
      const decoder = new ToonFormatDecoder();
      const { data } = await decoder.decode(toonContent);

      if (!data || data.length === 0) {
        warnings.push('No data found in ToonFormat content');
        return {
          content: toonContent,
          sourceFormat: 'toon',
          targetFormat: 'markdown',
          conversionTimeMs: Date.now() - startTime,
          warnings
        };
      }

      // Convert to markdown table
      const markdown = this.convertDataToMarkdown(data, config);

      const conversionTime = Date.now() - startTime;

      return {
        content: markdown,
        sourceFormat: 'toon',
        targetFormat: 'markdown',
        conversionTimeMs: conversionTime,
        warnings
      };
    } catch (error) {
      warnings.push(`ToonFormat to Markdown conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      
      return {
        content: '',
        sourceFormat: 'toon',
        targetFormat: 'markdown',
        conversionTimeMs: Date.now() - startTime,
        warnings
      };
    }
  }

  /**
   * Convert HTML to ToonFormat
   */
  async htmlToToonFormat(html: string, options: Partial<ConversionOptions> = {}): Promise<ConversionResult> {
    const startTime = Date.now();
    const warnings: string[] = [];

    try {
      // First convert HTML to Markdown
      const markdownResult = await this.htmlToMarkdown(html, options);
      
      if (markdownResult.warnings.length > 0) {
        warnings.push(...markdownResult.warnings);
      }

      // Then convert Markdown to ToonFormat
      const toonResult = await this.markdownToToonFormat(markdownResult.content, options);
      
      if (toonResult.warnings.length > 0) {
        warnings.push(...toonResult.warnings);
      }

      const conversionTime = Date.now() - startTime;

      return {
        content: toonResult.content,
        sourceFormat: 'html',
        targetFormat: 'toon',
        conversionTimeMs,
        warnings
      };
    } catch (error) {
      warnings.push(`HTML to ToonFormat conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      
      return {
        content: '',
        sourceFormat: 'html',
        targetFormat: 'toon',
        conversionTimeMs: Date.now() - startTime,
        warnings
      };
    }
  }

  /**
   * Convert ToonFormat to HTML
   */
  async toonFormatToHTML(toonContent: string, options: Partial<ConversionOptions> = {}): Promise<ConversionResult> {
    const startTime = Date.now();
    const warnings: string[] = [];

    try {
      // First convert ToonFormat to Markdown
      const markdownResult = await this.toonFormatToMarkdown(toonContent, options);
      
      if (markdownResult.warnings.length > 0) {
        warnings.push(...markdownResult.warnings);
      }

      // Then convert Markdown to HTML
      const htmlResult = await this.markdownToHTML(markdownResult.content, options);
      
      if (htmlResult.warnings.length > 0) {
        warnings.push(...htmlResult.warnings);
      }

      const conversionTime = Date.now() - startTime;

      return {
        content: htmlResult.content,
        sourceFormat: 'toon',
        targetFormat: 'html',
        conversionTimeMs,
        warnings
      };
    } catch (error) {
      warnings.push(`ToonFormat to HTML conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      
      return {
        content: '',
        sourceFormat: 'toon',
        targetFormat: 'html',
        conversionTimeMs: Date.now() - startTime,
        warnings
      };
    }
  }

  /**
   * Strip HTML tags and convert to markdown
   */
  private stripHTMLTags(html: string, options: ConversionOptions): string {
    let text = html;

    // Remove script and style tags
    text = text.replace(/<script[^>]*>.*?<\/script>/gis, '');
    text = text.replace(/<style[^>]*>.*?<\/style>/gis, '');

    // Convert basic HTML elements to markdown
    text = text.replace(/<strong[^>]*>(.*?)<\/strong>/gis, '**$1**');
    text = text.replace(/<b[^>]*>(.*?)<\/b>/gis, '**$1**');
    text = text.replace(/<em[^>]*>(.*?)<\/em>/gis, '*$1*');
    text = text.replace(/<i[^>]*>(.*?)<\/i>/gis, '*$1*');
    text = text.replace(/<code[^>]*>(.*?)<\/code>/gis, '`$1`');
    text = text.replace(/<br\s*\/?>/gi, '\n');
    text = text.replace(/<\/p>/gi, '\n\n');
    text = text.replace(/<[^>]+>/g, ''); // Remove remaining tags

    return text;
  }

  /**
   * Convert HTML tables to markdown
   */
  private convertHTMLTablesToMarkdown(html: string): string {
    // Simple table conversion - in production, use a proper HTML parser
    const tableRegex = /<table[^>]*>(.*?)<\/table>/gis;
    return html.replace(tableRegex, (match, tableContent) => {
      const rows = tableContent.match(/<tr[^>]*>(.*?)<\/tr>/gis) || [];
      
      if (rows.length === 0) return match;

      let markdown = '\n';
      
      for (let i = 0; i < rows.length; i++) {
        const cells = rows[i].match(/<t[dh][^>]*>(.*?)<\/t[dh]>/gis) || [];
        const cellText = cells.map(cell => cell.replace(/<[^>]+>/g, '').trim());
        
        markdown += '| ' + cellText.join(' | ') + ' |\n';
        
        if (i === 0) {
          markdown += '| ' + cells.map(() => '---').join(' | ') + ' |\n';
        }
      }
      
      return markdown + '\n';
    });
  }

  /**
   * Convert HTML headers to markdown
   */
  private convertHTMLHeaders(html: string): string {
    html = html.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n');
    html = html.replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n');
    html = html.replace(/<h3[^>]*>(.*?)<\/h3>/gi, '### $1\n\n');
    html = html.replace(/<h4[^>]*>(.*?)<\/h4>/gi, '#### $1\n\n');
    html = html.replace(/<h5[^>]*>(.*?)<\/h5>/gi, '##### $1\n\n');
    html = html.replace(/<h6[^>]*>(.*?)<\/h6>/gi, '###### $1\n\n');
    return html;
  }

  /**
   * Convert HTML lists to markdown
   */
  private convertHTMLLists(html: string): string {
    // Convert ordered lists
    html = html.replace(/<ol[^>]*>(.*?)<\/ol>/gis, (match, content) => {
      const items = content.match(/<li[^>]*>(.*?)<\/li>/gis) || [];
      return items.map((item, i) => `${i + 1}. ${item.replace(/<[^>]+>/g, '')}`).join('\n');
    });

    // Convert unordered lists
    html = html.replace(/<ul[^>]*>(.*?)<\/ul>/gis, (match, content) => {
      const items = content.match(/<li[^>]*>(.*?)<\/li>/gis) || [];
      return items.map(item => `- ${item.replace(/<[^>]+>/g, '')}`).join('\n');
    });

    return html;
  }

  /**
   * Convert HTML links to markdown
   */
  private convertHTMLLinks(html: string): string {
    html = html.replace(/<a[^>]+href="([^"]+)"[^>]*>(.*?)<\/a>/gi, '[$2]($1)');
    return html;
  }

  /**
   * Clean up markdown
   */
  private cleanMarkdown(text: string): string {
    // Remove excessive whitespace
    text = text.replace(/\n{3,}/g, '\n\n');
    
    // Trim whitespace
    text = text.trim();
    
    return text;
  }

  /**
   * Convert Markdown headers to HTML
   */
  private convertMarkdownHeaders(text: string): string {
    let result = text;
    result = result.replace(/^# (.*$)/gim, '<h1>$1</h1>');
    result = result.replace(/^## (.*$)/gim, '<h2>$1</h2>');
    result = result.replace(/^### (.*$)/gim, '<h3>$1</h3>');
    result = result.replace(/^#### (.*$)/gim, '<h4>$1</h4>');
    result = result.replace(/^##### (.*$)/gim, '<h5>$1</h5>');
    result = result.replace(/^###### (.*$)/gim, '<h6>$1</h6>');
    return result;
  }

  /**
   * Convert Markdown tables to HTML
   */
  private convertMarkdownTables(text: string): string {
    const tableRegex = /(\|.+\|\n)+/g;
    return text.replace(tableRegex, (match) => {
      const lines = match.trim().split('\n');
      const rows = lines.filter(line => line.includes('|'));
      
      if (rows.length === 0) return match;

      let html = '<table>\n';
      
      for (let i = 0; i < rows.length; i++) {
        const cells = rows[i].split('|').map(c => c.trim()).filter(c => c);
        const tag = i === 0 ? 'th' : 'td';
        
        html += '  <tr>\n';
        for (const cell of cells) {
          html += `    <${tag}>${cell}</${tag}>\n`;
        }
        html += '  </tr>\n';
      }
      
      html += '</table>\n';
      return html;
    });
  }

  /**
   * Convert Markdown lists to HTML
   */
  private convertMarkdownLists(text: string): string {
    let result = text;
    // Convert ordered lists
    result = result.replace(/^\d+\. (.*$)/gim, '<li>$1</li>');
    
    // Convert unordered lists
    result = result.replace(/^- (.*$)/gim, '<li>$1</li>');
    
    // Wrap in list tags (simplified)
    result = result.replace(/(<li>.*<\/li>\n)+/gim, '<ul>\n$1</ul>\n');
    
    return result;
  }

  /**
   * Convert Markdown links to HTML
   */
  private convertMarkdownLinks(text: string): string {
    let result = text;
    result = result.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');
    return result;
  }

  /**
   * Convert Markdown code blocks to HTML
   */
  private convertMarkdownCode(text: string): string {
    let result = text;
    // Code blocks
    result = result.replace(/```(\w+)?\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>');
    
    // Inline code
    result = result.replace(/`([^`]+)`/g, '<code>$1</code>');
    
    return result;
  }

  /**
   * Convert Markdown emphasis to HTML
   */
  private convertMarkdownEmphasis(text: string): string {
    let result = text;
    result = result.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
    result = result.replace(/\*([^*]+)\*/g, '<em>$1</em>');
    return result;
  }

  /**
   * Wrap content in HTML structure
   */
  private wrapInHTMLStructure(html: string, options: ConversionOptions): string {
    let wrapped = '<!DOCTYPE html>\n<html>\n<head>\n';
    
    if (options.includeMetadata) {
      wrapped += '  <meta charset="utf-8">\n';
      wrapped += '  <meta name="generator" content="AI Dashboard Format Converter">\n';
    }
    
    wrapped += '</head>\n<body>\n';
    wrapped += html;
    wrapped += '\n</body>\n</html>';
    
    return wrapped;
  }

  /**
   * Parse markdown tables to structured data
   */
  private parseMarkdownTables(markdown: string): QueryResultRow[] {
    const tableRegex = /(\|.+\|\n)+/g;
    const matches = markdown.match(tableRegex);
    
    if (!matches || matches.length === 0) return [];

    const tableMatch = matches[0];
    const lines = tableMatch.trim().split('\n');
    const rows = lines.filter(line => line.includes('|'));
    
    if (rows.length < 2) return [];

    // Extract headers
    const headers = rows[0].split('|').map(h => h.trim()).filter(h => h);
    
    // Extract data rows (skip separator row)
    const dataRows = rows.slice(2);
    const records: QueryResultRow[] = [];

    for (const row of dataRows) {
      const cells = row.split('|').map(c => c.trim()).filter(c => c);
      const record: QueryResultRow = {
        timestamp: cells[0] || new Date().toISOString()
      };

      // Map cells to record structure
      for (let i = 1; i < headers.length && i < cells.length; i++) {
        const header = headers[i];
        const value = cells[i];
        
        // Try to parse as number
        const numValue = parseFloat(value);
        
        if (!isNaN(numValue)) {
          if (!record.metrics) record.metrics = {};
          record.metrics[header] = numValue;
        } else {
          if (!record.dimensions) record.dimensions = {};
          record.dimensions[header] = value;
        }
      }

      records.push(record);
    }

    return records;
  }

  /**
   * Convert structured data to markdown table
   */
  private convertDataToMarkdown(data: QueryResultRow[], options: ConversionOptions): string {
    if (data.length === 0) return '# No Data\n\nNo data available.';

    let markdown = '# Analytics Data\n\n';
    
    if (options.includeMetadata) {
      markdown += `**Record Count:** ${data.length}\n`;
      markdown += `**Generated:** ${new Date().toISOString()}\n\n`;
    }

    // Extract headers
    const headers = this.extractHeaders(data[0]);

    // Create table
    markdown += '| ' + headers.join(' | ') + ' |\n';
    markdown += '| ' + headers.map(() => '---').join(' | ') + ' |\n';

    const maxRows = Math.min(data.length, options.maxTableRows);
    for (let i = 0; i < maxRows; i++) {
      const values = headers.map(header => {
        const value = this.getNestedValue(data[i], header);
        return this.formatCellValue(value);
      });
      markdown += '| ' + values.join(' | ') + ' |\n';
    }

    if (data.length > options.maxTableRows) {
      markdown += `\n*... and ${data.length - options.maxTableRows} more records*\n`;
    }

    return markdown;
  }

  /**
   * Extract headers from record
   */
  private extractHeaders(record: QueryResultRow): string[] {
    const headers: string[] = ['timestamp'];

    if (record.dimensions) {
      Object.keys(record.dimensions).forEach(key => {
        headers.push(`dimensions.${key}`);
      });
    }

    if (record.metrics) {
      Object.keys(record.metrics).forEach(key => {
        headers.push(`metrics.${key}`);
      });
    }

    return headers;
  }

  /**
   * Get nested value from record
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
   * Format cell value for markdown
   */
  private formatCellValue(value: any): string {
    if (value === null || value === undefined) return 'N/A';
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  }
}

// Export singleton instance
export const formatConverter = new FormatConverter();
