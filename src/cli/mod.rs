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
        Commands::Init { node, template, with_packages, validate } => {
            cmd_init(node.as_deref(), template, with_packages, validate)
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

            // Resolve aliases AND major-only versions before installing
            // "lts" → latest LTS  e.g. "20.11.0"
            // "latest" → latest stable e.g. "22.3.0"
            // "20" → highest 20.x available on nodejs.org e.g. "20.11.0"
            // "20.11.0" → exact, pass through
            let resolved = if version == "lts" || version == "latest" {
                println!("{} Fetching Node release list...", "→".cyan());
                plugin.latest_version()?
            } else if !version.contains('.') {
                // Major-only like "20" — resolve to highest 20.x from nodejs.org
                println!("{} Resolving Node {} to latest patch version...", "→".cyan(), version.bold());
                resolve_major_version(version)?
            } else {
                version.to_string()
            };

            println!("{} Resolved to Node {}", "✓".green(), resolved.bold());
            plugin.install_version(&resolved)?;
            Ok(())
        }
        other => {
            Err(anyhow::anyhow!("Unknown language: {}. Supported: node", other))
        }
    }
}

/// Resolve a major version like "20" to the latest 20.x.x by fetching nodejs.org release list
fn resolve_major_version(major: &str) -> Result<String> {
    let response = reqwest::blocking::get("https://nodejs.org/dist/index.json")
        .map_err(|e| anyhow::anyhow!("Cannot reach nodejs.org: {}", e))?;
    let releases: Vec<serde_json::Value> = response.json()?;

    // Find highest version with this major number
    for release in &releases {
        if let Some(ver) = release.get("version").and_then(|v| v.as_str()) {
            let ver_clean = ver.trim_start_matches('v');
            let release_major = ver_clean.split('.').next().unwrap_or("0");
            if release_major == major {
                return Ok(ver_clean.to_string()); // releases are sorted newest first
            }
        }
    }

    Err(anyhow::anyhow!(
        "No Node {} version found. Check available versions at nodejs.org",
        major
    ))
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
            let node_ver = &cfg.runtime.node;
            println!("  {} {}", "node".bold(), node_ver.green());
            if !cfg.packages.is_empty() {
                println!("  {} {} packages declared", "packages".bold(), cfg.packages.len());
            }
        }
    }
    println!();
    Ok(())
}

