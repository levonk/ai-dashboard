# AI Dashboard - Standard Developer UX Flow
# Following ADR: adr-20260131001-standard-developer-ux-flow.md
# Primary pattern: just x → devbox run x → just x-internal
# Fallback pattern: just x → devbox run <direct command>

# =============================================================================
# Normal targets - Developer interface (REQUIRED)
# =============================================================================

default:
    @just --list

# =============================================================================
# Core Development Commands
# =============================================================================

build:
    devbox run build

build-internal:
    # Nx monorepo build - builds all projects
    nx run-many -t build

test:
    devbox run test

test-internal:
    # Nx monorepo test - runs all tests
    nx run-many -t test

lint:
    devbox run lint

lint-internal:
    # Nx monorepo lint - runs all linting
    nx run-many -t lint

typecheck:
    devbox run typecheck

typecheck-internal:
    # Nx monorepo typecheck - runs all type checking
    nx run-many -t typecheck

dev:
    devbox run dev

dev-internal:
    # Nx monorepo dev - starts all development servers
    nx run-many -t dev

# =============================================================================
# Quality Checks (REQUIRED for all projects)
# =============================================================================

quality:
    just lint
    just test
    just typecheck

# =============================================================================
# Bootstrap and Prime (REQUIRED)
# =============================================================================

bootstrap:
    # Ensure devbox is available and environment is ready
    devbox run bootstrap

bootstrap-internal:
    # Internal bootstrap logic called by devbox init_hook
    # Install dependencies for monorepo
    echo "Installing pnpm dependencies..."
    pnpm install
    echo "Project bootstrap complete!"

prime:
    # Prime code indexing and analysis tools
    devbox run prime

prime-internal:
    # Internal prime logic for code indexing and analysis tools
    # Run universal-ctags, roam-code, desloppify, etc.
    echo "Priming code indexing and analysis tools..."
    # Add project-specific indexing commands here
    echo "Code indexing complete!"

# =============================================================================
# Health and Diagnostics (REQUIRED)
# =============================================================================

doctor:
    # Check development environment health
    devbox run doctor

doctor-internal:
    # Internal doctor logic
    echo "Checking development environment health..."
    echo "Checking pnpm installation..."
    pnpm --version
    echo "Checking Nx installation..."
    nx --version
    echo "Checking Rust installation..."
    cargo --version
    echo "Environment health check complete!"

# =============================================================================
# Clean up (REQUIRED)
# =============================================================================

clean:
    devbox run clean

clean-internal:
    # Remove build artifacts and caches
    rm -rf node_modules/.cache
    rm -rf .nx/cache
    rm -rf apps/*/dist
    rm -rf apps/*/build
    rm -rf apps/*/.next
    rm -rf packages/*/dist
    rm -rf packages/*/target
    echo "Build artifacts removed"

clean-all:
    devbox run clean-all

clean-all-internal:
    # Remove all build artifacts, dependencies, and caches
    rm -rf node_modules
    rm -rf .nx
    rm -rf apps/*/dist
    rm -rf apps/*/build
    rm -rf apps/*/.next
    rm -rf packages/*/dist
    rm -rf packages/*/target
    rm -rf packages/analytics-rs/Cargo.lock
    echo "All build artifacts and dependencies removed"

# =============================================================================
# Project-Specific Commands
# =============================================================================

# Proxy Service (Rust)
proxy-build:
    cd apps/proxy && cargo build --release

proxy-test:
    cd apps/proxy && cargo test

proxy-lint:
    cd apps/proxy && cargo clippy -- -D warnings

proxy-fmt:
    cd apps/proxy && cargo fmt

proxy-run:
    cd apps/proxy && cargo run

proxy-serve:
    cd apps/proxy && cargo run -- serve

# Web Application (Next.js)
web-build:
    nx build web

web-dev:
    nx dev web

web-test:
    nx test web

web-lint:
    nx lint web

web-typecheck:
    nx typecheck web

# Analytics Package (Rust)
analytics-build:
    cd packages/analytics-rs && cargo build --release

analytics-test:
    cd packages/analytics-rs && cargo test

analytics-lint:
    cd packages/analytics-rs && cargo clippy -- -D warnings

analytics-fmt:
    cd packages/analytics-rs && cargo fmt

analytics-bench:
    cd packages/analytics-rs && cargo bench

# =============================================================================
# Monorepo Operations
# =============================================================================

# Format all projects
format:
    prettier --write "**/*.{ts,tsx,js,jsx,json,md,css}"
    nx run-many -t format

# Format check
format-check:
    prettier --check "**/*.{ts,tsx,js,jsx,json,md,css}"
    nx run-many -t format

# Database operations (if needed)
db-migrate:
    # Run database migrations
    echo "Database migrations"

db-seed:
    # Seed database with test data
    echo "Seeding database"

# =============================================================================
# Deployment Commands
# =============================================================================

deploy:
    # Deploy to production
    echo "Deploying to production..."
    just build
    # Add deployment commands here

deploy-staging:
    # Deploy to staging
    echo "Deploying to staging..."
    just build
    # Add staging deployment commands here

# =============================================================================
# Documentation
# =============================================================================

docs:
    # Generate documentation
    echo "Generating documentation..."

docs-serve:
    # Serve documentation locally
    echo "Serving documentation..."

# =============================================================================
# CI/CD Support
# =============================================================================

ci:
    # Run CI pipeline locally
    just quality
    just build

ci-full:
    # Run full CI pipeline with additional checks
    just quality
    just build
    just analytics-bench