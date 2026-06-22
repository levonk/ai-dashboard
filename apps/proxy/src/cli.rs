use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;

mod emitter;

/// AI Analytics Proxy Server - Routes AI requests and collects telemetry
///
/// # Content-First Behavior
///
/// Running the CLI with no arguments shows the most relevant live content instead of a usage manual:
///   ai-analytics-proxy  # Shows state summary and contextual suggestions
///
/// This enables agents to see actual state immediately and act without a second call.
/// Detailed help remains available via the --help flag.
///
/// # Examples
///
/// Start HTTP server:
///   ai-analytics-proxy serve
///   ai-analytics-proxy serve --port 3000
///
/// Process files:
///   ai-analytics-proxy file1.txt file2.txt
///
/// Use glob patterns:
///   ai-analytics-proxy "**/*.txt"
///
/// Read from stdin:
///   ai-analytics-proxy -
///
/// Manage configuration:
///   ai-analytics-proxy config --show
///   ai-analytics-proxy config --validate
///
/// Manage daemon:
///   ai-analytics-proxy daemon --start
///   ai-analytics-proxy daemon --list-jobs running
///
/// Generate completions:
///   ai-analytics-proxy completion bash
///
/// Run diagnostics:
///   ai-analytics-proxy doctor --all
///
/// Start HTTP server:
///   ai-analytics-proxy serve
///   ai-analytics-proxy serve --port 3000
///
/// # Mode Selection
///
/// The CLI automatically detects the execution mode (agent vs human) based on:
/// - Explicit flags (--human, --interactive)
/// - Environment variables (AI_ANALYTICS_PROXY_MODE)
/// - Config file settings
/// - Auto-detection (TTY presence, agent session detection)
///
/// Force human mode:
///   ai-analytics-proxy --human file1.txt
///   ai-analytics-proxy --interactive file1.txt
///
/// Force agent mode via environment:
///   AI_ANALYTICS_PROXY_MODE=agent ai-analytics-proxy file1.txt
///
/// # Prompt Behavior
///
/// Interactive prompts are automatically suppressed in agent mode to enable non-interactive automation:
/// - Agent mode: All prompts are suppressed, operations proceed with flags alone
/// - Human mode: Prompts are shown by default for destructive operations
/// - --force flag: Bypasses all confirmation prompts in any mode
/// - --dry-run: Suppresses destructive confirmation prompts (preview only)
///
/// Force bypass of confirmation prompts:
///   ai-analytics-proxy --force file1.txt
///   ai-analytics-proxy --force --interactive file1.txt
///
/// # Output Format
///
/// The CLI supports multiple output formats optimized for different use cases:
/// - **TOON** (Token-Oriented Object Notation): Token-efficient format for AI agents (~40% savings vs JSON)
/// - **JSON**: Standard JSON format for machine processing
/// - **Human**: Human-readable formatted output
///
/// Format selection is automatic based on mode:
/// - Agent mode defaults to TOON format
/// - Human mode defaults to human-readable output
///
/// Override format:
///   ai-analytics-proxy --format toon file1.txt
///   ai-analytics-proxy --format json file1.txt
///   ai-analytics-proxy --format human file1.txt
///
/// Quick TOON output:
///   ai-analytics-proxy --toon file1.txt
///
/// # Content Truncation
///
/// Large text fields (descriptions, bodies, logs) are automatically truncated to reduce token consumption in agent mode.
/// Default truncation limit is 1000 characters (configurable via truncation_limit in config file).
///
/// Show full content without truncation:
///   ai-analytics-proxy --full file1.txt
///
/// Truncation metadata includes:
/// - Original content size
/// - Truncated content size
/// - Truncation indicator
/// - Help suggestion for retrieving full content (use --full flag)
///
/// # Aggregate Information
///
/// List commands include aggregate information to reduce the need for follow-up API calls:
/// - Total count: "count: 30 of 847 total" shows current page size vs total available
/// - Derived status: Lightweight summaries like "3/3 passed", "7 comments", "5m 30s"
///
/// Aggregates are automatically included in output when available from the backend.
/// This optimizes agent interactions by providing commonly-needed data upfront.
///
/// # Session Context
///
/// The CLI provides session context for ambient context injection into agent sessions:
/// - Directory: Current working directory
/// - Git: Repository name, branch, commit, dirty status (if in git repo)
/// - Config: Configuration file path and existence
/// - Presence: Tool availability in PATH and version
///
/// Get session context:
///   ai-analytics-proxy --session-context
///   ai-analytics-proxy session context
///
/// Session context is output in compact TOON format for token efficiency.
///
/// # Agent Hooks
///
/// The CLI can register session hooks with agent platforms (Claude Code, Codex):
/// - Hooks are registered in platform-specific config files
/// - Installation is idempotent (re-running updates existing hooks)
/// - Command paths are resolved to absolute paths when in PATH
///
/// Install hooks for Claude Code:
///   ai-analytics-proxy --install-agent-hooks claude
///   ai-analytics-proxy session install-hooks claude
///
/// Install hooks for Codex:
///   ai-analytics-proxy --install-agent-hooks codex
///   ai-analytics-proxy session install-hooks codex
///
/// # Agent Skill
///
/// The CLI provides an installable agent skill for discoverability via agentskills.io:
/// - Skill is generated from CLI help and examples
/// - Live state is stripped to ensure static content
/// - Command examples are in non-interactive form
/// - Skill includes trigger-shaped frontmatter for agent discovery
///
/// Generate skill:
///   ai-analytics-proxy skill generate --output SKILL.md
///
/// # Contextual Suggestions
///
/// The CLI provides intelligent, context-aware suggestions that help discover capabilities organically:
/// - 2-4 relevant suggestions per output based on current state
/// - Suggestions are formatted as structured `help[]` array in TOON output
/// - Suggestions are actionable (complete commands) and carry forward disambiguating flags
/// - Suggestions are context-aware (e.g., empty lists suggest creating, non-empty suggest viewing)
///
/// Suggestions are automatically included in TOON/JSON output for agent mode.
/// This enables agents to discover CLI capabilities without additional documentation calls.
///
/// Check skill freshness:
///   ai-analytics-proxy skill check
///
/// **Note**: You only need to use **one** integration method:
/// - Use **agent hooks** for session context and lifecycle integration
/// - Use **agent skill** for command discovery and documentation
/// - Both methods are optional; choose based on your use case
///
/// # Structured Errors
///
/// All errors are formatted in a structured format that enables programmatic error handling:
/// - Errors go to stdout in the same format as normal output (TOON/JSON/Human)
/// - Diagnostics and debug information go to stderr
/// - Error messages include actionable suggestions for resolution
/// - Error codes enable programmatic error handling
/// - Raw dependency output never leaks through
///
/// Error format includes:
/// - error: Error code (e.g., CONFIG_ERROR, NETWORK_ERROR)
/// - message: Human-readable error description
/// - context: Additional context information (when available)
/// - suggestions: Actionable suggestions for resolving the error
/// - exit_code: Numeric exit code for the error
///
/// Example error in TOON format:
///   error: CONFIG_ERROR
///   message: Configuration file not found
///   context:
///     path: /path/to/config.toml
///   suggestions:
///   - text: Initialize configuration with default settings
///     command: mytool --install
///     exit_code: 3
///
/// # Empty States
///
/// When commands return no results, the CLI provides definitive empty state messages:
/// - Clear indication that the query executed successfully
/// - Context about filter criteria and scope
/// - Suggestions for adjusting the query when applicable
/// - Exit code 0 (success) for empty results
///
/// Empty state format varies by output format:
/// - TOON/JSON: Structured object with "empty": true flag
/// - Human: Readable message with context and suggestions
///
/// This prevents agents from re-running commands with different flags to verify empty results.
///
/// # Idempotent Operations
///
/// State-changing operations (install, uninstall, start, stop, create, update, delete, close) are idempotent:
/// - Repeating a command when the desired state already exists returns exit code 0 (success)
/// - No-op operations provide descriptive acknowledgment messages
/// - Non-zero exit codes are reserved for situations where the intent cannot be satisfied
///
/// Idempotent behavior examples:
///   ai-analytics-proxy --install  # Already installed: "Completions already installed at /path/to/completion"
///   ai-analytics-proxy --uninstall  # Already uninstalled: "Completions already uninstalled (file not found)"
///
/// This enables agents to safely repeat operations without causing errors when the desired state is already achieved.
#[derive(Parser)]
#[command(name = "ai-analytics-proxy", version, about = "AI Analytics Proxy Server - Routes AI requests and collects telemetry", long_about = None)]
pub struct Cli {
    /// Input files or glob patterns. Use "-" for stdin.
    #[arg(value_name = "INPUTS")]
    pub inputs: Vec<String>,

