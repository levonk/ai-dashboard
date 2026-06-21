---
story_id: "02-003"
story_title: "Proxy Data Collection"
story_name: "proxy-collection"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 2
parallel_id: 3
branch: "feature/current/prd-ai-analytics/story-02-003-proxy-collection"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["01-003"]
parallel_safe: true
modules: ["proxy", "collection"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "proxy", "collection"]
due: "2025-02-28"
created_at: "2025-06-20"
updated_at: "2025-06-20"
---

## Summary

Implement data collection functionality in the proxy service to capture telemetry from AI requests and responses. The proxy collects comprehensive analytics including request metadata, timing metrics, provider information, model usage, and response data, then writes directly to the database in analytics mode.

## Sub-Tasks

- [x] Implement request metadata extraction (AI client, user, pipeline stage) — target: apps/proxy/src/collection/metadata.rs
- [x] Add timing metrics collection (request duration, provider latency) — target: apps/proxy/src/collection/timing.rs
- [x] Implement token counting and cost estimation — target: apps/proxy/src/collection/tokens.rs
- [x] Add error tracking and classification — target: apps/proxy/src/collection/errors.rs
- [x] Create database write operations for analytics mode — target: apps/proxy/src/collection/database.rs
- [x] Implement content hashing for request correlation — target: apps/proxy/src/collection/hashing.rs
- [x] Add multi-dimensional attribute collection — target: apps/proxy/src/collection/dimensions.rs
- [x] Create collection tests and validation — target: apps/proxy/tests/collection/

## Relevant Files

- `apps/proxy/src/collection/metadata.rs` — Request metadata extraction
- `apps/proxy/src/collection/timing.rs` — Timing metrics collection
- `apps/proxy/src/collection/tokens.rs` — Token counting and cost estimation
- `apps/proxy/src/collection/errors.rs` — Error tracking and classification
- `apps/proxy/src/collection/database.rs` — Database write operations
- `apps/proxy/src/collection/hashing.rs` — Content hashing for correlation
- `apps/proxy/src/collection/dimensions.rs` — Multi-dimensional attribute collection
- `apps/proxy/tests/collection/` — Collection tests
- `docs/collection-guide.md` — Data collection guide
- [ ] Implement OmniRoute analytics collector — target: src/collectors/omniroute.ts
- [ ] Implement Iron-Proxy analytics collector — target: src/collectors/iron-proxy.ts
- [ ] Create generic custom stage collector — target: src/collectors/custom-stage.ts
- [ ] Add pre-optimization collector for original requests — target: src/collectors/pre-optimization.ts
- [ ] Implement post-optimization collector for transformed requests — target: src/collectors/post-optimization.ts
- [ ] Create collector integration tests with mock pipeline — target: test/collectors/integration/
- [ ] Add collector configuration examples and documentation — target: docs/collector-examples.md

## Relevant Files

- `src/collectors/headroom.ts` — Headroom stage collector
- `src/collectors/omniroute.ts` — OmniRoute stage collector
- `src/collectors/iron-proxy.ts` — Iron-Proxy stage collector
- `src/collectors/custom-stage.ts` — Generic custom stage collector
- `src/collectors/pre-optimization.ts` — Pre-optimization collector
- `src/collectors/post-optimization.ts` — Post-optimization collector
- `test/collectors/integration/` — Integration tests
- `docs/collector-examples.md` — Usage examples
- `examples/collectors/` — Sample collector configurations

## Acceptance Criteria

- [ ] Headroom collector captures pre-routing analytics
- [ ] OmniRoute collector captures routing and optimization metrics
- [ ] Iron-Proxy collector captures final provider interaction metrics
- [ ] Custom stage collector works for user-defined pipeline stages
- [ ] Pre/post optimization collectors enable compression analytics
- [ ] All collectors use standardized metadata schema
- [ ] Integration tests verify collector behavior in mock pipeline
- [ ] Documentation provides clear integration examples

## Test Plan

- Unit: `npm test src/collectors/headroom.ts`
- Unit: `npm test src/collectors/omniroute.ts`
- Unit: `npm test src/collectors/iron-proxy.ts`
- Integration: Test collectors in mock pipeline environment
- E2E: Verify analytics flow through multiple collectors

## Observability

- Monitor collector success/failure rates
- Track collection latency per stage
- Log collector configuration changes
- Alert on collector failures or anomalies

## Compliance

- Ensure collectors don't capture sensitive request content
- Validate that API keys are only accessible to Iron-Proxy collector
- Document data retention policies for collected metrics

## Risks & Mitigations

- Risk: Collector failures may block pipeline — Mitigation: Implement async collection with error isolation
- Risk: Different pipeline stages may have incompatible data — Mitigation: Use standardized metadata schema
- Risk: Custom collectors may have security issues — Mitigation: Validate collector code and limit permissions

## Dependencies

- 01-003: Basic Collector Framework Architecture (collectors depend on framework interfaces)

## Notes

- Design collectors to be non-blocking for pipeline performance
- Focus on metadata collection rather than full request content
- Consider collector ordering and dependencies between stages
- Make custom collector SDK simple and well-documented