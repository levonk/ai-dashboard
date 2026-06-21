# AI Dashboard — Agent Instructions

## Project Overview

AI Dashboard is a comprehensive analytics platform for AI usage across multiple dimensions: company clients, AI clients (Claude Code, Codex, Pi, Devin, etc.), teams, pipeline stages, AI model suppliers (Anthropic, OpenAI, Google, Microsoft, AWS, OpenRouter, etc.), models, and input types (text/chat, image, audio, etc.).

**Architecture:**
- **Open-source version:** 2-service architecture (proxy + web) for single-tenant deployments
- **Commercial version:** 4-service architecture (proxy + collector + analytics + web) for multi-tenant scale
- **Monorepo:** Nx-based with pnpm workspaces
- **Environment:** Devbox for consistent development environments

## Development Commands

**IMPORTANT:** All commands must be prefixed with `devbox run --` to ensure they run within the devbox environment. This is critical for consistent tooling and dependencies.

### Root-level Commands

```bash
# Build all projects
devbox run -- nx run-many -t build

# Development mode (all projects)
devbox run -- nx run-many -t dev

# Lint all projects
devbox run -- nx run-many -t lint

# Type check all projects
devbox run -- nx run-many -t typecheck

# Clean all projects
devbox run -- nx run-many -t clean

# Format all projects
devbox run -- prettier --write "**/*.{ts,tsx,js,jsx,json,md,css}" && nx run-many -t format
```

### Project-specific Commands

#### Proxy Service (Rust)

```bash
# Navigate to proxy directory first
cd apps/proxy

# Build proxy
devbox run -- cargo build --release

# Run proxy
devbox run -- cargo run

# Run proxy in serve mode
devbox run -- cargo run -- serve

# Test proxy
devbox run -- cargo test

# Lint proxy
devbox run -- cargo clippy -- -D warnings

# Format proxy
devbox run -- cargo fmt

# Type check proxy
devbox run -- cargo check
```

#### Web Application (Next.js)

```bash
# Navigate to web directory first
cd apps/web

# Build web
devbox run -- nx build web

# Development server
devbox run -- nx dev web

# Production server
devbox run -- nx start web

# Lint web
devbox run -- nx lint web

# Type check web
devbox run -- nx typecheck web
```

#### Analytics Package (Rust)

```bash
# Navigate to analytics-rs directory first
cd packages/analytics-rs

# Build analytics package
devbox run -- cargo build --release

# Test analytics package
devbox run -- cargo test

# Lint analytics package
devbox run -- cargo clippy -- -D warnings

# Format analytics package
devbox run -- cargo fmt

# Type check analytics package
devbox run -- cargo check
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

1. **Always use `devbox run --` prefix** for all commands
2. Work in the appropriate app/package directory for project-specific commands
3. Use root-level Nx commands for multi-project operations
4. Follow the PRD task structure in `internal-docs/feature/prd-multi-tenant-ai-analytics/tasks/`
5. Reference individual app-specific AGENTS.md files when available (currently none exist)

## PRD Implementation

The project follows a structured PRD implementation process:

- **PRD Location:** `docs/feature/prd-multi-tenant-ai-analytics.md`
- **Task Index:** `internal-docs/feature/prd-multi-tenant-ai-analytics/tasks/index-prd-multi-tenant-ai-analytics.md`
- **Task Processing:** Follow the tasks-processor workflow for systematic implementation

## Testing

- Run tests with `devbox run -- nx run-many -t test` for all projects
- Run project-specific tests by navigating to the project directory first
- Always ensure tests pass before committing changes

## Code Quality

- **Linting:** `devbox run -- nx run-many -t lint` (zero warnings policy)
- **Type Checking:** `devbox run -- nx run-many -t typecheck`
- **Formatting:** `devbox run -- prettier --write "**/*.{ts,tsx,js,jsx,json,md,css}" && nx run-many -t format`

## Notes

- This project uses devbox for environment management - never skip the `devbox run --` prefix
- Individual apps/packages may have their own AGENTS.md files in the future (currently none exist)
- Always check the project.json files for the correct command structure
- The proxy service uses Just for additional command aliases - check the justfile for more options
