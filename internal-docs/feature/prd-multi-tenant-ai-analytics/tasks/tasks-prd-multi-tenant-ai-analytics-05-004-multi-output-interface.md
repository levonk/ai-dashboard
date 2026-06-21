---
story_id: "05-004"
story_title: "Multi-Output Interface Support"
story_name: "multi-output-interface"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 5
parallel_id: 4
branch: "feature/current/prd-multi-tenant-ai-analytics/story-05-004-multi-output-interface"
status: "todo"
assignee: ""
reviewer: ""
dependencies: ["04-001", "04-002", "04-003"]
parallel_safe: true
modules: ["api", "frontend", "export"]
priority: "SHOULD"
risk_level: "medium"
tags: ["feat", "api", "frontend", "export", "ai-agents"]
due: "2025-04-30"
created_at: "2025-06-21"
updated_at: "2025-06-21"
---

## Summary

Implement multi-output interface support to serve both human users (HTML) and AI agents (Markdown/ToonFormat) from the same underlying data. This includes ToonFormat integration for bulk data transfer to minimize token usage, JSON for single record exchanges, and comprehensive AI agent interface support.

## Sub-Tasks

- [ ] Add ToonFormat library dependency and integration — target: packages/analytics-rs/Cargo.toml, src/export/toonformat.rs
- [ ] Implement ToonFormat encoder for bulk data export — target: src/export/formats/toonformat.ts
- [ ] Add JSON format for single record plaintext protocol exchanges — target: src/export/formats/json-single.ts
- [ ] Create AI agent interface endpoints with markdown output — target: src/api/endpoints/ai-agents.ts
- [ ] Implement multi-format content negotiation (HTML/Markdown/ToonFormat) — target: src/api/middleware/content-negotiation.ts
- [ ] Add AI agent detection and automatic format selection — target: src/api/middleware/agent-detection.ts
- [ ] Update dashboard to support AI agent view mode — target: frontend/src/components/ai-agent-view/
- [ ] Create ToonFormat compression utilities for token optimization — target: src/export/toonformat-optimizer.ts
- [ ] Add API documentation for AI agent interfaces — target: docs/ai-agent-api-guide.md
- [ ] Implement format conversion utilities (HTML ↔ Markdown ↔ ToonFormat) — target: src/export/converters.ts
- [ ] Note: Do not use binary formats (Protocol Buffers, Apache Thrift, Captain Proto, Apache Avro) for plaintext protocols

## Relevant Files

- `packages/analytics-rs/Cargo.toml` — ToonFormat dependency
- `src/export/toonformat.rs` — Rust ToonFormat integration
- `src/export/formats/toonformat.ts` — ToonFormat export
- `src/export/formats/json-single.ts` — Single record JSON export
- `src/api/endpoints/ai-agents.ts` — AI agent endpoints
- `src/api/middleware/content-negotiation.ts` — Content negotiation
- `src/api/middleware/agent-detection.ts` — AI agent detection
- `frontend/src/components/ai-agent-view/` — AI agent view components
- `src/export/toonformat-optimizer.ts` — ToonFormat optimization
- `docs/ai-agent-api-guide.md` — AI agent API documentation
- `src/export/converters.ts` — Format conversion utilities
- `test/export/toonformat.test.ts` — ToonFormat tests
- `test/api/ai-agents.test.ts` — AI agent API tests

## Acceptance Criteria

- [ ] ToonFormat is integrated and functional for bulk data export
- [ ] Single record JSON format works for plaintext protocol exchanges
- [ ] AI agent endpoints provide markdown output on request
- [ ] Content negotiation correctly serves HTML/Markdown/ToonFormat based on client
- [ ] AI agent detection automatically selects optimal format
- [ ] Dashboard includes AI agent view mode with markdown output
- [ ] ToonFormat compression reduces token usage by >30% compared to JSON
- [ ] Format conversion utilities work bidirectionally
- [ ] API documentation clearly explains AI agent interface options
- [ ] All formats are tested and validated

## Test Plan

- Unit: `npm test src/export/formats/toonformat.test.ts`
- Unit: `npm test src/api/endpoints/ai-agents.test.ts`
- Integration: Test content negotiation with different client types
- Performance: Compare token usage between JSON and ToonFormat for bulk data
- Conversion: Test format conversion accuracy and fidelity
- E2E: Test AI agent consumption of different output formats

## Observability

- Monitor format selection by client type (human vs AI agent)
- Track ToonFormat compression ratios and token savings
- Log AI agent detection accuracy and fallbacks
- Alert on format conversion failures or performance issues

## Compliance

- Ensure ToonFormat doesn't lose data fidelity during compression
- Maintain data privacy in all output formats
- Support GDPR right to data portability in all formats
- Document format-specific data handling practices

## Risks & Mitigations

- Risk: ToonFormat may not be widely adopted — Mitigation: Maintain JSON as fallback format
- Risk: AI agent detection may be inaccurate — Mitigation: Allow manual format override via headers
- Risk: Format conversion may lose information — Mitigation: Implement strict validation and testing
- Risk: Multi-format support may increase complexity — Mitigation: Design clean abstraction layer

## Dependencies

- 04-001: Dashboard UI Framework (AI agent view extends dashboard)
- 04-002: REST API Implementation (AI agent endpoints extend API)
- 04-003: Data Export Capabilities (ToonFormat extends export system)

## Notes

- ToonFormat (https://toonformat.dev/) should be used for bulk data to minimize token usage
- JSON should be used for single record exchanges in plaintext protocols
- Do not use binary formats (Protocol Buffers, Apache Thrift, Captain Proto, Apache Avro) for plaintext protocols
- AI agent interfaces should support both direct API access and markdown output
- Content negotiation should use standard Accept headers and user-agent detection
- Consider adding rate limiting specific to AI agent endpoints to prevent abuse