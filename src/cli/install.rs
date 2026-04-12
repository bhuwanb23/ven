use anyhow::Result;
use colored::Colorize;
use crate::plugins::{PluginRegistry, LanguagePlugin};

/// Resolve a major version like "20" to the latest 20.x.x by fetching nodejs.org release list
fn resolve_major_version(plugin: &dyn LanguagePlugin, major: &str) -> Result<String> {
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

// ── ven install node <version> ────────────────────────────────────
pub fn cmd_install(language: &str, version: &str) -> Result<()> {
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;

    // Resolve aliases AND major-only versions before installing
    // "lts" → latest LTS  e.g. "20.11.0"
    // "latest" → latest stable e.g. "22.3.0"
    // "20" → highest 20.x available on nodejs.org e.g. "20.11.0"
    // "20.11.0" → exact, pass through
    let resolved = if version == "lts" || version == "latest" {
        println!("{} Fetching {} release list...", "→".cyan(), language.bold());
        plugin.latest_version()?
    } else if !version.contains('.') {
        // Major-only like "20" — resolve to highest 20.x from nodejs.org
        println!("{} Resolving {} {} to latest patch version...", "→".cyan(), language.bold(), version.bold());
        resolve_major_version(plugin, version)?
    } else {
        version.to_string()
    };

    println!("{} Resolved to {} {}", "✓".green(), language.bold(), resolved.bold());
    plugin.install_version(&resolved)?;
    Ok(())
}
