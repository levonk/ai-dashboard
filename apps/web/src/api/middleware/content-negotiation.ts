/**
 * Content Negotiation Middleware
 * 
 * Handles content negotiation for multiple output formats (HTML, Markdown, ToonFormat, JSON)
 * Based on Accept headers and query parameters for AI agent optimization
 */

import { NextRequest, NextResponse } from 'next/server';

/**
 * Supported content types
 */
export enum ContentType {
  HTML = 'text/html',
  MARKDOWN = 'text/markdown',
  TOON = 'text/plain',
  JSON = 'application/json',
  JSONL = 'application/jsonl',
  JSON_SINGLE = 'application/json'
}

/**
 * Content negotiation options
 */
export interface ContentNegotiationOptions {
  defaultFormat: ContentType;
  allowQueryOverride: boolean;
  prioritizeTokenEfficiency: boolean;
}

/**
 * Content negotiation result
 */
export interface ContentNegotiationResult {
  contentType: ContentType;
  format: string;
  isAIAgent: boolean;
  tokenEfficiency: 'low' | 'medium' | 'high';
  charset: string;
}

/**
 * Default content negotiation options
 */
const defaultOptions: ContentNegotiationOptions = {
  defaultFormat: ContentType.HTML,
  allowQueryOverride: true,
  prioritizeTokenEfficiency: false
};

/**
 * Parse Accept header and determine preferred content type
 */
function parseAcceptHeader(acceptHeader: string | null): ContentType[] {
  if (!acceptHeader) {
    return [];
  }

  const types = acceptHeader.split(',').map(part => {
    const [type, ...params] = part.trim().split(';');
    const qParam = params.find(p => p.trim().startsWith('q='));
    const q = qParam ? parseFloat(qParam.split('=')[1]) : 1.0;
    return { type: type.trim() as ContentType, q };
  });

  // Sort by quality value
  types.sort((a, b) => b.q - a.q);

  return types.map(t => t.type);
}

/**
 * Detect if the client is an AI agent based on user agent
 */
function detectAIAgent(userAgent: string | null): boolean {
  if (!userAgent) {
    return false;
  }

  const aiAgentPatterns = [
    /claude-ai/i,
    /gpt-4/i,
    /openai/i,
    /anthropic/i,
    /ai-agent/i,
    /bot/i,
    /crawler/i,
    /spider/i
  ];

  return aiAgentPatterns.some(pattern => pattern.test(userAgent));
}

/**
 * Determine token efficiency level for a content type
 */
function getTokenEfficiency(contentType: ContentType): 'low' | 'medium' | 'high' {
  switch (contentType) {
    case ContentType.TOON:
      return 'high';
    case ContentType.MARKDOWN:
    case ContentType.JSONL:
    case ContentType.JSON_SINGLE:
      return 'medium';
    case ContentType.HTML:
    case ContentType.JSON:
    default:
      return 'low';
  }
}

/**
 * Get the most token-efficient format for AI agents
 */
function getMostTokenEfficientFormat(): ContentType {
  return ContentType.TOON;
}

/**
 * Get human-readable format
 */
function getHumanReadableFormat(): ContentType {
  return ContentType.HTML;
}

/**
 * Negotiate content type based on request
 */
export function negotiateContent(
  request: NextRequest,
  options: Partial<ContentNegotiationOptions> = {}
): ContentNegotiationResult {
  const config = { ...defaultOptions, ...options };
  
  const acceptHeader = request.headers.get('accept');
  const userAgent = request.headers.get('user-agent');
  const formatQuery = request.nextUrl.searchParams.get('format');
  
  const isAIAgent = detectAIAgent(userAgent);
  
  // Check for query parameter override
  if (config.allowQueryOverride && formatQuery) {
    const requestedFormat = formatQuery.toLowerCase();
    let contentType: ContentType;

    switch (requestedFormat) {
      case 'html':
        contentType = ContentType.HTML;
        break;
      case 'markdown':
      case 'md':
        contentType = ContentType.MARKDOWN;
        break;
      case 'toon':
      case 'toonformat':
        contentType = ContentType.TOON;
        break;
      case 'json':
        contentType = ContentType.JSON;
        break;
      case 'jsonl':
        contentType = ContentType.JSONL;
        break;
      case 'json-single':
        contentType = ContentType.JSON_SINGLE;
        break;
      default:
        contentType = config.defaultFormat;
    }

    return {
      contentType,
      format: requestedFormat,
      isAIAgent,
      tokenEfficiency: getTokenEfficiency(contentType),
      charset: 'utf-8'
    };
  }

  // Parse Accept header
  const acceptedTypes = parseAcceptHeader(acceptHeader);

  // If AI agent and prioritizing token efficiency, prefer ToonFormat
  if (isAIAgent && config.prioritizeTokenEfficiency) {
    // Check if ToonFormat is accepted
    if (acceptedTypes.includes(ContentType.TOON) || acceptedTypes.includes(ContentType.MARKDOWN)) {
      return {
        contentType: ContentType.TOON,
        format: 'toon',
        isAIAgent,
        tokenEfficiency: 'high',
        charset: 'utf-8'
      };
    }
  }

  // If AI agent, prefer markdown over HTML
  if (isAIAgent) {
    if (acceptedTypes.includes(ContentType.MARKDOWN)) {
      return {
        contentType: ContentType.MARKDOWN,
        format: 'markdown',
        isAIAgent,
        tokenEfficiency: getTokenEfficiency(ContentType.MARKDOWN),
        charset: 'utf-8'
      };
    }
    
    if (acceptedTypes.includes(ContentType.TOON)) {
      return {
        contentType: ContentType.TOON,
        format: 'toon',
        isAIAgent,
        tokenEfficiency: getTokenEfficiency(ContentType.TOON),
        charset: 'utf-8'
      };
    }
  }

  // Use first accepted type that we support
  const supportedTypes = [
    ContentType.HTML,
    ContentType.MARKDOWN,
    ContentType.TOON,
    ContentType.JSON,
    ContentType.JSONL,
    ContentType.JSON_SINGLE
  ];

  for (const acceptedType of acceptedTypes) {
    if (supportedTypes.includes(acceptedType)) {
      return {
        contentType: acceptedType,
        format: acceptedType.split('/')[1] || acceptedType,
        isAIAgent,
        tokenEfficiency: getTokenEfficiency(acceptedType),
        charset: 'utf-8'
      };
    }
  }

  // Fall back to default
  return {
    contentType: config.defaultFormat,
    format: config.defaultFormat.split('/')[1] || 'html',
    isAIAgent,
    tokenEfficiency: getTokenEfficiency(config.defaultFormat),
    charset: 'utf-8'
  };
}

