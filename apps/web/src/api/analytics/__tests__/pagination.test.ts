/**
 * Tests for Query Pagination
 */

import { QueryPaginator, CursorPaginator, PaginationOptions } from '../pagination';
import { QueryResultRow } from '../types';

describe('QueryPaginator', () => {
  let paginator: QueryPaginator;

  beforeEach(() => {
    paginator = new QueryPaginator();
  });

  const createMockResults = (count: number): QueryResultRow[] => {
    return Array.from({ length: count }, (_, i) => ({
      dimensions: {
        provider: i % 2 === 0 ? 'anthropic' : 'openai',
        model: i % 3 === 0 ? 'claude-3.5-opus' : 'gpt-4'
      },
      metrics: {
        requestCount: 100 + i * 10,
        cost: 10 + i
      }
    }));
  };

  describe('Basic pagination', () => {
    it('should paginate results correctly', () => {
      const results = createMockResults(25);
      const options: PaginationOptions = {
        limit: 10,
        offset: 0
      };

      const paginated = paginator.paginate(results, options);

      expect(paginated.data).toHaveLength(10);
      expect(paginated.pagination.total).toBe(25);
      expect(paginated.pagination.limit).toBe(10);
      expect(paginated.pagination.offset).toBe(0);
      expect(paginated.pagination.hasMore).toBe(true);
      expect(paginated.pagination.currentPage).toBe(1);
      expect(paginated.pagination.totalPages).toBe(3);
    });

    it('should handle last page', () => {
      const results = createMockResults(25);
      const options: PaginationOptions = {
        limit: 10,
        offset: 20
      };

      const paginated = paginator.paginate(results, options);

      expect(paginated.data).toHaveLength(5);
      expect(paginated.pagination.hasMore).toBe(false);
      expect(paginated.pagination.currentPage).toBe(3);
    });

    it('should handle empty results', () => {
      const results: QueryResultRow[] = [];
      const options: PaginationOptions = {
        limit: 10,
        offset: 0
      };

      const paginated = paginator.paginate(results, options);

      expect(paginated.data).toHaveLength(0);
      expect(paginated.pagination.total).toBe(0);
      expect(paginated.pagination.hasMore).toBe(false);
    });

    it('should handle offset beyond results', () => {
      const results = createMockResults(10);
      const options: PaginationOptions = {
        limit: 10,
        offset: 100
      };

      const paginated = paginator.paginate(results, options);

      expect(paginated.data).toHaveLength(0);
      expect(paginated.pagination.hasMore).toBe(false);
    });
  });

  describe('Option normalization', () => {
    it('should use default limit when not provided', () => {
      const results = createMockResults(200);
      const options: PaginationOptions = {
        limit: 0, // Will be normalized to default
        offset: 0
      };

      const paginated = paginator.paginate(results, options);

      expect(paginated.data).toHaveLength(100); // default limit
    });

    it('should enforce maximum limit', () => {
      const results = createMockResults(20000);
      const options: PaginationOptions = {
        limit: 20000, // Will be clamped to max
        offset: 0
      };

      const paginated = paginator.paginate(results, options);

      expect(paginated.pagination.limit).toBe(10000); // max limit
    });

    it('should prevent negative offset', () => {
      const results = createMockResults(10);
      const options: PaginationOptions = {
        limit: 10,
        offset: -10
      };

      const paginated = paginator.paginate(results, options);

      expect(paginated.pagination.offset).toBe(0);
    });
  });

  describe('Sorting', () => {
    it('should sort by metric field ascending', () => {
      const results = createMockResults(10);
      const options: PaginationOptions = {
        limit: 10,
        offset: 0,
        sortBy: 'requestCount',
        sortOrder: 'asc'
      };

      const paginated = paginator.paginate(results, options);

      const requestCounts = paginated.data.map(r => r.metrics.requestCount);
      for (let i = 1; i < requestCounts.length; i++) {
        expect(requestCounts[i]).toBeGreaterThanOrEqual(requestCounts[i - 1]);
      }
    });

    it('should sort by metric field descending', () => {
      const results = createMockResults(10);
      const options: PaginationOptions = {
        limit: 10,
        offset: 0,
        sortBy: 'cost',
        sortOrder: 'desc'
      };

      const paginated = paginator.paginate(results, options);

      const costs = paginated.data.map(r => r.metrics.cost);
      for (let i = 1; i < costs.length; i++) {
        expect(costs[i]).toBeLessThanOrEqual(costs[i - 1]);
      }
    });

    it('should sort by dimension field', () => {
      const results = createMockResults(10);
      const options: PaginationOptions = {
        limit: 10,
        offset: 0,
        sortBy: 'provider',
        sortOrder: 'asc'
      };

      const paginated = paginator.paginate(results, options);

      const providers = paginated.data.map(r => r.dimensions?.provider);
      const sortedProviders = [...providers].sort();
      expect(providers).toEqual(sortedProviders);
    });
  });

  describe('Pagination metadata', () => {
    it('should create correct pagination metadata', () => {
      const metadata = paginator.createPaginationMetadata(100, 10, 0);

      expect(metadata.total).toBe(100);
      expect(metadata.limit).toBe(10);
      expect(metadata.offset).toBe(0);
      expect(metadata.hasMore).toBe(true);
    });

    it('should indicate no more results on last page', () => {
      const metadata = paginator.createPaginationMetadata(100, 10, 95);

      expect(metadata.hasMore).toBe(false);
    });
  });

  describe('Pagination helpers', () => {
    it('should get pagination from request with limit/offset', () => {
      const request = {
        limit: 50,
        offset: 100
      };

      const options = QueryPaginator.getPaginationFromRequest(request);

      expect(options.limit).toBe(50);
      expect(options.offset).toBe(100);
    });

    it('should get pagination from request with page/pageSize', () => {
      const request = {
        page: 3,
        pageSize: 25
      };

      const options = QueryPaginator.getPaginationFromRequest(request);

      expect(options.limit).toBe(25);
      expect(options.offset).toBe(50); // (3-1) * 25
    });

    it('should calculate next page params', () => {
      const current: PaginationOptions = {
        limit: 10,
        offset: 0
      };

      const next = paginator.getNextPageParams(current, 100);

      expect(next?.offset).toBe(10);
    });

    it('should return null for next page when at end', () => {
      const current: PaginationOptions = {
        limit: 10,
        offset: 95
      };

      const next = paginator.getNextPageParams(current, 100);

      expect(next).toBeNull();
    });

    it('should calculate previous page params', () => {
      const current: PaginationOptions = {
        limit: 10,
        offset: 20
      };

      const prev = paginator.getPrevPageParams(current);

      expect(prev?.offset).toBe(10);
    });

    it('should return null for previous page when at start', () => {
      const current: PaginationOptions = {
        limit: 10,
        offset: 0
      };

      const prev = paginator.getPrevPageParams(current);

      expect(prev).toBeNull();
    });

    it('should calculate optimal page size', () => {
      const optimalSize = paginator.calculateOptimalPageSize(100000, 1024);

      // Target ~1MB per page
      expect(optimalSize).toBeGreaterThan(0);
      expect(optimalSize).toBeLessThanOrEqual(10000); // max limit
    });
  });
});

