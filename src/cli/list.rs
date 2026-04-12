use anyhow::Result;
use colored::Colorize;
use crate::plugins::PluginRegistry;
use crate::core::{find_ven_toml, parse_ven_toml, resolve_node_version};

// ── ven list [language] ───────────────────────────────────────────────
pub fn cmd_list(language: Option<&str>) -> Result<()> {
    let lang = language.unwrap_or("node");
    let registry = PluginRegistry::new();
    let plugin = registry.require(lang)?;
    
    // Get installed versions
    let versions = plugin.list_installed()?;

    if versions.is_empty() {
        println!("{} No {} versions installed. Run: ven install {} latest", 
            "[WARN]".yellow(), 
            lang.bold(), 
            lang.bold()
        );
        return Ok(());
    }

    // Detect active version from ven.toml
    let active_version = detect_active_version(lang)?;

    // Display versions with metadata
    display_versions_with_metadata(lang, &versions, &active_version)?;
    
    Ok(())
}

/// Detect which version is currently active (from ven.toml)
fn detect_active_version(language: &str) -> Result<Option<String>> {
    // Only support node for now
    if language != "node" {
        return Ok(None);
    }

    // Find ven.toml in current directory
    let current_dir = std::env::current_dir()?;
    let toml_path = match find_ven_toml(&current_dir) {
        Some(p) => p,
        None => return Ok(None), // No ven.toml found
    };

    // Parse config
    let config = parse_ven_toml(&toml_path)?;
    let node_spec = &config.runtime.node;
    
    if node_spec.is_empty() {
        return Ok(None);
    }

    // Get installed versions and resolve
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;
    let installed = plugin.list_installed().unwrap_or_default();
    
    // Resolve version spec to actual version
    match resolve_node_version(node_spec, &installed) {
        Ok(resolved) => Ok(Some(resolved)),
        Err(_) => Ok(None), // If resolution fails, no active version
    }
}

/// Get version status based on major version number
fn get_version_status(version: &str) -> (&'static str, &'static str) {
    let major = version.split('.').next().unwrap_or("0");
    let major_num: u32 = major.parse().unwrap_or(0);
    
    match major_num {
        0..=14 => ("DEPRECATED", "End-of-life"),
        15..=16 => ("DEPRECATED", "Maintenance ended"),
        18 => ("LTS", "Active LTS"),
        20 => ("LTS", "Active LTS (Recommended)"),
        21 => ("DEPRECATED", "Maintenance ended"),
        22 => ("CURRENT", "Active development"),
        23..=99 => ("CURRENT", "Latest stable"),
        _ => ("UNKNOWN", "Unknown status"),
    }
}

/// Display versions with metadata and active indicator
fn display_versions_with_metadata(
    language: &str,
    versions: &[String],
    active_version: &Option<String>,
) -> Result<()> {
    let count = versions.len();
    
    println!("\n  {} ({} versions installed)", language.bold().cyan(), count.to_string().bold());
    println!();
    
    for version in versions {
        let (status, description) = get_version_status(version);
        
        // Determine marker
        let is_active = active_version.as_ref() == Some(version);
        let marker = if is_active {
            "▸".bold().green()
        } else {
            "•".dimmed()
        };
        
        // Determine status color and tag
        let status_tag = match status {
            "LTS" => format!("[LTS] ⭐"),
            "CURRENT" => format!("[CURRENT]"),
            "DEPRECATED" => format!("[DEPRECATED]"),
            _ => format!("[{}] ", status),
        };
        
        // Print version line
        if is_active {
            println!(
                "    {} {}  {} {}",
                marker,
                version.bold().green(),
                status_tag,
                format!("- {}", description).dimmed()
            );
        } else {
            println!(
                "    {} {}  {} {}",
                marker,
                version,
                status_tag,
                format!("- {}", description).dimmed()
            );
        }
    }
    
    // Show helpful tips
    println!();
    if let Some(active) = active_version {
        println!("  {} Currently active: {}", "[ACTIVE]".green().bold(), active.bold());
    }
    
    // Check for deprecated versions
    let deprecated_count = versions.iter()
        .filter(|v| get_version_status(v).0 == "DEPRECATED")
        .count();
    
    if deprecated_count > 0 {
        println!("  {} {} deprecated version(s) - consider removing to free space", 
            "[TIP]".yellow(), 
            deprecated_count
        );
    }
    
    println!();
    Ok(())
}