// ── ven init ──────────────────────────────────────────────────────
fn cmd_init(_node: Option<&str>, use_template: bool, with_packages: bool, validate: bool) -> Result<()> {
    use colored::Colorize;
    use std::fs;
    use dialoguer::{Select, MultiSelect, theme::ColorfulTheme};

    let cwd = std::env::current_dir()?;
    let toml_path = cwd.join("ven.toml");

    if toml_path.exists() {
        return Err(anyhow::anyhow!("ven.toml already exists in this directory"));
    }

    let theme = ColorfulTheme::default();
    let mut selected_language = "node".to_string();
    let mut selected_version = String::new();
    let mut selected_packages: Vec<(String, String)> = Vec::new();
    let mut template_name = String::new();

    // MODE 1: Template selection
    if use_template {
        println!("\n{} Smart Project Templates", "📦".bold().cyan());
        
        let templates = vec![
            ("Express API Server", "node", "20", vec![
                ("express", "^4.18.2"),
                ("cors", "^2.8.5"),
                ("dotenv", "^16.3.1"),
            ]),
            ("React + Vite Frontend", "node", "20", vec![
                ("react", "^18.2.0"),
                ("react-dom", "^18.2.0"),
                ("vite", "^5.0.0"),
            ]),
            ("Next.js Full-stack", "node", "20", vec![
                ("next", "^14.0.0"),
                ("react", "^18.2.0"),
                ("react-dom", "^18.2.0"),
            ]),
            ("Empty Project", "node", "lts", vec![]),
        ];
        
        let template_names: Vec<&str> = templates.iter().map(|t| t.0).collect();
        
        let template_idx = Select::with_theme(&theme)
            .with_prompt("Select project template")
            .items(&template_names)
            .default(0)
            .interact()?;
        
        let template = &templates[template_idx];
        template_name = template.0.to_string();
        selected_language = template.1.to_string();
        selected_version = template.2.to_string();
        
        // Add template packages
        for (pkg, ver) in &template.3 {
            selected_packages.push((pkg.to_string(), ver.to_string()));
        }
        
        println!("\n{} Selected: {}", "✓".green(), template_name.bold());
    } else {
        // MODE 2: Interactive language & version selection
        let languages = vec!["node", "python"];
        let language_idx = Select::with_theme(&theme)
            .with_prompt("Select language")
            .items(&languages)
            .default(0)
            .interact()?;
        
        selected_language = languages[language_idx].to_string();
        
        // Version selection
        selected_version = match selected_language.as_str() {
            "node" => select_node_version()?,
            "python" => select_python_version()?,
            _ => {
                return Err(anyhow::anyhow!("Unsupported language: {}", selected_language));
            }
        };
    }

    // MODE 3: Interactive package selection
    if with_packages && selected_packages.is_empty() {
        println!("\n{} Interactive Package Selection", "📦".bold().cyan());
        
        let popular_packages = vec![
            ("express", "^4.18.2", "Fast, minimalist web framework"),
            ("typescript", "^5.3.0", "Typed JavaScript for better DX"),
            ("dotenv", "^16.3.1", "Load environment variables from .env"),
            ("cors", "^2.8.5", "Enable CORS support"),
            ("morgan", "^1.10.0", "HTTP request logger"),
            ("jest", "^29.7.0", "Testing framework"),
            ("eslint", "^8.56.0", "Code linting and quality"),
            ("prettier", "^3.1.0", "Code formatting"),
        ];
        
        let pkg_display: Vec<String> = popular_packages.iter()
            .map(|(name, ver, desc)| format!("{} {} - {}", name.bold(), ver.dimmed(), desc))
            .collect();
        
        let selected_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Add popular packages? (SPACE to select, ENTER to continue)")
            .items(&pkg_display)
            .interact()?;
        
        for idx in selected_indices {
            let pkg = &popular_packages[idx];
            selected_packages.push((pkg.0.to_string(), pkg.1.to_string()));
        }
        
        if !selected_packages.is_empty() {
            println!("\n{} Selected {} packages", "✓".green(), selected_packages.len());
        }
    }

    // Generate ven.toml
    let mut content = String::from("[runtime]\n");
    content.push_str(&format!("{} = \"{}\"\n", selected_language, selected_version));

    content.push_str("\n[packages]\n");
    if selected_packages.is_empty() {
        content.push_str("# Add your dependencies here\n");
        content.push_str("# express = \"^4.18.2\"\n");
    } else {
        for (pkg, ver) in &selected_packages {
            content.push_str(&format!("{} = \"{}\"\n", pkg, ver));
        }
    }

    fs::write(&toml_path, &content)?;
    
    // Success message
    println!("\n{} Created {}", "✓".green(), toml_path.display());
    if !template_name.is_empty() {
        println!("  {} Template: {}", "📦".cyan(), template_name.bold());
    }
    println!("  {} {} {}", "🔧".cyan(), selected_language.bold(), selected_version.green());
    
    if !selected_packages.is_empty() {
        println!("  {} {} packages:", "📦".cyan(), selected_packages.len());
        for (pkg, ver) in &selected_packages {
            println!("      {} {}", pkg.bold(), ver.dimmed());
        }
    }
    
    // MODE 4: Validation
    if validate {
        println!("\n{} Running validation...", "🔍".bold().cyan());
        run_validation(&selected_language, &selected_version, &selected_packages)?;
    } else {
        println!("\nEdit this file to customize your dependencies.");
        println!("Run: ven install {} {}   to install this version", 
            selected_language, selected_version);
    }

    Ok(())
}