    /// Override config file
    #[arg(long, env = "AI_ANALYTICS_PROXY_CONFIG")]
    pub config: Option<PathBuf>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Output as TOON format (token-efficient for agents)
    #[arg(long, help = "Output in TOON format (token-efficient for AI agents)")]
    pub toon: bool,

    /// Output format: toon, json, or human (default: auto-detect based on mode)
    #[arg(long, value_name = "FORMAT", help = "Output format: toon, json, or human (default: auto-detect based on mode)")]
    pub format: Option<String>,

    /// Select specific fields for output (comma-separated, e.g., --fields id,name,status)
    #[arg(long, value_name = "FIELDS", help = "Select specific output fields (comma-separated, e.g., id,name,status)")]
    pub fields: Option<String>,

    /// Show full content without truncation
    #[arg(long, help = "Disable content truncation and show full output")]
    pub full: bool,

    /// Quiet mode - suppress all output except errors
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// Verbose mode - increase logging verbosity (can be used multiple times)
    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Debug mode - enable debug-level logging
    #[arg(long, short = 'd')]
    pub debug: bool,

    /// Human mode - force human-optimized interaction
    #[arg(long, help = "Force human-optimized interaction mode")]
    pub human: bool,

    /// Color mode: auto, always, never (default: auto)
    #[arg(long, value_name = "MODE")]
    pub color: Option<String>,

