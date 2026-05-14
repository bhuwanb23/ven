use anyhow::Result;
use clap::{Parser, Subcommand};

// Command modules
pub mod add;
pub mod check;
pub mod check_add;
pub mod docs;
pub mod graph;
pub mod init;
pub mod install;
pub mod list;
pub mod lockfile;
pub mod remove;
pub mod resolve;
pub mod scan;
pub mod setup;
pub mod shell;
pub mod status;
pub mod sync;
pub mod upgrade;
pub mod why;

/// ven — Any-first intelligent version and dependency manager
///
/// A modern, fast, and secure runtime/version manager.
/// Manages language runtimes and project tooling with intelligent compatibility checking.
#[derive(Parser)]
#[command(
    name = "ven",
    version,
    about,
    long_about = None,
    after_help = "Examples:\n  ven setup                    # Shell hooks + profiles\n  ven install node 20          # Install Node.js\n  ven install python 3.12.7    # Install Python runtime\n  ven install go 1.21.5        # Install Go toolchain\n  ven install rust 1.75.0      # Install Rust toolchain\n  ven install java 21          # Install Java JDK\n  ven install deno 1.40.0      # Install Deno runtime\n  ven install ruby 3.4.2       # MRI Ruby (Win: RubyInstaller2; Unix: ruby-builder)\n  ven list                     # All installed runtimes (node, python, go, rust, java, deno, ruby …)\n  ven use                      # Export PATH/env for cwd (evaluate in shell)\n  ven deactivate               # Undo PATH overlay in this terminal\n  ven init --template          # Create ven.toml interactively\n  ven add express vite         # Add packages + sync ven.toml\n  ven status --verbose         # Show project runtime + packages\n  ven upgrade --all --apply    # Upgrade pinned packages\n  ven remove --cleanup         # Remove orphaned packages\n\nDocumentation (repo): docs/README.md — Language & command reference: docs/languages.md, docs/commands-reference.md"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a language runtime (e.g., Node.js, Python, Go)
    ///
    /// Downloads and installs runtime versions. Supports version specs like
    /// "20", "3.12", "1.21", "lts", or "latest".
    ///
    /// Examples:
    ///   ven install node 20        # Install latest Node.js 20.x
    ///   ven install node 20.11.0   # Install specific version
    ///   ven install node           # Show available versions
    ///   ven install                # Interactive mode
    #[command(
        visible_alias = "i",
        long_about = "Install a language runtime (e.g., Node.js, Python, Go, Rust, Java, Deno)\n\nDownloads and installs versions from official sources.\nSupports version specs like \"20\", \"3.12\", \"1.21\", \"1.75\", \"21\", \"1.40\", \"lts\", or \"latest\".\n\nExamples:\n  ven install node 20        # Install latest Node.js 20.x\n  ven install python 3.12.7  # Install specific Python patch\n  ven install go 1.21        # Install latest Go 1.21.x\n  ven install rust 1.75      # Install Rust 1.75.x\n  ven install java 21        # Install Java 21.x\n  ven install deno 1.40      # Install latest Deno 1.40.x\n  ven install node           # Show available versions\n  ven install                # Interactive mode"
    )]
    Install {
        /// Language to install (e.g. "node", "python", "go")
        language: Option<String>,

        /// Version to install: "20", "20.11.0", "lts", "latest"
        version: Option<String>,
    },

    /// List installed language runtimes (node, python, …)
    ///
    /// With no language argument, shows every registered language. Use a name
    /// to filter (e.g. `node`, `python`).
    ///
    /// Examples:
    ///   ven list                   # All languages installed under ven
    ///   ven list node              # Node.js only
    ///   ven list python            # Python only
    ///   ven list --verbose         # Disk usage and install date
    ///   ven list --json            # JSON (object keyed by language if listing all)
    #[command(
        long_about = "List installed runtimes managed by ven\n\nWithout a language, prints Node, Python, and any other registered languages.\nPass a language name to restrict output.\n\nExamples:\n  ven list\n  ven list node\n  ven list python\n  ven list --verbose\n  ven list --json"
    )]
    List {
        /// Language to list (omit to show all languages)
        language: Option<String>,

        /// Show detailed info (installation date, disk size)
        #[arg(short, long)]
        verbose: bool,

        /// Output as JSON for scripting and automation
        #[arg(long)]
        json: bool,
    },

    /// Apply nearest ven.toml runtime to your shell session (prints exports; evaluate in shell)
    ///
    /// Same behavior as `ven shell activate`. After `ven setup`, hooks define **`ven-use`** so you
    /// rarely call this by hand.
    ///
    /// Examples:
    ///   ven use
    ///   ven use .
    ///   ven use path/to/project
    #[command(
        long_about = "Apply nearest ven.toml runtime settings for a directory.\n\nPrints PATH/env assignments for your shell — they must be evaluated in-process\n(Run `ven-use` once hooks are installed, or pipe to Invoke-Expression / eval).\n\nExamples:\n  ven use .\n  ven use ~/my-app"
    )]
    Use {
        /// Directory (default: current folder)
        #[arg(default_value = ".")]
        dir: String,
    },

    /// Print commands to revert ven's PATH overlay in this terminal (same session as hooks).
    #[command(
        visible_alias = "d",
        long_about = "Print shell code to undo ven's PATH/environment overlay and pause auto-prepending of the project \"venv/\" (sets VEN_SKIP_PROJECT_VENV=1). Run \"ven-use\" to resume putting the env on PATH.\n\nRequires the shell hook globals (e.g. VEN_ORIGINAL_PATH). Evaluate in-process:\n\n  PowerShell:  iex ((ven deactivate) -join \"`n\")\n  bash/zsh:    eval \"$(ven deactivate)\"\n\nExamples:\n  ven deactivate"
    )]
    Deactivate,

    /// Show current project status and configuration
    ///
    /// Displays ven.toml configuration, installed runtimes, package status (when applicable),
    /// environment variables, and a short health summary.
    ///
    /// Examples:
    ///   ven status                 # Show basic status
    ///   ven status --verbose       # Show detailed info with disk usage
    ///   ven status --json          # JSON output for CI/CD
    ///   ven status --fix           # Auto-fix detected issues (where supported)
    #[command(
        long_about = "Show current project status and configuration\n\nDisplays ven.toml configuration, installed runtimes,\npackage status, environment variables, and project health information.\n\nExamples:\n  ven status                 # Show basic status\n  ven status --verbose       # Show detailed info with disk usage\n  ven status --json          # JSON output for CI/CD\n  ven status --fix           # Auto-fix detected issues"
    )]
    Status {
        /// Output as JSON for scripting and CI/CD pipelines
        #[arg(long)]
        json: bool,

        /// Show detailed info (disk usage, package compatibility, env vars)
        #[arg(short, long)]
        verbose: bool,

        /// Fix all detected issues automatically (install missing packages)
        #[arg(long)]
        fix: bool,
    },

    /// Initialize a new ven.toml configuration file
    ///
    /// Creates a ven.toml file in the current directory with a runtime
    /// version and package declarations for project isolation.
    ///
    /// Examples:
    ///   ven init                   # Create with default settings
    ///   ven init --template        # Interactive template selection
    ///   ven init --with-packages   # Add packages interactively
    ///   ven init --validate        # Validate after creation
    #[command(
        long_about = "Initialize a new ven.toml configuration file\n\nCreates a ven.toml file in the current directory with a runtime version\nand optional package declarations. This enables project-specific version management\nand dependency isolation.\n\nExamples:\n  ven init                   # Create with default settings\n  ven init --template        # Interactive template selection\n  ven init --with-packages   # Add packages interactively\n  ven init --validate        # Validate after creation"
    )]
    Init {
        /// Use interactive template selection (React, Vue, Next.js, etc.)
        #[arg(long)]
        template: bool,

        /// Add popular packages interactively after creating ven.toml
        #[arg(long)]
        with_packages: bool,

        /// Validate the setup after creation (check runtime is installed)
        #[arg(long)]
        validate: bool,

        /// Node.js version to use (legacy option, kept for backward compatibility)
        #[arg(short, long)]
        node: Option<String>,

        /// Non-interactive: scaffold ven.toml with the given language (and version).
        /// Picks the newest installed runtime if `--ver` is omitted. Required in CI.
        #[arg(short = 'l', long)]
        lang: Option<String>,

        /// Pin the language version (e.g. `20.20.2`). Honoured only with --lang.
        #[arg(long)]
        ver: Option<String>,

        /// Skip every prompt; combine with --lang to fully scaffold a project headlessly.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Add package(s) to the project with compatibility checking
    ///
    /// Installs npm packages and adds them to ven.toml (node projects)
    /// compatibility verification.
    ///
    /// Examples:
    ///   ven add express              # Add latest Express
    ///   ven add express@4.18.2       # Add specific version
    ///   ven add react vite           # Add multiple packages
    ///   ven add lodash --skip-check  # Skip node compatibility check
    ///   ven add express --dry-run    # Preview before installing
    ///   ven add socket.io --verbose  # Show full dependency tree
    #[command(
        long_about = "Add package(s) to the project with compatibility checking\n\nInstalls packages and automatically adds them to ven.toml.\nFor node projects, performs runtime compatibility checking to prevent issues.\n\nExamples:\n  ven add express              # Add latest Express\n  ven add express@4.18.2       # Add specific version\n  ven add react vite           # Add multiple packages\n  ven add lodash --skip-check  # Skip compatibility check\n  ven add express --dry-run    # Preview before installing\n  ven add socket.io --verbose  # Show full dependency tree"
    )]
    Add {
        /// Package name(s) with optional version, e.g., "express" or "express@4.18.2"
        #[arg(required = true)]
        packages: Vec<String>,
        /// Skip compatibility checking (use with caution)
        #[arg(long)]
        skip_check: bool,
        /// Show dependency preview without installing
        #[arg(long)]
        dry_run: bool,
        /// Show full dependency tree
        #[arg(short, long)]
        verbose: bool,
        /// Assume "yes" for the final install prompt (default for non-TTY shells / CI)
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Check add compatibility without installing (dependency intelligence)
    ///
    /// Examples:
    ///   ven check-add express
    ///   ven check-add react@18 --json
    #[command(visible_alias = "ca")]
    CheckAdd {
        /// Package(s) with optional version (`pkg` or `pkg@1.2.3`)
        #[arg(required = true)]
        packages: Vec<String>,
        /// JSON output
        #[arg(long)]
        json: bool,
    },

    /// Show dependency graph / last intelligence snapshot
    ///
    /// Examples:
    ///   ven graph
    ///   ven graph --json
    ///   ven graph --resolve
    Graph {
        /// JSON output
        #[arg(long)]
        json: bool,
        /// Skip SQLite snapshot and show live manifest / node_modules snapshot
        #[arg(long)]
        resolve: bool,
    },

    /// Show why a package is installed (reverse dependency lookup)
    ///
    /// Displays the dependency chain showing all packages that depend on the target,
    /// and traces back to the root ven.toml entries. Also indicates whether it's safe to remove.
    ///
    /// Examples:
    ///   ven why express         # Why is express installed?
    ///   ven why accepts         # What depends on accepts?
    Why {
        /// Package name to analyze
        #[arg(required = true)]
        package: String,
    },

    /// Remove package(s) with dependency analysis and safety checks
    ///
    /// Safely removes npm packages after checking for dependents
    /// and providing interactive warnings.
    ///
    /// Examples:
    ///   ven remove express           # Remove with dependency check
    ///   ven remove lodash --force    # Force remove without checks
    ///   ven remove react vite        # Remove multiple packages
    ///   ven remove --dry-run         # Preview what would be removed
    ///   ven remove --cleanup         # Find and remove orphaned packages
    #[command(
        long_about = "Remove package(s) with dependency analysis and safety checks\n\nSafely removes npm packages from your project. Analyzes the dependency\ngraph to warn about packages that depend on the target before removal.\n\nExamples:\n  ven remove express           # Remove with dependency check\n  ven remove lodash --force    # Force remove without checks\n  ven remove react vite        # Remove multiple packages\n  ven remove --dry-run         # Preview what would be removed\n  ven remove --cleanup         # Find and remove orphaned packages"
    )]
    Remove {
        /// Package name(s) to remove (supports multiple packages)
        packages: Vec<String>,

        /// Skip dependency checking and force removal
        #[arg(long)]
        force: bool,

        /// Preview removal without actually removing packages
        #[arg(long)]
        dry_run: bool,

        /// Output removal status as JSON for CI/CD automation
        #[arg(long)]
        json: bool,

        /// Show detailed analysis (disk space freed, transitive dependencies)
        #[arg(short, long)]
        verbose: bool,

        /// Find and remove orphaned/unused dependencies automatically
        #[arg(long)]
        cleanup: bool,

        /// Assume "yes" for confirmation prompts (default for non-TTY shells / CI)
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Upgrade packages to latest compatible versions
    ///
    /// Shows available upgrades and optionally applies them with
    /// Runtime compatibility verification.
    ///
    /// Examples:
    ///   ven upgrade express          # Preview Express upgrade
    ///   ven upgrade express --apply  # Apply the upgrade
    ///   ven upgrade --all            # Preview all upgrades
    ///   ven upgrade --all --apply    # Upgrade all packages
    ///   ven upgrade react --dry-run  # Preview without changes
    ///   ven upgrade --all --apply --force  # CI/CD mode (no prompts)
    #[command(
        long_about = "Upgrade packages to latest compatible versions\n\nChecks for available package upgrades and verifies compatibility\nbefore applying them. Shows preview by default, use --apply to upgrade.\n\nExamples:\n  ven upgrade express          # Preview Express upgrade\n  ven upgrade express --apply  # Apply the upgrade\n  ven upgrade --all            # Preview all upgrades\n  ven upgrade --all --apply    # Upgrade all packages\n  ven upgrade react --dry-run  # Preview without changes\n  ven upgrade --all --apply --force  # CI/CD mode (no prompts)"
    )]
    Upgrade {
        /// Package name(s) to upgrade (supports multiple packages)
        packages: Vec<String>,

        /// Actually perform the upgrade (default: preview only)
        #[arg(long)]
        apply: bool,

        /// Preview upgrades without making any changes
        #[arg(long)]
        dry_run: bool,

        /// Output upgrade status as JSON for CI/CD automation
        #[arg(long)]
        json: bool,

        /// Show detailed analysis (changelog URLs, disk space, compatibility)
        #[arg(short, long)]
        verbose: bool,

        /// Upgrade all packages declared in ven.toml
        #[arg(long)]
        all: bool,

        /// Skip prompts and force apply (for CI/CD automation)
        #[arg(long)]
        force: bool,
    },

    /// Write `ven.lock` from resolved graphs for `[packages]` (npm / Node-Bun projects)
    ///
    /// Merges per-package simulations into one pinned graph and records a content hash.
    #[command(
        long_about = "Generate ven.lock\n\nResolves each package declared in ven.toml, merges the dependency graphs,\nand writes ven.lock with a cryptographic content hash for integrity checks.\n\nExample:\n  ven lock"
    )]
    Lock,

    /// Validate `ven.lock` and install pinned root packages
    ///
    /// Verifies internal graph consistency and semver constraints before `npm install`.
    #[command(
        long_about = "Sync from ven.lock\n\nReads ven.lock, validates structure and constraints, updates the intelligence cache,\nand installs each root package at its locked version.\n\nExamples:\n  ven sync                  # validate + install\n  ven sync --dry-run        # validate, print install plan, exit 0\n  ven sync --check          # CI mode: report drift; exit 1 if any\n  ven sync --json"
    )]
    Sync {
        /// Validate only; do not run npm install. Always exits 0 on a valid lock.
        #[arg(long)]
        dry_run: bool,
        /// Drift-aware CI mode: validate the lock, then compare it against
        /// `node_modules/` (npm) or installed pip packages (Python). Exits
        /// non-zero when any drift is found. Implies no install.
        #[arg(long)]
        check: bool,
        /// Machine-readable result
        #[arg(long)]
        json: bool,
        /// Skip validation (not recommended)
        #[arg(long)]
        skip_validate: bool,
    },

    /// Automatically resolve dependency conflicts and apply fixes
    ///
    /// Scans the current dependency graph, identifies package and engine
    /// conflicts, computes an optimal version resolution set, and applies
    /// fixes with one command.
    #[command(
        long_about = "Automatically resolve dependency conflicts and apply fixes.\n\nScans the current dependency graph, estimates the best resolution set,\nand updates package versions to restore compatibility.\n\nExample:\n  ven resolve"
    )]
    Resolve,

    /// Health report — security advisories (OSV) + runtime EOL alerts
    ///
    /// Default behavior runs both security and EOL checks; use `--security` or
    /// `--eol` to scope. Exits non-zero on any HIGH/CRITICAL CVE or passed-EOL
    /// runtime so it's CI-safe.
    ///
    /// Examples:
    ///   ven check
    ///   ven check --security
    ///   ven check --eol
    ///   ven check --json
    #[command(
        long_about = "Health report for the current project: package CVEs from osv.dev plus\nruntime end-of-life status from endoflife.date. With no flags, both checks run.\n\nExit code:\n  0  no actionable issues\n  1  any HIGH/CRITICAL CVE OR a passed-EOL runtime\n\nResults are cached locally (CVEs 6h, EOL 24h) and served stale on network\nfailure so you can keep working offline.\n\nExamples:\n  ven check\n  ven check --security    # CVE only\n  ven check --eol         # EOL only\n  ven check --json        # CI / scripting"
    )]
    Check {
        /// CVE scan only
        #[arg(long)]
        security: bool,
        /// Runtime end-of-life check only
        #[arg(long)]
        eol: bool,
        /// JSON output
        #[arg(long)]
        json: bool,
    },

    /// Source-tree scanners (currently: ghost dependency detection)
    ///
    /// `--ghosts` walks your source tree (gitignore-aware) to find packages
    /// you `import`/`require` but never declared in any manifest. `--fix`
    /// adds them to `ven.toml [packages]`.
    ///
    /// Examples:
    ///   ven scan --ghosts
    ///   ven scan --ghosts --fix
    ///   ven scan --ghosts --json
    #[command(
        long_about = "Source-tree scanners.\n\n--ghosts       Walk source files (.gitignore-aware) and report packages\n               imported but not declared in ven.toml or any native\n               manifest (package.json, requirements.txt, Cargo.toml,\n               go.mod, Gemfile, pom.xml, deno.json).\n\n--fix          Add each ghost to ven.toml [packages] using `latest` as the\n               spec (npm-family resolves to highest Node-compatible).\n\nExit code: 1 in JSON mode when ghosts are found (CI), 0 otherwise."
    )]
    Scan {
        /// Detect undeclared package imports
        #[arg(long)]
        ghosts: bool,
        /// Add detected ghosts to ven.toml [packages]
        #[arg(long)]
        fix: bool,
        /// JSON output
        #[arg(long)]
        json: bool,
    },

    /// Show docs for an installed package — version-pinned to ven.lock
    ///
    /// Resolves the version from `ven.lock` → `ven.toml [packages]` → installed
    /// manifest, then renders the README/description in the terminal.
    /// `--browser` opens the canonical docs URL in your default browser.
    /// `--diff v1 v2` fetches the same package's READMEs at two versions
    /// and shows a unified line diff.
    ///
    /// Examples:
    ///   ven docs express
    ///   ven docs requests --browser
    ///   ven docs express --diff 4.18.0 4.18.2
    #[command(
        long_about = "Open or render documentation for a package, pinned to the version\nrecorded in ven.lock (or ven.toml / installed manifest if no lock exists).\n\nSupported sources:\n  Node / Bun       npm registry README\n  Python           PyPI description\n  Rust             docs.rs HTML\n  Go               pkg.go.dev\n  Java             javadoc.io (URL only)\n  Ruby             rubygems.org\n  Deno             deno.land / jsr.io (URL only)\n\n--browser        open the canonical URL (cross-platform via webbrowser)\n--diff V1 V2     fetch readme at V1 and V2, render a unified diff\n--json           machine-readable\n\nResults are cached for 7 days (docs rarely change for a fixed version)."
    )]
    Docs {
        /// Package name (must be in `ven.toml [packages]` or installed)
        #[arg(required = true)]
        package: String,
        /// Open canonical docs URL in default browser
        #[arg(long)]
        browser: bool,
        /// Diff README between two versions
        #[arg(long, num_args = 2, value_names = ["V1", "V2"])]
        diff: Option<Vec<String>>,
        /// JSON output
        #[arg(long)]
        json: bool,
    },

    /// One-time setup: Install shell hooks for automatic version switching
    ///
    /// Configures your shell (bash, zsh, fish, PowerShell) to automatically
    /// apply runtime settings when you change directories.
    ///
    /// Examples:
    ///   ven setup                  # Run interactive setup
    ///
    /// After setup, add to your shell rc file:
    ///   eval "$(ven shell hook <shell>)"
    #[command(
        long_about = "One-time setup: Install shell hooks for automatic runtime switching\n\nConfigures your shell (bash, zsh, fish, PowerShell) to automatically\napply runtime settings when you change directories with a ven.toml file.\n\nThis is a one-time setup that enables seamless version management.\n\nExamples:\n  ven setup                  # Run interactive setup\n\nAfter setup, the installer will show you what to add to your shell rc file:\n  eval \"$(ven shell hook <shell>)\"\n\nSupported shells: bash, zsh, fish, powershell"
    )]
    Setup,

    /// Shell integration (internal — called by shell hook)
    #[command(hide = true)] // hides from --help
    Shell {
        #[command(subcommand)]
        action: ShellCommands,
    },
}

