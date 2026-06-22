/**
 * Query Result Pagination
 * 
 * Handles pagination for query results to efficiently handle large datasets
 */

import { QueryResultRow, AnalyticsQueryResponse } from './types';

export interface PaginationOptions {
  limit: number;
  offset: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResult<T> {
  data: T[];
  pagination: {
    total: number;
    limit: number;
    offset: number;
    hasMore: boolean;
    currentPage: number;
    totalPages: number;
  };
}

export class QueryPaginator {
  private defaultLimit: number;
  private maxLimit: number;

  constructor(defaultLimit: number = 100, maxLimit: number = 10000) {
    this.defaultLimit = defaultLimit;
    this.maxLimit = maxLimit;
  }

  /**
   * Paginate query results
   */
  paginate<T extends QueryResultRow>(
    results: T[],
    options: PaginationOptions
  ): PaginatedResult<T> {
    // Validate and normalize options
    const normalizedOptions = this.normalizeOptions(options);

    // Sort results if requested
    let sortedResults = results;
    if (normalizedOptions.sortBy) {
      sortedResults = this.sortResults(results, normalizedOptions.sortBy, normalizedOptions.sortOrder);
    }

    // Apply pagination
    const total = sortedResults.length;
    const offset = normalizedOptions.offset;
    const limit = normalizedOptions.limit;
    
    const paginatedData = sortedResults.slice(offset, offset + limit);
    const hasMore = offset + limit < total;

    const currentPage = Math.floor(offset / limit) + 1;
    const totalPages = Math.ceil(total / limit);

    return {
      data: paginatedData,
      pagination: {
        total,
        limit,
        offset,
        hasMore,
        currentPage,
        totalPages
      }
    };
  }

  /**
   * Normalize pagination options
   */
  private normalizeOptions(options: PaginationOptions): PaginationOptions {
    return {
      limit: Math.min(
        Math.max(options.limit || this.defaultLimit, 1),
        this.maxLimit
      ),
      offset: Math.max(options.offset || 0, 0),
      sortBy: options.sortBy,
      sortOrder: options.sortOrder || 'asc'
    };
  }

  /**
   * Sort results by field
   */
  private sortResults<T extends QueryResultRow>(
    results: T[],
    sortBy: string,
    sortOrder: 'asc' | 'desc' = 'asc'
  ): T[] {
    return [...results].sort((a, b) => {
      let aValue: any;
      let bValue: any;

      // Try to get value from metrics first
      if (a.metrics && a.metrics[sortBy]) {
        aValue = a.metrics[sortBy];
        bValue = b.metrics[sortBy];
      }
      // Then try dimensions
      else if (a.dimensions && a.dimensions[sortBy]) {
        aValue = a.dimensions[sortBy];
        bValue = b.dimensions[sortBy];
      }
      // Then try direct property
      else if ((a as any)[sortBy]) {
        aValue = (a as any)[sortBy];
        bValue = (b as any)[sortBy];
      } else {
        return 0;
      }

      // Handle different types
      if (typeof aValue === 'number' && typeof bValue === 'number') {
        return sortOrder === 'asc' ? aValue - bValue : bValue - aValue;
      }

      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return sortOrder === 'asc' 
          ? aValue.localeCompare(bValue)
          : bValue.localeCompare(aValue);
      }

      // Fallback to string comparison
      const aStr = String(aValue);
      const bStr = String(bValue);
      return sortOrder === 'asc' 
        ? aStr.localeCompare(bStr)
        : bStr.localeCompare(aStr);
    });
  }

  /**
   * Create pagination metadata for API response
   */
  createPaginationMetadata(
    total: number,
    limit: number,
    offset: number
  ): AnalyticsQueryResponse['pagination'] {
    const hasMore = offset + limit < total;
    const currentPage = Math.floor(offset / limit) + 1;
    const totalPages = Math.ceil(total / limit);

    return {
      total,
      limit,
      offset,
      hasMore
    };
  }

  /**
   * Get pagination info from request
   */
  static getPaginationFromRequest(request: {
    limit?: number;
    offset?: number;
    page?: number;
    pageSize?: number;
  }): PaginationOptions {
    // Support both limit/offset and page/pageSize patterns
    const limit = request.limit || request.pageSize || 100;
    const offset = request.offset || ((request.page || 1) - 1) * (request.pageSize || 100);

    return {
      limit,
      offset
    };
  }

  /**
   * Get next page parameters
   */
  getNextPageParams(current: PaginationOptions, total: number): PaginationOptions | null {
    const nextOffset = current.offset + current.limit;
    if (nextOffset >= total) {
      return null;
    }

    return {
      ...current,
      offset: nextOffset
    };
  }

  /**
   * Get previous page parameters
   */
  getPrevPageParams(current: PaginationOptions): PaginationOptions | null {
    const prevOffset = current.offset - current.limit;
    if (prevOffset < 0) {
      return null;
    }

    return {
      ...current,
      offset: Math.max(0, prevOffset)
    };
  }

  /**
   * Calculate optimal page size based on data characteristics
   */
  calculateOptimalPageSize(totalRecords: number, avgRecordSize: number): number {
    // Target ~1MB per page
    const targetPageSize = 1024 * 1024 / avgRecordSize;
    
    // Clamp between reasonable bounds
    return Math.min(
      Math.max(Math.round(targetPageSize), 10),
      this.maxLimit
    );
  }
}

/**
 * Cursor-based pagination alternative for large datasets
 */
export class CursorPaginator {
  /**
   * Create a cursor from a result
   */
  static createCursor(result: QueryResultRow, index: number): string {
    const cursorData = {
      index,
      timestamp: result.timestamp || Date.now(),
      id: result.dimensions?.id || index
    };
    return Buffer.from(JSON.stringify(cursorData)).toString('base64');
  }

  /**
   * Parse a cursor
   */
  static parseCursor(cursor: string): any {
    try {
      const data = Buffer.from(cursor, 'base64').toString('utf-8');
      return JSON.parse(data);
    } catch (error) {
      return null;
    }
  }

  /**
   * Get results after a cursor
   */
  static getResultsAfterCursor<T extends QueryResultRow>(
    results: T[],
    cursor: string,
    limit: number
  ): T[] {
    const cursorData = this.parseCursor(cursor);
    if (!cursorData) {
      return results.slice(0, limit);
    }

    const startIndex = cursorData.index + 1;
    return results.slice(startIndex, startIndex + limit);
  }
}

/**
 * Singleton instance
 */
export const queryPaginator = new QueryPaginator();