    /// Disable colored output (deprecated: use --color=never)
    #[arg(long)]
    pub nocolor: bool,

    /// Disable pager
    #[arg(long)]
    pub no_pager: bool,

    /// Install shell completions and initialize config
    #[arg(long, help = "Install shell completion scripts and initialize configuration")]
    pub install: bool,

    /// Uninstall shell completions
    #[arg(long, help = "Remove shell completion scripts")]
    pub uninstall: bool,

    /// Interactive TUI mode (alias: --tui)
    /// Launch a terminal UI to configure arguments before execution
    #[arg(long, alias = "tui", help = "Launch interactive TUI mode for argument configuration")]
    pub interactive: bool,

    /// Dry run mode - preview changes without execution
    #[arg(long, help = "Preview changes without executing them")]
    pub dry_run: bool,

    /// Force mode - bypass confirmation prompts for destructive operations
    #[arg(long, help = "Bypass confirmation prompts for destructive operations")]
    pub force: bool,

    /// Daemon mode - pre-launch daemon and wait for jobs
    #[arg(long, help = "Pre-launch daemon process and wait for background jobs")]
    pub daemon: bool,

    /// No-daemon mode - force synchronous in-process operation
    #[arg(long, help = "Force synchronous in-process operation (disable daemon)")]
    pub no_daemon: bool,

    /// Privacy mode - enable privacy features with ignore lists
    #[arg(long, help = "Enable privacy mode with identifier anonymization")]
    pub privacy: bool,

    /// Display man page
    #[arg(long, help = "Display manual page")]
    pub man: bool,

    /// Output session context in compact TOON format
    #[arg(long, help = "Output session context in compact TOON format for ambient context injection")]
    pub session_context: bool,

    /// Install agent hooks for platform
    #[arg(long, value_name = "PLATFORM", help = "Install agent hooks (platform: claude, codex, generic)")]
    pub install_agent_hooks: Option<String>,

    /// Maximum memory limit (e.g., 512M, 2G)
    #[arg(long, value_name = "LIMIT", help = "Set maximum memory limit for operations")]
    pub max_memory: Option<String>,

    /// Maximum CPU usage as percentage (1-100)
    #[arg(long, value_name = "PERCENT", help = "Set maximum CPU usage percentage")]
    pub max_cpu: Option<u8>,

    /// Subcommand for management operations
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Management subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Manage configuration (show, reset, validate)
    ///
    /// # Examples
    ///
    /// Show current configuration:
    ///   ai-analytics-proxy config --show
    ///
    /// Reset to defaults:
    ///   ai-analytics-proxy config --reset
    ///
    /// Validate configuration:
    ///   ai-analytics-proxy config --validate
    Config {
        /// Show current configuration
        #[arg(long, help = "Display current configuration settings")]
        show: bool,

        /// Reset configuration to defaults
        #[arg(long, help = "Reset configuration to default values")]
        reset: bool,

        /// Validate configuration
        #[arg(long, help = "Validate configuration syntax and values")]
        validate: bool,
    },

