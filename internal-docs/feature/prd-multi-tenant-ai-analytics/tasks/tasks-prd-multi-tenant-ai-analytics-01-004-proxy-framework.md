---
story_id: "01-004"
story_title: "Proxy Service Framework"
story_name: "proxy-framework"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 1
parallel_id: 4
branch: "feature/current/prd-ai-analytics/story-01-004-proxy-framework"
status: "in-progress"
assignee: ""
reviewer: ""
dependencies: []
parallel_safe: true
modules: ["proxy", "framework"]
priority: "MUST"
risk_level: "medium"
tags: ["feat", "proxy", "framework"]
due: "2025-01-31"
created_at: "2025-06-20"
updated_at: "2025-06-20"
---

## Summary

Design and implement the proxy service framework that routes AI requests to providers and collects telemetry data. The proxy operates in analytics mode by default (writes telemetry directly to database) with support for emitter mode (sends telemetry to downstream collector) for future enterprise deployments.

## Sub-Tasks

- [x] Design proxy service architecture with analytics/emitter modes — target: apps/proxy/src/lib.rs
- [x] Implement HTTP routing to AI providers (Anthropic, OpenAI, Google, etc.) — target: apps/proxy/src/routing.rs
- [x] Create telemetry collection middleware — target: apps/proxy/src/telemetry.rs
- [x] Implement analytics mode (direct database writes) — target: apps/proxy/src/analytics_mode.rs
- [x] Add configuration for proxy mode switching — target: apps/proxy/src/config.rs
- [x] Implement API key security and request processing — target: apps/proxy/src/security.rs
- [x] Add health check endpoints and monitoring — target: apps/proxy/src/health.rs
- [x] Create proxy SDK for AI client integration — target: apps/proxy/src/sdk/

## Relevant Files

- `apps/proxy/src/lib.rs` — Proxy service entry point
- `apps/proxy/src/routing.rs` — HTTP routing to AI providers
- `apps/proxy/src/telemetry.rs` — Telemetry collection middleware
- `apps/proxy/src/analytics_mode.rs` — Analytics mode implementation
- `apps/proxy/src/config.rs` — Configuration handling (updated with proxy_mode)
- `apps/proxy/src/security.rs` — API key security and request processing
- `apps/proxy/src/health.rs` — Health check endpoints
- `apps/proxy/src/sdk/` — Proxy SDK for AI client integration
- `apps/proxy/tests/` — Proxy service tests
- `docs/proxy-guide.md` — Proxy deployment guide

## Acceptance Criteria

- [x] Proxy service architecture supports both analytics and emitter modes
- [x] HTTP routing correctly forwards requests to AI providers (Anthropic, OpenAI, Google, etc.)
- [x] Telemetry collection middleware captures all relevant request/response data
- [x] Analytics mode writes telemetry directly to database with proper error handling
- [x] Configuration system enables seamless switching between proxy modes
- [x] API key security prevents unauthorized access and properly validates requests
- [x] Health check endpoints provide visibility into proxy service status
- [x] Proxy SDK enables easy integration for AI clients (Claude Code, Codex, Pi, Devin, etc.)

## Test Plan

- Unit: `devbox run -- cargo test --lib proxy::routing`
- Unit: `devbox run -- cargo test --lib proxy::telemetry`
- Unit: `devbox run -- cargo test --lib proxy::analytics_mode`
- Unit: `devbox run -- cargo test --lib proxy::config`
- Unit: `devbox run -- cargo test --lib proxy::security`
- Integration: Test end-to-end proxy request flow with mock AI providers
- Integration: Test telemetry collection and database writes in analytics mode
- SDK: Test SDK integration with sample AI client

## Observability

- Add proxy startup/shutdown logging
- Monitor proxy health status and request metrics
- Track request latency and error rates
- Log proxy mode configuration changes
- Monitor database connection health in analytics mode

## Compliance

- Ensure proxy doesn't expose sensitive API keys or request data
- Validate all incoming requests to prevent injection attacks
- Document data retention policies for collected telemetry
- Implement proper authentication for proxy management endpoints

## Risks & Mitigations

- Risk: Proxy failures may block AI requests — Mitigation: Implement circuit breakers and fallback mechanisms
- Risk: Telemetry collection may impact performance — Mitigation: Use async collection and batch writes
- Risk: API key exposure in logs — Mitigation: Sanitize logs and implement secure key storage
- Risk: Database connection issues in analytics mode — Mitigation: Implement connection pooling and retry logic

## Dependencies

- 01-003 (Analytics Package Foundation) - uses analytics-rs package for data processing

## Notes

- Design for high availability and minimal latency
- Keep proxy mode switching seamless without service restart
- Focus on minimizing performance impact on AI request latency
- Consider rate limiting and quota management for future enhancements