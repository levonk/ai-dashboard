/**
 * Query Error Handling and Retry Logic
 * 
 * Comprehensive error handling for analytics queries with retry logic
 * and user-friendly error messages
 */

import { QueryError, AnalyticsQueryRequest } from './types';

export enum ErrorCode {
  // Validation errors
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  INVALID_REQUEST = 'INVALID_REQUEST',
  MISSING_REQUIRED_FIELD = 'MISSING_REQUIRED_FIELD',
  INVALID_TIME_RANGE = 'INVALID_TIME_RANGE',
  
  // Execution errors
  EXECUTION_ERROR = 'EXECUTION_ERROR',
  QUERY_TIMEOUT = 'QUERY_TIMEOUT',
  RESOURCE_EXHAUSTED = 'RESOURCE_EXHAUSTED',
  
  // Data errors
  DATA_NOT_FOUND = 'DATA_NOT_FOUND',
  DATA_TOO_LARGE = 'DATA_TOO_LARGE',
  
  // System errors
  INTERNAL_ERROR = 'INTERNAL_ERROR',
  SERVICE_UNAVAILABLE = 'SERVICE_UNAVAILABLE',
  
  // Rate limiting
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED'
}

export class AnalyticsQueryError extends Error {
  code: ErrorCode;
  details?: any;
  timestamp: string;
  retryable: boolean;

  constructor(
    code: ErrorCode,
    message: string,
    details?: any,
    retryable: boolean = false
  ) {
    super(message);
    this.name = 'AnalyticsQueryError';
    this.code = code;
    this.details = details;
    this.timestamp = new Date().toISOString();
    this.retryable = retryable;
  }

  toJSON(): QueryError {
    return {
      code: this.code,
      message: this.message,
      details: this.details,
      timestamp: this.timestamp
    };
  }
}

export class ErrorHandler {
  /**
   * Handle an error and convert to appropriate response
   */
  static handleError(error: any): AnalyticsQueryError {
    // Already an AnalyticsQueryError
    if (error instanceof AnalyticsQueryError) {
      return error;
    }

    // Handle common error types
    if (error.name === 'ValidationError') {
      return new AnalyticsQueryError(
        ErrorCode.VALIDATION_ERROR,
        error.message || 'Query validation failed',
        { originalError: error.message },
        false
      );
    }

    if (error.name === 'TimeoutError') {
      return new AnalyticsQueryError(
        ErrorCode.QUERY_TIMEOUT,
        'Query execution timed out',
        { timeout: error.timeout },
        true
      );
    }

    if (error.message && error.message.includes('rate limit')) {
      return new AnalyticsQueryError(
        ErrorCode.RATE_LIMIT_EXCEEDED,
        'Rate limit exceeded. Please retry later.',
        { retryAfter: error.retryAfter },
        true
      );
    }

    if (error.message && error.message.includes('service unavailable')) {
      return new AnalyticsQueryError(
        ErrorCode.SERVICE_UNAVAILABLE,
        'Analytics service temporarily unavailable',
        { originalError: error.message },
        true
      );
    }

    // Default internal error
    return new AnalyticsQueryError(
      ErrorCode.INTERNAL_ERROR,
      error.message || 'An unexpected error occurred',
      { originalError: error.message },
      false
    );
  }

  /**
   * Create user-friendly error message
   */
  static getUserFriendlyMessage(error: AnalyticsQueryError): string {
    switch (error.code) {
      case ErrorCode.VALIDATION_ERROR:
        return 'The query request is invalid. Please check your parameters.';
      case ErrorCode.INVALID_TIME_RANGE:
        return 'The time range is invalid. Please ensure start time is before end time.';
      case ErrorCode.QUERY_TIMEOUT:
        return 'The query took too long to execute. Try refining your filters or reducing the time range.';
      case ErrorCode.DATA_TOO_LARGE:
        return 'The result set is too large. Please add filters or use pagination.';
      case ErrorCode.RATE_LIMIT_EXCEEDED:
        return 'You have exceeded the rate limit. Please wait before trying again.';
      case ErrorCode.SERVICE_UNAVAILABLE:
        return 'The analytics service is temporarily unavailable. Please try again later.';
      case ErrorCode.DATA_NOT_FOUND:
        return 'No data found for the specified query parameters.';
      default:
        return 'An error occurred while processing your query.';
    }
  }
}

export class RetryHandler {
  private maxRetries: number;
  private baseDelay: number;
  private maxDelay: number;

  constructor(
    maxRetries: number = 3,
    baseDelay: number = 1000,
    maxDelay: number = 10000
  ) {
    this.maxRetries = maxRetries;
    this.baseDelay = baseDelay;
    this.maxDelay = maxDelay;
  }

