use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};
use crate::core::{find_ven_toml, parse_ven_toml, resolve_node_version};
use crate::core::config::VenConfig;

// ── ven status ────────────────────────────────────────────────────
pub fn cmd_status(json: bool, verbose: bool, fix: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    
    // Find ven.toml
    let toml_path = match find_ven_toml(&cwd) {
        Some(p) => p,
        None => {
            if json {
                println!("{{\"error\": \"No ven.toml found\"}}");
            } else {
                println!("\n  {} {}", "ven status".bold(), cwd.display());
                println!("  {} No ven.toml found in this directory tree.", "[WARN]".yellow());
                println!("  {} Run: ven init   to create one.", "[TIP]".cyan());
                println!();
            }
            return Ok(());
        }
    };
    
    let config = parse_ven_toml(&toml_path)?;
    
    if json {
        output_json_status(&cwd, &toml_path, &config, verbose)?;
    } else if verbose {
        display_verbose_status(&cwd, &toml_path, &config, fix)?;
    } else {
        display_basic_status(&cwd, &toml_path, &config)?;
    }
    
    Ok(())
}

// ── Basic Status Display ─────────────────────────────────────────
fn display_basic_status(cwd: &Path, toml_path: &Path, config: &VenConfig) -> Result<()> {
    println!("\n  {} {}", "ven status".bold(), cwd.display());
    println!("  {} {}", "Config".dimmed(), toml_path.display());
    println!();
    
    // Runtime section
    if !config.runtime.node.is_empty() {
        let node_spec = &config.runtime.node;
        let resolved = resolve_version_for_display(node_spec)?;
        let installed = is_version_installed(node_spec);
        
        let status_icon = if installed { "✓" } else { "✗" };
        
        println!("  {} node {} {}", 
            status_icon, 
            node_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        
        if !installed {
            println!("    {} Run: ven install node {}", "[!]".yellow(), node_spec);
        }
    } else {
        println!("  {} node {}", "[!]".yellow(), "not specified".dimmed());
    }
    
    // Packages section
    let pkg_count = config.packages.len();
    if pkg_count > 0 {
        // Count installed packages
        let installed_count = config.packages.keys()
            .filter(|pkg| is_package_installed(pkg))
            .count();
        
        println!("  {} {} package(s) declared, {} installed", 
            "packages".bold(), 
            pkg_count,
            installed_count
        );
        
        // Show tip if packages are missing
        if installed_count < pkg_count {
            println!("    {} Install missing: ven add --sync or npm install", "[TIP]".cyan());
        }
    } else {
        println!("  {} {}", "packages".bold(), "none".dimmed());
    }
    
    // Environment variables
    let env_count = config.env.len();
    if env_count > 0 {
        println!("  {} {} variable(s) defined", "env".bold(), env_count);
    }
    
    println!();
    Ok(())
}

// ── Verbose Status Display ───────────────────────────────────────
fn display_verbose_status(cwd: &Path, toml_path: &Path, config: &VenConfig, fix: bool) -> Result<()> {
    println!("\n  {} {}", "ven status".bold().cyan(), cwd.display());
    println!("  {} {}", "Config".dimmed(), toml_path.display());
    println!();
    
    // ── Runtime Status ──
    println!("  {}", "Runtime".bold().underline());
    
    if !config.runtime.node.is_empty() {
        let node_spec = &config.runtime.node;
        let resolved = resolve_version_for_display(node_spec)?;
        let installed = is_version_installed(node_spec);
        
        if installed {
            let bin_path = get_bin_path_for_version(node_spec)?;
            let version_size = calculate_dir_size(&bin_path.parent().unwrap())?;
            
            println!("    {} node {} ({})", "✓".green(), node_spec.bold(), resolved);
            println!("      {} {}", "Binary:".dimmed(), bin_path.display());
            println!("      {} {}", "Size:".dimmed(), format_bytes(version_size));
            
            // Check if active in PATH
            let is_active = check_if_version_active(node_spec)?;
            if is_active {
                println!("      {} {}", "Status:".dimmed(), "[ACTIVE]".green());
            } else {
                println!("      {} {}", "Status:".dimmed(), "[INACTIVE]".yellow());
            }
        } else {
            println!("    {} node {} - {}", "✗".red(), node_spec.bold(), "not installed");
            println!("      {} Run: ven install node {}", "[!]".yellow(), node_spec);
            
            if fix {
                auto_install_version(node_spec)?;
            }
        }
    }
    
    println!();
    
    // ── Package Status ──
    let pkg_count = config.packages.len();
    if pkg_count > 0 {
        println!("  {}", "Packages".bold().underline());
        
        let mut installed_count = 0;
        let mut missing_count = 0;
        let mut incompatible_count = 0;
        
        for (pkg_name, pkg_version) in &config.packages {
            let is_installed = is_package_installed(pkg_name);
            
            if is_installed {
                installed_count += 1;
                if let Ok(installed_ver) = get_installed_package_version(pkg_name) {
                    // Check compatibility
                    if let Ok(compatible) = check_package_compatibility(pkg_name, &installed_ver, &config.runtime.node) {
                        if compatible {
                            println!("    {} {}@{} {}", "✓".green(), pkg_name, installed_ver, "[compatible]".dimmed());
                        } else {
                            incompatible_count += 1;
                            println!("    {} {}@{} {}", "⚠".yellow(), pkg_name, installed_ver, "[incompatible]".yellow());
                        }
                    } else {
                        println!("    {} {}@{}", "✓".green(), pkg_name, installed_ver);
                    }
                    
                    // Verbose: show more details (we're already in verbose mode)
                    let pkg_path = std::env::current_dir()
                        .unwrap_or_default()
                        .join("node_modules")
                        .join(pkg_name);
                    println!("      {} {}", "Location:".dimmed(), pkg_path.display());
                }
            } else {
                missing_count += 1;
                println!("    {} {}@{} {}", "✗".red(), pkg_name, pkg_version, "[not installed]".red());
                
                if fix {
                    auto_install_package(pkg_name, pkg_version)?;
                }
            }
        }
        
        println!();
        println!("    {} {} installed, {} missing, {} incompatible", 
            "Summary:".dimmed(),
            installed_count.to_string().green(),
            missing_count.to_string().red(),
            incompatible_count.to_string().yellow()
        );
        
        if missing_count > 0 && !fix {
            println!("    {} Run: ven add --sync  or  npm install", "[TIP]".cyan());
        }
    }
    
    println!();
    
    // ── Environment Variables ──
    let env_count = config.env.len();
    if env_count > 0 {
        println!("  {}", "Environment".bold().underline());
        
        for (key, value) in &config.env {
            let key_str = key.as_str();
            let value_str = value.as_str();
            let current = std::env::var(key_str).ok();
            let is_set = current.as_deref() == Some(value_str);
            
            let icon = if is_set { "✓" } else { "○" };
            let status = if is_set { "[active]".green() } else { "[not set]".yellow() };
            
            println!("    {} {}={} {}", icon, key_str.bold(), value_str.dimmed(), status);
        }
        println!();
    }
    
    // ── Health Summary ──
    println!("  {}", "Health Summary".bold().underline());
    print_health_summary(config)?;
    
    println!();
    Ok(())
}

// ── JSON Output Mode ─────────────────────────────────────────────
fn output_json_status(cwd: &Path, toml_path: &Path, config: &VenConfig, verbose: bool) -> Result<()> {
    use serde_json::json;
    
    let mut runtime_info = json!({
        "name": "node",
        "version_required": config.runtime.node,
    });
    
    if !config.runtime.node.is_empty() {
        let node_spec = &config.runtime.node;
        let resolved = resolve_version_for_display(node_spec)?;
        let installed = is_version_installed(node_spec);
        
        runtime_info["version_resolved"] = json!(resolved);
        runtime_info["installed"] = json!(installed);
        
        if verbose && installed {
            if let Ok(bin_path) = get_bin_path_for_version(node_spec) {
                runtime_info["binary_path"] = json!(bin_path.to_string_lossy());
                
                if let Ok(size) = calculate_dir_size(&bin_path.parent().unwrap()) {
                    runtime_info["size_bytes"] = json!(size);
                }
                
                // Check if active
                let is_active = check_if_version_active(node_spec).unwrap_or(false);
                runtime_info["active"] = json!(is_active);
            }
        }
    }
    
    // Build package list
    let mut pkg_list = Vec::new();
    let mut installed_count = 0;
    
    for (name, version) in &config.packages {
        let is_installed = is_package_installed(name);
        let mut pkg_info = json!({
            "name": name,
            "version_declared": version,
            "installed": is_installed
        });
        
        if is_installed {
            installed_count += 1;
            if let Ok(installed_ver) = get_installed_package_version(name) {
                pkg_info["version_installed"] = json!(installed_ver);
                
                if verbose {
                    // Get package location
                    let pkg_location = std::env::current_dir()
                        .unwrap_or_default()
                        .join("node_modules")
                        .join(name)
                        .to_string_lossy()
                        .to_string();
                    pkg_info["location"] = json!(pkg_location);
                    
                    // Check compatibility
                    if let Ok(compatible) = check_package_compatibility(name, &installed_ver, &config.runtime.node) {
                        pkg_info["compatible"] = json!(compatible);
                    }
                }
            }
        } else {
            pkg_info["version_installed"] = serde_json::Value::Null;
        }
        
        pkg_list.push(pkg_info);
    }
    
    let packages_info = json!({
        "declared": config.packages.len(),
        "installed": installed_count,
        "list": pkg_list
    });
    
    let mut status = json!({
        "project_root": cwd.to_string_lossy(),
        "config_path": toml_path.to_string_lossy(),
        "runtime": runtime_info,
        "packages": packages_info
    });
    
    // Add lock file info in verbose mode
    if verbose {
        let lock_file = cwd.join("ven.lock");
        status["lock_file"] = json!({
            "exists": lock_file.exists(),
            "path": lock_file.to_string_lossy()
        });
    }
    
    // Add env vars if present
    if !config.env.is_empty() {
        let mut env_list = Vec::new();
        for (key, value) in &config.env {
            let current = std::env::var(key).ok();
            env_list.push(json!({
                "key": key,
                "required": value,
                "active": current.as_deref() == Some(value.as_str())
            }));
        }
        status["environment"] = json!(env_list);
    }
    
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

// ── Helper Functions ─────────────────────────────────────────────

fn resolve_version_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    
    let registry = PluginRegistry::new();
    let plugin = registry.require("node")?;
    let installed = plugin.list_installed().unwrap_or_default();
    
    match resolve_node_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string())
    }
}

