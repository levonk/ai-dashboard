---
story_id: "02-002"
story_title: "Content Hashing and Token Estimation"
story_name: "hashing-token-estimation"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 2
parallel_id: 2
branch: "feature/current/prd-multi-tenant-ai-analytics/story-02-002-hashing-token-estimation"
status: "todo"
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

- [ ] Implement content hashing algorithm for request fingerprinting — target: src/utils/hashing.ts
- [ ] Create token estimation for text/chat models — target: src/utils/tokens/text.ts
- [ ] Implement token estimation for image models — target: src/utils/tokens/image.ts
- [ ] Add token estimation for audio/video models — target: src/utils/tokens/media.ts
- [ ] Create model-specific token estimation registry — target: src/utils/tokens/registry.ts
- [ ] Implement timing utilities for high-precision metrics — target: src/utils/timing.ts
- [ ] Add input type detection and classification — target: src/utils/input-type.ts
- [ ] Create utility test suite with model-specific test cases — target: test/utils/

## Relevant Files

- `src/utils/hashing.ts` — Content hashing for request correlation
- `src/utils/tokens/text.ts` — Text token estimation
- `src/utils/tokens/image.ts` — Image token estimation
- `src/utils/tokens/media.ts` — Audio/video token estimation
- `src/utils/tokens/registry.ts` — Model-specific token registry
- `src/utils/timing.ts` — High-precision timing utilities
- `src/utils/input-type.ts` — Input type detection
- `test/utils/` — Utility tests
- `docs/token-estimation.md` — Token estimation documentation

## Acceptance Criteria

- [ ] Content hashing produces consistent fingerprints for identical requests
- [ ] Token estimation is accurate for major model providers (Anthropic, OpenAI, Google)
- [ ] Token estimation supports different input types (text, image, audio, video)
- [ ] Model registry allows easy addition of new models
- [ ] Timing utilities provide microsecond precision
- [ ] Input type detection correctly classifies different content types
- [ ] Test suite covers major models and edge cases
- [ ] Documentation explains token estimation methodology

## Test Plan

- Unit: `npm test src/utils/hashing.ts`
- Unit: `npm test src/utils/tokens/`
- Unit: `npm test src/utils/timing.ts`
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