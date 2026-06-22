/**
 * Authentication and Authorization Middleware
 * 
 * Handles API authentication using API keys, JWT tokens, or other auth mechanisms
 */

import { NextRequest, NextResponse } from 'next/server';

export interface AuthConfig {
  apiKey?: string;
  jwtSecret?: string;
  enableApiKeyAuth?: boolean;
  enableJwtAuth?: boolean;
}

export interface AuthContext {
  userId?: string;
  apiKey?: string;
  permissions: string[];
  authenticated: boolean;
}

/**
 * Authentication middleware
 * Validates authentication credentials and attaches auth context to request
 */
export async function authMiddleware(
  request: NextRequest,
  config: AuthConfig = {}
): Promise<NextResponse | null> {
  const authHeader = request.headers.get('authorization');
  
  // Try API key authentication
  if (config.enableApiKeyAuth !== false) {
    const apiKey = request.headers.get('x-api-key');
    if (apiKey) {
      const isValid = await validateApiKey(apiKey, config);
      if (isValid) {
        // Attach auth context to request headers for downstream handlers
        const requestWithAuth = new NextRequest(request.url, request);
        requestWithAuth.headers.set('x-authenticated', 'true');
        requestWithAuth.headers.set('x-auth-method', 'api-key');
        requestWithAuth.headers.set('x-api-key', apiKey);
        return null; // Success - continue to next middleware/handler
      }
    }
  }
  
  // Try JWT authentication
  if (config.enableJwtAuth !== false && authHeader) {
    const token = authHeader.replace('Bearer ', '');
    const isValid = await validateJwtToken(token, config);
    if (isValid) {
      const requestWithAuth = new NextRequest(request.url, request);
      requestWithAuth.headers.set('x-authenticated', 'true');
      requestWithAuth.headers.set('x-auth-method', 'jwt');
      requestWithAuth.headers.set('x-auth-token', token);
      return null; // Success - continue to next middleware/handler
    }
  }
  
  // Authentication failed
  return NextResponse.json(
    {
      error: 'Authentication required',
      message: 'Valid API key or JWT token required',
      code: 'AUTH_REQUIRED'
    },
    { status: 401 }
  );
}

/**
 * Validate API key
 */
async function validateApiKey(apiKey: string, config: AuthConfig): Promise<boolean> {
  // TODO: Implement proper API key validation
  // For now, accept any non-empty key in development
  if (process.env.NODE_ENV === 'development') {
    return apiKey.length > 0;
  }
  
  // In production, validate against database or key management service
  return false;
}

/**
 * Validate JWT token
 */
async function validateJwtToken(token: string, config: AuthConfig): Promise<boolean> {
  // TODO: Implement proper JWT validation
  // For now, accept any non-empty token in development
  if (process.env.NODE_ENV === 'development') {
    return token.length > 0;
  }
  
  // In production, validate signature and expiration
  return false;
}

/**
 * Extract auth context from request
 */
export function getAuthContext(request: NextRequest): AuthContext {
  const authenticated = request.headers.get('x-authenticated') === 'true';
  const authMethod = request.headers.get('x-auth-method');
  
  return {
    authenticated,
    apiKey: authMethod === 'api-key' ? request.headers.get('x-api-key') || undefined : undefined,
    permissions: [], // TODO: Implement permission extraction
    userId: undefined // TODO: Implement user ID extraction
  };
}

/**
 * Authorization check helper
 */
export function hasPermission(authContext: AuthContext, permission: string): boolean {
  return authContext.permissions.includes(permission) || authContext.permissions.includes('*');
}

/**
 * Require specific permission
 */
export function requirePermission(permission: string) {
  return (request: NextRequest): NextResponse | null => {
    const authContext = getAuthContext(request);
    
    if (!authContext.authenticated) {
      return NextResponse.json(
        { error: 'Authentication required', code: 'AUTH_REQUIRED' },
        { status: 401 }
      );
    }
    
    if (!hasPermission(authContext, permission)) {
      return NextResponse.json(
        { error: 'Insufficient permissions', code: 'FORBIDDEN' },
        { status: 403 }
      );
    }
    
    return null; // Success - continue
  };
}
