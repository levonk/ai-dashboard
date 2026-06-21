# ai-analytics-proxy

AI Analytics Proxy Server - Routes AI requests and collects telemetry

## Quick Start

```bash
# Build the project
just build

# Run tests
just test

# Run linting
just lint

# Run in development mode
just dev
```

## Usage

### Dry Run Mode

Preview changes without executing them:

```bash
ai-analytics-proxy --dry-run input.txt
```

Dry run mode shows what would happen without making any changes. Output includes:
- Source file information
- Operation details
- Destructive operation warnings (if applicable)
- Status indicating no changes were made

### Destructive Operations

The CLI detects potentially destructive operations and prompts for confirmation before executing them. Destructive operations include:
- File deletion
- Directory deletion
- File overwrites
- Configuration changes
- Database operations

#### Confirmation Prompts

When a destructive operation is detected, you'll see:

```
⚠️  Delete file: important_file.txt [y/N]:
```

#### Bypassing Confirmation

Use the `--force` flag to bypass confirmation prompts:

```bash
ai-analytics-proxy --force delete important_file.txt
```

**Note:** Use `--force` with caution, especially in automation scripts.

#### Safe Mode

Dry run mode automatically bypasses confirmation prompts since no changes are made:

```bash
ai-analytics-proxy --dry-run delete important_file.txt
```

### Output Formats and Field Selection

The CLI supports multiple output formats optimized for different use cases:

#### Output Formats

- **TOON** (Token-Oriented Object Notation): Token-efficient format for AI agents (~40% savings vs JSON)
- **JSON**: Standard JSON format for machine processing
- **Human**: Human-readable formatted output

Format selection is automatic based on mode:
- Agent mode defaults to TOON format
- Human mode defaults to human-readable output

```bash
# Override format
ai-analytics-proxy --format toon input.txt
ai-analytics-proxy --format json input.txt
ai-analytics-proxy --format human input.txt

# Quick TOON output
ai-analytics-proxy --toon input.txt
```

#### Field Selection

In agent mode, the CLI uses minimal default schemas to reduce token consumption. You can select specific fields using the `--fields` flag:

```bash
# Select specific fields (comma-separated)
ai-analytics-proxy --fields id,name,status input.txt

# Combine with format selection
ai-analytics-proxy --format toon --fields id,name input.txt
```

Default schemas include 3-4 essential fields (identifier, title, status) while excluding long-form content (descriptions, bodies) from list views. Use `--fields` to request additional fields when needed.

### Man Pages

The CLI includes traditional Unix man pages for comprehensive documentation.

#### Viewing Man Pages

You can view the man page in two ways:

```bash
# Using the --man flag
ai-analytics-proxy --man

# Or using the man command (after installation)
man ai-analytics-proxy
```

#### Generating Man Pages

Man pages are generated from templates during the build process:

```bash
# Generate man pages
just generate-man-pages

# Validate man page format
just validate-man-pages
```

#### Installing Man Pages

To install man pages system-wide (requires sudo):

```bash
just install-man-pages
```

This installs the man page to `/usr/local/share/man/man1/` and updates the man database.

#### Man Page Sections

The man page includes the following sections:
- **NAME**: Command name and brief description
- **SYNOPSIS**: Command usage syntax
- **DESCRIPTION**: Detailed description of the command
- **OPTIONS**: All available command-line options
- **COMMANDS**: Available subcommands
- **ENVIRONMENT**: Environment variables that affect behavior
- **FILES**: Configuration and data file locations
- **EXAMPLES**: Usage examples
- **EXIT STATUS**: Exit code meanings
- **BUGS**: Where to report bugs
- **SEE ALSO**: Related documentation

## Agent Integration

The CLI provides two integration methods for AI agents:

### Agent Hooks

The CLI can register session hooks with agent platforms (Claude Code, Codex):
- Hooks are registered in platform-specific config files
- Installation is idempotent (re-running updates existing hooks)
- Command paths are resolved to absolute paths when in PATH

Install hooks for Claude Code:
  ai-analytics-proxy --install-agent-hooks claude
  ai-analytics-proxy session install-hooks claude

Install hooks for Codex:
  ai-analytics-proxy --install-agent-hooks codex
  ai-analytics-proxy session install-hooks codex

### Agent Skill

The CLI provides an installable agent skill for discoverability via agentskills.io:
- Skill is generated from CLI help and examples
- Live state is stripped to ensure static content
- Command examples are in non-interactive form
- Skill includes trigger-shaped frontmatter for agent discovery

Generate skill:
  ai-analytics-proxy skill generate --output SKILL.md

Check skill freshness:
  ai-analytics-proxy skill check

**Note**: You only need to use **one** integration method:
- Use **agent hooks** for session context and lifecycle integration
- Use **agent skill** for command discovery and documentation
- Both methods are optional; choose based on your use case

## Development

This project uses **devbox** for environment management and **just** for task running.

### Prerequisites

- [devbox](https://www.jetify.com/devbox)
- [just](https://github.com/casey/just)

### Setup

```bash
# Enter the devbox shell
devbox shell

# Or run commands directly
devbox run just build
```

## Project Structure

```
.
├── src/
│   └── main.rs          # CLI entry point
├── tests/
│   └── cli_tests.rs     # Integration tests
├── docs/                # Documentation
├── internal-docs/       # Architecture decisions and specs
├── Cargo.toml           # Rust dependencies
├── devbox.json          # Development environment
├── justfile             # Task runner
└── README.md            # This file
```

## License

[MIT](LICENSE)
