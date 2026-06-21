# Contributing to AI Analytics Dashboard

Thank you for your interest in contributing to the AI Analytics Dashboard! This document provides guidelines and instructions for contributing to the project.

## Contributor License Agreement (CLA)

**IMPORTANT:** This project uses a dual licensing model (AGPL-3.0 and Commercial). To enable this dual licensing, all contributors must sign a Contributor License Agreement (CLA).

### Why Do We Need a CLA?

The CLA ensures that:
- The project can offer both AGPL-3.0 and commercial licenses
- Your contributions can be used in both the open-source and commercial versions
- The project maintainers have the necessary rights to manage the dual licensing
- Your intellectual property rights are protected

### CLA Process

1. **Automatic CLA Check**
   - When you open a pull request, a CLA bot will automatically check if you've signed the CLA
   - If you haven't signed, you'll receive a comment with a link to the CLA

2. **Sign the CLA**
   - Click the link provided by the CLA bot
   - Review the CLA terms
   - Sign electronically using your GitHub account
   - The CLA bot will automatically update your PR status

3. **Corporate Contributions**
   - If you're contributing on behalf of your company, your company may need to sign a corporate CLA
   - Contact us at cla@example.com for corporate CLA arrangements

### CLA Summary

By signing the CLA, you:
- Grant the project a license to use your contribution in both AGPL-3.0 and commercial versions
- Retain copyright to your contribution
- Ensure that your contribution is your original work or properly licensed
- Allow the project to re-license your contribution as needed for the dual licensing model

## Development Setup

### Prerequisites

- Node.js 18 or higher
- pnpm 8 or higher
- Rust toolchain (for proxy service and analytics package)
- Devbox (for consistent development environments)
- Git

### Getting Started

1. **Fork and Clone**
   ```bash
   # Fork the repository on GitHub
   git clone https://github.com/your-username/ai-dashboard.git
   cd ai-dashboard
   ```

2. **Set Up Development Environment**
   ```bash
   # Install dependencies using devbox
   devbox run -- pnpm install

   # Install Rust toolchain if not already installed
   rustup install stable
   ```

3. **Run Development Servers**
   ```bash
   # All services (proxy + web)
   devbox run -- nx run-many -t dev

   # Or individually:
   # Proxy service (Rust)
   cd apps/proxy
   devbox run -- cargo run

   # Web application (Next.js)
   cd apps/web
   devbox run -- nx dev web
   ```

## Code Style and Standards

### Rust Code
- Follow Rust style guidelines (rustfmt)
- Use `cargo fmt` for code formatting
- Use `cargo clippy` for linting
- Use `cargo check` for type checking
- Write documentation comments for all public functions and modules

### TypeScript/Next.js Code
- Follow ESLint rules
- Use Prettier for code formatting
- Use TypeScript for type safety
- Write JSDoc comments for public functions

### Git Commit Messages
Follow conventional commits format:
```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example:
```
feat(collector): add support for Claude Code client

- Implement Claude Code specific metadata extraction
- Add Claude Code request/response parsing
- Update collector documentation

Closes #123
```

## Testing

### Running Tests

```bash
# All tests
devbox run -- nx run-many -t test

# Rust tests (proxy service)
cd apps/proxy
devbox run -- cargo test

# TypeScript tests (web application)
cd apps/web
devbox run -- nx test web

# Analytics package tests
cd packages/analytics-rs
devbox run -- cargo test
```

### Test Requirements

- All new features must include unit tests
- Maintain >80% code coverage
- Integration tests for critical paths
- E2E tests for user workflows

## Pull Request Process

1. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**
   - Write code following the style guidelines
   - Add tests for your changes
   - Update documentation as needed

3. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

4. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   # Create PR on GitHub
   ```

5. **PR Checklist**
   - [ ] CLA signed (automatic check)
   - [ ] Tests pass locally
   - [ ] Code follows style guidelines
   - [ ] Documentation updated
   - [ ] Commit messages follow conventional commits
   - [ ] No merge conflicts

## Project Structure

```
ai-dashboard/
├── apps/
│   ├── proxy/          # Rust-based proxy service
│   │   ├── src/        # Source code
│   │   ├── tests/      # Tests
│   │   ├── Cargo.toml  # Rust dependencies
│   │   ├── justfile    # Just command runner
│   │   └── devbox.json # Devbox configuration
│   └── web/            # Next.js web application
│       ├── src/        # Source code
│       ├── public/     # Static assets
│       └── package.json
├── packages/
│   ├── analytics-rs/   # Rust analytics package
│   │   ├── src/        # Source code
│   │   └── Cargo.toml
│   └── ui/             # UI components package
├── docs/               # Public documentation
├── internal-docs/      # Internal documentation and PRDs
├── devbox.json         # Root devbox configuration
├── nx.json             # Nx workspace configuration
└── package.json        # Root package.json
```

## Feature Development

### Adding New Collectors

1. Create collector in the proxy service (apps/proxy/src/collectors/)
2. Implement standard collector interface in Rust
3. Add tests in apps/proxy/tests/
4. Update documentation in docs/
5. Submit PR

### Adding New Analytics Features

1. Update data model in packages/analytics-rs/ if needed
2. Implement processing logic in Rust
3. Add dashboard visualizations in apps/web/
4. Write tests for both Rust and TypeScript components
5. Update documentation

### Architecture Considerations

When adding features, consider:
- Maintain single-tenant simplicity while keeping architecture extensible
- Design for future multi-tenant capabilities when appropriate
- Keep commercial feature requirements in mind for future licensing
- Document any design decisions that affect future commercial features
- Use devbox for all command execution to ensure consistency

## Documentation

### Updating Documentation

- Keep README.md up to date
- Update API documentation for new endpoints
- Add inline code comments for complex logic
- Update architecture diagrams for structural changes

### Documentation Standards

- Use clear, concise language
- Include code examples
- Add diagrams for complex concepts
- Keep documentation in sync with code changes

## Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

### Communication

- Use GitHub issues for bug reports and feature requests
- Use GitHub discussions for questions and ideas
- Be patient with maintainers and community members
- Search existing issues before creating new ones

## Release Process

Releases are managed by maintainers following semantic versioning:
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality additions
- PATCH version for backwards-compatible bug fixes

## Getting Help

- **Documentation**: Check the [docs](docs/) directory
- **Issues**: Search or create GitHub issues
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: Join our Discord community (link in README)

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project documentation

Thank you for contributing to the AI Analytics Dashboard!