    /// Manage daemon processes (start, stop, restart, status, jobs)
    ///
    /// # Examples
    ///
    /// Start daemon:
    ///   ai-analytics-proxy daemon --start
    ///
    /// List running jobs:
    ///   ai-analytics-proxy daemon --list-jobs running
    ///
    /// Cancel a job:
    ///   ai-analytics-proxy daemon --cancel-job <job-id>
    Daemon {
        /// Start daemon in background
        #[arg(long, help = "Start daemon process in background")]
        start: bool,

        /// Stop running daemon
        #[arg(long, help = "Stop running daemon process")]
        stop: bool,

        /// Restart daemon
        #[arg(long, help = "Restart daemon process")]
        restart: bool,

        /// Show daemon status
        #[arg(long, help = "Display daemon status information")]
        status: bool,

        /// List background jobs
        #[arg(long, value_name = "FILTER", help = "List jobs (filter: pending, running, completed, failed, cancelled)")]
        list_jobs: Option<String>,

        /// Cancel a background job by ID
        #[arg(long, value_name = "ID", help = "Cancel a specific background job by its ID")]
        cancel_job: Option<String>,
    },

    /// Generate shell completion scripts
    ///
    /// # Examples
    ///
    /// Generate bash completions:
    ///   ai-analytics-proxy completion bash > ~/.local/share/bash-completion/completions/ai-analytics-proxy
    ///
    /// Generate zsh completions:
    ///   ai-analytics-proxy completion zsh > ~/.local/share/zsh/site-functions/_ai-analytics-proxy
    Completion {
        /// Shell type (bash, zsh, fish, elvish, powershell)
        #[arg(value_name = "SHELL", help = "Shell type for completion script")]
        shell: String,
    },

    /// Run diagnostics and health checks
    ///
    /// # Examples
    ///
    /// Run all diagnostics:
    ///   ai-analytics-proxy doctor --all
    ///
    /// Check specific components:
    ///   ai-analytics-proxy doctor --config --terminal
    Doctor {
        /// Run all diagnostic checks
        #[arg(long, help = "Run all diagnostic checks")]
        all: bool,

        /// Check configuration
        #[arg(long, help = "Check configuration file and settings")]
        config: bool,

        /// Check daemon status
        #[arg(long, help = "Check daemon process status")]
        daemon: bool,

        /// Check terminal capabilities
        #[arg(long, help = "Check terminal size and capabilities")]
        terminal: bool,
    },

    /// Export collected data to portable formats
    ///
    /// # Examples
    ///
    /// Export to JSON:
    ///   ai-analytics-proxy export --format json --output data.json
    ///
    /// Export to CSV:
    ///   ai-analytics-proxy export --format csv --output data.csv
    Export {
        /// Export format (json, csv, yaml)
        #[arg(long, value_name = "FORMAT", help = "Export format (json, csv, yaml)")]
        format: Option<String>,

        /// Output file path
        #[arg(long, value_name = "PATH", help = "Output file path for exported data")]
        output: Option<PathBuf>,

        /// Include metadata in export
        #[arg(long, help = "Include metadata in export")]
        include_metadata: bool,

        /// Compress output file
        #[arg(long, help = "Compress output file")]
        compress: bool,
    },

    /// Manage emitter mode (status, validate, buffer, test, reset)
    ///
    /// # Examples
    ///
    /// Show emitter status:
    ///   ai-analytics-proxy emitter status
    ///
    /// Validate emitter configuration:
    ///   ai-analytics-proxy emitter validate
    ///
    /// Test connectivity:
    ///   ai-analytics-proxy emitter test --collector --queue
    Emitter {
        /// Show detailed status
        #[arg(long, help = "Show detailed emitter status")]
        detailed: bool,
        
        /// Validate configuration
        #[arg(long, help = "Validate emitter configuration")]
        validate: bool,
        
        /// Test collector connectivity
        #[arg(long, help = "Test collector connectivity")]
        test_collector: bool,
        
        /// Test queue connectivity
        #[arg(long, help = "Test queue connectivity")]
        test_queue: bool,
    },

