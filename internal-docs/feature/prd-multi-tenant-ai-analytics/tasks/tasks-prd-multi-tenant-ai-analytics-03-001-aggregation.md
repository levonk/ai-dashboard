---
story_id: "03-001"
story_title: "On-Demand Analytics Queries"
story_name: "analytics-queries"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 3
parallel_id: 1
branch: "feature/current/prd-ai-analytics/story-03-001-analytics-queries"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["02-001", "02-002", "02-003"]
parallel_safe: true
modules: ["web", "analytics"]
priority: "MUST"
risk_level: "high"
tags: ["feat", "analytics", "queries"]
due: "2025-03-31"
created_at: "2025-06-20"
updated_at: "2025-06-20"
---

## Summary

Implement on-demand analytics query system in the web service to support dashboard visualization and data exploration. This system executes SQL queries against the PostgreSQL database to generate aggregated metrics, trends, and comparisons on-demand for dashboard display.

## Sub-Tasks

- [ ] Design query API and response schemas — target: apps/web/src/app/api/analytics/query/route.ts
- [ ] Implement multi-dimensional filtering queries — target: apps/web/src/lib/analytics/filters.ts
- [ ] Add time-series aggregation queries — target: apps/web/src/lib/analytics/timeseries.ts
- [ ] Create comparative analysis queries (providers, models, clients) — target: apps/web/src/lib/analytics/comparison.ts
- [ ] Implement cost calculation queries — target: apps/web/src/lib/analytics/costs.ts
- [ ] Add query optimization and caching — target: apps/web/src/lib/analytics/cache.ts
- [ ] Create query validation and sanitization — target: apps/web/src/lib/analytics/validation.ts
- [ ] Add query tests and performance benchmarks — target: apps/web/tests/analytics/

## Relevant Files

- `apps/web/src/app/api/analytics/query/route.ts` — Query API endpoints
- `apps/web/src/lib/analytics/filters.ts` — Multi-dimensional filtering
- `apps/web/src/lib/analytics/timeseries.ts` — Time-series aggregation
- `apps/web/src/lib/analytics/comparison.ts` — Comparative analysis
- `apps/web/src/lib/analytics/costs.ts` — Cost calculation
- `apps/web/src/lib/analytics/cache.ts` — Query caching
- `apps/web/src/lib/analytics/validation.ts` — Query validation
- `apps/web/tests/analytics/` — Analytics query tests
- `docs/analytics-query-guide.md` — Analytics query guide
- [ ] Implement event ingestion pipeline — target: src/analytics/streaming/ingestion.ts
- [ ] Create real-time aggregation functions — target: src/analytics/streaming/aggregation.ts
- [ ] Implement sliding window time-series aggregation — target: src/analytics/streaming/windows.ts
- [ ] Add multi-dimensional filtering and grouping — target: src/analytics/streaming/filters.ts
- [ ] Create real-time metric storage layer — target: src/analytics/streaming/storage.ts
- [ ] Implement stream processing error handling and recovery — target: src/analytics/streaming/errors.ts
- [ ] Add real-time aggregation performance monitoring — target: src/analytics/streaming/monitoring.ts

## Relevant Files

- `src/analytics/streaming/architecture.ts` — Stream processing design
- `src/analytics/streaming/ingestion.ts` — Event ingestion pipeline
- `src/analytics/streaming/aggregation.ts` — Real-time aggregation functions
- `src/analytics/streaming/windows.ts` — Sliding window implementation
- `src/analytics/streaming/filters.ts` — Multi-dimensional filtering
- `src/analytics/streaming/storage.ts` — Real-time metric storage
- `src/analytics/streaming/errors.ts` — Error handling and recovery
- `src/analytics/streaming/monitoring.ts` — Performance monitoring
- `test/analytics/streaming/` — Stream processing tests
- `docs/streaming-architecture.md` — Architecture documentation

## Acceptance Criteria

- [ ] Stream processing handles 10,000+ events/second
- [ ] Real-time aggregation latency <5 seconds
- [ ] Sliding windows support multiple time ranges (1m, 5m, 15m, 1h)
- [ ] Multi-dimensional filtering works across all dimensions
- [ ] Error handling prevents stream processing failures
- [ ] Storage layer supports high-frequency writes
- [ ] Monitoring provides visibility into stream health
- [ ] System gracefully handles backpressure and spikes

## Test Plan

- Unit: `npm test src/analytics/streaming/ingestion.ts`
- Unit: `npm test src/analytics/streaming/aggregation.ts`
- Load: Test with 10,000+ events/second
- Latency: Verify <5 second aggregation latency
- Failure: Test error handling and recovery scenarios

## Observability

- Monitor stream processing throughput and latency
- Track aggregation window processing times
- Alert on stream backpressure or failures
- Log filter performance and cardinality

## Compliance

- Ensure real-time processing doesn't expose sensitive data
- Implement data retention for real-time metrics
- Support real-time data deletion requests

## Risks & Mitigations

- Risk: High throughput may overwhelm system — Mitigation: Implement backpressure handling and scaling
- Risk: Real-time aggregation may be resource-intensive — Mitigation: Optimize algorithms and use efficient data structures
- Risk: Stream failures may lose data — Mitigation: Implement durable queuing and recovery mechanisms

## Dependencies

- 02-001: Standardized Metadata Schema (aggregation depends on consistent metadata)
- 02-002: Content Hashing and Token Estimation (aggregation uses hashed request IDs and token counts)
- 02-003: Pipeline Stage Collectors (aggregation processes collected analytics events)

## Notes

- Focus on horizontal scalability for stream processing
- Consider using Redis or similar for real-time metric storage
- Design for fault tolerance and automatic recovery
- Balance real-time freshness with system stability