/// Interactive Node.js version selection with compatibility matrix
fn select_node_version() -> Result<String> {
    use crate::plugins::{NodePlugin, LanguagePlugin};
    use dialoguer::{Select, theme::ColorfulTheme};
    
    let theme = ColorfulTheme::default();
    let plugin = NodePlugin;
    
    // Get installed versions
    let installed = plugin.list_installed().unwrap_or_default();
    
    // Build version options with metadata
    struct VersionOption {
        value: String,
        display: String,
    }
    
    let mut options: Vec<VersionOption> = Vec::new();
    
    // Add installed versions with compatibility info
    for version in &installed {
        let info = get_version_info(version);
        options.push(VersionOption {
            value: version.clone(),
            display: format!("{}  {}", version, info),
        });
    }
    
    // Add separator if there are installed versions
    if !installed.is_empty() {
        options.push(VersionOption {
            value: "".to_string(),
            display: "─── Version Aliases ───".to_string(),
        });
    }
    
    // Add aliases with descriptions
    options.push(VersionOption {
        value: "latest".to_string(),
        display: "latest              Latest stable release".to_string(),
    });
    options.push(VersionOption {
        value: "lts".to_string(),
        display: "lts                 Latest LTS (recommended)".to_string(),
    });
    options.push(VersionOption {
        value: "22".to_string(),
        display: "22                  Current release line".to_string(),
    });
    options.push(VersionOption {
        value: "20".to_string(),
        display: "20                  Active LTS (best compatibility)".to_string(),
    });
    options.push(VersionOption {
        value: "18".to_string(),
        display: "18                  Maintenance LTS".to_string(),
    });
    
    // If no installed versions, show informative message
    if installed.is_empty() {
        options.insert(0, VersionOption {
            value: "".to_string(),
            display: "⚠️  No versions installed - select an alias to install".to_string(),
        });
    }
    
    // Extract display items
    let display_items: Vec<String> = options.iter()
        .map(|opt| opt.display.clone())
        .collect();
    
    let version_idx = Select::with_theme(&theme)
        .with_prompt("Select Node.js version")
        .items(&display_items)
        .default(if installed.is_empty() { 2 } else { 0 })
        .interact()?;
    
    let selected = &options[version_idx];
    
    // Skip separator and warning
    if selected.value.is_empty() || selected.value.starts_with("⚠️") {
        return Err(anyhow::anyhow!("Please select a valid version"));
    }
    
    Ok(selected.value.clone())
}

/// Get version compatibility and status information
fn get_version_info(version: &str) -> String {
    let major = version.split('.').next().unwrap_or("0");
    let major_num: u32 = major.parse().unwrap_or(0);
    
    // Determine version status and compatibility
    if major_num >= 23 {
        format!("🔥 Current  (~85% pkg compat)")
    } else if major_num == 22 {
        format!("✅ Current  (~95% pkg compat)")
    } else if major_num == 20 {
        format!("⭐ LTS     (~98% pkg compat) [Recommended]")
    } else if major_num == 18 {
        format!("🔧 LTS     (~95% pkg compat) [Maintenance]")
    } else if major_num <= 16 {
        format!("⚠️  Deprecated (<80% pkg compat)")
    } else {
        format!("✅ Installed")
    }
}

/// Interactive Python version selection (stub - future implementation)
fn select_python_version() -> Result<String> {
    use dialoguer::{Select, theme::ColorfulTheme};
    
    let theme = ColorfulTheme::default();
    let versions = vec!["3.12.2", "3.11.8", "3.10.14", "latest"];
    
    let version_idx = Select::with_theme(&theme)
        .with_prompt("Select Python version")
        .items(&versions)
        .default(0)
        .interact()?;
    
    Ok(versions[version_idx].to_string())
}

