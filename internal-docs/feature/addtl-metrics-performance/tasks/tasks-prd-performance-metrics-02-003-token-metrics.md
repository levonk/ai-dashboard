---
story_id: "02-003"
story_title: "Token Processing Metrics"
story_name: "token-metrics"
prd_name: "prd-performance-metrics"
prd_file: "internal-docs/feature/addtl-metrics-performance/prd-performance-metrics.md"
phase: 2
parallel_id: 3
branch: "feature/current/prd-perf-metrics/story-02-003-token-metrics"
status: "pending"
assignee: ""
reviewer: ""
dependencies: ["01-001"]
parallel_safe: true
modules: ["proxy", "processing"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "collection"]
due: "2025-07-22"
created_at: "2025-06-23"
updated_at: "2025-06-23"
---

## Summary

Implement detailed token processing metrics to provide insights into token generation efficiency, throughput, and patterns. This story complements the AI performance metrics with deeper token-level analysis.

## Sub-Tasks

- [x] Implement token throughput tracking (tokens/second overall) — target: apps/proxy/src/metrics/tokens.rs
- [x] Implement token generation rate analysis — target: apps/proxy/src/metrics/tokens.rs
- [x] Implement token cost estimation per request — target: apps/proxy/src/metrics/tokens.rs
- [x] Implement token caching efficiency metrics — target: apps/proxy/src/metrics/tokens.rs
- [x] Implement token streaming metrics (if applicable) — target: apps/proxy/src/metrics/tokens.rs
- [x] Add token-level correlation with AI provider responses — target: apps/proxy/src/metrics/tokens.rs
- [x] Implement token metrics aggregation functions — target: apps/proxy/src/metrics/aggregation.rs
- [x] Integrate token metrics into request pipeline — target: apps/proxy/src/analytics.rs
- [x] Add token metrics storage integration — target: apps/proxy/src/collection/token_storage.rs
- [x] Write unit tests for token metrics — target: apps/proxy/src/collection/

## Relevant Files

- `apps/proxy/src/collection/tokens.rs` — Token processing metrics implementation (extended with throughput, generation rate, caching, streaming, and provider correlation)
- `apps/proxy/src/collection/aggregation.rs` — Token metrics aggregation functions (new file)
- `apps/proxy/src/analytics.rs` — Request pipeline integration (extended with token metrics processing)
- `apps/proxy/src/collection/token_storage.rs` — Metrics storage integration (new file)
- `apps/proxy/src/collection/mod.rs` — Module exports updated

## Acceptance Criteria

- [x] Token throughput is accurately calculated and recorded
- [x] Token generation rate analysis provides meaningful insights
- [x] Token cost estimation is accurate per request
- [x] Token caching efficiency is tracked when caching is enabled
- [x] Token streaming metrics capture real-time generation patterns
- [x] Token metrics are correlated with AI provider responses
- [x] Token metrics aggregation functions provide useful summaries
- [x] Token metrics collection adds minimal overhead (<2ms)
- [x] All token metrics are linked to request IDs
- [x] Unit tests cover all token metric calculations
- [x] Integration tests verify end-to-end token metrics collection

## Test Plan

- Unit tests for each token metric calculation
- Integration tests with actual AI model requests
- Performance tests to measure overhead (target: <2ms)
- Accuracy tests comparing calculated metrics to expected values
- Aggregation tests for summary statistics

## Observability

- Add logging for token metrics collection errors
- Monitor token metrics collection success rate
- Track token throughput distribution over time

## Compliance

- Ensure token metrics respect privacy requirements
- Document what token-level data is collected
- Follow existing data handling practices

## Risks & Mitigations

- Risk: Token counting may vary between AI providers — Mitigation: Provider-specific implementations and normalization
- Risk: Token cost estimation may be inaccurate — Mitigation: Regular updates to pricing models and validation
- Risk: Token streaming metrics may be complex to capture — Mitigation: Simplified metrics for initial implementation

## Dependencies

- 01-001: Performance Metrics Data Model (database schema must exist)

## Notes

- Focus on actionable token metrics for optimization
- Design for provider-agnostic collection where possible
- Consider future enhancements for advanced token analysis
- Ensure token metrics are available even when requests fail