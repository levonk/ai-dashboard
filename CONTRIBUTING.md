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

- Python 3.10 or higher
- Node.js 18 or higher
- Docker and Docker Compose
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
   # Install Python dependencies
   pip install -r requirements.txt
   pip install -r requirements-dev.txt

   # Install Node dependencies
   cd frontend
   npm install
   cd ..

   # Start development services
   docker-compose up -d redis postgres
   ```

3. **Run Development Servers**
   ```bash
   # Backend API
   python -m api.main

   # Frontend (in another terminal)
   cd frontend
   npm run dev
   ```

## Code Style and Standards

### Python Code
- Follow PEP 8 style guidelines
- Use `black` for code formatting
- Use `flake8` for linting
- Use `mypy` for type checking
- Write docstrings for all public functions and classes

### JavaScript/TypeScript Code
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
# Python tests
pytest tests/

# JavaScript tests
cd frontend
npm test

# Integration tests
pytest tests/integration/

# Coverage report
pytest --cov=src tests/
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
├── api/                 # Backend API
│   ├── collectors/     # Data collectors
│   ├── processors/     # Analytics processors
│   ├── models/         # Data models
│   └── main.py         # API entry point
├── frontend/           # Frontend dashboard
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   └── utils/
│   └── package.json
├── collectors/         # Standalone collectors
│   ├── python/
│   ├── go/
│   └── javascript/
├── tests/              # Test files
├── docs/              # Documentation
├── scripts/           # Utility scripts
└── docker-compose.yml # Development services
```

## Feature Development

### Adding New Collectors

1. Create collector in appropriate language directory
2. Implement standard collector interface
3. Add tests
4. Update documentation
5. Submit PR

### Adding New Analytics Features

1. Update data model if needed
2. Implement processing logic
3. Add dashboard visualizations
4. Write tests
5. Update documentation

### Architecture Considerations

When adding features, consider:
- Maintain single-tenant simplicity while keeping architecture extensible
- Design for future multi-tenant capabilities when appropriate
- Keep commercial feature requirements in mind for future licensing
- Document any design decisions that affect future commercial features

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