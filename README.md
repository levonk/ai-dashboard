# AI Analytics Dashboard

A comprehensive analytics platform for AI usage across multiple dimensions: company clients, AI clients (Claude Code, Codex, Pi, Devin, etc.), teams, pipeline stages, AI model suppliers (Anthropic, OpenAI, Google, Microsoft, AWS, OpenRouter, etc.), models, and input types (text/chat, image, audio, etc.).

## Architecture

**Open-Source Version (AGPL-3.0):**
- 2-service architecture: proxy + web
- Single-tenant deployment
- Real-time analytics and dashboards

**Commercial Version:**
- 4-service architecture: proxy + collector + analytics + web
- Multi-tenant scale
- Enterprise features and white-label options

## Tech Stack

- **Monorepo:** Nx with pnpm workspaces
- **Proxy Service:** Rust with Cargo and Just
- **Web Application:** Next.js with TypeScript
- **Analytics Engine:** Rust
- **Environment:** Devbox for consistent development environments
- **Package Manager:** pnpm

## Project Structure

```
ai-dashboard/
├── apps/
│   ├── proxy/          # Rust-based proxy service
│   └── web/            # Next.js web application
├── packages/
│   ├── analytics-rs/   # Rust analytics package
│   └── ui/             # UI components package
├── docs/               # Public documentation
├── internal-docs/      # Internal documentation and PRDs
└── devbox.json         # Devbox configuration
```

## Getting Started

### Prerequisites

- Node.js 18 or higher
- pnpm 8 or higher
- Rust toolchain (for proxy service and analytics package)
- Devbox (for consistent development environments)
- Git

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/levonk/ai-dashboard.git
   cd ai-dashboard
   ```

2. **Install dependencies**
   ```bash
   devbox run -- pnpm install
   ```

3. **Run development servers**
   ```bash
   # All services (proxy + web)
   devbox run -- nx run-many -t dev

   # Or individually:
   # Proxy service
   cd apps/proxy
   devbox run -- cargo run

   # Web application
   cd apps/web
   devbox run -- nx dev web
   ```

## Development Commands

**IMPORTANT:** All commands must be prefixed with `devbox run --` to ensure they run within the devbox environment.

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
cd apps/proxy

# Build proxy
devbox run -- cargo build --release

# Run proxy
devbox run -- cargo run

# Test proxy
devbox run -- cargo test

# Lint proxy
devbox run -- cargo clippy -- -D warnings

# Format proxy
devbox run -- cargo fmt
```

#### Web Application (Next.js)

```bash
cd apps/web

# Build web
devbox run -- nx build web

# Development server
devbox run -- nx dev web

# Lint web
devbox run -- nx lint web

# Type check web
devbox run -- nx typecheck web
```

## Testing

```bash
# Run all tests
devbox run -- nx run-many -t test

# Run specific project tests
cd apps/proxy
devbox run -- cargo test

cd apps/web
devbox run -- nx test web
```

## Documentation

- [Public Documentation](docs/)
- [Internal Documentation](internal-docs/)
- [Contributing Guidelines](CONTRIBUTING.md)
- [License Information](LICENSE.md)

## License

This project is dual-licensed:

- **AGPL-3.0** for open-source use
- **Commercial License** for enterprise features and multi-tenant deployment

See [LICENSE.md](LICENSE.md) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Support

- **Issues:** [GitHub Issues](https://github.com/levonk/ai-dashboard/issues)
- **Discussions:** [GitHub Discussions](https://github.com/levonk/ai-dashboard/discussions)
- **Commercial License:** ai-dashboard@a3isolutions.com

## Acknowledgments

Built with:
- [Nx](https://nx.dev) for monorepo management
- [Devbox](https://www.jetify.com/devbox) for development environments
- [Next.js](https://nextjs.org) for the web application
- [Rust](https://www.rust-lang.org) for high-performance services
