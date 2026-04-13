use crate::cli::add::update_ven_toml_package;
use crate::core::{load_config, packages::*};
use anyhow::Result;
use colored::Colorize;

/// Upgrade a package to latest compatible version
pub fn cmd_upgrade(package: &str, apply: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    // Get currently installed version from node_modules
    let current_ver = get_installed_version(package).unwrap_or_else(|_| "unknown".to_string());

    // Fetch latest compatible version
    let info = fetch_npm_info(package)?;
    let latest = find_compatible_version(&info, &node_version)
        .ok_or_else(|| anyhow::anyhow!("No compatible version found"))?;

    if current_ver == latest {
        println!(
            "{} {} is already up to date ({})",
            "✓".green(),
            package.bold(),
            latest
        );
        return Ok(());
    }

    println!(
        "\n  {} {}  →  {}  (latest compatible)",
        package.bold(),
        current_ver.dimmed(),
        latest.green()
    );
    println!(
        "\n  Compatibility: {} Node {} supported",
        "✓".green(),
        node_version
    );

    // Show changelog hint
    let notes = fetch_release_notes(package, &current_ver, &latest);
    println!("\n  Release notes: {}", notes.dimmed());

    if !apply {
        println!(
            "\n  Run  {} to upgrade",
            format!("ven upgrade {} --apply", package).bold()
        );
        return Ok(());
    }

    npm_install(package, &latest)?;
    update_ven_toml_package(package, &latest)?;
    Ok(())
}
