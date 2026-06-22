# AI Dashboard — Agent Instructions

## Project Overview

AI Dashboard is a comprehensive analytics platform for AI usage across multiple dimensions: company clients, AI clients (Claude Code, Codex, Pi, Devin, etc.), teams, pipeline stages, AI model suppliers (Anthropic, OpenAI, Google, Microsoft, AWS, OpenRouter, etc.), models, and input types (text/chat, image, audio, etc.).

**Architecture:**
- **Open-source version:** 2-service architecture (proxy + web) for single-tenant deployments
- **Commercial version:** 4-service architecture (proxy + collector + analytics + web) for multi-tenant scale
- **Monorepo:** Nx-based with pnpm workspaces
- **Environment:** Devbox for consistent development environments

## Development Commands

**IMPORTANT:** This project follows the Standard Developer UX Flow (ADR-20260131001). AI agents should use `devbox run just x-internal` for automated operations, while human developers can use `just x` for convenience.

### AI Agent Commands (Automated)

```bash
# Core development commands (AI agents use these)
devbox run just build-internal      # Build all projects
devbox run just test-internal       # Run all tests
devbox run just lint-internal       # Lint all projects
devbox run just typecheck-internal  # Type check all projects
devbox run just dev-internal        # Start all dev servers
devbox run just quality             # Run quality gates (lint + test + typecheck)
devbox run just ci                  # Run CI pipeline locally
```

### Human Developer Commands (Convenient)

```bash
# Human developers can use these simpler commands
just build      # Build all projects
just test       # Run all tests
just lint       # Lint all projects
just typecheck  # Type check all projects
just dev        # Start all dev servers
just quality    # Run quality gates
just ci         # Run CI pipeline locally
```

### Project-specific Commands

#### Proxy Service (Rust)

```bash
# AI agent commands
devbox run just proxy-build
devbox run just proxy-test
devbox run just proxy-lint
devbox run just proxy-fmt
devbox run just proxy-run
devbox run just proxy-serve

# Human developer commands
just proxy-build
just proxy-test
just proxy-lint
just proxy-fmt
just proxy-run
just proxy-serve
```

#### Web Application (Next.js)

```bash
# AI agent commands
devbox run just web-build
devbox run just web-dev
devbox run just web-test
devbox run just web-lint
devbox run just web-typecheck

# Human developer commands
just web-build
just web-dev
just web-test
just web-lint
just web-typecheck
```

#### Analytics Package (Rust)

```bash
# AI agent commands
devbox run just analytics-build
devbox run just analytics-test
devbox run just analytics-lint
devbox run just analytics-fmt
devbox run just analytics-bench

# Human developer commands
just analytics-build
just analytics-test
just analytics-lint
just analytics-fmt
just analytics-bench
```

### Utility Commands

```bash
# Bootstrap and environment setup
devbox run just bootstrap-internal    # Install dependencies
devbox run just prime-internal        # Prime code indexing

# Health and diagnostics
devbox run just doctor-internal       # Check environment health

# Cleanup
devbox run just clean-internal        # Remove build artifacts
devbox run just clean-all-internal    # Remove all artifacts and dependencies

# Formatting
devbox run just format                # Format all projects
devbox run just format-check          # Check formatting

# Deployment
devbox run just deploy                # Deploy to production
devbox run just deploy-staging        # Deploy to staging
```

## Project Structure

```
ai-dashboard/
├── apps/
│   ├── proxy/          # Rust-based proxy service (devbox + cargo/just)
│   └── web/            # Next.js web application
├── packages/
│   ├── analytics-rs/   # Rust analytics package
│   └── ui/             # UI components package
├── docs/               # Public documentation
├── internal-docs/      # Internal documentation and PRDs
└── devbox.json         # Devbox configuration
```

## Key Technologies

- **Monorepo:** Nx with pnpm workspaces
- **Proxy Service:** Rust with Cargo and Just
- **Web Application:** Next.js with TypeScript
- **Analytics Engine:** Rust
- **Environment:** Devbox for consistent development environments
- **Package Manager:** pnpm

## Licensing

- **Open Source:** AGPL 3.0 for all open-source features
- **Commercial:** Commercial license available for multi-tenant, white-label, or proprietary use
- **Dual Licensing:** Clear separation between open-source and commercial features

## Development Workflow

This project follows the Standard Developer UX Flow (ADR-20260131001):

1. **AI Agents**: Use `devbox run just x-internal` for automated operations
2. **Human Developers**: Use `just x` for convenient one-off commands
3. **Power Users**: Use `just x-internal` directly when already in devbox shell
4. **Environment**: Devbox with direnv for automatic environment activation
5. **Quality Gates**: Run `just quality` before committing changes
6. **PRD Implementation**: Follow the PRD task structure in `internal-docs/feature/prd-multi-tenant-ai-analytics/tasks/`
7. **Reference**: Individual app-specific AGENTS.md files when available (currently none exist)

## PRD Implementation

The project follows a structured PRD implementation process:

- **PRD Location:** `docs/feature/prd-multi-tenant-ai-analytics.md`
- **Task Index:** `internal-docs/feature/prd-multi-tenant-ai-analytics/tasks/index-prd-multi-tenant-ai-analytics.md`
- **Task Processing:** Follow the tasks-processor workflow for systematic implementation

## Testing

- Run tests with `devbox run just test-internal` for all projects (AI agents)
- Run tests with `just test` for all projects (human developers)
- Run project-specific tests: `just proxy-test`, `just web-test`, `just analytics-test`
- Always ensure tests pass before committing changes

## Code Quality

- **Quality Gates:** `devbox run just quality` or `just quality` (runs lint + test + typecheck)
- **Linting:** `devbox run just lint-internal` or `just lint` (zero warnings policy)
- **Type Checking:** `devbox run just typecheck-internal` or `just typecheck`
- **Formatting:** `devbox run just format` or `just format`
- **Format Check:** `devbox run just format-check` or `just format-check`

## Notes

- This project follows ADR-20260131001 Standard Developer UX Flow
- AI agents use `devbox run just x-internal` for automated operations
- Human developers use `just x` for convenient one-off commands
- Devbox with direnv provides automatic environment activation
- Individual apps/packages may have their own AGENTS.md files in the future (currently none exist)
- Use `just --list` to see all available commands
- Use `just doctor` to check environment health
