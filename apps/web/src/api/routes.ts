/**
 * Central API Architecture and Route Definitions
 * 
 * This file defines the complete REST API structure for the AI Analytics Dashboard.
 * It serves as the central routing hub that organizes all API endpoints by domain.
 */

import { NextRequest, NextResponse } from 'next/server';

// Import route handlers from domain-specific modules
import { analyticsRoutes } from './domains/analytics';
import { costRoutes } from './domains/cost';
import { filterRoutes } from './domains/filters';
import { aggregationRoutes } from './domains/aggregation';
import { exportRoutes } from './domains/export';
import { webhookRoutes } from './domains/webhooks';

// Import middleware
import { authMiddleware } from './middleware/auth';
import { rateLimitMiddleware } from './middleware/rate-limit';
import { errorHandler } from './middleware/error-handler';

/**
 * API Route Configuration
 * Defines the structure and metadata for all API endpoints
 */
export interface ApiRouteConfig {
  path: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  handler: (request: NextRequest) => Promise<NextResponse>;
  middleware?: Array<(request: NextRequest) => Promise<NextResponse | null>>;
  authRequired?: boolean;
  rateLimit?: {
    requests: number;
    window: number; // seconds
  };
  description: string;
  tags?: string[];
}

/**
 * Complete API Route Registry
 * Central registry of all available API endpoints
 */
