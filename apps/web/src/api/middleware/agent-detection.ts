/**
 * AI Agent Detection Middleware
 * 
 * Sophisticated AI agent detection and automatic format selection
 * Uses multiple detection methods and heuristics for accurate identification
 */

import { NextRequest } from 'next/server';
import { ContentType } from './content-negotiation';

/**
 * AI Agent types
 */
export enum AgentType {
  CLAUDE = 'claude',
  GPT = 'gpt',
  OPENAI = 'openai',
  ANTHROPIC = 'anthropic',
  GENERIC_BOT = 'generic-bot',
  CRAWLER = 'crawler',
  UNKNOWN = 'unknown'
}

/**
 * Detection confidence levels
 */
export enum DetectionConfidence {
  HIGH = 'high',
  MEDIUM = 'medium',
  LOW = 'low'
}

/**
 * Agent detection result
 */
export interface AgentDetectionResult {
  isAIAgent: boolean;
  agentType: AgentType;
  confidence: DetectionConfidence;
  detectedBy: string[];
  userAgent: string | null;
  recommendedFormat: ContentType;
  reasoning: string[];
}

/**
 * Known AI agent patterns
 */
const AGENT_PATTERNS = [
  {
    type: AgentType.CLAUDE,
    patterns: [/claude-ai/i, /claude/i, /anthropic-claude/i],
    confidence: DetectionConfidence.HIGH
  },
  {
    type: AgentType.GPT,
    patterns: [/gpt-4/i, /gpt-3/i, /chatgpt/i],
    confidence: DetectionConfidence.HIGH
  },
  {
    type: AgentType.OPENAI,
    patterns: [/openai/i, /openai-crawler/i, /gptbot/i],
    confidence: DetectionConfidence.HIGH
  },
  {
    type: AgentType.ANTHROPIC,
    patterns: [/anthropic/i, /anthropic-ai/i],
    confidence: DetectionConfidence.HIGH
  },
  {
    type: AgentType.GENERIC_BOT,
    patterns: [/bot/i, /spider/i, /crawler/i],
    confidence: DetectionConfidence.MEDIUM
  },
  {
    type: AgentType.CRAWLER,
    patterns: [/googlebot/i, /bingbot/i, /slurp/i, /duckduckbot/i],
    confidence: DetectionConfidence.MEDIUM
  }
];

/**
 * Known AI agent IP ranges (simplified for demonstration)
 */
const AI_AGENT_IP_RANGES = [
  // OpenAI IP ranges (example)
  '104.28.0.0/16',
  // Add other known AI agent IP ranges as needed
];

/**
 * Request behavior patterns that suggest AI agents
 */
const BEHAVIOR_PATTERNS = {
  highRequestRate: 100, // requests per minute
  unusualHeaders: ['x-api-key', 'x-ai-model', 'x-agent-version'],
  compressionPreference: ['toon', 'markdown', 'jsonl'],
  bulkDataRequests: true
};

/**
 * Detect AI agent from user agent string
 */
function detectFromUserAgent(userAgent: string | null): {
  agentType: AgentType;
  confidence: DetectionConfidence;
  reasoning: string;
} | null {
  if (!userAgent) {
    return null;
  }

  for (const agent of AGENT_PATTERNS) {
    for (const pattern of agent.patterns) {
      if (pattern.test(userAgent)) {
        return {
          agentType: agent.type,
          confidence: agent.confidence,
          reasoning: `User agent matches pattern: ${pattern.toString()}`
        };
      }
    }
  }

  return null;
}

/**
 * Detect AI agent from IP address
 */
function detectFromIP(ip: string | null): {
  agentType: AgentType;
  confidence: DetectionConfidence;
  reasoning: string;
} | null {
  if (!ip) {
    return null;
  }

  // Check against known AI agent IP ranges
  for (const range of AI_AGENT_IP_RANGES) {
    if (isIPInRange(ip, range)) {
      return {
        agentType: AgentType.GENERIC_BOT,
        confidence: DetectionConfidence.MEDIUM,
        reasoning: `IP address matches known AI agent range: ${range}`
      };
    }
  }

  return null;
}

/**
 * Check if IP is in CIDR range
 */
function isIPInRange(ip: string, cidr: string): boolean {
  // Simplified IP range check - in production, use proper CIDR library
  const [range, mask] = cidr.split('/');
  return ip.startsWith(range.split('.')[0] + '.');
}

/**
 * Detect AI agent from request headers
 */
function detectFromHeaders(headers: Headers): {
  agentType: AgentType;
  confidence: DetectionConfidence;
  reasoning: string;
} | null {
  const detectedBy: string[] = [];

  // Check for AI-specific headers
  for (const header of BEHAVIOR_PATTERNS.unusualHeaders) {
    if (headers.has(header)) {
      detectedBy.push(`Header present: ${header}`);
    }
  }

  // Check for compression preference
  const accept = headers.get('accept');
  if (accept) {
    for (const format of BEHAVIOR_PATTERNS.compressionPreference) {
      if (accept.toLowerCase().includes(format)) {
        detectedBy.push(`Accept header includes: ${format}`);
        break;
      }
    }
  }

  if (detectedBy.length > 0) {
    return {
      agentType: AgentType.GENERIC_BOT,
      confidence: DetectionConfidence.MEDIUM,
      reasoning: detectedBy.join(', ')
    };
  }

  return null;
}

