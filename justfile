# AI Dashboard - Standard Developer UX Flow
# Following ADR: adr-20260131001-standard-developer-ux-flow.md
# Primary pattern: just x → _devbox x_impl

_log := '
_jv_has() {
  local cat="$1"
  local v="${JUST_LOG:-0}"
  case "$v" in
    1|all) return 0 ;;
    0|"") return 1 ;;
  esac
  v="${v//startend/start,end}"
  echo ",$v," | grep -q ",$cat,"
}
log_info()   { _jv_has info   && echo "$*" || true; }
log_start()  { _jv_has start  && echo "▶ $*" || true; }
log_end()    { _jv_has end    && echo "✔ $*" || true; }
log_status() { _jv_has status && echo "$*" || true; }
log_warn()   { echo "⚠️  $*" >&2; }
log_error()  { echo "❌ $*" >&2; }
log_startend() {
  local msg="$1"; shift
  local rc
  _jv_has start && echo "▶ $msg" || true
  rc=0; "$@" || rc=$?
  _jv_has end && echo "✔ $msg complete" || true
  return $rc
}
'

# Devbox auto-detection: run impl target directly if in devbox,
# re-exec via devbox run if not, or fail with doctor diagnostic.
_devbox target *args:
    #!/usr/bin/env bash
    {{_log}}
    if [ "${DEVBOX_SHELL_ENABLED:-0}" = "1" ]; then
        exec just "{{target}}" {{args}}
    elif command -v devbox >/dev/null 2>&1; then
        exec devbox run -- just "{{target}}" {{args}}
    else
        log_error "devbox not found in PATH."
        log_warn "Running doctor to diagnose environment issues..."
        just doctor 2>/dev/null || true
        exit 1
    fi

# =============================================================================
# Normal targets - Developer interface (REQUIRED)
# =============================================================================

default:
    @just --list

# =============================================================================
# Core Development Commands
# =============================================================================

build:
    @just _devbox build_impl

test:
    @just _devbox test_impl

lint:
    @just _devbox lint_impl

typecheck:
    @just _devbox typecheck_impl

dev:
    @just _devbox dev_impl

# =============================================================================
# Quality Checks (REQUIRED for all projects)
# =============================================================================

quality:
    @just _devbox quality_impl

# =============================================================================
# Bootstrap and Prime (REQUIRED)
# =============================================================================

bootstrap:
    @just _devbox bootstrap_impl

prime:
    @just _devbox prime_impl

# =============================================================================
# Health and Diagnostics (REQUIRED)
# =============================================================================

doctor:
    @just _devbox doctor_impl

# =============================================================================
# Clean up (REQUIRED)
# =============================================================================

clean:
    @just _devbox clean_impl

clean-all:
    @just _devbox clean_all_impl

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
    #!/usr/bin/env bash
    {{_log}}
    # Run database migrations
    log_info "Database migrations"

db-seed:
    #!/usr/bin/env bash
    {{_log}}
    # Seed database with test data
    log_info "Seeding database"

# =============================================================================
# Deployment Commands
# =============================================================================

deploy:
    @just _devbox deploy_impl

deploy-staging:
    @just _devbox deploy_staging_impl

# =============================================================================
# Documentation
# =============================================================================

docs:
    #!/usr/bin/env bash
    {{_log}}
    # Generate documentation
    log_info "Generating documentation..."

docs-serve:
    #!/usr/bin/env bash
    {{_log}}
    # Serve documentation locally
    log_info "Serving documentation..."

# =============================================================================
# CI/CD Support
# =============================================================================

ci:
    # Run CI pipeline locally
    @just quality
    @just build

ci-full:
    # Run full CI pipeline with additional checks
    @just quality
    @just build
    @just analytics-bench

# =============================================================================
# Implementation targets (private)
# =============================================================================

[private]
build_impl:
    # Nx monorepo build - builds all projects
    nx run-many -t build

[private]
test_impl:
    # Nx monorepo test - runs all tests
    nx run-many -t test

[private]
lint_impl:
    # Nx monorepo lint - runs all linting
    nx run-many -t lint

[private]
typecheck_impl:
    # Nx monorepo typecheck - runs all type checking
    nx run-many -t typecheck

[private]
dev_impl:
    # Nx monorepo dev - starts all development servers
    nx run-many -t dev

[private]
quality_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    {{_log}}
    just lint_impl
    just test_impl
    just typecheck_impl

[private]
bootstrap_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    {{_log}}
    log_start "Installing pnpm dependencies"
    pnpm install
    log_end "Project bootstrap complete"

[private]
prime_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    {{_log}}
    log_start "Priming code indexing and analysis tools"
    # Add project-specific indexing commands here
    log_end "Code indexing complete"

[private]
doctor_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Checking development environment health..."
    echo "Checking pnpm installation..."
    pnpm --version
    echo "Checking Nx installation..."
    nx --version
    echo "Checking Rust installation..."
    cargo --version
    echo "Environment health check complete!"

[private]
clean_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    {{_log}}
    # Remove build artifacts and caches
    rm -rf node_modules/.cache
    rm -rf .nx/cache
    rm -rf apps/*/dist
    rm -rf apps/*/build
    rm -rf apps/*/.next
    rm -rf packages/*/dist
    rm -rf packages/*/target
    log_end "Build artifacts removed"

[private]
clean_all_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    {{_log}}
    # Remove all build artifacts, dependencies, and caches
    rm -rf node_modules
    rm -rf .nx
    rm -rf apps/*/dist
    rm -rf apps/*/build
    rm -rf apps/*/.next
    rm -rf packages/*/dist
    rm -rf packages/*/target
    rm -rf packages/analytics-rs/Cargo.lock
    log_end "All build artifacts and dependencies removed"

[private]
deploy_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    {{_log}}
    log_warn "Deploy target is a placeholder — wire per environment in CI/CD."
    exit 1

[private]
deploy_staging_impl:
    #!/usr/bin/env bash
    set -euo pipefail
    {{_log}}
    log_warn "Deploy-staging target is a placeholder — wire per environment in CI/CD."
    exit 1