describe('CursorPaginator', () => {
  describe('Cursor creation and parsing', () => {
    it('should create valid cursor', () => {
      const result: QueryResultRow = {
        dimensions: { id: 'test-id' },
        metrics: { requestCount: 100 },
        timestamp: '2024-01-01T00:00:00Z'
      };

      const cursor = CursorPaginator.createCursor(result, 5);

      expect(cursor).toBeTruthy();
      expect(typeof cursor).toBe('string');
    });

    it('should parse valid cursor', () => {
      const result: QueryResultRow = {
        dimensions: { id: 'test-id' },
        metrics: { requestCount: 100 },
        timestamp: '2024-01-01T00:00:00Z'
      };

      const cursor = CursorPaginator.createCursor(result, 5);
      const parsed = CursorPaginator.parseCursor(cursor);

      expect(parsed).not.toBeNull();
      expect(parsed.index).toBe(5);
    });

    it('should return null for invalid cursor', () => {
      const parsed = CursorPaginator.parseCursor('invalid-cursor');

      expect(parsed).toBeNull();
    });
  });

  describe('Cursor-based pagination', () => {
    it('should get results after cursor', () => {
      const results = Array.from({ length: 10 }, (_, i) => ({
        dimensions: { id: `id-${i}` },
        metrics: { requestCount: i * 10 }
      }));

      const cursor = CursorPaginator.createCursor(results[2], 2);
      const paginated = CursorPaginator.getResultsAfterCursor(results, cursor, 3);

      expect(paginated).toHaveLength(3);
      expect(paginated[0].dimensions?.id).toBe('id-3');
    });

    it('should handle invalid cursor by returning first page', () => {
      const results = Array.from({ length: 10 }, (_, i) => ({
        dimensions: { id: `id-${i}` },
        metrics: { requestCount: i * 10 }
      }));

      const paginated = CursorPaginator.getResultsAfterCursor(results, 'invalid', 3);

      expect(paginated).toHaveLength(3);
      expect(paginated[0].dimensions?.id).toBe('id-0');
    });

    it('should handle cursor beyond results', () => {
      const results = Array.from({ length: 5 }, (_, i) => ({
        dimensions: { id: `id-${i}` },
        metrics: { requestCount: i * 10 }
      }));

      const cursor = CursorPaginator.createCursor(results[4], 4);
      const paginated = CursorPaginator.getResultsAfterCursor(results, cursor, 3);

      expect(paginated).toHaveLength(0);
    });
  });
});
