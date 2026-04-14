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

/// ven — Node.js version manager
#[derive(Parser)]
#[command(name = "ven", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a language version
    #[command(visible_alias = "i")]
    Install {
        /// Language: node
        language: Option<String>,

        /// Version: 20.11.0, 20, lts, latest
        version: Option<String>,
    },

    /// List installed versions
    List {
        /// Language: node (optional)
        language: Option<String>,
        
        /// Show detailed info (installation date, disk size)
        #[arg(short, long)]
        verbose: bool,
        
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
    },

    /// Show current active versions
    Status {
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
        
        /// Show detailed info (disk usage, compatibility)
        #[arg(short, long)]
        verbose: bool,
        
        /// Fix all detected issues automatically
        #[arg(long)]
        fix: bool,
    },

    /// Initialize a new ven.toml file
    Init {
        /// Use interactive template selection
        #[arg(long)]
        template: bool,
        
        /// Add popular packages interactively
        #[arg(long)]
        with_packages: bool,
        
        /// Validate setup after creation
        #[arg(long)]
        validate: bool,
        
        /// Node.js version to use (legacy, kept for backward compatibility)
        #[arg(short, long)]
        node: Option<String>,
    },

    /// Add package(s) to this project
    Add {
        /// Package name(s), e.g. express or express@4.18.2
        #[arg(required = true)]
        packages: Vec<String>,
        /// Skip compatibility check
        #[arg(long)]
        skip_check: bool,
    },

    /// Remove packages (single or batch)
    Remove {
        /// Package names to remove (supports multiple)
        packages: Vec<String>,
        
        /// Skip dependency check
        #[arg(long)]
        force: bool,
        
        /// Preview removal without executing
        #[arg(long)]
        dry_run: bool,
        
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
        
        /// Show detailed analysis (disk space, transitive deps)
        #[arg(short, long)]
        verbose: bool,
        
        /// Find and remove orphaned/unused dependencies
        #[arg(long)]
        cleanup: bool,
    },

    /// Upgrade a package (preview or apply)
    Upgrade {
        /// Package name
        package: String,
        /// Actually apply the upgrade (default: preview only)
        #[arg(long)]
        apply: bool,
    },

    /// One-time setup: install shell hook
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
        Commands::Add { packages, skip_check } => {
            add::cmd_add(&packages, skip_check)
        }
        Commands::Remove { packages, force, dry_run, json, verbose, cleanup } => {
            remove::cmd_remove(&packages, force, dry_run, json, verbose, cleanup)
        }
        Commands::Upgrade { package, apply } => {
            upgrade::cmd_upgrade(&package, apply)
        }
        Commands::Setup => {
            setup::cmd_setup()
        }
        Commands::Shell { action } => match action {
            ShellCommands::Hook { shell } => shell::cmd_shell_hook(&shell),
            ShellCommands::Activate { dir } => shell::cmd_shell_activate(&dir),
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