export const apiRoutes: ApiRouteConfig[] = [
  // Analytics Query Endpoints
  {
    path: '/api/analytics/query',
    method: 'POST',
    handler: analyticsRoutes.executeQuery,
    middleware: [rateLimitMiddleware({ requests: 100, window: 60 })],
    authRequired: true,
    description: 'Execute an analytics query with filtering and aggregation',
    tags: ['analytics', 'query']
  },
  {
    path: '/api/analytics/query/health',
    method: 'GET',
    handler: analyticsRoutes.healthCheck,
    authRequired: false,
    description: 'Health check for analytics query service',
    tags: ['analytics', 'health']
  },
  {
    path: '/api/analytics/query/stats',
    method: 'GET',
    handler: analyticsRoutes.getStats,
    authRequired: true,
    description: 'Get analytics query system statistics',
    tags: ['analytics', 'monitoring']
  },

  // Cost Analysis Endpoints
  {
    path: '/api/cost/breakdown',
    method: 'POST',
    handler: costRoutes.breakdown,
    middleware: [rateLimitMiddleware({ requests: 50, window: 60 })],
    authRequired: true,
    description: 'Analyze cost breakdown by dimensions',
    tags: ['cost', 'analysis']
  },
  {
    path: '/api/cost/trends',
    method: 'POST',
    handler: costRoutes.trends,
    middleware: [rateLimitMiddleware({ requests: 50, window: 60 })],
    authRequired: true,
    description: 'Analyze cost trends over time',
    tags: ['cost', 'trends']
  },
  {
    path: '/api/cost/forecast',
    method: 'POST',
    handler: costRoutes.forecast,
    middleware: [rateLimitMiddleware({ requests: 20, window: 60 })],
    authRequired: true,
    description: 'Generate cost forecast',
    tags: ['cost', 'forecast']
  },
  {
    path: '/api/cost/optimization',
    method: 'POST',
    handler: costRoutes.optimization,
    middleware: [rateLimitMiddleware({ requests: 20, window: 60 })],
    authRequired: true,
    description: 'Get cost optimization recommendations',
    tags: ['cost', 'optimization']
  },
  {
    path: '/api/cost/comparison',
    method: 'POST',
    handler: costRoutes.comparison,
    middleware: [rateLimitMiddleware({ requests: 30, window: 60 })],
    authRequired: true,
    description: 'Compare costs across dimensions',
    tags: ['cost', 'comparison']
  },
  {
    path: '/api/cost/health',
    method: 'GET',
    handler: costRoutes.healthCheck,
    authRequired: false,
    description: 'Health check for cost analysis service',
    tags: ['cost', 'health']
  },

  // Filtering Endpoints
  {
    path: '/api/filters/dimensions',
    method: 'GET',
    handler: filterRoutes.getDimensions,
    authRequired: true,
    description: 'Get available filter dimensions',
    tags: ['filters', 'metadata']
  },
  {
    path: '/api/filters/validate',
    method: 'POST',
    handler: filterRoutes.validate,
    middleware: [rateLimitMiddleware({ requests: 100, window: 60 })],
    authRequired: true,
    description: 'Validate filter configuration',
    tags: ['filters', 'validation']
  },
  {
    path: '/api/filters/saved',
    method: 'GET',
    handler: filterRoutes.listSaved,
    authRequired: true,
    description: 'List saved filter configurations',
    tags: ['filters', 'saved']
  },
  {
    path: '/api/filters/saved',
    method: 'POST',
    handler: filterRoutes.save,
    authRequired: true,
    description: 'Save a filter configuration',
    tags: ['filters', 'saved']
  },
  {
    path: '/api/filters/saved/:id',
    method: 'DELETE',
    handler: filterRoutes.delete,
    authRequired: true,
    description: 'Delete a saved filter configuration',
    tags: ['filters', 'saved']
  },

  // Aggregation Endpoints
  {
    path: '/api/aggregation/functions',
    method: 'GET',
    handler: aggregationRoutes.getFunctions,
    authRequired: true,
    description: 'Get available aggregation functions',
    tags: ['aggregation', 'metadata']
  },
  {
    path: '/api/aggregation/execute',
    method: 'POST',
    handler: aggregationRoutes.execute,
    middleware: [rateLimitMiddleware({ requests: 50, window: 60 })],
    authRequired: true,
    description: 'Execute aggregation query',
    tags: ['aggregation', 'query']
  },
  {
    path: '/api/aggregation/trends',
    method: 'POST',
    handler: aggregationRoutes.trends,
    middleware: [rateLimitMiddleware({ requests: 30, window: 60 })],
    authRequired: true,
    description: 'Get aggregated trend analysis',
    tags: ['aggregation', 'trends']
  },

  // Export Endpoints
  {
    path: '/api/export/execute',
    method: 'POST',
    handler: exportRoutes.execute,
    middleware: [rateLimitMiddleware({ requests: 20, window: 60 })],
    authRequired: true,
    description: 'Execute data export',
    tags: ['export', 'data']
  },
  {
    path: '/api/export/formats',
    method: 'GET',
    handler: exportRoutes.getFormats,
    authRequired: true,
    description: 'Get available export formats',
    tags: ['export', 'metadata']
  },
  {
    path: '/api/export/status/:id',
    method: 'GET',
    handler: exportRoutes.getStatus,
    authRequired: true,
    description: 'Get export job status',
    tags: ['export', 'status']
  },
  {
    path: '/api/export/download/:id',
    method: 'GET',
    handler: exportRoutes.download,
    authRequired: true,
    description: 'Download exported data',
    tags: ['export', 'download']
  },

  // Webhook Endpoints
  {
    path: '/api/webhooks',
    method: 'GET',
    handler: webhookRoutes.list,
    authRequired: true,
    description: 'List configured webhooks',
    tags: ['webhooks', 'management']
  },
  {
    path: '/api/webhooks',
    method: 'POST',
    handler: webhookRoutes.create,
    authRequired: true,
    description: 'Create a new webhook',
    tags: ['webhooks', 'management']
  },
  {
    path: '/api/webhooks/:id',
    method: 'GET',
    handler: webhookRoutes.get,
    authRequired: true,
    description: 'Get webhook details',
    tags: ['webhooks', 'management']
  },
  {
    path: '/api/webhooks/:id',
    method: 'PUT',
    handler: webhookRoutes.update,
    authRequired: true,
    description: 'Update webhook configuration',
    tags: ['webhooks', 'management']
  },
  {
    path: '/api/webhooks/:id',
    method: 'DELETE',
    handler: webhookRoutes.delete,
    authRequired: true,
    description: 'Delete a webhook',
    tags: ['webhooks', 'management']
  },
  {
    path: '/api/webhooks/:id/test',
    method: 'POST',
    handler: webhookRoutes.test,
    authRequired: true,
    description: 'Test webhook delivery',
    tags: ['webhooks', 'testing']
  },
  {
    path: '/api/webhooks/:id/disable',
    method: 'POST',
    handler: webhookRoutes.disable,
    authRequired: true,
    description: 'Disable a webhook',
    tags: ['webhooks', 'management']
  },
  {
    path: '/api/webhooks/:id/enable',
    method: 'POST',
    handler: webhookRoutes.enable,
    authRequired: true,
    description: 'Enable a webhook',
    tags: ['webhooks', 'management']
  },

  // API Metadata Endpoints
  {
    path: '/api',
    method: 'GET',
    handler: getApiMetadata,
    authRequired: false,
    description: 'Get API metadata and available endpoints',
    tags: ['metadata', 'api']
  },
  {
    path: '/api/health',
    method: 'GET',
    handler: getHealthCheck,
    authRequired: false,
    description: 'Overall API health check',
    tags: ['health', 'api']
  }
];

