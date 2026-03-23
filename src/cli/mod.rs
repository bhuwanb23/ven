use clap::{Parser, Subcommand};

use anyhow::Result;

use crate::core::{load_config};
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

    /// Add a package to this project
    Add {
        /// Package name, e.g. express or express@4.18.2
        package: String,
        /// Skip compatibility check
        #[arg(long)]
        skip_check: bool,
    },

    /// Remove a package from this project
    Remove {
        /// Package name
        package: String,
        /// Skip dependency check
        #[arg(long)]
        force: bool,
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
        Commands::Add { package, skip_check } => {
            cmd_add(&package, skip_check)
        }
        Commands::Remove { package, force } => {
            cmd_remove(&package, force)
        }
        Commands::Upgrade { package, apply } => {
            cmd_upgrade(&package, apply)
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

// ── ven add <package> ──────────────────────────────────────────────
fn cmd_add(package_spec: &str, skip_check: bool) -> Result<()> {
    use colored::Colorize;
    use crate::core::{load_config, packages::*};

    // Split "express@4.18.2" into name + optional version pin
    let (pkg_name, pinned_version) = if package_spec.contains('@') && !package_spec.starts_with('@') {
        let parts: Vec<&str> = package_spec.splitn(2, '@').collect();
        (parts[0], Some(parts[1]))
    } else {
        (package_spec, None)
    };

    // Get current Node version from ven.toml
    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .and_then(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    println!("{} Checking {} against Node {}...",
        "→".cyan(), pkg_name.bold(), node_version.bold());

    // Fetch npm metadata
    let info = fetch_npm_info(pkg_name)?;

    // Determine version to install
    let version_to_install = if let Some(pinned) = pinned_version {
        pinned.to_string()
    } else if skip_check {
        info.dist_tags.get("latest").cloned()
            .ok_or_else(|| anyhow::anyhow!("No latest version found"))?
    } else {
        // Find best compatible
        find_compatible_version(&info, &node_version)
            .ok_or_else(|| anyhow::anyhow!(
                "No compatible version of {} found for Node {}",
                pkg_name, node_version
            ))?
    };

    println!("  {} {} — compatible with Node {}",
        "✓".green(), format!("{}@{}", pkg_name, version_to_install).bold(), node_version);

    // Run npm install
    npm_install(pkg_name, &version_to_install)?;

    // Update ven.toml
    update_ven_toml_package(pkg_name, &version_to_install)?;

    Ok(())
}

// ── Update ven.toml with new package ────────────────────────────────
fn update_ven_toml_package(pkg: &str, version: &str) -> Result<()> {
    use crate::core::{find_ven_toml, parse_ven_toml};

    let cwd = std::env::current_dir()?;
    let toml_path = find_ven_toml(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    // Read current config as raw TOML string, append package
    let mut content = std::fs::read_to_string(&toml_path)?;

    let entry = format!("{} = \"{}\"", pkg, version);

    if content.contains("[packages]") {
        // Insert after [packages] header
        content = content.replace("[packages]", &format!("[packages]\n{}", entry));
    } else {
        // Add [packages] section
        content.push_str(&format!("\n[packages]\n{}\n", entry));
    }

    std::fs::write(&toml_path, content)?;
    println!("  {} Updated ven.toml", "✓".green());
    Ok(())
}

// ── ven remove <package> ────────────────────────────────────────────
fn cmd_remove(package: &str, force: bool) -> Result<()> {
    use colored::Colorize;
    use crate::core::packages::*;

    if !force {
        let dependents = find_dependents(package)?;
        if !dependents.is_empty() {
            println!(
                "\n  {} {} packages depend on {}:",
                "Warning:".yellow().bold(),
                dependents.len(),
                package.bold()
            );
            for (dep, ver) in &dependents {
                println!(
                    "    {} {}  requires  {}",
                    "•".dimmed(),
                    format!("{} {}", dep, ver).bold(),
                    package
                );
            }
            println!();
            println!("  Removing {} may break these packages.", package);
            print!("  Remove anyway? [y/N]: ");
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let answer = stdin.lock().lines().next()
                .and_then(|l| l.ok())
                .unwrap_or_default();
            if answer.trim().to_lowercase() != "y" {
                println!("  Cancelled.");
                return Ok(());
            }
        }
    }

    npm_uninstall(package)?;
    println!("{} Removed {}", "✓".green(), package.bold());
    Ok(())
}

// ── ven upgrade <package> ────────────────────────────────────────────
fn cmd_upgrade(package: &str, apply: bool) -> Result<()> {
    use colored::Colorize;
    use crate::core::{load_config, packages::*};

    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .and_then(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    // Get currently installed version from node_modules
    let current_ver = get_installed_version(package)
        .unwrap_or_else(|_| "unknown".to_string());

    // Fetch latest compatible version
    let info = fetch_npm_info(package)?;
    let latest = find_compatible_version(&info, &node_version)
        .ok_or_else(|| anyhow::anyhow!("No compatible version found"))?;

    if current_ver == latest {
        println!("{} {} is already up to date ({})", "✓".green(), package.bold(), latest);
        return Ok(());
    }

    println!("\n  {} {}  →  {}  (latest compatible)", package.bold(), current_ver.dimmed(), latest.green());
    println!("\n  Compatibility: {} Node {} supported", "✓".green(), node_version);

    // Show changelog hint
    let notes = fetch_release_notes(package, &current_ver, &latest);
    println!("\n  Release notes: {}", notes.dimmed());

    if !apply {
        println!("\n  Run  {} to upgrade", format!("ven upgrade {} --apply", package).bold());
        return Ok(());
    }

    npm_install(package, &latest)?;
    update_ven_toml_package(package, &latest)?;
    Ok(())
}
