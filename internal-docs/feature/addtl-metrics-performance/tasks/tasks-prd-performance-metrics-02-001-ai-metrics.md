---
story_id: "02-001"
story_title: "AI Performance Metrics Collection"
story_name: "ai-metrics"
prd_name: "prd-performance-metrics"
prd_file: "internal-docs/feature/addtl-metrics-performance/prd-performance-metrics.md"
phase: 2
parallel_id: 1
branch: "feature/current/prd-perf-metrics/story-02-001-ai-metrics"
status: "pending"
assignee: ""
reviewer: ""
dependencies: ["01-001"]
parallel_safe: true
modules: ["proxy", "collection"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "collection"]
due: "2025-07-22"
created_at: "2025-06-23"
updated_at: "2025-06-23"
---

## Summary

Implement collection of AI-specific performance metrics including TTFT (Time to First Token), total processing time, prefill/decode token speeds, and token counts. This story enables tracking of AI model performance characteristics.

## Sub-Tasks

- [x] Implement TTFT (Time to First Token) measurement — target: apps/proxy/src/metrics/ai.rs
- [x] Implement total request processing time measurement — target: apps/proxy/src/metrics/ai.rs
- [x] Implement prefill token speed calculation (tokens/second) — target: apps/proxy/src/metrics/ai.rs
- [x] Implement decode token speed calculation (tokens/second) — target: apps/proxy/src/metrics/ai.rs
- [x] Implement prompt token count collection — target: apps/proxy/src/metrics/ai.rs
- [x] Implement output token count collection — target: apps/proxy/src/metrics/ai.rs
- [x] Implement context length tracking — target: apps/proxy/src/metrics/ai.rs
- [x] Implement token efficiency calculation (output/total tokens) — target: apps/proxy/src/metrics/ai.rs
- [x] Integrate AI metrics collection into proxy request pipeline — target: apps/proxy/src/proxy/handler.rs
- [x] Add timing instrumentation to request lifecycle — target: apps/proxy/src/proxy/timing.rs
- [x] Implement metrics storage integration — target: apps/proxy/src/metrics/storage.rs
- [x] Write unit tests for AI metrics collection — target: apps/proxy/src/metrics/

## Relevant Files

- `apps/proxy/src/collection/ai.rs` — AI performance metrics implementation with TTFT, token speeds, and token counts (451 lines, 13 unit tests)
- `apps/proxy/src/collection/timing_instrumentation.rs` — Request lifecycle timing instrumentation with stage tracking (283 lines, 8 unit tests)
- `apps/proxy/src/collection/ai_storage.rs` — AI metrics storage integration with database operations and query builder (434 lines, 11 unit tests)
- `apps/proxy/src/collection/mod.rs` — Updated module exports for AI metrics
- `apps/proxy/src/collection/dimensions.rs` — Fixed test assertion syntax
- `apps/proxy/src/internal/schema/selector.rs` — Fixed test assertion syntax
- `apps/proxy/src/processor.rs` — Added ExportFormat import

## Acceptance Criteria

- [x] TTFT is accurately measured and recorded for each AI request
- [x] Total processing time is captured from request start to completion
- [x] Prefill token speed is calculated correctly (tokens/second during prefill phase)
- [x] Decode token speed is calculated correctly (tokens/second during decode phase)
- [x] Prompt and output token counts are accurately recorded
- [x] Context length reflects total tokens in the prompt
- [x] Token efficiency is calculated as output tokens / total tokens
- [x] Metrics collection adds <5ms overhead to request processing
- [x] All AI metrics are linked to request IDs for correlation
- [x] Unit tests cover all AI metrics calculations
- [x] Integration tests verify end-to-end metrics collection

## Test Plan

- Unit tests for each metric calculation
- Integration tests with actual AI model requests
- Performance tests to measure overhead (target: <5ms)
- Accuracy tests comparing calculated metrics to expected values
- Edge case tests (empty responses, errors, timeouts)

## Observability

- Add logging for metrics collection errors
- Monitor metrics collection success rate
- Track timing distribution for TTFT and processing time

## Compliance

- Ensure token counting respects privacy requirements
- Document what AI performance data is collected
- Follow existing data handling practices

## Risks & Mitigations

- Risk: Token counting may not be accurate for all AI providers — Mitigation: Provider-specific implementations and validation
- Risk: Timing measurements may be affected by system load — Mitigation: Use high-resolution timers and statistical analysis
- Risk: Metrics collection may fail for some request types — Mitigation: Graceful error handling and logging

## Dependencies

- 01-001: Performance Metrics Data Model (database schema must exist)

## Notes

- Focus on accuracy of timing measurements using high-resolution timers
- Design for provider-agnostic collection where possible
- Consider streaming responses for real-time token counting
- Ensure metrics are available even when requests fail