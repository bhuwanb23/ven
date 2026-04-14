use anyhow::Result;
use colored::Colorize;
use toml_edit::{DocumentMut, value};
use crate::core::{load_config, packages::*};

/// Package entry for batch processing
struct PackageEntry {
    name: String,
    pinned_version: Option<String>,
}

/// Add packages with Node.js compatibility checking
pub fn cmd_add(package_specs: &[String], skip_check: bool) -> Result<()> {
    if package_specs.is_empty() {
        println!("  {} No packages specified", "[ERROR]".red());
        println!("  {} Usage: ven add <package> [package...] [--skip-check]", "[TIP]".cyan());
        return Ok(());
    }

    // Parse all package specs
    let packages: Vec<PackageEntry> = package_specs
        .iter()
        .map(|spec| {
            // Split "express@4.18.2" into name + optional version pin
            let (name, pinned_version) = if spec.contains('@') && !spec.starts_with('@') {
                let parts: Vec<&str> = spec.splitn(2, '@').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (spec.clone(), None)
            };
            PackageEntry { name, pinned_version }
        })
        .collect();

    // Get current Node version from ven.toml
    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    println!("\n{}", "ven add".bold().cyan());
    println!("  {} {} package(s)", "[PLAN]".cyan(), packages.len());
    println!("  {} Node.js {}", "[RUNTIME]".cyan(), node_version);
    println!();

    // Process each package
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut results: Vec<(String, String, bool)> = Vec::new(); // (name, version, success)

    for pkg in &packages {
        println!("{} Processing {}...", "→".cyan(), pkg.name.bold());

        match process_package(pkg, &node_version, skip_check) {
            Ok(version) => {
                results.push((pkg.name.clone(), version.clone(), true));
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {}", "[ERROR]".red(), e.to_string().red());
                results.push((pkg.name.clone(), String::new(), false));
                fail_count += 1;
            }
        }
    }

    // Print summary
    println!();
    println!("  {}", "Summary".bold().cyan());
    println!("    {} {} package(s) processed", "Total:".dimmed(), packages.len());
    println!("    {} {}", "Success:".dimmed(), success_count.to_string().green());
    
    if fail_count > 0 {
        println!("    {} {}", "Failed:".dimmed(), fail_count.to_string().red());
        println!();
        for (name, _, success) in &results {
            if !success {
                println!("    {} {}", "[FAIL]".red(), name);
            }
        }
    } else {
        println!("    {} {}", "Failed:".dimmed(), "0".green());
    }

    // Update ven.toml once with all successful packages
    let successful_packages: Vec<_> = results
        .iter()
        .filter(|(_, _, success)| *success)
        .map(|(name, version, _)| (name.clone(), version.clone()))
        .collect();

    if !successful_packages.is_empty() {
        println!();
        update_ven_toml_packages(&successful_packages)?;
    }

    println!();
    Ok(())
}

/// Process a single package: check compatibility and install
fn process_package(
    pkg: &PackageEntry,
    node_version: &str,
    skip_check: bool,
) -> Result<String> {
    let version_to_install = if let Some(ref pinned) = pkg.pinned_version {
        // User specified exact version
        pinned.clone()
    } else if skip_check {
        // Skip compatibility check
        println!("  {} Skipping compatibility check", "[SKIP]".yellow());
        "latest".to_string()
    } else {
        // Normal path: fetch npm metadata and find compatible version
        println!("  {} Checking against Node {}...", "[CHECK]".cyan(), node_version);
        
        // Fetch npm metadata
        let info = fetch_npm_info(&pkg.name)?;
        
        // Find best compatible
        find_compatible_version(&info, node_version)
            .ok_or_else(|| anyhow::anyhow!(
                "No compatible version of {} found for Node.js {}\n  Hint: Use --skip-check to bypass",
                pkg.name, node_version
            ))?
    };

    println!("  {} {} — compatible with Node {}", 
        "[OK]".green(), 
        format!("{}@{}", pkg.name, version_to_install).bold(), 
        node_version
    );

    // Run npm install
    npm_install(&pkg.name, &version_to_install)?;

    Ok(version_to_install)
}

/// Update ven.toml with multiple packages using proper TOML parsing
pub fn update_ven_toml_packages(packages: &[(String, String)]) -> Result<()> {
    use crate::core::find_ven_toml;

    let cwd = std::env::current_dir()?;
    let toml_path = find_ven_toml(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    // Read and parse TOML
    let content = std::fs::read_to_string(&toml_path)?;
    let mut doc: DocumentMut = content.parse::<DocumentMut>()
        .map_err(|e| anyhow::anyhow!("Failed to parse ven.toml: {}", e))?;

    // Ensure [packages] table exists
    if !doc.contains_key("packages") {
        doc["packages"] = toml_edit::table();
    }

    // Add or update each package
    let packages_table = doc["packages"].as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("Failed to access [packages] table"))?;

    for (pkg_name, version) in packages {
        // Check if package already exists
        let action = if packages_table.contains_key(pkg_name) {
            "Updated"
        } else {
            "Added"
        };

        // Insert or update the package
        packages_table.insert(pkg_name, value(version));
        
        println!("  {} {} {} = \"{}\"", "[TOML]".cyan(), action, pkg_name, version);
    }

    // Write back to file (preserves formatting and comments)
    std::fs::write(&toml_path, doc.to_string())?;
    println!("  {} ven.toml updated with {} package(s)", "[OK]".green(), packages.len());

    Ok(())
}
