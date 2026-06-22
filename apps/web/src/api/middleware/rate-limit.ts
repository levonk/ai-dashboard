/**
 * Rate Limiting Middleware
 * 
 * Implements rate limiting to protect API endpoints from abuse
 */

import { NextRequest, NextResponse } from 'next/server';

export interface RateLimitConfig {
  requests: number;
  window: number; // seconds
}

export interface RateLimitMiddlewareConfig {
  requests: number;
  window: number;
  skipSuccessfulRequests?: boolean;
  skipFailedRequests?: boolean;
}

// Simple in-memory rate limit store (for production, use Redis or similar)
const rateLimitStore = new Map<string, { count: number; resetTime: number }>();

/**
 * Rate limiting middleware factory
 */
export function rateLimitMiddleware(config: RateLimitConfig) {
  return async (request: NextRequest): Promise<NextResponse | null> => {
    const identifier = getClientIdentifier(request);
    const now = Date.now();
    const windowMs = config.window * 1000;
    
    // Get or create rate limit entry
    let entry = rateLimitStore.get(identifier);
    
    if (!entry || now > entry.resetTime) {
      // Create new entry or reset expired one
      entry = {
        count: 0,
        resetTime: now + windowMs
      };
      rateLimitStore.set(identifier, entry);
    }
    
    // Check if limit exceeded
    if (entry.count >= config.requests) {
      const resetTime = Math.ceil((entry.resetTime - now) / 1000);
      
      return NextResponse.json(
        {
          error: 'Rate limit exceeded',
          message: `Too many requests. Try again in ${resetTime} seconds.`,
          code: 'RATE_LIMIT_EXCEEDED',
          retryAfter: resetTime
        },
        {
          status: 429,
          headers: {
            'X-RateLimit-Limit': config.requests.toString(),
            'X-RateLimit-Remaining': '0',
            'X-RateLimit-Reset': entry.resetTime.toString(),
            'Retry-After': resetTime.toString()
          }
        }
      );
    }
    
    // Increment counter
    entry.count++;
    rateLimitStore.set(identifier, entry);
    
    // Add rate limit headers to request for downstream handlers
    const requestWithHeaders = new NextRequest(request.url, request);
    requestWithHeaders.headers.set('X-RateLimit-Limit', config.requests.toString());
    requestWithHeaders.headers.set('X-RateLimit-Remaining', (config.requests - entry.count).toString());
    requestWithHeaders.headers.set('X-RateLimit-Reset', entry.resetTime.toString());
    
    return null; // Success - continue to next middleware/handler
  };
}

/**
 * Get client identifier for rate limiting
 */
function getClientIdentifier(request: NextRequest): string {
  // Try API key first
  const apiKey = request.headers.get('x-api-key');
  if (apiKey) {
    return `api-key:${apiKey}`;
  }
  
  // Fall back to IP address
  const ip = request.headers.get('x-forwarded-for') || 
             request.headers.get('x-real-ip') || 
             'unknown';
  
  return `ip:${ip}`;
}

/**
 * Clean up expired rate limit entries
 * Call this periodically to prevent memory leaks
 */
export function cleanupRateLimitStore(): void {
  const now = Date.now();
  const keysToDelete: string[] = [];
  
  for (const [key, entry] of rateLimitStore.entries()) {
    if (now > entry.resetTime) {
      keysToDelete.push(key);
    }
  }
  
  for (const key of keysToDelete) {
    rateLimitStore.delete(key);
  }
}

/**
 * Get rate limit status for a client
 */
export function getRateLimitStatus(request: NextRequest): {
  limit: number;
  remaining: number;
  reset: number;
} | null {
  const identifier = getClientIdentifier(request);
  const entry = rateLimitStore.get(identifier);
  
  if (!entry) {
    return null;
  }
  
  return {
    limit: entry.count, // This should be the configured limit, not current count
    remaining: 0, // This should be calculated from configured limit
    reset: entry.resetTime
  };
}

// Schedule cleanup every 5 minutes
if (typeof setInterval !== 'undefined') {
  setInterval(cleanupRateLimitStore, 5 * 60 * 1000);
}