    /// Session context and hook management
    ///
    /// # Examples
    ///
    /// Get session context:
    ///   ai-analytics-proxy session context
    ///
    /// Install agent hooks:
    ///   ai-analytics-proxy session install-hooks claude
    ///
    /// Install hooks for specific platform:
    ///   ai-analytics-proxy session install-hooks codex
    Session {
        /// Get session context in compact TOON format
        #[arg(long, help = "Output session context in compact TOON format")]
        context: bool,

        /// Install agent hooks for platform
        #[arg(long, value_name = "PLATFORM", help = "Install agent hooks (platform: claude, codex, generic)")]
        install_hooks: Option<String>,

        /// Register session-end hook
        #[arg(long, help = "Register session-end hook")]
        register_hook: bool,
    },

    /// Process exported data offline
    ///
    /// # Examples
    ///
    /// Analyze data:
    ///   ai-analytics-proxy process --input data.json --operation analyze
    ///
    /// Filter data:
    ///   ai-analytics-proxy process --input data.json --operation filter --filter-config '{"record_type": "test"}'
    Process {
        /// Input data file path
        #[arg(long, value_name = "PATH", help = "Input data file path")]
        input: PathBuf,

        /// Processing operation (filter, aggregate, transform, analyze)
        #[arg(long, value_name = "OP", help = "Processing operation (filter, aggregate, transform, analyze)")]
        operation: String,

        /// Output file path for results
        #[arg(long, value_name = "PATH", help = "Output file path for processing results")]
        output: Option<PathBuf>,

        /// Operation-specific configuration (JSON)
        #[arg(long, value_name = "JSON", help = "Operation-specific configuration as JSON")]
        operation_config: Option<String>,
    },

    /// Agent skill management (generate, check)
    ///
    /// # Examples
    ///
    /// Generate skill:
    ///   ai-analytics-proxy skill generate --output SKILL.md
    ///
    /// Check skill freshness:
    ///   ai-analytics-proxy skill check
    Skill {
        /// Generate SKILL.md content
        #[arg(long, help = "Generate SKILL.md content")]
        generate: bool,

        /// Check if skill is stale
        #[arg(long, help = "Check if committed SKILL.md is stale")]
        check: bool,

        /// Output file path for generated skill
        #[arg(long, value_name = "PATH", help = "Output file path for generated skill")]
        output: Option<PathBuf>,
    },

/// Start HTTP server
    ///
    /// # Examples
    ///
    /// Start server on default port:
    ///   ai-analytics-proxy serve
    ///
    /// Start server on custom port:
    ///   ai-analytics-proxy serve --port 3000
    ///
    /// Start server on specific host:
    ///   ai-analytics-proxy serve --host 127.0.0.1
    Serve {
        /// Port to listen on
        #[arg(long, value_name = "PORT", help = "Port to listen on")]
        port: Option<u16>,

        /// Host to bind to
        #[arg(long, value_name = "HOST", help = "Host to bind to")]
        host: Option<String>,
    },
}

impl Cli {
    /// Print usage information
    pub fn print_usage(&self) -> Result<()> {
        use clap::CommandFactory;
        Self::command().print_help()?;
        Ok(())
    }

    /// Print version information
    pub fn print_version(&self) -> Result<()> {
        use clap::CommandFactory;
        println!("{} {}", Self::command().get_name(), Self::command().get_version().unwrap_or("unknown"));
        Ok(())
    }

    /// Check if progress indicators should be shown
    pub fn show_progress(&self) -> bool {
        !self.quiet && atty::is(atty::Stream::Stdout)
    }

    /// Check if any subcommand is provided
    pub fn has_subcommand(&self) -> bool {
        self.command.is_some()
    }
}

/// Progress indicator utilities
pub struct Progress {
    quiet: bool,
    multi: MultiProgress,
}

impl Progress {
    /// Create a new progress indicator manager
    pub fn new(quiet: bool) -> Self {
        Self {
            quiet,
            multi: MultiProgress::new(),
        }
    }

    /// Create a progress bar for operations with known progress
    pub fn bar(&self, len: u64, message: &str) -> Option<ProgressBar> {
        if self.quiet {
            return None;
        }

        let bar = self.multi.add(ProgressBar::new(len));
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Some(bar)
    }

    /// Create a spinner for operations with unknown duration
    pub fn spinner(&self, message: &str) -> Option<ProgressBar> {
        if self.quiet {
            return None;
        }

        let spinner = self.multi.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(100));
        Some(spinner)
    }

    /// Clear all progress indicators
    pub fn clear(&self) {
        self.multi.clear().ok();
    }
}

#[cfg(test)]
mod tests;
