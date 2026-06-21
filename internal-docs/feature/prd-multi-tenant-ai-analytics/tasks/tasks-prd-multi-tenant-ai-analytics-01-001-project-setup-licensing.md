---
story_id: "01-001"
story_title: "Project Setup and Licensing Framework"
story_name: "project-setup-licensing"
prd_name: "prd-multi-tenant-ai-analytics"
prd_file: "docs/feature/prd-multi-tenant-ai-analytics.md"
phase: 1
parallel_id: 1
branch: "feature/current/prd-multi-tenant-ai-analytics/story-01-001-project-setup-licensing"
status: "in-progress"
assignee: ""
reviewer: ""
dependencies: []
parallel_safe: true
modules: ["project-root", "licensing"]
priority: "MUST"
risk_level: "low"
tags: ["feat", "foundation"]
due: "2025-01-31"
created_at: "2025-06-20"
updated_at: "2025-06-20"
---

## Summary

Establish the project foundation including build system, development environment, licensing framework (AGPL 3.0 + commercial license), and contribution agreements. This story creates the scaffolding for the entire analytics platform.

## Sub-Tasks

- [x] Initialize project structure with TypeScript, Node.js, and build tools — target: package.json, tsconfig.json, build scripts
- [x] Set up AGPL 3.0 licensing framework with commercial license references — target: LICENSE.md, LICENSE-COMMERCIAL.md
- [x] Create Contributor License Agreement (CLA) process and documentation — target: CLA.md, .github/cla-assistant/
- [x] Update development tooling configuration for actual stack (Nx, Rust, Next.js) — target: nx.json, devbox.json, project.json files
- [x] Update CONTRIBUTING.md to reflect actual Rust/Next.js stack (not Python) — target: CONTRIBUTING.md
- [x] Set up Git workflow with branch protection rules — target: .github/workflows/, branch protection config
- [x] Create/update README with project overview and setup instructions — target: README.md
- [x] Configure CI/CD pipeline for automated testing and validation — target: .github/workflows/ci.yml
- [x] Set up documentation structure and contributing guidelines — target: CONTRIBUTING.md, docs/

## Relevant Files

- `package.json` — Project dependencies and scripts
- `tsconfig.json` — TypeScript configuration
- `LICENSE.md` — AGPL 3.0 license with commercial license references
- `LICENSE-COMMERCIAL.md` — Commercial license terms template
- `CLA.md` — Contributor License Agreement documentation
- `.github/cla-assistant/` — CLA automation configuration
- `.eslintrc.json` — ESLint linting rules
- `.prettierrc` — Code formatting configuration
- `jest.config.js` — Test configuration
- `README.md` — Project overview and setup guide
- `CONTRIBUTING.md` — Contribution guidelines
- `.github/workflows/ci.yml` — CI/CD pipeline configuration

## Acceptance Criteria

- [x] Project structure is configured with TypeScript, Node.js, and build tools
- [x] Development tooling configuration is set up for actual stack (Nx, Rust, Next.js)
- [x] License files clearly distinguish AGPL 3.0 vs commercial features
- [x] CLA process is documented and automated via GitHub
- [x] CI/CD pipeline is configured for automated testing and validation
- [x] README provides clear setup instructions for new developers
- [x] CONTRIBUTING.md reflects actual Rust/Next.js stack
- [x] Git workflow and branch protection rules are documented
- [ ] Full build/lint/test cycle passes (requires pnpm/Rust environment setup)

## Test Plan

- Lint: `npm run lint`
- Types: `npm run typecheck` 
- Build: `npm run build`
- Test: `npm test`
- CI: Verify GitHub Actions workflow runs successfully

## Observability

- Add build status badge to README
- Configure CI/CD to report build/test results
- Set up dependency monitoring for security updates

## Compliance

- Ensure AGPL 3.0 compliance in all open-source components
- Document commercial license boundaries clearly
- Implement CLA for all contributions to enable dual licensing

## Risks & Mitigations

- Risk: License complexity may confuse contributors — Mitigation: Clear documentation and simplified CLA process
- Risk: Build tooling choices may limit future options — Mitigation: Use standard, well-supported tools with migration paths

## Dependencies

None - this is a foundation story

## Notes

- Focus on standard, well-maintained tooling to minimize technical debt
- Keep commercial license boundaries clear in documentation
- Ensure CLA process is frictionless for contributors