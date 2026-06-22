---
story_id: "03-002"
story_title: "Cost Calculation Engine"
story_name: "cost-calculation"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 3
parallel_id: 2
branch: "feature/current/prd-ai-analytics/story-03-002-cost-calculation"
status: "in-progress"
assignee: ""
reviewer: ""
dependencies: ["01-003"]
parallel_safe: true
modules: ["analytics-rs", "pricing"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "analytics-rs", "pricing"]
due: "2025-03-31"
created_at: "2025-06-20"
updated_at: "2025-06-20"
---

## Summary

Implement multi-provider cost calculation engine in the analytics-rs package with accurate pricing data for different AI models and input types. This engine is execution-context agnostic and can be used by proxy (open-source), analytics service (commercial), or Spark jobs (commercial batch processing).

## Sub-Tasks

- [x] Design pricing data model and update system — target: packages/analytics-rs/src/cost.rs
- [x] Implement provider-specific pricing (Anthropic, OpenAI, Google, etc.) — target: packages/analytics-rs/src/cost.rs
- [x] Add model-specific cost calculation — target: packages/analytics-rs/src/cost.rs
- [x] Implement input type cost factors (text, image, audio) — target: packages/analytics-rs/src/cost.rs
- [x] Create cost aggregation functions — target: packages/analytics-rs/src/cost.rs
- [x] Add budget monitoring and alerting — target: packages/analytics-rs/src/cost.rs
- [x] Implement cost optimization suggestions — target: packages/analytics-rs/src/cost.rs
- [x] Create pricing data update mechanism — target: packages/analytics-rs/src/cost.rs
- [x] Add cost calculation tests — target: packages/analytics-rs/tests/cost/
- [x] Create pricing data validation — target: packages/analytics-rs/tests/cost/pricing_data.rs

## Relevant Files

- `packages/analytics-rs/src/cost.rs` — Cost calculation engine (expanded with pricing data model, provider-specific pricing, model-specific costs, input type factors, cost aggregation, budget monitoring, optimization suggestions, and pricing data update mechanism)
- `packages/analytics-rs/src/pricing_data.rs` — Pricing data structures and validation (new file)
- `packages/analytics-rs/tests/cost_test.rs` — Cost calculation tests (new file with 16 comprehensive tests)
- `packages/analytics-rs/src/lib.rs` — Updated exports for new cost-related types
- `packages/analytics-rs/src/processing.rs` — Updated to use new CostCalculator instance methods

## Acceptance Criteria

- [x] Cost calculations are accurate within 0.1% of provider pricing
- [x] All major providers supported (Anthropic, OpenAI, Google, Microsoft, AWS, OpenRouter)
- [x] Model-specific pricing handles different input/output token costs
- [x] Input type cost factors correctly apply for text, image, audio
- [x] Cost aggregation works across all dimensions (client, provider, model, team)
- [x] Budget monitoring alerts trigger at configured thresholds
- [x] Cost optimization suggestions provide actionable recommendations
- [x] Pricing data can be updated without code changes
- [x] Test coverage exceeds 90% for cost calculation functions

## Test Plan

- Unit: `devbox run -- cargo test cost` (packages/analytics-rs)
- Validation: Test pricing data accuracy against provider documentation
- Integration: Test cost calculation with sample analytics data
- Edge cases: Zero costs, negative costs, extreme values
- Performance: Verify <50ms cost calculation for 10K records

## Observability

- Log cost calculation execution times
- Track pricing data version and update history
- Monitor budget alert triggers
- Track cost optimization suggestion adoption

## Compliance

- Ensure cost data doesn't expose sensitive usage patterns
- Support data deletion requests in cost calculations
- Maintain pricing data audit trail

## Risks & Mitigations

- Risk: Provider pricing changes frequently — Mitigation: Implement automated pricing data updates
- Risk: Complex pricing models may have edge cases — Mitigation: Comprehensive test coverage with provider-specific tests
- Risk: Cost calculations may be inaccurate for new models — Mitigation: Validation system for pricing data

## Dependencies

- 01-003: Analytics Package Foundation (cost calculation builds on the analytics package structure)

## Definition of Done

- Cost calculation engine implemented and tested
- All major providers supported with accurate pricing
- Budget monitoring and alerting functional
- Cost optimization suggestions working
- Documentation complete with pricing data format
- Code review approved
- All acceptance criteria verified

## Commit Conventions

- Use conventional commits with module scoping: `feat(analytics-rs): add cost calculation engine`
- Reference story ID in commit messages: "Related to 03-002 in PRD multi-tenant-ai-analytics"