/**
 * Main API Router
 * Routes incoming requests to appropriate handlers with middleware
 */
export async function apiRouter(request: NextRequest): Promise<NextResponse> {
  const { pathname, method } = new URL(request.url);
  
  try {
    // Find matching route
    const route = apiRoutes.find(r => {
      // Handle dynamic parameters (e.g., :id)
      const routePattern = r.path.replace(/:([^/]+)/g, '[^/]+');
      const regex = new RegExp(`^${routePattern}$`);
      return regex.test(pathname) && r.method === method;
    });

    if (!route) {
      return NextResponse.json(
        { error: 'Endpoint not found', path: pathname, method },
        { status: 404 }
      );
    }

    // Apply authentication middleware if required
    if (route.authRequired) {
      const authResult = await authMiddleware(request);
      if (authResult) {
        return authResult; // Auth failed
      }
    }

    // Apply custom middleware
    if (route.middleware) {
      for (const middleware of route.middleware) {
        const result = await middleware(request);
        if (result) {
          return result; // Middleware blocked the request
        }
      }
    }

    // Execute route handler
    return await route.handler(request);

  } catch (error) {
    return errorHandler(error, request);
  }
}

/**
 * Get API Metadata
 * Returns information about available endpoints and API version
 */
async function getApiMetadata(request: NextRequest): Promise<NextResponse> {
  return NextResponse.json({
    name: 'AI Analytics Dashboard API',
    version: '1.0.0',
    description: 'Comprehensive REST API for AI usage analytics',
    endpoints: apiRoutes.map(route => ({
      path: route.path,
      method: route.method,
      description: route.description,
      authRequired: route.authRequired,
      tags: route.tags
    })),
    documentation: '/docs/api-spec.yaml'
  });
}

/**
 * Overall API Health Check
 * Checks health of all API services
 */
async function getHealthCheck(request: NextRequest): Promise<NextResponse> {
  const healthChecks = await Promise.allSettled([
    analyticsRoutes.healthCheck(request),
    costRoutes.healthCheck(request)
  ]);

  const services = {
    analytics: healthChecks[0].status === 'fulfilled',
    cost: healthChecks[1].status === 'fulfilled'
  };

  const overallHealthy = Object.values(services).every(status => status);

  return NextResponse.json({
    status: overallHealthy ? 'healthy' : 'degraded',
    timestamp: new Date().toISOString(),
    services
  }, { status: overallHealthy ? 200 : 503 });
}

/**
 * Route lookup utility for testing and documentation
 */
export function findRoute(path: string, method: string): ApiRouteConfig | undefined {
  return apiRoutes.find(r => {
    const routePattern = r.path.replace(/:([^/]+)/g, '[^/]+');
    const regex = new RegExp(`^${routePattern}$`);
    return regex.test(path) && r.method === method;
  });
}

/**
 * Get routes by tag
 */
export function getRoutesByTag(tag: string): ApiRouteConfig[] {
  return apiRoutes.filter(route => route.tags?.includes(tag));
}

/**
 * Get all authenticated routes
 */
export function getAuthenticatedRoutes(): ApiRouteConfig[] {
  return apiRoutes.filter(route => route.authRequired);
}