  /**
   * Execute a function with retry logic
   */
  async executeWithRetry<T>(
    fn: () => Promise<T>,
    context: string = 'operation'
  ): Promise<T> {
    let lastError: Error;

    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      try {
        return await fn();
      } catch (error) {
        lastError = error as Error;
        const analyticsError = error instanceof AnalyticsQueryError 
          ? error 
          : ErrorHandler.handleError(error);

        // Don't retry if error is not retryable
        if (!analyticsError.retryable) {
          throw analyticsError;
        }

        // Don't retry on last attempt
        if (attempt === this.maxRetries) {
          throw analyticsError;
        }

        // Calculate delay with exponential backoff
        const delay = this.calculateDelay(attempt);
        console.warn(
          `${context} failed (attempt ${attempt + 1}/${this.maxRetries + 1}), ` +
          `retrying in ${delay}ms. Error: ${analyticsError.message}`
        );

        await this.sleep(delay);
      }
    }

    throw lastError!;
  }

  /**
   * Calculate delay with exponential backoff
   */
  private calculateDelay(attempt: number): number {
    const delay = Math.min(
      this.baseDelay * Math.pow(2, attempt),
      this.maxDelay
    );
    // Add jitter to prevent thundering herd
    return delay + Math.random() * 1000;
  }

  /**
   * Sleep for specified milliseconds
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Check if error is retryable
   */
  static isRetryableError(error: AnalyticsQueryError): boolean {
    return error.retryable;
  }
}

/**
 * Circuit breaker pattern for preventing cascading failures
 */
export class CircuitBreaker {
  private failureCount: number = 0;
  private lastFailureTime: number = 0;
  private isOpen: boolean = false;
  private cooldownPeriod: number;
  private failureThreshold: number;

  constructor(
    failureThreshold: number = 5,
    cooldownPeriod: number = 60000 // 1 minute
  ) {
    this.failureThreshold = failureThreshold;
    this.cooldownPeriod = cooldownPeriod;
  }

  /**
   * Execute function with circuit breaker protection
   */
  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.isOpen) {
      if (this.shouldAttemptReset()) {
        this.isOpen = false;
        this.failureCount = 0;
      } else {
        throw new AnalyticsQueryError(
          ErrorCode.SERVICE_UNAVAILABLE,
          'Circuit breaker is open. Service temporarily unavailable.',
          { cooldownEndsAt: new Date(this.lastFailureTime + this.cooldownPeriod).toISOString() },
          true
        );
      }
    }

    try {
      const result = await fn();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  /**
   * Handle successful execution
   */
  private onSuccess(): void {
    this.failureCount = 0;
    this.isOpen = false;
  }

  /**
   * Handle failed execution
   */
  private onFailure(): void {
    this.failureCount++;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= this.failureThreshold) {
      this.isOpen = true;
      console.error('Circuit breaker opened due to repeated failures');
    }
  }

  /**
   * Check if circuit breaker should attempt to reset
   */
  private shouldAttemptReset(): boolean {
    return Date.now() - this.lastFailureTime > this.cooldownPeriod;
  }

  /**
   * Get circuit breaker state
   */
  getState(): {
    isOpen: boolean;
    failureCount: number;
    lastFailureTime: string | null;
    cooldownEndsAt: string | null;
  } {
    return {
      isOpen: this.isOpen,
      failureCount: this.failureCount,
      lastFailureTime: this.lastFailureTime ? new Date(this.lastFailureTime).toISOString() : null,
      cooldownEndsAt: this.lastFailureTime ? new Date(this.lastFailureTime + this.cooldownPeriod).toISOString() : null
    };
  }
}

/**
 * Error logging and monitoring
 */
export class ErrorLogger {
  private errorLog: Map<string, AnalyticsQueryError[]>;

  constructor() {
    this.errorLog = new Map();
  }

  /**
   * Log an error
   */
  log(error: AnalyticsQueryError, context?: string): void {
    const key = error.code;
    
    if (!this.errorLog.has(key)) {
      this.errorLog.set(key, []);
    }

    const errors = this.errorLog.get(key)!;
    errors.push(error);

    // Keep only last 100 errors per code
    if (errors.length > 100) {
      errors.shift();
    }

    // Log to console with context
    console.error(
      `[${error.timestamp}] ${error.code}: ${error.message}`,
      context ? `(Context: ${context})` : '',
      error.details ? JSON.stringify(error.details) : ''
    );
  }

  /**
   * Get error statistics
   */
  getStats(): {
    totalErrors: number;
    errorsByCode: Record<string, number>;
    recentErrors: AnalyticsQueryError[];
  } {
    let totalErrors = 0;
    const errorsByCode: Record<string, number> = {};
    const recentErrors: AnalyticsQueryError[] = [];

    for (const [code, errors] of this.errorLog.entries()) {
      totalErrors += errors.length;
      errorsByCode[code] = errors.length;
      recentErrors.push(...errors.slice(-10));
    }

    return {
      totalErrors,
      errorsByCode,
      recentErrors: recentErrors.sort((a, b) => 
        new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      )
    };
  }
}

/**
 * Singleton instances
 */
export const retryHandler = new RetryHandler();
export const circuitBreaker = new CircuitBreaker();
export const errorLogger = new ErrorLogger();
