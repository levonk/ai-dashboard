/**
 * Tests for AI Agent Detection Middleware
 */

import { detectAIAgent, agentDetectionMiddleware, AgentType, DetectionConfidence, getDetectionStats, validateDetection } from '../agent-detection';
import { NextRequest } from 'next/server';
import { ContentType } from '../content-negotiation';

describe('AI Agent Detection', () => {
  describe('detectAIAgent', () => {
    it('should detect Claude AI from user agent', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Claude-AI/1.0' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.agentType).toBe(AgentType.CLAUDE);
      expect(result.confidence).toBe(DetectionConfidence.HIGH);
      expect(result.detectedBy).toContain('user-agent');
    });

    it('should detect GPT from user agent', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'GPT-4-Bot/1.0' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.agentType).toBe(AgentType.GPT);
      expect(result.confidence).toBe(DetectionConfidence.HIGH);
    });

    it('should detect OpenAI from user agent', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'OpenAI-Crawler/1.0' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.agentType).toBe(AgentType.OPENAI);
      expect(result.confidence).toBe(DetectionConfidence.HIGH);
    });

    it('should detect generic bots from user agent', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'MyBot/1.0' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.agentType).toBe(AgentType.GENERIC_BOT);
      expect(result.confidence).toBe(DetectionConfidence.MEDIUM);
    });

    it('should detect crawlers from user agent', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'GoogleBot/2.1' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.agentType).toBe(AgentType.CRAWLER);
      expect(result.confidence).toBe(DetectionConfidence.MEDIUM);
    });

    it('should not detect regular browsers as AI agents', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(false);
      expect(result.agentType).toBe(AgentType.UNKNOWN);
    });

    it('should detect from AI-specific headers', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'x-api-key': 'test-key' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.agentType).toBe(AgentType.GENERIC_BOT);
      expect(result.detectedBy).toContain('headers');
    });

    it('should detect from compression preference in Accept header', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'accept': 'text/toon, text/markdown' }
      });
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.detectedBy).toContain('headers');
    });

    it('should detect from behavior patterns', () => {
      const request = new NextRequest('http://example.com/api/data?format=toon&maxRecords=1000');
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(true);
      expect(result.detectedBy).toContain('behavior');
    });

    it('should recommend ToonFormat for LLM agents', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Claude-AI/1.0' }
      });
      const result = detectAIAgent(request);

      expect(result.recommendedFormat).toBe('text/plain'); // TOON format
    });

    it('should recommend Markdown for generic bots', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'MyBot/1.0' }
      });
      const result = detectAIAgent(request);

      expect(result.recommendedFormat).toBe('text/markdown');
    });

    it('should handle missing user agent gracefully', () => {
      const request = new NextRequest('http://example.com/api/data');
      const result = detectAIAgent(request);

      expect(result.isAIAgent).toBe(false);
      expect(result.userAgent).toBeNull();
    });

    it('should include reasoning in detection result', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Claude-AI/1.0' }
      });
      const result = detectAIAgent(request);

      expect(result.reasoning).toHaveLength(1);
      expect(result.reasoning[0]).toContain('User agent matches pattern');
    });
  });

  describe('agentDetectionMiddleware', () => {
    it('should return detection result and headers', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Claude-AI/1.0' }
      });
      const middleware = agentDetectionMiddleware(request);

      expect(middleware.detection).toBeDefined();
      expect(middleware.shouldOptimizeForAI).toBe(true);
      expect(middleware.recommendedHeaders).toBeDefined();
      expect(middleware.recommendedHeaders['X-AI-Agent-Detected']).toBe('true');
    });

    it('should not optimize for low confidence detections', () => {
      const request = new NextRequest('http://example.com/api/data?format=toon');
      const middleware = agentDetectionMiddleware(request);

      expect(middleware.shouldOptimizeForAI).toBe(false);
    });

    it('should include detection confidence in headers', () => {
      const request = new NextRequest('http://example.com/api/data', {
        headers: { 'user-agent': 'Claude-AI/1.0' }
      });
      const middleware = agentDetectionMiddleware(request);

      expect(middleware.recommendedHeaders['X-Detection-Confidence']).toBe('high');
    });
  });

  describe('getDetectionStats', () => {
    it('should return detection statistics', () => {
      const stats = getDetectionStats();

      expect(stats.supportedAgentTypes).toContain(AgentType.CLAUDE);
      expect(stats.supportedAgentTypes).toContain(AgentType.GPT);
      expect(stats.confidenceLevels).toContain(DetectionConfidence.HIGH);
      expect(stats.detectionMethods).toContain('user-agent');
      expect(stats.knownPatterns).toBeGreaterThan(0);
    });
  });

  describe('validateDetection', () => {
    it('should validate correct detection', () => {
      const result = {
        isAIAgent: true,
        agentType: AgentType.CLAUDE,
        confidence: DetectionConfidence.HIGH,
        detectedBy: ['user-agent'],
        userAgent: 'Claude-AI/1.0',
        recommendedFormat: ContentType.TOON,
        reasoning: ['User agent matches pattern']
      };

      const validation = validateDetection(result);

      expect(validation.valid).toBe(true);
      expect(validation.issues).toHaveLength(0);
    });

    it('should detect AI agent with no detection methods', () => {
      const result = {
        isAIAgent: true,
        agentType: AgentType.UNKNOWN,
        confidence: DetectionConfidence.LOW,
        detectedBy: [],
        userAgent: null,
        recommendedFormat: ContentType.HTML,
        reasoning: []
      };

      const validation = validateDetection(result);

      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('AI agent detected but no detection methods specified');
    });

    it('should detect low confidence with single method', () => {
      const result = {
        isAIAgent: true,
        agentType: AgentType.GENERIC_BOT,
        confidence: DetectionConfidence.LOW,
        detectedBy: ['behavior'],
        userAgent: null,
        recommendedFormat: ContentType.MARKDOWN,
        reasoning: ['Requesting compressed format']
      };

      const validation = validateDetection(result);

      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Low confidence detection with single method may be unreliable');
    });

    it('should detect unknown agent type for AI agent', () => {
      const result = {
        isAIAgent: true,
        agentType: AgentType.UNKNOWN,
        confidence: DetectionConfidence.HIGH,
        detectedBy: ['user-agent'],
        userAgent: 'Unknown-AI/1.0',
        recommendedFormat: ContentType.HTML,
        reasoning: ['User agent matches pattern']
      };

      const validation = validateDetection(result);

      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('AI agent detected but type is unknown');
    });
  });
});