fn is_version_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("node") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_node_version(spec, &installed).is_ok()
    } else {
        false
    }
}

fn get_bin_path_for_version(spec: &str) -> Result<PathBuf> {
    use crate::plugins::PluginRegistry;
    
    let registry = PluginRegistry::new();
    let plugin = registry.require("node")?;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = resolve_node_version(spec, &installed)?;
    
    plugin.bin_path(&resolved)
}

fn is_package_installed(package: &str) -> bool {
    let pkg_json = std::env::current_dir()
        .unwrap_or_default()
        .join("node_modules")
        .join(package)
        .join("package.json");
    
    pkg_json.exists()
}

fn get_installed_package_version(package: &str) -> Result<String> {
    crate::core::packages::get_installed_version(package)
}

fn check_package_compatibility(pkg_name: &str, _pkg_version: &str, node_spec: &str) -> Result<bool> {
    use crate::core::packages::{fetch_npm_info, find_compatible_version};
    
    let info = fetch_npm_info(pkg_name)?;
    let compatible_version = find_compatible_version(&info, node_spec);
    
    Ok(compatible_version.is_some())
}

fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total_size = 0;
    
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                total_size += entry.metadata()?.len();
            } else if path.is_dir() {
                total_size += calculate_dir_size(&path)?;
            }
        }
    }
    
    Ok(total_size)
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn check_if_version_active(spec: &str) -> Result<bool> {
    let current_path = std::env::var("PATH").unwrap_or_default();
    let bin_path = get_bin_path_for_version(spec)?;
    
    Ok(current_path.contains(&bin_path.parent().unwrap().to_string_lossy().to_string()))
}

