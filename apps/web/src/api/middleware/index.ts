/**
 * API Middleware Index
 * 
 * Central exports for API middleware
 */

export {
  negotiateContent,
  createNegotiatedResponse,
  contentNegotiationMiddleware,
  getAvailableFormats,
  isContentTypeSupported,
  formatToContentType
} from './content-negotiation';

export { ContentType } from './content-negotiation';

export type {
  ContentNegotiationOptions,
  ContentNegotiationResult
} from './content-negotiation';

export {
  detectAIAgent,
  agentDetectionMiddleware,
  getDetectionStats,
  validateDetection
} from './agent-detection';

export {
  AgentType,
  DetectionConfidence
} from './agent-detection';

export type {
  AgentDetectionResult
} from './agent-detection';
