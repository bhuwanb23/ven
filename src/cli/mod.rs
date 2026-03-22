use clap::{Parser, Subcommand};

use anyhow::Result;

use crate::core::{load_config, resolve_node_version};
use crate::plugins::{LanguagePlugin, NodePlugin};

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
    Install {
        /// Language: node
        language: String,

        /// Version: 20.11.0, 20, lts, latest
        version: String,
    },

    /// List installed versions
    List {
        /// Language: node (optional)
        language: Option<String>,
    },

    /// Show current active versions
    Status,

    /// Initialize a new ven.toml file
    Init {
        /// Node.js version to use
        #[arg(short, long)]
        node: Option<String>,
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
            cmd_install(&language, &version)
        }
        Commands::List { language } => {
            cmd_list(language.as_deref())
        }
        Commands::Status => {
            cmd_status()
        }
        Commands::Init { node } => {
            cmd_init(node.as_deref())
        }
        Commands::Setup => {
            cmd_setup()
        }
        Commands::Shell { action } => match action {
            ShellCommands::Hook { shell } => cmd_shell_hook(&shell),
            ShellCommands::Activate { dir } => cmd_shell_activate(&dir),
        },
    }
}

// ── ven install node <version> ────────────────────────────────────
fn cmd_install(language: &str, version: &str) -> Result<()> {
    use colored::Colorize;

    match language {
        "node" => {
            let plugin = NodePlugin;
            
            // Resolve alias first: "lts" or "latest" → real version number
            let resolved = if version == "lts" || version == "latest" {
                println!("{} Fetching latest Node version...", "🔍".cyan());
                plugin.latest_version()?
            } else {
                version.to_string()
            };

            plugin.install_version(&resolved)?;
            Ok(())
        }
        other => {
            Err(anyhow::anyhow!("Unknown language: {}. Supported: node", other))
        }
    }
}

// ── ven list (node) ───────────────────────────────────────────────
fn cmd_list(language: Option<&str>) -> Result<()> {
    use colored::Colorize;

    match language.unwrap_or("node") {
        "node" => {
            let plugin = NodePlugin;
            let versions = plugin.list_installed()?;

            if versions.is_empty() {
                println!("{} No Node versions installed. Run: ven install node latest", "⚠️".yellow());
                return Ok(());
            }

            println!("\n  {}", "node".bold().cyan());
            for v in &versions {
                println!("    {} {}", "•".dimmed(), v);
            }
            println!();
            Ok(())
        }
        other => {
            Err(anyhow::anyhow!("Unknown language: {}. Supported: node", other))
        }
    }
}

// ── ven status ────────────────────────────────────────────────────
#[allow(non_snake_case)]
fn cmd_status() -> Result<()> {
    use colored::Colorize;

    let cwd = std::env::current_dir()?;
    let config = load_config(&cwd)?;

    println!("\n  {} {}", "ven status".bold(), cwd.display());

    match config {
        None => {
            println!("  No ven.toml found in this directory tree.");
            println!("  Run: ven init   to create one.");
        }
        Some(cfg) => {
            if let Some(node_ver) = &cfg.runtime.node {
                println!("  {} {}", "node".bold(), node_ver.green());
            }
            if let Some(pkgs) = &cfg.packages {
                println!("  {} {} packages declared", "packages".bold(), pkgs.len());
            }
        }
    }
    println!();
    Ok(())
}

// ── ven init ──────────────────────────────────────────────────────
fn cmd_init(node: Option<&str>) -> Result<()> {
    use colored::Colorize;
    use std::fs;
    use std::path::Path;

    let cwd = std::env::current_dir()?;
    let toml_path = cwd.join("ven.toml");

    if toml_path.exists() {
        return Err(anyhow::anyhow!("ven.toml already exists in this directory"));
    }

    // Build ven.toml content
    let mut content = String::from("[runtime]\n");
    
    if let Some(version) = node {
        content.push_str(&format!("node = \"{}\"\n", version));
    } else {
        // Try to detect current Node version
        let output = std::process::Command::new("node")
            .arg("--version")
            .output();

        if let Ok(out) = output {
            let version = String::from_utf8_lossy(&out.stdout)
                .trim()
                .trim_start_matches('v')
                .to_string();
            content.push_str(&format!("node = \"{}\"\n", version));
            println!("{} Detected current Node version: {}", "✓".green(), version);
        } else {
            content.push_str("node = \"latest\"\n");
            println!("{} Using 'latest' as default Node version", "ℹ️".blue());
        }
    }

    content.push_str("\n[packages]\n");
    content.push_str("# Add your dependencies here\n");
    content.push_str("# express = \"^4.18.2\"\n");

    fs::write(&toml_path, &content)?;
    println!("{} Created {}", "✓".green(), toml_path.display());
    println!("\nEdit this file to customize your Node version and dependencies.");
    println!("Run: ven install node <version>   to install a specific version");

    Ok(())
}

// ── ven setup ─────────────────────────────────────────────────────
fn cmd_setup() -> Result<()> {
    use colored::Colorize;
    use std::io::Write;

    // Detect which shell the user is running
    let shell_path = std::env::var("SHELL").unwrap_or_default();
    let shell_name = std::path::Path::new(&shell_path)
        .file_name().and_then(|n| n.to_str())
        .unwrap_or("bash");

    println!("\n  {} ven setup", "→".cyan());
    println!("  Detected shell: {}", shell_name.bold());

    // Find the rc file
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    let rc_file = match shell_name {
        "zsh"  => home.join(".zshrc"),
        "fish" => home.join(".config/fish/config.fish"),
        _      => home.join(".bashrc"),
    };

    // The line to add to the rc file
    let hook_line = format!("\n# ven shell hook\neval \"$(ven shell {})\"", shell_name);

    // Check if already installed
    let existing = std::fs::read_to_string(&rc_file).unwrap_or_default();
    if existing.contains("ven shell hook") {
        println!("  {} Shell hook already installed in {}", "✓".green(), rc_file.display());
        return Ok(());
    }

    // Append to rc file
    let mut file = std::fs::OpenOptions::new().append(true).open(&rc_file)?;
    writeln!(file, "{}", hook_line)?;

    println!("  {} Written to {}", "✓".green(), rc_file.display());
    println!();
    println!("  Restart your shell or run:");
    println!("  {}", format!("source {}", rc_file.display()).bold());
    println!();
    Ok(())
}

// ── ven shell hook <shell> ────────────────────────────────────────
fn cmd_shell_hook(shell: &str) -> Result<()> {
    use crate::shell::generate_hook;
    
    // Just print the hook code — user wraps this in eval "$(ven shell hook bash)"
    print!("{}", generate_hook(shell));
    Ok(())
}

// ── ven shell activate <dir> ──────────────────────────────────────
#[allow(non_snake_case)]
fn cmd_shell_activate(dir: &str) -> Result<()> {
    use crate::shell::compute_exports;
    
    let path = std::path::Path::new(dir);
    match compute_exports(path)? {
        Some(exports) => print!("{}", exports),
        None          => {}  // no ven.toml = print nothing = no eval
    }
    Ok(())
}