fn print_health_summary(config: &VenConfig) -> Result<()> {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut ok_items = Vec::new();
    
    // Check Node.js
    if config.runtime.node.is_empty() {
        issues.push("No Node.js version specified".to_string());
    } else if !is_version_installed(&config.runtime.node) {
        issues.push(format!("Node.js {} not installed", config.runtime.node));
    } else {
        ok_items.push(format!("Node.js {} ready", config.runtime.node));
    }
    
    // Check packages
    let pkg_count = config.packages.len();
    if pkg_count > 0 {
        let mut missing = 0;
        for pkg_name in config.packages.keys() {
            if !is_package_installed(pkg_name) {
                missing += 1;
            }
        }
        
        if missing == 0 {
            ok_items.push(format!("All {} packages installed", pkg_count));
        } else {
            warnings.push(format!("{}/{} packages missing", missing, pkg_count));
        }
    }
    
    // Print summary
    if !issues.is_empty() {
        for issue in &issues {
            println!("    {} {}", "✗".red(), issue);
        }
    }
    
    if !warnings.is_empty() {
        for warning in &warnings {
            println!("    {} {}", "⚠".yellow(), warning);
        }
    }
    
    for ok in &ok_items {
        println!("    {} {}", "✓".green(), ok);
    }
    
    if issues.is_empty() && warnings.is_empty() {
        println!("    {} {}", "✓".green(), "All checks passed!".green());
    }
    
    Ok(())
}

fn auto_install_version(spec: &str) -> Result<()> {
    println!("\n  {} Installing node {}...", "[AUTO-FIX]".cyan(), spec);
    crate::cli::install::cmd_install("node", spec)?;
    println!("  {} Node.js {} installed", "✓".green(), spec);
    Ok(())
}

fn auto_install_package(pkg_name: &str, pkg_version: &str) -> Result<()> {
    println!("  {} Installing {}@{}...", "[AUTO-FIX]".cyan(), pkg_name, pkg_version);
    
    let packages = vec![format!("{}@{}", pkg_name, pkg_version)];
    crate::cli::add::cmd_add(&packages, false)?;
    
    println!("  {} {}@{} installed", "✓".green(), pkg_name, pkg_version);
    Ok(())
}
