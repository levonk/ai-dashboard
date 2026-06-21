---
story_id: "02-002"
story_title: "Content Hashing and Token Estimation"
story_name: "hashing-token-estimation"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 2
parallel_id: 2
branch: "feature/current/prd-multi-tenant-ai-analytics/story-02-002-hashing-token-estimation"
status: "in-progress"
assignee: ""
reviewer: ""
dependencies: ["01-002"]
parallel_safe: true
modules: ["utils", "processing"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "utils", "processing"]
due: "2025-02-28"
created_at: "2025-06-20"
updated_at: "2025-06-20"
---

## Summary

Implement content hashing for request correlation across pipeline stages and accurate token estimation for different AI models and input types. These utilities are essential for tracking requests through the analytics pipeline and calculating costs.

## Sub-Tasks

- [x] Implement content hashing algorithm for request fingerprinting — target: src/hashing.rs
- [~] Create token estimation for text/chat models — target: src/utils/tokens/text.rs
- [x] Implement token estimation for image models — target: src/image_tokens.rs
- [x] Add token estimation for audio/video models — target: src/media_tokens.rs
- [x] Create model-specific token estimation registry — target: src/token_registry.rs
- [x] Implement timing utilities for high-precision metrics — target: src/timing.rs
- [x] Add input type detection and classification — target: src/input_type.rs
- [x] Create utility test suite with model-specific test cases — target: tests/utils_integration_test.rs

## Relevant Files

- `src/hashing.rs` — Content hashing for request correlation
- `src/tokens.rs` — Text token estimation and model-specific token counting
- `src/image_tokens.rs` — Image token estimation
- `src/media_tokens.rs` — Audio/video token estimation
- `src/token_registry.rs` — Model-specific token registry
- `src/timing.rs` — High-precision timing utilities
- `src/input_type.rs` — Input type detection
- `tests/utils/` — Utility tests
- `docs/token-estimation.md` — Token estimation documentation

## Acceptance Criteria

- [x] Content hashing produces consistent fingerprints for identical requests
- [x] Token estimation is accurate for major model providers (Anthropic, OpenAI, Google)
- [x] Token estimation supports different input types (text, image, audio, video)
- [x] Model registry allows easy addition of new models
- [x] Timing utilities provide microsecond precision
- [x] Input type detection correctly classifies different content types
- [x] Test suite covers major models and edge cases
- [x] Documentation explains token estimation methodology

## Test Plan

- Unit: `cargo test hashing`
- Unit: `cargo test tokens`
- Unit: `cargo test timing`
- Accuracy: Compare token estimates against actual API usage
- Performance: Benchmark hashing and token estimation performance

## Observability

- Log token estimation accuracy vs actual usage
- Monitor hashing collision rates
- Track timing utility performance
- Alert on unsupported model token estimation

## Compliance

- Ensure hashing doesn't expose sensitive content
- Document token estimation limitations and accuracy
- Support regional model variations in token counting

## Risks & Mitigations

- Risk: Token estimation may be inaccurate for new models — Mitigation: Make registry easily extensible
- Risk: Hashing collisions may cause incorrect correlation — Mitigation: Use strong hashing algorithm and monitor collisions
- Risk: Timing overhead may impact performance — Mitigation: Optimize timing utilities and use only when needed

## Dependencies

- 01-002: Core Data Model and Schema Design (database must support hashed request IDs and token counts)

## Notes

- Focus on accuracy for major models first, add niche models over time
- Consider provider-specific token counting differences
- Hash algorithm should be fast but minimize collisions
- Token estimation should handle both input and output tokens