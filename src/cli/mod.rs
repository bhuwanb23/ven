use clap::{Parser, Subcommand};
use anyhow::Result;

// Command modules
pub mod install;
pub mod list;
pub mod status;
pub mod setup;
pub mod shell;
pub mod init;
pub mod add;
pub mod remove;
pub mod upgrade;

/// ven — Any-first intelligent version and dependency manager
///
/// A modern, fast, and secure Node.js version and package manager.
/// Manages Node.js versions and project dependencies with intelligent compatibility checking.
#[derive(Parser)]
#[command(
    name = "ven",
    version,
    about,
    long_about = None,
    after_help = "Examples:\n  ven install node 20          # Install Node.js 20.x\n  ven init --template          # Create ven.toml interactively\n  ven add express vite         # Add packages to project\n  ven status --verbose         # Show detailed project status\n  ven upgrade --all --apply    # Upgrade all packages\n  ven remove --cleanup         # Remove orphaned packages\n\nDocumentation: https://github.com/your-org/ven"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a language version (e.g., Node.js)
    ///
    /// Downloads and installs Node.js versions. Supports version specs like
    /// "20", "20.11.0", "lts", or "latest".
    ///
    /// Examples:
    ///   ven install node 20        # Install latest Node.js 20.x
    ///   ven install node 20.11.0   # Install specific version
    ///   ven install node           # Show available versions
    ///   ven install                # Interactive mode
    #[command(visible_alias = "i", long_about = "Install a language version (e.g., Node.js)\n\nDownloads and installs Node.js versions from official sources.\nSupports version specs like \"20\", \"20.11.0\", \"lts\", or \"latest\".\n\nExamples:\n  ven install node 20        # Install latest Node.js 20.x\n  ven install node 20.11.0   # Install specific version\n  ven install node           # Show available versions\n  ven install                # Interactive mode")]
    Install {
        /// Language to install (currently only "node" is supported)
        language: Option<String>,

        /// Version to install: "20", "20.11.0", "lts", "latest"
        version: Option<String>,
    },

    /// List installed Node.js versions
    ///
    /// Shows all Node.js versions installed on your system.
    /// Optionally filter by language and show detailed information.
    ///
    /// Examples:
    ///   ven list                   # List all installed versions
    ///   ven list node              # List Node.js versions
    ///   ven list --verbose         # Show disk usage and install date
    ///   ven list --json            # JSON output for scripting
    #[command(long_about = "List installed Node.js versions\n\nShows all Node.js versions installed on your system with optional\ndetails like disk usage and installation date.\n\nExamples:\n  ven list                   # List all installed versions\n  ven list node              # List Node.js versions\n  ven list --verbose         # Show disk usage and install date\n  ven list --json            # JSON output for scripting")]
    List {
        /// Language to list (default: shows all)
        language: Option<String>,
        
        /// Show detailed info (installation date, disk size)
        #[arg(short, long)]
        verbose: bool,
        
        /// Output as JSON for scripting and automation
        #[arg(long)]
        json: bool,
    },

    /// Show current project status and configuration
    ///
    /// Displays ven.toml configuration, installed versions,
    /// package status, and project health information.
    ///
    /// Examples:
    ///   ven status                 # Show basic status
    ///   ven status --verbose       # Show detailed info with disk usage
    ///   ven status --json          # JSON output for CI/CD
    ///   ven status --fix           # Auto-fix detected issues
    #[command(long_about = "Show current project status and configuration\n\nDisplays ven.toml configuration, installed Node.js versions,\npackage status, environment variables, and project health information.\n\nExamples:\n  ven status                 # Show basic status\n  ven status --verbose       # Show detailed info with disk usage\n  ven status --json          # JSON output for CI/CD\n  ven status --fix           # Auto-fix detected issues")]
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
    /// Creates a ven.toml file in the current directory with Node.js
    /// version and package declarations for project isolation.
    ///
    /// Examples:
    ///   ven init                   # Create with default settings
    ///   ven init --template        # Interactive template selection
    ///   ven init --with-packages   # Add packages interactively
    ///   ven init --validate        # Validate after creation
    #[command(long_about = "Initialize a new ven.toml configuration file\n\nCreates a ven.toml file in the current directory with Node.js version\nand package declarations. This enables project-specific version management\nand dependency isolation.\n\nExamples:\n  ven init                   # Create with default settings\n  ven init --template        # Interactive template selection\n  ven init --with-packages   # Add packages interactively\n  ven init --validate        # Validate after creation")]
    Init {
        /// Use interactive template selection (React, Vue, Next.js, etc.)
        #[arg(long)]
        template: bool,
        
        /// Add popular packages interactively after creating ven.toml
        #[arg(long)]
        with_packages: bool,
        
        /// Validate the setup after creation (check Node.js is installed)
        #[arg(long)]
        validate: bool,
        
        /// Node.js version to use (legacy option, kept for backward compatibility)
        #[arg(short, long)]
        node: Option<String>,
    },

    /// Add package(s) to the project with compatibility checking
    ///
    /// Installs npm packages and adds them to ven.toml with Node.js
    /// compatibility verification.
    ///
    /// Examples:
    ///   ven add express              # Add latest Express
    ///   ven add express@4.18.2       # Add specific version
    ///   ven add react vite           # Add multiple packages
    ///   ven add lodash --skip-check  # Skip Node.js compatibility check
    ///   ven add express --dry-run    # Preview before installing
    ///   ven add socket.io --verbose  # Show full dependency tree
    #[command(long_about = "Add package(s) to the project with compatibility checking\n\nInstalls npm packages and automatically adds them to ven.toml.\nPerforms Node.js version compatibility checking to prevent issues.\n\nExamples:\n  ven add express              # Add latest Express\n  ven add express@4.18.2       # Add specific version\n  ven add react vite           # Add multiple packages\n  ven add lodash --skip-check  # Skip Node.js compatibility check\n  ven add express --dry-run    # Preview before installing\n  ven add socket.io --verbose  # Show full dependency tree")]
    Add {
        /// Package name(s) with optional version, e.g., "express" or "express@4.18.2"
        #[arg(required = true)]
        packages: Vec<String>,
        /// Skip Node.js compatibility checking (use with caution)
        #[arg(long)]
        skip_check: bool,
        /// Show dependency preview without installing
        #[arg(long)]
        dry_run: bool,
        /// Show full dependency tree
        #[arg(short, long)]
        verbose: bool,
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
    #[command(long_about = "Remove package(s) with dependency analysis and safety checks\n\nSafely removes npm packages from your project. Analyzes the dependency\ngraph to warn about packages that depend on the target before removal.\n\nExamples:\n  ven remove express           # Remove with dependency check\n  ven remove lodash --force    # Force remove without checks\n  ven remove react vite        # Remove multiple packages\n  ven remove --dry-run         # Preview what would be removed\n  ven remove --cleanup         # Find and remove orphaned packages")]
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
    },

    /// Upgrade packages to latest compatible versions
    ///
    /// Shows available upgrades and optionally applies them with
    /// Node.js compatibility verification.
    ///
    /// Examples:
    ///   ven upgrade express          # Preview Express upgrade
    ///   ven upgrade express --apply  # Apply the upgrade
    ///   ven upgrade --all            # Preview all upgrades
    ///   ven upgrade --all --apply    # Upgrade all packages
    ///   ven upgrade react --dry-run  # Preview without changes
    ///   ven upgrade --all --apply --force  # CI/CD mode (no prompts)
    #[command(long_about = "Upgrade packages to latest compatible versions\n\nChecks for available package upgrades and verifies Node.js compatibility\nbefore applying them. Shows preview by default, use --apply to upgrade.\n\nExamples:\n  ven upgrade express          # Preview Express upgrade\n  ven upgrade express --apply  # Apply the upgrade\n  ven upgrade --all            # Preview all upgrades\n  ven upgrade --all --apply    # Upgrade all packages\n  ven upgrade react --dry-run  # Preview without changes\n  ven upgrade --all --apply --force  # CI/CD mode (no prompts)")]
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

    /// One-time setup: Install shell hooks for automatic version switching
    ///
    /// Configures your shell (bash, zsh, fish, PowerShell) to automatically
    /// switch Node.js versions when you change directories.
    ///
    /// Examples:
    ///   ven setup                  # Run interactive setup
    ///
    /// After setup, add to your shell rc file:
    ///   eval "$(ven shell hook <shell>)"
    #[command(long_about = "One-time setup: Install shell hooks for automatic version switching\n\nConfigures your shell (bash, zsh, fish, PowerShell) to automatically\nswitch Node.js versions when you change directories with a ven.toml file.\n\nThis is a one-time setup that enables seamless version management.\n\nExamples:\n  ven setup                  # Run interactive setup\n\nAfter setup, the installer will show you what to add to your shell rc file:\n  eval \"$(ven shell hook <shell>)\"\n\nSupported shells: bash, zsh, fish, powershell")]
    Setup,

    /// Shell integration (internal — called by shell hook)
    #[command(hide = true)]  // hides from --help
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
        Commands::List { language, verbose, json } => {
            list::cmd_list(language.as_deref(), verbose, json)
        }
        Commands::Status { json, verbose, fix } => {
            status::cmd_status(json, verbose, fix)
        }
        Commands::Init { node, template, with_packages, validate } => {
            init::cmd_init(node.as_deref(), template, with_packages, validate)
        }
        Commands::Add { packages, skip_check, dry_run, verbose } => {
            add::cmd_add(&packages, skip_check, dry_run, verbose)
        }
        Commands::Remove { packages, force, dry_run, json, verbose, cleanup } => {
            remove::cmd_remove(&packages, force, dry_run, json, verbose, cleanup)
        }
        Commands::Upgrade { packages, apply, dry_run, json, verbose, all, force } => {
            upgrade::cmd_upgrade(&packages, apply, dry_run, json, verbose, all, force)
        }
        Commands::Setup => {
            setup::cmd_setup()
        }
        Commands::Shell { action } => match action {
            ShellCommands::Hook { shell } => shell::cmd_shell_hook(&shell),
            ShellCommands::Activate { dir } => shell::cmd_shell_activate(&dir),
            ShellCommands::Install => shell::cmd_shell_install(),
        },

    }
}

// All command implementations have been moved to their own modules:
// - src/cli/install.rs
// - src/cli/list.rs
// - src/cli/status.rs
// - src/cli/setup.rs
// - src/cli/shell.rs
// - src/cli/init.rs
// - src/cli/add.rs
// - src/cli/remove.rs
// - src/cli/upgrade.rs

