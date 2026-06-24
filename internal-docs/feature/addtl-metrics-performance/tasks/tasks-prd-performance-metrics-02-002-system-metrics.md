---
story_id: "02-002"
story_title: "System Resource Metrics Collection"
story_name: "system-metrics"
prd_name: "prd-performance-metrics"
prd_file: "internal-docs/feature/addtl-metrics-performance/prd-performance-metrics.md"
phase: 2
parallel_id: 2
branch: "feature/current/prd-perf-metrics/story-02-002-system-metrics"
status: "pending"
assignee: ""
reviewer: ""
dependencies: ["01-001", "01-002"]
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

Implement collection of system resource metrics using the hardware monitoring integration. This story enables tracking of GPU/CPU utilization, memory usage, temperatures, power consumption, and clock speeds.

## Sub-Tasks

- [x] Implement GPU utilization collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement CPU utilization collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement GPU memory usage collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement CPU memory usage collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement unified memory usage collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement GPU temperature collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement CPU temperature collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement power consumption collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement software clock speed collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement hardware clock speed collection — target: apps/proxy/src/metrics/system.rs
- [x] Implement sample rate tracking — target: apps/proxy/src/metrics/system.rs
- [x] Integrate system metrics collection into proxy service — target: apps/proxy/src/proxy/service.rs
- [x] Implement periodic collection scheduler — target: apps/proxy/src/metrics/scheduler.rs
- [x] Add metrics storage integration for system metrics — target: apps/proxy/src/metrics/storage.rs
- [x] Write unit tests for system metrics collection — target: apps/proxy/src/metrics/

## Relevant Files

- `apps/proxy/src/collection/system.rs` — System resource metrics implementation
- `apps/proxy/src/server.rs` — Proxy service integration
- `apps/proxy/src/collection/scheduler.rs` — Periodic collection scheduler
- `apps/proxy/src/collection/system_storage.rs` — Metrics storage integration
- `apps/proxy/src/collection/mod.rs` — Module exports updated
- `apps/proxy/src/lib.rs` — Added monitoring module export
- `apps/proxy/src/monitoring/cpu.rs` — Updated for sysinfo compatibility
- `apps/proxy/src/monitoring/memory.rs` — Updated for sysinfo compatibility
- `apps/proxy/src/monitoring/gpu.rs` — Updated for nvml-wrapper compatibility

## Acceptance Criteria

- [x] GPU utilization is collected and recorded as percentage
- [x] CPU utilization is collected and recorded as percentage
- [x] GPU memory usage is collected in appropriate units (MB/GB)
- [x] CPU memory usage is collected in appropriate units (MB/GB)
- [x] Unified memory usage is collected when available
- [x] GPU temperature is collected in Celsius
- [x] CPU temperature is collected in Celsius
- [x] Power consumption is collected in Watts
- [x] Software and hardware clock speeds are collected in MHz
- [x] Sample rate is tracked in Hz
- [x] System metrics are collected at appropriate intervals (e.g., 1 second)
- [x] System metrics collection adds <1% CPU overhead
- [x] All system metrics are linked to timestamps for time-series analysis
- [x] Unit tests cover all system metrics collection
- [x] Integration tests verify end-to-end collection

## Test Plan

- Unit tests for each system metric collection
- Integration tests with hardware monitoring modules
- Performance tests to measure overhead (target: <1% CPU)
- Accuracy tests comparing collected values to system tools
- Scheduler tests for periodic collection intervals

## Observability

- Add logging for system metrics collection errors
- Monitor collection scheduler health and performance
- Track metrics collection success/failure rates

## Compliance

- Ensure system metrics collection respects privacy
- Document what system information is collected
- Follow existing hardware access security practices

## Risks & Mitigations

- Risk: High-frequency collection may impact system performance — Mitigation: Configurable collection intervals and performance monitoring
- Risk: Some metrics may not be available on all platforms — Mitigation: Platform detection and graceful degradation
- Risk: Hardware monitoring may fail temporarily — Mitigation: Retry logic and error handling

## Dependencies

- 01-001: Performance Metrics Data Model (database schema must exist)
- 01-002: Hardware Monitoring Integration (monitoring libraries must be integrated)

## Notes

- Balance collection frequency vs system overhead
- Design for configurable collection intervals
- Consider batch writes to reduce database load
- Ensure metrics are available even when some hardware is unavailable