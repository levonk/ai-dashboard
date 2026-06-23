/**
 * Tests for Content Negotiation Middleware
 */

import { negotiateContent, createNegotiatedResponse, ContentType, getAvailableFormats, isContentTypeSupported, formatToContentType } from '../content-negotiation';
import { NextRequest } from 'next/server';

describe('Content Negotiation', () => {
  describe('negotiateContent', () => {
    it('should use default format when no Accept header', () => {
      const request = new NextRequest('http://example.com/api/data');
      const result = negotiateContent(request);

      expect(result.contentType).toBe(ContentType.HTML);
      expect(result.format).toBe('html');
      expect(result.isAIAgent).toBe(false);
    });

    it('should respect Accept header for HTML', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'accept': 'text/html' }
      });
      const result = negotiateContent(request);

      expect(result.contentType).toBe(ContentType.HTML);
      expect(result.format).toBe('html');
    });

    it('should respect Accept header for Markdown', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'accept': 'text/markdown' }
      });
      const result = negotiateContent(request);

      expect(result.contentType).toBe(ContentType.MARKDOWN);
      expect(result.format).toBe('markdown');
    });

    it('should respect Accept header for ToonFormat', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'accept': 'text/plain' }
      });
      const result = negotiateContent(request, { defaultFormat: ContentType.TOON });

      expect(result.contentType).toBe(ContentType.TOON);
      expect(result.format).toBe('toon');
    });

    it('should respect Accept header for JSON', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'accept': 'application/json' }
      });
      const result = negotiateContent(request);

      expect(result.contentType).toBe(ContentType.JSON);
      expect(result.format).toBe('json');
    });

    it('should detect AI agents from user agent', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Claude-AI/1.0' }
      });
      const result = negotiateContent(request);

      expect(result.isAIAgent).toBe(true);
    });

    it('should detect AI agents from various patterns', () => {
      const aiAgents = [
        'Claude-AI/1.0',
        'GPT-4-Bot/1.0',
        'OpenAI-Crawler/1.0',
        'Anthropic-Agent/1.0',
        'AI-Agent/1.0',
        'GoogleBot/1.0'
      ];

      for (const userAgent of aiAgents) {
        const request = new NextRequest('http://example.com/api/data', {
          headers: { 'user-agent': userAgent }
        });
        const result = negotiateContent(request);
        expect(result.isAIAgent).toBe(true);
      }
    });

    it('should not detect regular browsers as AI agents', () => {
      const browsers = [
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
      ];

      for (const userAgent of browsers) {
        const request = new NextRequest('http://example.com/api/data', {
          headers: { 'user-agent': userAgent }
        });
        const result = negotiateContent(request);
        expect(result.isAIAgent).toBe(false);
      }
    });

    it('should prioritize token efficiency for AI agents when enabled', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: {
          'accept': 'text/html, text/markdown, text/plain',
          'user-agent': 'Claude-AI/1.0'
        }
      });
      const result = negotiateContent(request, { prioritizeTokenEfficiency: true });

      expect(result.contentType).toBe(ContentType.TOON);
      expect(result.tokenEfficiency).toBe('high');
    });

    it('should allow query parameter override', () => {
      const request = new NextRequest('http://example.com/api/data?format=markdown');
      const result = negotiateContent(request, { allowQueryOverride: true });

      expect(result.contentType).toBe(ContentType.MARKDOWN);
      expect(result.format).toBe('markdown');
    });

    it('should respect various format query parameters', () => {
      const formats = [
        { query: 'format=html', expected: ContentType.HTML },
        { query: 'format=markdown', expected: ContentType.MARKDOWN },
        { query: 'format=md', expected: ContentType.MARKDOWN },
        { query: 'format=toon', expected: ContentType.TOON },
        { query: 'format=json', expected: ContentType.JSON },
        { query: 'format=jsonl', expected: ContentType.JSONL }
      ];

      for (const { query, expected } of formats) {
        const request = new NextRequest(`http://example.com/api/data?${query}`);
        const result = negotiateContent(request, { allowQueryOverride: true });
        expect(result.contentType).toBe(expected);
      }
    });

    it('should handle quality values in Accept header', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'accept': 'text/html;q=0.5, text/markdown;q=0.9, application/json;q=0.8' }
      });
      const result = negotiateContent(request);

      expect(result.contentType).toBe(ContentType.MARKDOWN);
    });

    it('should return correct token efficiency levels', () => {
      const request = new NextRequest('http://example.com/api/data');

      const htmlResult = negotiateContent(request, { defaultFormat: ContentType.HTML });
      expect(htmlResult.tokenEfficiency).toBe('low');

      const markdownResult = negotiateContent(request, { defaultFormat: ContentType.MARKDOWN });
      expect(markdownResult.tokenEfficiency).toBe('medium');

      const toonResult = negotiateContent(request, { defaultFormat: ContentType.TOON });
      expect(toonResult.tokenEfficiency).toBe('high');
    });
  });

  describe('createNegotiatedResponse', () => {
    it('should create response with negotiated content type', () => {
      const request = new NextRequest('http://example.com/api/data');
      const content = 'Test content';
      const response = createNegotiatedResponse(request, content);

      expect(response.headers.get('Content-Type')).toContain('text/html');
      expect(response.headers.get('X-Content-Format')).toBe('html');
      expect(response.headers.get('X-Token-Efficiency')).toBe('low');
    });

    it('should include AI agent detection header', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Claude-AI/1.0' }
      });
      const content = 'Test content';
      const response = createNegotiatedResponse(request, content);

      expect(response.headers.get('X-AI-Agent-Detected')).toBe('true');
    });

    it('should include Vary header', () => {
      const request = new NextRequest('http://example.com/api/data');
      const content = 'Test content';
      const response = createNegotiatedResponse(request, content);

      expect(response.headers.get('Vary')).toBe('Accept, User-Agent');
    });
  });

  describe('getAvailableFormats', () => {
    it('should return all available formats', () => {
      const formats = getAvailableFormats();

      expect(formats).toHaveLength(6);
      expect(formats.map(f => f.format)).toContain('html');
      expect(formats.map(f => f.format)).toContain('markdown');
      expect(formats.map(f => f.format)).toContain('toon');
      expect(formats.map(f => f.format)).toContain('json');
      expect(formats.map(f => f.format)).toContain('jsonl');
      expect(formats.map(f => f.format)).toContain('json-single');
    });

    it('should include format metadata', () => {
      const formats = getAvailableFormats();
      const htmlFormat = formats.find(f => f.format === 'html');

      expect(htmlFormat).toBeDefined();
      expect(htmlFormat?.description).toBeDefined();
      expect(htmlFormat?.tokenEfficiency).toBeDefined();
      expect(htmlFormat?.humanReadable).toBeDefined();
    });
  });

  describe('isContentTypeSupported', () => {
    it('should return true for supported content types', () => {
      expect(isContentTypeSupported('text/html')).toBe(true);
      expect(isContentTypeSupported('text/markdown')).toBe(true);
      expect(isContentTypeSupported('text/plain')).toBe(true);
      expect(isContentTypeSupported('application/json')).toBe(true);
      expect(isContentTypeSupported('application/jsonl')).toBe(true);
    });

    it('should return false for unsupported content types', () => {
      expect(isContentTypeSupported('application/xml')).toBe(false);
      expect(isContentTypeSupported('text/csv')).toBe(false);
      expect(isContentTypeSupported('application/pdf')).toBe(false);
    });
  });

  describe('formatToContentType', () => {
    it('should convert format strings to content types', () => {
      expect(formatToContentType('html')).toBe(ContentType.HTML);
      expect(formatToContentType('markdown')).toBe(ContentType.MARKDOWN);
      expect(formatToContentType('md')).toBe(ContentType.MARKDOWN);
      expect(formatToContentType('toon')).toBe(ContentType.TOON);
      expect(formatToContentType('toonformat')).toBe(ContentType.TOON);
      expect(formatToContentType('json')).toBe(ContentType.JSON);
      expect(formatToContentType('jsonl')).toBe(ContentType.JSONL);
      expect(formatToContentType('json-single')).toBe(ContentType.JSON_SINGLE);
    });

    it('should default to HTML for unknown formats', () => {
      expect(formatToContentType('unknown')).toBe(ContentType.HTML);
      expect(formatToContentType('xml')).toBe(ContentType.HTML);
    });
  });
});