#[derive(Subcommand)]
pub enum ShellCommands {
    /// Print shell hook code (eval this in your rc file)
    Hook { shell: String },
    /// Compute and print PATH exports for current directory
    Activate { dir: String },
    /// Print commands to clear ven PATH overlay (same as `ven deactivate`)
    Deactivate,
    /// Install hook into shell profile for auto-loading
    Install,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Install { language, version } => {
            match (language, version) {
                (Some(lang), Some(ver)) => {
                    // Both provided: direct install
                    install::cmd_install(&lang, &ver)
                }
                (Some(lang), None) => {
                    // Language only: show available versions
                    install::cmd_install_with_version_list(&lang)
                }
                (None, None) => {
                    // Neither provided: interactive mode
                    install::cmd_install_interactive()
                }
                _ => {
                    // Partial args (version without language): show error
                    Err(anyhow::anyhow!(
                        "Provide language first, then version.\n\nExamples:\n  ven install node 20        # Direct install\n  ven install node           # Show available versions\n  ven install                # Interactive mode"
                    ))
                }
            }
        }
        Commands::List {
            language,
            verbose,
            json,
        } => list::cmd_list(language.as_deref(), verbose, json),
        Commands::Use { dir } => shell::cmd_use(&dir),
        Commands::Deactivate => shell::cmd_shell_deactivate(),
        Commands::Status { json, verbose, fix } => status::cmd_status(json, verbose, fix),
        Commands::Init {
            node,
            template,
            with_packages,
            validate,
            lang,
            ver,
            yes,
        } => init::cmd_init(
            node.as_deref(),
            template,
            with_packages,
            validate,
            lang.as_deref(),
            ver.as_deref(),
            yes,
        ),
        Commands::Add {
            packages,
            skip_check,
            dry_run,
            verbose,
            yes,
        } => add::cmd_add(&packages, skip_check, dry_run, verbose, yes),
        Commands::CheckAdd { packages, json } => check_add::cmd_check_add(&packages, json),
        Commands::Graph { json, resolve } => graph::cmd_graph(json, resolve),
        Commands::Why { package } => why::cmd_why(&package),
        Commands::Remove {
            packages,
            force,
            dry_run,
            json,
            verbose,
            cleanup,
            yes,
        } => remove::cmd_remove(&packages, force, dry_run, json, verbose, cleanup, yes),
        Commands::Upgrade {
            packages,
            apply,
            dry_run,
            json,
            verbose,
            all,
            force,
        } => upgrade::cmd_upgrade(&packages, apply, dry_run, json, verbose, all, force),
        Commands::Lock => lockfile::cmd_lock(),
        Commands::Sync {
            dry_run,
            check,
            json,
            skip_validate,
        } => sync::cmd_sync(dry_run, check, json, skip_validate),
        Commands::Resolve => resolve::cmd_resolve(),
        Commands::Check {
            security,
            eol,
            json,
        } => check::cmd_check(security, eol, json),
        Commands::Scan { ghosts, fix, json } => scan::cmd_scan(ghosts, fix, json),
        Commands::Docs {
            package,
            browser,
            diff,
            json,
        } => docs::cmd_docs(&package, browser, diff.as_deref(), json),
        Commands::Setup => setup::cmd_setup(),
        Commands::Shell { action } => match action {
            ShellCommands::Hook { shell } => shell::cmd_shell_hook(&shell),
            ShellCommands::Activate { dir } => shell::cmd_shell_activate(&dir),
            ShellCommands::Deactivate => shell::cmd_shell_deactivate(),
            ShellCommands::Install => shell::cmd_shell_install(),
        },
    }
}

// All command implementations have been moved to their own modules:
// - src/cli/install.rs
// - src/cli/list.rs
// - src/cli/status/
// - src/cli/setup.rs
// - src/cli/shell.rs
// - src/cli/init.rs
// - src/cli/add.rs
// - src/cli/remove.rs
// - src/cli/upgrade.rs
