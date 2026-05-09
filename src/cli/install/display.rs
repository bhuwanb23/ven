use anyhow::Result;
use colored::Colorize;

use crate::plugins::PluginRegistry;

/// Display concise install list summary before interactive selection.
pub(super) fn display_version_list(versions: &[String], language: &str) -> Result<()> {
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;
    let installed = plugin.list_installed().unwrap_or_default();

    let installed_count = installed.len();
    println!();
    if installed_count > 0 {
        println!(
            "  {} {} version(s) available, {} installed",
            "[INFO]".cyan(),
            versions.len(),
            installed_count
        );
    } else {
        println!(
            "  {} {} version(s) available",
            "[INFO]".cyan(),
            versions.len()
        );
    }
    if language == "node" {
        println!(
            "  {} Use {} or {} to install quickly",
            "[TIP]".yellow(),
            "latest".green(),
            "lts".green()
        );
    } else {
        println!(
            "  {} Use {} to install quickly",
            "[TIP]".yellow(),
            "latest".green()
        );
    }

    Ok(())
}
