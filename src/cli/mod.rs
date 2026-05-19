use anyhow::Result;
use clap::{Parser, Subcommand};

// Command modules
pub mod add;
pub mod check;
pub mod check_add;
pub mod delete;
pub mod docs;
pub mod graph;
pub mod init;
pub mod install;
pub mod list;
pub mod lockfile;
pub mod path;
pub mod remove;
pub mod resolve;
pub mod scan;
pub mod setup;
pub mod shell;
pub mod status;
pub mod sync;
pub mod uninstall;
pub mod update;
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
    after_help = "Examples:\n  ven setup                    # Shell hooks + profiles\n  ven install node 20          # Install Node.js\n  ven install python 3.12.7    # Install Python runtime\n  ven install go 1.21.5        # Install Go toolchain\n  ven install rust 1.75.0      # Install Rust toolchain\n  ven install java 21          # Install Java JDK\n  ven install deno 1.40.0      # Install Deno runtime\n  ven install ruby 3.4.2       # MRI Ruby (Win: RubyInstaller2; Unix: ruby-builder)\n  ven list                     # All installed runtimes (node, python, go, rust, java, deno, ruby …)\n  ven list python              # Only Python versions\n  ven delete                   # Wizard: pick a runtime to remove\n  ven delete python 3.12.7     # Delete a specific version\n  ven path show                # Where ven keeps its data on disk (size, free space, source)\n  ven path set D:\\ven          # Relocate storage to a new drive (move data + persist VEN_HOME)\n  ven use                      # Export PATH/env for cwd (evaluate in shell)\n  ven deactivate               # Undo PATH overlay in this terminal\n  ven init --template          # Create ven.toml interactively\n  ven add express vite         # Add packages + sync ven.toml\n  ven status --verbose         # Show project runtime + packages\n  ven upgrade --all --apply    # Upgrade pinned packages\n  ven remove --cleanup         # Remove orphaned packages\n  ven update                   # Self-update ven + ven-launcher to the latest release\n  ven uninstall --dry-run      # Preview the full-nuke teardown plan (since v0.1.7)\n  ven uninstall                # Remove ven, all runtimes, PATH entries, persisted env\n\nDocumentation (repo): docs/README.md — Language & command reference: docs/languages.md, docs/commands-reference.md"
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

    /// Delete an installed language runtime
    ///
    /// Removes the directory at `$VEN_HOME/<language>/<version>/`. Distinct
    /// from `ven remove`, which uninstalls *packages* from a project. By
    /// default refuses to delete the runtime currently resolved by the
    /// nearest `ven.toml` — pass `--force` to override.
    ///
    /// Examples:
    ///   ven delete                   # Wizard: pick language, then version
    ///   ven delete python            # Pick a Python version to delete
    ///   ven delete python 3.12.7     # Confirm, then delete
    ///   ven delete python 3.12.7 -y  # Skip the confirm prompt (CI)
    ///   ven delete python 3.12.7 --force --json
    #[command(
        long_about = "Delete an installed language runtime\n\nRemoves `$VEN_HOME/<language>/<version>/`. Distinct from `ven remove`,\nwhich uninstalls packages (npm / pip / cargo / gem / ...).\n\nBy default refuses to delete the runtime currently resolved by the\nnearest `ven.toml` (passing --force overrides this guard so you can\nclean up actively-pinned versions you no longer want).\n\nExamples:\n  ven delete                   # Wizard: pick language, then version\n  ven delete python            # Skip language picker; pick version\n  ven delete python 3.12.7     # Confirm, then delete\n  ven delete python 3.12.7 -y  # Skip confirm too (CI / scripts)\n  ven delete python 3.12.7 --force        # Allow deleting the active runtime\n  ven delete python 3.12.7 -y --json      # Machine-readable result"
    )]
    Delete {
        /// Language whose runtime should be deleted (omit to enter the wizard)
        language: Option<String>,

        /// Specific version to delete (omit to pick from a list)
        version: Option<String>,

        /// Skip the confirmation prompt (CI / scripts)
        #[arg(short = 'y', long)]
        yes: bool,

        /// Allow deleting the version that is currently active in `ven.toml`
        #[arg(long)]
        force: bool,

        /// Machine-readable output. Requires explicit <language> [version]
        /// args and -y / --yes (no interactive prompts in JSON mode).
        #[arg(long)]
        json: bool,
    },

    /// Manage where ven stores its data (since v0.1.6)
    ///
    /// Use this when your default `~/.ven` (or `%USERPROFILE%\.ven`) is on
    /// a full drive and you want to move every runtime, cache, and
    /// lockfile-state file somewhere else. The relocation is atomic and
    /// rolls back on failure; the new location is recorded in a global
    /// pointer file AND persisted as `VEN_HOME` in your user environment
    /// so child shells and external tools (npm, pip) pick it up too.
    ///
    /// Subcommands:
    ///   show      Print the current $VEN_HOME, its source, size, free disk
    ///   set DIR   Relocate to DIR (wizard by default; --move / --no-move / --pointer-only / -y / --json)
    ///   reset     Clear the pointer; ven home reverts to ~/.ven
    ///
    /// Examples:
    ///   ven path                       # alias for `ven path show`
    ///   ven path show                  # see what's currently in effect
    ///   ven path set D:\\ven           # wizard: ask about moving existing data
    ///   ven path set D:\\ven --move    # move data, no prompt
    ///   ven path set D:\\ven --pointer-only   # leave data where it is, just point future installs at D:\\ven
    ///   ven path set D:\\ven -y --json # CI: default to move, machine-readable
    ///   ven path reset --move          # revert to ~/.ven, move data back
    #[command(
        long_about = "Manage where ven stores its data on disk.\n\nDefault is $HOME/.ven (Linux/macOS) or %USERPROFILE%\\.ven (Windows). When\nthat drive fills up, `ven path set <dir>` relocates the whole storage\nroot — runtimes, cache, lockfile state — atomically, with rollback on\nfailure. The new location is recorded in:\n\n  1. A pointer file at ~/.config/ven/config.toml (ven's source of truth)\n  2. VEN_HOME in your user environment (so npm / pip / new shells see it)\n\nResolution precedence ven uses to find the storage root:\n  $VEN_HOME env var > $VEN_STORAGE_PATH > portable sibling .ven/ > pointer file > ~/.ven\n\nSubcommands:\n  show      Print the current $VEN_HOME, the resolver source, size, free disk\n  set DIR   Relocate to DIR (--move / --no-move / --pointer-only / -y / --json)\n  reset     Clear the pointer; ven home reverts to ~/.ven\n\nExamples:\n  ven path show\n  ven path set D:\\ven                    # wizard\n  ven path set D:\\ven --move             # move, no prompt\n  ven path set D:\\ven --pointer-only     # just update the pointer\n  ven path set /mnt/data/ven -y --json   # CI: default to move, JSON output\n  ven path reset --move                  # revert to ~/.ven, move data back"
    )]
    Path {
        #[command(subcommand)]
        cmd: Option<path::PathCmd>,
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

    /// Update ven itself to the latest release (self-update)
    ///
    /// Detects the install location of the running `ven` binary, fetches the
    /// latest release from GitHub, verifies the SHA256 from the release's
    /// SHA256SUMS manifest, and swaps `ven` + `ven-launcher` in place. On
    /// Windows the running .exe is moved to `*.exe.old` first (left for
    /// next-boot cleanup); on Unix the inode is unlinked + rewritten while
    /// the old process keeps running until it exits.
    ///
    /// If the install directory is not writable (system install on
    /// Windows / `/usr/local/bin`), ven re-launches itself elevated via UAC
    /// (Windows) or `sudo` (Unix). Set `--reentry` to bypass the elevation
    /// step — used internally; do not pass it by hand.
    ///
    /// Examples:
    ///   ven update                    # check for + apply latest stable
    ///   ven update --check            # report only, no download
    ///   ven update --version v0.1.6   # install a specific tag (rollback)
    ///   ven update --yes              # skip the confirmation prompt
    ///   ven update --force            # reinstall even if already current
    ///   ven update --json             # machine-readable result
    #[command(
        long_about = "Self-update ven to the latest published release.\n\nDownloads the platform-specific 'combined' release asset\n(ven-{os}-{arch}.{zip|tar.gz}) from https://github.com/bhuwanb23/ven/releases,\nverifies it against the SHA256SUMS manifest in the same release, and swaps\nboth `ven` and `ven-launcher` in place — no need to re-run the installer\nor edit PATH.\n\nFlags:\n  --check               Report the available version without downloading.\n  --version <tag>       Install a specific tag (`v0.1.6` or `0.1.6`).\n                        Lets you roll back to an older release.\n  --yes / -y            Skip the interactive confirmation.\n  --force               Reinstall even when already at the target version.\n  --json                Emit a machine-readable update report and exit.\n\nElevation:\n  When the install dir is not writable by the current user (system installs\n  on Windows / Unix), ven re-launches itself elevated through UAC or sudo.\n  The `--reentry` flag is used internally to break the elevation loop and\n  should not be passed by hand.\n\nExit codes:\n  0  No-op (already current) OR update applied successfully\n  1  Network / verification / write failure\n  2  Aborted by user at the confirmation prompt"
    )]
    Update {
        /// Report the available version without downloading or applying anything
        #[arg(long)]
        check: bool,
        /// Install a specific release tag (`v0.1.6` or `0.1.6`). Default: latest stable.
        #[arg(long)]
        version: Option<String>,
        /// Skip the confirmation prompt (CI / scripts)
        #[arg(short = 'y', long)]
        yes: bool,
        /// Reinstall even if the running version already matches the target
        #[arg(long)]
        force: bool,
        /// Machine-readable output for CI gates
        #[arg(long)]
        json: bool,
        /// Internal: we already re-launched ourselves with elevation. Do not pass by hand.
        #[arg(long, hide = true)]
        reentry: bool,
    },

    /// Fully remove ven from this machine (binary + runtimes + state + env) — since v0.1.7
    ///
    /// Replaces the long copy-paste PowerShell/shell snippet that used to live on the
    /// install page. Deletes the user install root (`~/.ven` or
    /// `%USERPROFILE%\.ven`), the system install (if present and you're elevated),
    /// the persisted `VEN_HOME` env var, the global pointer file, and every
    /// ven-managed block in your shell rc files.
    ///
    /// Honors a relocated storage root: if you ran `ven path set D:\ven`, this
    /// removes BOTH `~/.ven` (the binary install) AND `D:\ven` (the data).
    ///
    /// Examples:
    ///   ven uninstall                # Interactive: shows plan, prompts before nuking
    ///   ven uninstall --dry-run      # Print the plan; touch nothing
    ///   ven uninstall -y             # Skip the confirm prompt (CI / scripts)
    ///   ven uninstall --user-only    # Skip the system install layer
    ///   ven uninstall --system-only  # Skip the user install layer (rare; for admins)
    ///   ven uninstall --json -y      # Machine-readable result (CI gates)
    ///   ven uninstall --json --dry-run # Plan as JSON without executing
    #[command(
        long_about = "Fully remove ven from this machine (binary + runtimes + state + env).\n\nReplaces the long copy-paste PowerShell/shell snippet on the install page with\na single confirmed, dry-run-capable command. Native Rust implementation;\n`scripts/uninstall.{ps1,sh}` ship alongside the binary as fallback for the\n\"my ven binary is broken\" recovery case.\n\nWhat gets removed (in scope All):\n  • User install root      ~/.ven  or  %USERPROFILE%\\.ven\n  • Relocated data dir     whatever ven path set wrote (if different from above)\n  • System install         /usr/local/bin/{ven,ven-launcher,ven-setup}\n                           +  /etc/profile.d/ven.sh  on Unix\n                           %ProgramFiles%\\ven\\         on Windows\n  • User env vars          VEN_HOME (removed via the same helper ven path set uses)\n  • Pointer file           ~/.config/ven/config.toml (or platform equivalent)\n  • User PATH entries      ~/.ven/bin  (Windows: User-scope registry edit)\n  • System PATH entries    %ProgramFiles%\\ven\\bin  (Windows: Machine-scope, elevated)\n  • Shell rc-file blocks   `# >>> ven env >>>`, `# >>> ven-setup PATH >>>`,\n                           `# >>> ven shell hook >>>` from .bashrc/.zshrc/\n                           .profile/.bash_profile/.zprofile/fish/config.fish\n                           AND any orphan PATH line referencing .ven/bin\n\nWhat survives:\n  • Project files (ven.toml, ven.lock) — those are part of your repo.\n  • node_modules / venv / etc. created inside individual projects.\n\nFlags:\n  --dry-run                  Print the plan; change nothing.\n  -y / --yes                 Skip the confirmation prompt (CI / scripts).\n  --user-only                Skip the system install layer.\n  --system-only              Skip the user install layer (rare; for admins).\n  --json                     Machine-readable output. Requires --dry-run OR -y.\n\nExit codes:\n  0  Uninstall succeeded (or no-op when nothing was installed)\n  1  Partial failure (see report) OR needs elevation for the system layer\n\nElevation:\n  The system install lives in dirs only writable by root/Admin. If detected,\n  ven prints a clear hint to re-run with `sudo ven uninstall` (Unix) or from\n  an elevated PowerShell (Windows). Use --user-only to skip the elevation\n  requirement when you only need to drop the per-user install."
    )]
    Uninstall {
        /// Skip the confirmation prompt (CI / scripts)
        #[arg(short = 'y', long)]
        yes: bool,

        /// Print the plan; touch nothing
        #[arg(long)]
        dry_run: bool,

        /// Only touch the user-mode install + user-scope env state
        #[arg(long, conflicts_with = "system_only")]
        user_only: bool,

        /// Only touch the system-mode install + system-scope env state
        #[arg(long, conflicts_with = "user_only")]
        system_only: bool,

        /// Machine-readable output. Requires --dry-run OR -y / --yes.
        #[arg(long)]
        json: bool,
    },

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
        Commands::Delete {
            language,
            version,
            yes,
            force,
            json,
        } => delete::cmd_delete(language, version, yes, force, json),
        Commands::Path { cmd } => path::cmd_path(cmd),
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
        Commands::Update {
            check,
            version,
            yes,
            force,
            json,
            reentry,
        } => update::cmd_update(check, version.as_deref(), yes, force, json, reentry),
        Commands::Uninstall {
            yes,
            dry_run,
            user_only,
            system_only,
            json,
        } => uninstall::cmd_uninstall(yes, json, dry_run, user_only, system_only),
        Commands::Shell { action } => match action {
            ShellCommands::Hook { shell } => shell::cmd_shell_hook(&shell),
            ShellCommands::Activate { dir } => shell::cmd_shell_activate(&dir),
            ShellCommands::Deactivate => shell::cmd_shell_deactivate(),
            ShellCommands::Install => shell::cmd_shell_install(),
        },
    }
}

// All command implementations have been moved to their own modules:
// - src/cli/install/
// - src/cli/list.rs (+ list/helpers.rs, also reused by delete.rs)
// - src/cli/delete.rs   ← removes an installed runtime, complement of `remove`
// - src/cli/path.rs     ← `ven path show / set / reset` (storage relocation; v0.1.6+)
// - src/cli/uninstall.rs ← `ven uninstall` full-nuke teardown (v0.1.7+)
// - src/cli/status/
// - src/cli/setup.rs
// - src/cli/shell.rs
// - src/cli/init.rs
// - src/cli/add/
// - src/cli/remove.rs   ← removes packages (npm / pip / cargo / ...)
// - src/cli/upgrade.rs