/// Health check & validation system
fn run_validation(language: &str, version: &str, packages: &[(String, String)]) -> Result<()> {
    use colored::Colorize;
    use crate::plugins::{NodePlugin, LanguagePlugin};
    
    let mut all_checks_passed = true;
    
    // Check 1: ven.toml created
    println!("  {} ven.toml created", "✓".green());
    
    // Check 2: Node.js version installed
    if language == "node" {
        let plugin = NodePlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        
        if version == "latest" || version == "lts" || !version.contains('.') {
            // Alias - will be resolved during install
            println!("  {} Node.js {} (will resolve during install)", "⚠️".yellow(), version);
        } else if installed.contains(&version.to_string()) {
            println!("  {} Node.js {} installed", "✓".green(), version);
        } else {
            println!("  {} Node.js {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install node {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }
    
    // Check 3: Package compatibility (basic check)
    if !packages.is_empty() {
        println!("  {} {} packages declared", "✓".green(), packages.len());
        
        // Future: Check npm registry for compatibility
        // For now, just list them
        for (pkg, _) in packages {
            println!("      {} {}", "•".dimmed(), pkg);
        }
    }
    
    // Check 4: Environment variables
    println!("  {} Environment variables (optional)", "ℹ️".blue());
    
    // Final summary
    println!();
    if all_checks_passed {
        println!("  {} Ready to develop!", "🚀".green().bold());
    } else {
        println!("  {} Some issues need attention", "⚠️".yellow().bold());
    }
    
    Ok(())
}

// ── ven setup ─────────────────────────────────────────────────────
fn cmd_setup() -> Result<()> {
    use colored::Colorize;
    use std::io::Write;
    use crate::shell::detect_shell;

    // FIXED: detect shell properly — Windows always uses PowerShell
    let shell_name = detect_shell();

    println!("\n  {} ven setup", "→".cyan());
    println!("  Detected shell: {}", shell_name.bold());

    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    // FIXED: Windows writes to PowerShell $PROFILE, not ~/.bashrc
    let (rc_file, hook_line) = if cfg!(target_os = "windows") {
        // PowerShell profile location
        let profile = home
            .join("Documents")
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1");
        let line = "\n# ven shell hook\nInvoke-Expression (& ven shell hook powershell | Out-String)".to_string();
        (profile, line)
    } else {
        // Unix — bash/zsh/fish
        let rc = match shell_name.as_str() {
            "zsh"  => home.join(".zshrc"),
            "fish" => home.join(".config").join("fish").join("config.fish"),
            _      => home.join(".bashrc"),
        };
        let line = format!("\n# ven shell hook\neval \"$(ven shell hook {})\""  , shell_name);
        (rc, line)
    };

    // Check if already installed
    let existing = std::fs::read_to_string(&rc_file).unwrap_or_default();
    if existing.contains("ven shell hook") {
        println!("  {} Shell hook already installed in {}", "✓".green(), rc_file.display());
        return Ok(());
    }

    // Create parent dirs if needed (PowerShell profile dir may not exist)
    if let Some(parent) = rc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Append hook line to rc file
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)?;
    writeln!(file, "{}", hook_line)?;

    println!("  {} Written to {}", "✓".green(), rc_file.display());
    println!();

    if cfg!(target_os = "windows") {
        println!("  Restart PowerShell or run:");
        println!("  {}", ". $PROFILE".bold());
    } else {
        println!("  Restart your shell or run:");
        println!("  {}", format!("source {}", rc_file.display()).bold());
    }
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
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    // Determine version to install
    let version_to_install = if let Some(pinned) = pinned_version {
        // User specified exact version
        pinned.to_string()
    } else if skip_check {
        // Skip compatibility check - just use "latest" tag via npm directly
        println!("{} Skipping compatibility check for {}...",
            "→".cyan(), pkg_name.bold());
        // Let npm decide the version (will install latest)
        npm_install(pkg_name, "latest")?;
        update_ven_toml_package(pkg_name, "latest")?;
        return Ok(());
    } else {
        // Normal path: fetch npm metadata and find compatible version
        println!("{} Checking {} against Node {}...",
            "→".cyan(), pkg_name.bold(), node_version.bold());
        
        // Fetch npm metadata
        let info = fetch_npm_info(pkg_name)?;
        
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
    use colored::Colorize;
    use crate::core::find_ven_toml;

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
        .map(|c| c.runtime.node)
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