/**
 * Create response with negotiated content type
 */
export function createNegotiatedResponse(
  request: NextRequest,
  content: string,
  options: Partial<ContentNegotiationOptions> = {}
): NextResponse {
  const negotiation = negotiateContent(request, options);

  const response = new NextResponse(content, {
    headers: {
      'Content-Type': `${negotiation.contentType}; charset=${negotiation.charset}`,
      'X-Content-Format': negotiation.format,
      'X-Token-Efficiency': negotiation.tokenEfficiency,
      'X-AI-Agent-Detected': negotiation.isAIAgent.toString(),
      'Vary': 'Accept, User-Agent'
    }
  });

  return response;
}

/**
 * Middleware function for content negotiation
 */
export function contentNegotiationMiddleware(options: Partial<ContentNegotiationOptions> = {}) {
  return (request: NextRequest, response: NextResponse): NextResponse => {
    const negotiation = negotiateContent(request, options);

    // Add content negotiation headers to response
    response.headers.set('X-Content-Format', negotiation.format);
    response.headers.set('X-Token-Efficiency', negotiation.tokenEfficiency);
    response.headers.set('X-AI-Agent-Detected', negotiation.isAIAgent.toString());
    response.headers.set('Vary', 'Accept, User-Agent');

    return response;
  };
}

/**
 * Get available formats for API documentation
 */
export function getAvailableFormats(): Array<{
  contentType: ContentType;
  format: string;
  description: string;
  tokenEfficiency: 'low' | 'medium' | 'high';
  humanReadable: boolean;
}> {
  return [
    {
      contentType: ContentType.HTML,
      format: 'html',
      description: 'Standard HTML for web browsers',
      tokenEfficiency: 'low',
      humanReadable: true
    },
    {
      contentType: ContentType.MARKDOWN,
      format: 'markdown',
      description: 'Markdown format for documentation and AI agents',
      tokenEfficiency: 'medium',
      humanReadable: true
    },
    {
      contentType: ContentType.TOON,
      format: 'toon',
      description: 'ToonFormat for token-efficient AI consumption',
      tokenEfficiency: 'high',
      humanReadable: true
    },
    {
      contentType: ContentType.JSON,
      format: 'json',
      description: 'Standard JSON format',
      tokenEfficiency: 'low',
      humanReadable: true
    },
    {
      contentType: ContentType.JSONL,
      format: 'jsonl',
      description: 'JSON Lines format for streaming',
      tokenEfficiency: 'medium',
      humanReadable: false
    },
    {
      contentType: ContentType.JSON_SINGLE,
      format: 'json-single',
      description: 'JSON single record format',
      tokenEfficiency: 'medium',
      humanReadable: false
    }
  ];
}

/**
 * Validate if a content type is supported
 */
export function isContentTypeSupported(contentType: string): boolean {
  const supportedTypes = [
    ContentType.HTML,
    ContentType.MARKDOWN,
    ContentType.TOON,
    ContentType.JSON,
    ContentType.JSONL,
    ContentType.JSON_SINGLE
  ];

  return supportedTypes.includes(contentType as ContentType);
}

/**
 * Get content type from format string
 */
export function formatToContentType(format: string): ContentType {
  switch (format.toLowerCase()) {
    case 'html':
      return ContentType.HTML;
    case 'markdown':
    case 'md':
      return ContentType.MARKDOWN;
    case 'toon':
    case 'toonformat':
      return ContentType.TOON;
    case 'json':
      return ContentType.JSON;
    case 'jsonl':
      return ContentType.JSONL;
    case 'json-single':
      return ContentType.JSON_SINGLE;
    default:
      return ContentType.HTML;
  }
}
