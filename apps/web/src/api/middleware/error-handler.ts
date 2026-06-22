/**
 * Error Handler Middleware
 * 
 * Centralized error handling for API endpoints
 */

import { NextRequest, NextResponse } from 'next/server';

export interface ApiError {
  code: string;
  message: string;
  statusCode: number;
  details?: any;
  timestamp: string;
}

/**
 * Error handler middleware
 * Catches and formats errors consistently across all API endpoints
 */
export function errorHandler(error: any, request: NextRequest): NextResponse {
  console.error('API Error:', {
    error: error.message,
    stack: error.stack,
    url: request.url,
    method: request.method,
    timestamp: new Date().toISOString()
  });

  // Handle known error types
  if (error.code) {
    return NextResponse.json(
      formatError(error.code, error.message, error.statusCode || 500, error.details),
      { status: error.statusCode || 500 }
    );
  }

  // Handle validation errors
  if (error.name === 'ValidationError') {
    return NextResponse.json(
      formatError('VALIDATION_ERROR', error.message, 400, error.details),
      { status: 400 }
    );
  }

  // Handle authentication errors
  if (error.name === 'AuthenticationError') {
    return NextResponse.json(
      formatError('AUTHENTICATION_ERROR', error.message, 401),
      { status: 401 }
    );
  }

  // Handle authorization errors
  if (error.name === 'AuthorizationError') {
    return NextResponse.json(
      formatError('AUTHORIZATION_ERROR', error.message, 403),
      { status: 403 }
    );
  }

  // Handle not found errors
  if (error.name === 'NotFoundError') {
    return NextResponse.json(
      formatError('NOT_FOUND', error.message, 404),
      { status: 404 }
    );
  }

  // Handle rate limit errors
  if (error.name === 'RateLimitError') {
    return NextResponse.json(
      formatError('RATE_LIMIT_EXCEEDED', error.message, 429, error.details),
      { status: 429 }
    );
  }

  // Handle database errors
  if (error.name === 'DatabaseError') {
    return NextResponse.json(
      formatError('DATABASE_ERROR', 'A database error occurred', 500),
      { status: 500 }
    );
  }

  // Default to internal server error
  return NextResponse.json(
    formatError('INTERNAL_ERROR', 'An internal server error occurred', 500),
    { status: 500 }
  );
}

/**
 * Format error response
 */
function formatError(
  code: string,
  message: string,
  statusCode: number,
  details?: any
): ApiError {
  return {
    code,
    message: process.env.NODE_ENV === 'production' ? getSanitizedMessage(code, message) : message,
    statusCode,
    details: process.env.NODE_ENV === 'production' ? undefined : details,
    timestamp: new Date().toISOString()
  };
}

/**
 * Get sanitized error message for production
 */
function getSanitizedMessage(code: string, originalMessage: string): string {
  const productionMessages: Record<string, string> = {
    'VALIDATION_ERROR': 'Invalid request data',
    'AUTHENTICATION_ERROR': 'Authentication required',
    'AUTHORIZATION_ERROR': 'Insufficient permissions',
    'NOT_FOUND': 'Resource not found',
    'RATE_LIMIT_EXCEEDED': 'Too many requests',
    'DATABASE_ERROR': 'A database error occurred',
    'INTERNAL_ERROR': 'An internal server error occurred'
  };

  return productionMessages[code] || 'An error occurred';
}

/**
 * Custom error classes
 */
export class ValidationError extends Error {
  constructor(message: string, public details?: any) {
    super(message);
    this.name = 'ValidationError';
    this.code = 'VALIDATION_ERROR';
  }
  code: string;
}

export class AuthenticationError extends Error {
  constructor(message: string = 'Authentication required') {
    super(message);
    this.name = 'AuthenticationError';
    this.code = 'AUTHENTICATION_ERROR';
  }
  code: string;
}

export class AuthorizationError extends Error {
  constructor(message: string = 'Insufficient permissions') {
    super(message);
    this.name = 'AuthorizationError';
    this.code = 'AUTHORIZATION_ERROR';
  }
  code: string;
}

export class NotFoundError extends Error {
  constructor(message: string = 'Resource not found') {
    super(message);
    this.name = 'NotFoundError';
    this.code = 'NOT_FOUND';
  }
  code: string;
}

export class RateLimitError extends Error {
  constructor(message: string, public details?: any) {
    super(message);
    this.name = 'RateLimitError';
    this.code = 'RATE_LIMIT_EXCEEDED';
  }
  code: string;
}

export class DatabaseError extends Error {
  constructor(message: string = 'A database error occurred') {
    super(message);
    this.name = 'DatabaseError';
    this.code = 'DATABASE_ERROR';
  }
  code: string;
}

/**
 * Async wrapper to catch errors in route handlers
 */
export function asyncHandler(
  fn: (request: NextRequest) => Promise<NextResponse>
) {
  return async (request: NextRequest): Promise<NextResponse> => {
    try {
      return await fn(request);
    } catch (error) {
      return errorHandler(error, request);
    }
  };
}