/**
 * Detect AI agent from request behavior
 */
function detectFromBehavior(request: NextRequest): {
  agentType: AgentType;
  confidence: DetectionConfidence;
  reasoning: string;
} | null {
  const detectedBy: string[] = [];

  // Check for bulk data request indicators
  const url = request.nextUrl.searchParams;
  if (url.has('format') && BEHAVIOR_PATTERNS.compressionPreference.includes(url.get('format')!)) {
    detectedBy.push('Requesting compressed format');
  }

  if (url.has('maxRecords') && parseInt(url.get('maxRecords')!) > 100) {
    detectedBy.push('Requesting large record count');
  }

  if (detectedBy.length > 0) {
    return {
      agentType: AgentType.GENERIC_BOT,
      confidence: DetectionConfidence.LOW,
      reasoning: detectedBy.join(', ')
    };
  }

  return null;
}

/**
 * Get recommended format for agent type
 */
function getRecommendedFormat(agentType: AgentType): ContentType {
  switch (agentType) {
    case AgentType.CLAUDE:
    case AgentType.GPT:
    case AgentType.OPENAI:
    case AgentType.ANTHROPIC:
      return ContentType.TOON; // Most token-efficient for LLMs
    case AgentType.GENERIC_BOT:
    case AgentType.CRAWLER:
      return ContentType.MARKDOWN; // Good balance of readability and efficiency
    default:
      return ContentType.HTML; // Default for unknown
  }
}

/**
 * Main AI agent detection function
 */
export function detectAIAgent(request: NextRequest): AgentDetectionResult {
  const detectedBy: string[] = [];
  const reasoning: string[] = [];
  let agentType = AgentType.UNKNOWN;
  let confidence = DetectionConfidence.LOW;
  let isAIAgent = false;

  const userAgent = request.headers.get('user-agent');
  const clientIP = request.headers.get('x-forwarded-for') || request.headers.get('x-real-ip');

  // Detect from user agent
  const userAgentResult = detectFromUserAgent(userAgent);
  if (userAgentResult) {
    agentType = userAgentResult.agentType;
    confidence = userAgentResult.confidence;
    reasoning.push(userAgentResult.reasoning);
    detectedBy.push('user-agent');
    isAIAgent = true;
  }

  // Detect from IP
  const ipResult = detectFromIP(clientIP);
  if (ipResult) {
    if (!isAIAgent || ipResult.confidence === DetectionConfidence.HIGH) {
      agentType = ipResult.agentType;
      confidence = ipResult.confidence;
    }
    reasoning.push(ipResult.reasoning);
    detectedBy.push('ip-address');
    isAIAgent = true;
  }

  // Detect from headers
  const headerResult = detectFromHeaders(request.headers);
  if (headerResult) {
    if (!isAIAgent || headerResult.confidence === DetectionConfidence.HIGH) {
      agentType = headerResult.agentType;
      confidence = headerResult.confidence;
    }
    reasoning.push(headerResult.reasoning);
    detectedBy.push('headers');
    isAIAgent = true;
  }

  // Detect from behavior
  const behaviorResult = detectFromBehavior(request);
  if (behaviorResult) {
    if (!isAIAgent) {
      agentType = behaviorResult.agentType;
      confidence = behaviorResult.confidence;
    }
    reasoning.push(behaviorResult.reasoning);
    detectedBy.push('behavior');
    isAIAgent = true;
  }

  return {
    isAIAgent,
    agentType,
    confidence,
    detectedBy,
    userAgent,
    recommendedFormat: getRecommendedFormat(agentType),
    reasoning
  };
}

/**
 * Middleware function for AI agent detection
 */
export function agentDetectionMiddleware(request: NextRequest) {
  const detection = detectAIAgent(request);

  return {
    detection,
    shouldOptimizeForAI: detection.isAIAgent && detection.confidence !== DetectionConfidence.LOW,
    recommendedHeaders: {
      'X-AI-Agent-Detected': detection.isAIAgent.toString(),
      'X-Agent-Type': detection.agentType,
      'X-Detection-Confidence': detection.confidence,
      'X-Detected-By': detection.detectedBy.join(',')
    }
  };
}

/**
 * Get detection statistics for monitoring
 */
export function getDetectionStats() {
  return {
    supportedAgentTypes: Object.values(AgentType),
    confidenceLevels: Object.values(DetectionConfidence),
    detectionMethods: ['user-agent', 'ip-address', 'headers', 'behavior'],
    knownPatterns: AGENT_PATTERNS.length,
    knownIPRanges: AI_AGENT_IP_RANGES.length
  };
}

/**
 * Validate detection result
 */
export function validateDetection(result: AgentDetectionResult): {
  valid: boolean;
  issues: string[];
} {
  const issues: string[] = [];

  if (result.isAIAgent && result.detectedBy.length === 0) {
    issues.push('AI agent detected but no detection methods specified');
  }

  if (result.isAIAgent && result.confidence === DetectionConfidence.LOW && result.detectedBy.length === 1) {
    issues.push('Low confidence detection with single method may be unreliable');
  }

  if (result.agentType === AgentType.UNKNOWN && result.isAIAgent) {
    issues.push('AI agent detected but type is unknown');
  }

  return {
    valid: issues.length === 0,
    issues
  };
}
