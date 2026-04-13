use crate::core::{load_config, packages::*};
use anyhow::Result;
use colored::Colorize;

/// Add a package with Node.js compatibility checking
pub fn cmd_add(package_spec: &str, skip_check: bool) -> Result<()> {
    // Split "express@4.18.2" into name + optional version pin
    let (pkg_name, pinned_version) = if package_spec.contains('@') && !package_spec.starts_with('@')
    {
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
        println!(
            "{} Skipping compatibility check for {}...",
            "→".cyan(),
            pkg_name.bold()
        );
        // Let npm decide the version (will install latest)
        npm_install(pkg_name, "latest")?;
        update_ven_toml_package(pkg_name, "latest")?;
        return Ok(());
    } else {
        // Normal path: fetch npm metadata and find compatible version
        println!(
            "{} Checking {} against Node {}...",
            "→".cyan(),
            pkg_name.bold(),
            node_version.bold()
        );

        // Fetch npm metadata
        let info = fetch_npm_info(pkg_name)?;

        // Find best compatible
        find_compatible_version(&info, &node_version).ok_or_else(|| {
            anyhow::anyhow!(
                "No compatible version of {} found for Node {}",
                pkg_name,
                node_version
            )
        })?
    };

    println!(
        "  {} {} — compatible with Node {}",
        "✓".green(),
        format!("{}@{}", pkg_name, version_to_install).bold(),
        node_version
    );

    // Run npm install
    npm_install(pkg_name, &version_to_install)?;

    // Update ven.toml
    update_ven_toml_package(pkg_name, &version_to_install)?;

    Ok(())
}

/// Update ven.toml with new package
pub fn update_ven_toml_package(pkg: &str, version: &str) -> Result<()> {
    use crate::core::find_ven_toml;

    let cwd = std::env::current_dir()?;
    let toml_path =
        find_ven_toml(&cwd).ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

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
