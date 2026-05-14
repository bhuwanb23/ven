mod languages;

use crate::core::load_config;
use crate::core::packages::*;
use anyhow::Result;
use colored::Colorize;
use languages::{
    cmd_remove_bun, cmd_remove_deno, cmd_remove_go, cmd_remove_java, cmd_remove_python,
    cmd_remove_ruby, cmd_remove_rust,
};
use std::collections::HashSet;
use std::path::Path;

/// Remove packages with batch support and advanced features
pub fn cmd_remove(
    packages: &[String],
    force: bool,
    dry_run: bool,
    json: bool,
    verbose: bool,
    cleanup: bool,
    yes: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let python_mode = load_config(&cwd)?
        .map(|c| {
            !c.runtime.python.is_empty() && c.runtime.node.is_empty() && c.runtime.bun.is_empty()
        })
        .unwrap_or(false);
    let rust_mode = load_config(&cwd)?
        .map(|c| {
            !c.runtime.rust.is_empty()
                && c.runtime.node.is_empty()
                && c.runtime.python.is_empty()
                && c.runtime.go.is_empty()
                && c.runtime.ruby.is_empty()
                && c.runtime.bun.is_empty()
        })
        .unwrap_or(false);
    let go_mode = load_config(&cwd)?
        .map(|c| {
            !c.runtime.go.is_empty()
                && c.runtime.node.is_empty()
                && c.runtime.python.is_empty()
                && c.runtime.rust.is_empty()
                && c.runtime.java.is_empty()
                && c.runtime.deno.is_empty()
                && c.runtime.ruby.is_empty()
                && c.runtime.bun.is_empty()
        })
        .unwrap_or(false);
    let java_mode = load_config(&cwd)?
        .map(|c| {
            !c.runtime.java.is_empty()
                && c.runtime.node.is_empty()
                && c.runtime.python.is_empty()
                && c.runtime.go.is_empty()
                && c.runtime.rust.is_empty()
                && c.runtime.ruby.is_empty()
                && c.runtime.bun.is_empty()
        })
        .unwrap_or(false);
    let deno_mode = load_config(&cwd)?
        .map(|c| {
            !c.runtime.deno.is_empty()
                && c.runtime.node.is_empty()
                && c.runtime.python.is_empty()
                && c.runtime.go.is_empty()
                && c.runtime.rust.is_empty()
                && c.runtime.java.is_empty()
                && c.runtime.ruby.is_empty()
                && c.runtime.bun.is_empty()
        })
        .unwrap_or(false);
    let bun_mode = load_config(&cwd)?
        .map(|c| {
            !c.runtime.bun.is_empty()
                && c.runtime.node.is_empty()
                && c.runtime.python.is_empty()
                && c.runtime.go.is_empty()
                && c.runtime.rust.is_empty()
                && c.runtime.java.is_empty()
                && c.runtime.deno.is_empty()
                && c.runtime.ruby.is_empty()
        })
        .unwrap_or(false);
    let ruby_mode = load_config(&cwd)?
        .map(|c| {
            !c.runtime.ruby.is_empty()
                && c.runtime.node.is_empty()
                && c.runtime.python.is_empty()
                && c.runtime.go.is_empty()
                && c.runtime.rust.is_empty()
                && c.runtime.java.is_empty()
                && c.runtime.deno.is_empty()
                && c.runtime.bun.is_empty()
        })
        .unwrap_or(false);
    if python_mode && !cleanup {
        return cmd_remove_python(packages, dry_run, json);
    }
    if ruby_mode && !cleanup {
        return cmd_remove_ruby(packages, dry_run, json);
    }
    if rust_mode && !cleanup {
        return cmd_remove_rust(packages, dry_run, json);
    }
    if go_mode && !cleanup {
        return cmd_remove_go(packages, dry_run, json);
    }
    if java_mode && !cleanup {
        return cmd_remove_java(packages, dry_run, json);
    }
    if deno_mode && !cleanup {
        return cmd_remove_deno(packages, dry_run, json);
    }
    if bun_mode && !cleanup {
        return cmd_remove_bun(packages, dry_run, json);
    }

    // Handle cleanup mode separately
    if cleanup {
        return cmd_cleanup(json, verbose, dry_run, force, yes);
    }

    if packages.is_empty() {
        if json {
            println!("{{\"error\": \"No packages specified\"}}");
        } else {
            println!("\n  {} No packages specified", "[ERROR]".red());
            println!(
                "  {} Usage: ven remove <package> [package...] [FLAGS]",
                "[TIP]".cyan()
            );
            println!();
        }
        return Ok(());
    }

    if json {
        output_json_removal(packages, force, dry_run, verbose)?;
    } else if verbose {
        display_verbose_removal(packages, force, dry_run)?;
    } else if dry_run {
        display_dry_run(packages, force)?;
    } else {
        execute_batch_removal(packages, force, yes)?;
    }

    Ok(())
}

// Language-specific remove handlers live in `remove/languages.rs`.

/// Execute batch removal with dependency checking
fn execute_batch_removal(packages: &[String], force: bool, yes: bool) -> Result<()> {
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut skipped_count = 0;
    let mut results: Vec<(String, String, bool)> = Vec::new();

    for package in packages {
        println!("[INFO] Processing {}...", package.bold());

        // Check if package is installed
        if !is_package_installed_locally(package) {
            println!("  {} {} not installed", "[SKIP]".yellow(), package);
            skipped_count += 1;
            results.push((package.clone(), "not installed".to_string(), false));
            continue;
        }

        // Dependency check (unless force)
        if !force {
            let dependents =
                crate::intelligence::engine::DependencyIntelligenceService::list_npm_lockfile_dependents(
                    package,
                )?;
            if !dependents.is_empty() {
                println!(
                    "\n  {} {} packages depend on {}:",
                    "Warning:".yellow().bold(),
                    dependents.len(),
                    package.bold()
                );
                for (dep, ver) in &dependents {
                    println!(
                        "    {} {} {}  requires  {}",
                        "•".dimmed(),
                        dep.bold(),
                        ver.dimmed(),
                        package
                    );
                }
                println!();
                println!("  Removing {} may break these packages.", package);

                let auto_yes = yes || !crate::core::runtime_bin::stdin_is_interactive();
                if auto_yes {
                    println!(
                        "  Proceeding anyway{} since --yes/non-TTY mode is active.",
                        if yes {
                            " (--yes)"
                        } else {
                            " (non-interactive)"
                        }
                    );
                } else {
                    print!("  Remove anyway? [y/N]: ");
                    use std::io::{self, BufRead};
                    let stdin = io::stdin();
                    let answer = stdin
                        .lock()
                        .lines()
                        .next()
                        .and_then(|l| l.ok())
                        .unwrap_or_default();

                    if answer.trim().to_lowercase() != "y" {
                        println!("  Cancelled removal of {}.", package);
                        skipped_count += 1;
                        results.push((package.clone(), "cancelled".to_string(), false));
                        continue;
                    }
                }
                println!();
            }
        }

        // Execute removal
        let npm_result = npm_uninstall(package);

        // Always remove from ven.toml (regardless of npm success)
        if let Err(e) = remove_from_ven_toml(package) {
            println!(
                "  {} Warning: Failed to remove {} from ven.toml: {}",
                "[WARN]".yellow(),
                package,
                e
            );
        }

        match npm_result {
            Ok(_) => {
                println!(
                    "  {} {}",
                    "[OK]".green(),
                    format!("Removed {}", package.bold())
                );
                success_count += 1;
                results.push((package.clone(), "removed".to_string(), true));
            }
            Err(e) => {
                println!(
                    "  {} {} (but removed from ven.toml)",
                    "[WARN]".yellow(),
                    e.to_string()
                );
                fail_count += 1;
                results.push((package.clone(), format!("{} (ven.toml updated)", e), false));
            }
        }
    }

    // Print summary
    println!();
    println!("  {}", "Summary".bold().cyan());
    println!(
        "    {} {} package(s) processed",
        "Total:".dimmed(),
        packages.len()
    );
    println!(
        "    {} {}",
        "Removed:".dimmed(),
        success_count.to_string().green()
    );

    if fail_count > 0 {
        println!(
            "    {} {}",
            "Failed:".dimmed(),
            fail_count.to_string().red()
        );
    }
    if skipped_count > 0 {
        println!(
            "    {} {}",
            "Skipped:".dimmed(),
            skipped_count.to_string().yellow()
        );
    }

    if fail_count > 0 {
        println!();
        for (name, reason, success) in &results {
            if !success {
                println!("    {} {} ({})", "[FAIL]".red(), name, reason);
            }
        }
    }

    println!();
    Ok(())
}

/// Display dry-run preview without executing
fn display_dry_run(packages: &[String], force: bool) -> Result<()> {
    println!(
        "\n  {} {}",
        "ven remove".bold().cyan(),
        "[DRY RUN]".yellow()
    );
    println!(
        "  {} Preview mode - no changes will be made\n",
        "[INFO]".cyan()
    );

    let mut would_remove = Vec::new();
    let mut would_skip = Vec::new();
    let mut total_dependents = 0;

    for package in packages {
        if !is_package_installed_locally(package) {
            would_skip.push((package.clone(), "not installed".to_string()));
            continue;
        }

        let dependents = if force {
            vec![]
        } else {
            crate::intelligence::engine::DependencyIntelligenceService::list_npm_lockfile_dependents(package)?
        };
        total_dependents += dependents.len();

        let installed_version =
            get_installed_version(package).unwrap_or_else(|_| "unknown".to_string());
        would_remove.push((package.clone(), installed_version, dependents));
    }

    // Display what would be removed
    if !would_remove.is_empty() {
        println!(
            "  {} Packages that would be removed:",
            "[REMOVE]".green().bold()
        );
        for (pkg, version, dependents) in &would_remove {
            if dependents.is_empty() {
                println!(
                    "    {} {} ({})",
                    "[OK]".green(),
                    pkg.bold(),
                    version.dimmed()
                );
            } else {
                println!(
                    "    {} {} ({}) {} {}",
                    "[WARN]".yellow(),
                    pkg.bold(),
                    version.dimmed(),
                    format!("[{} dependents]", dependents.len()).yellow(),
                    "may break".dimmed()
                );
            }
        }
    }

    if !would_skip.is_empty() {
        println!();
        println!(
            "  {} Packages that would be skipped:",
            "[SKIP]".yellow().bold()
        );
        for (pkg, reason) in &would_skip {
            println!("    {} {} ({})", "[FAIL]".red(), pkg, reason);
        }
    }

    // Summary
    println!();
    println!("  {}", "Summary".bold().cyan());
    println!(
        "    {} {} package(s) would be removed",
        "Remove:".dimmed(),
        would_remove.len().to_string().green()
    );
    println!(
        "    {} {} package(s) would be skipped",
        "Skip:".dimmed(),
        would_skip.len().to_string().yellow()
    );

    if total_dependents > 0 && !force {
        println!(
            "    {} {} dependent package(s) may break",
            "Warning:".dimmed(),
            total_dependents.to_string().red()
        );
    }

    if force {
        println!();
        println!(
            "  {} Using --force: dependency checks bypassed",
            "[!]".yellow().bold()
        );
    }

    println!();
    Ok(())
}

/// Display verbose mode with detailed analysis
fn display_verbose_removal(packages: &[String], force: bool, dry_run: bool) -> Result<()> {
    println!(
        "\n  {} {}",
        "ven remove".bold().cyan(),
        "[VERBOSE]".yellow()
    );
    println!(
        "  {} Analyzing {} package(s)\n",
        "[INFO]".cyan(),
        packages.len()
    );

    let mut total_disk_space = 0u64;
    let mut removal_details = Vec::new();

    for package in packages {
        println!("{} Analyzing {}...", "[INFO]".cyan(), package.bold());

        if !is_package_installed_locally(package) {
            println!(
                "  {} {} not installed, skipping\n",
                "[SKIP]".yellow(),
                package
            );
            continue;
        }

        let installed_version = get_installed_version(package)?;
        let pkg_size = calculate_package_size(package)?;
        total_disk_space += pkg_size;

        // Find direct dependents
        let dependents = if force {
            vec![]
        } else {
            crate::intelligence::engine::DependencyIntelligenceService::list_npm_lockfile_dependents(package)?
        };

        // Find transitive dependencies (what this package depends on)
        let transitive_deps = get_transitive_dependencies(package)?;

        println!("  {} Version: {}", "Package:".dimmed(), installed_version);
        println!(
            "  {} Size: {}",
            "Disk Usage:".dimmed(),
            format_bytes(pkg_size)
        );
        println!(
            "  {} {} direct dependent(s)",
            "Dependents:".dimmed(),
            dependents.len()
        );
        println!(
            "  {} {} transitive dependency(ies)",
            "Dependencies:".dimmed(),
            transitive_deps.len()
        );

        if !dependents.is_empty() {
            println!("    {}", "Direct dependents:".dimmed());
            for (dep, ver) in &dependents {
                println!(
                    "      {} {} {}",
                    "•".dimmed(),
                    dep.bold(),
                    format!("({})", ver).dimmed()
                );
            }
        }

        if !transitive_deps.is_empty() {
            println!("    {}", "Transitive dependencies:".dimmed());
            for dep in transitive_deps.iter().take(5) {
                println!("      {} {}", "•".dimmed(), dep);
            }
            if transitive_deps.len() > 5 {
                println!(
                    "      {} ... and {} more",
                    "...".dimmed(),
                    transitive_deps.len() - 5
                );
            }
        }

        if dry_run {
            println!("  {} Would remove (dry run)", "[DRY RUN]".yellow());
        } else {
            println!("  {} Ready to remove", "[OK]".green());
        }

        removal_details.push((
            package.clone(),
            installed_version,
            pkg_size,
            dependents.len(),
        ));
        println!();
    }

    // Total impact summary
    println!("  {}", "Total Impact".bold().underline());
    println!(
        "    {} {} package(s) analyzed",
        "Analyzed:".dimmed(),
        packages.len().to_string().cyan()
    );
    println!(
        "    {} {} total disk space",
        "Disk Space:".dimmed(),
        format_bytes(total_disk_space)
    );

    let total_dependents: usize = removal_details.iter().map(|(_, _, _, deps)| deps).sum();
    if total_dependents > 0 {
        println!(
            "    {} {} dependent package(s) may be affected",
            "Warning:".dimmed(),
            total_dependents.to_string().red()
        );
    }

    println!();
    Ok(())
}

/// Output JSON format for scripting
fn output_json_removal(
    packages: &[String],
    force: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    use serde_json::json;

    let mut output = json!({
        "mode": if dry_run { "dry_run" } else { "execute" },
        "force": force,
        "packages_requested": packages
    });

    let mut results = Vec::new();

    for package in packages {
        let mut pkg_info = json!({
            "name": package,
            "installed": is_package_installed_locally(package)
        });

        if pkg_info["installed"].as_bool().unwrap_or(false) {
            let installed_version = get_installed_version(package).unwrap_or_default();
            let dependents = if force {
                vec![]
            } else {
                crate::intelligence::engine::DependencyIntelligenceService::list_npm_lockfile_dependents(package)?
            };

            pkg_info["installed_version"] = json!(installed_version);
            pkg_info["dependent_count"] = json!(dependents.len());

            if verbose {
                let pkg_size = calculate_package_size(package)?;
                let transitive_deps = get_transitive_dependencies(package)?;

                pkg_info["size_bytes"] = json!(pkg_size);
                pkg_info["transitive_dependencies"] = json!(transitive_deps.len());
            }

            if !dry_run {
                // Actually remove
                match npm_uninstall(package) {
                    Ok(_) => {
                        pkg_info["status"] = json!("removed");
                        pkg_info["success"] = json!(true);
                    }
                    Err(e) => {
                        pkg_info["status"] = json!("failed");
                        pkg_info["error"] = json!(e.to_string());
                        pkg_info["success"] = json!(false);
                    }
                }
            } else {
                pkg_info["status"] = json!("would_remove");
                pkg_info["success"] = json!(null);
            }
        } else {
            pkg_info["status"] = json!("not_installed");
            pkg_info["success"] = json!(false);
        }

        results.push(pkg_info);
    }

    output["results"] = json!(results);

    let success_count = results
        .iter()
        .filter(|r| r["success"].as_bool().unwrap_or(false))
        .count();

    output["summary"] = json!({
        "total": packages.len(),
        "success": success_count,
        "failed": results.len() - success_count
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Orphan cleanup mode
fn cmd_cleanup(json: bool, verbose: bool, dry_run: bool, force: bool, yes: bool) -> Result<()> {
    println!(
        "\n  {} {}",
        "ven remove".bold().cyan(),
        "[CLEANUP]".yellow()
    );
    println!("  {} Finding orphaned dependencies...\n", "[INFO]".cyan());

    let orphans = find_orphaned_packages()?;

    if orphans.is_empty() {
        if json {
            println!("{{\"orphans_found\": 0, \"message\": \"No orphaned packages found\"}}");
        } else {
            println!("  {} No orphaned packages found", "[OK]".green());
            println!(
                "  {} All installed packages are required by dependencies",
                "[OK]".green()
            );
            println!();
        }
        return Ok(());
    }

    if json {
        output_json_cleanup(&orphans, dry_run, force)?;
    } else {
        display_cleanup_results(&orphans, verbose, dry_run, force, yes)?;
    }

    Ok(())
}

/// Find packages that are installed but not required by anything
fn find_orphaned_packages() -> Result<Vec<(String, String, u64)>> {
    let lock_path = std::env::current_dir()?.join("package-lock.json");

    if !lock_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&lock_path)?;
    let lock: serde_json::Value = serde_json::from_str(&content)?;

    let mut orphans = Vec::new();
    let mut all_packages = HashSet::new();
    let mut required_packages = HashSet::new();

    // Collect all packages from lock file
    if let Some(packages) = lock["packages"].as_object() {
        for name in packages.keys() {
            if name.is_empty() {
                continue;
            }
            let clean_name = name.trim_start_matches("node_modules/");
            all_packages.insert(clean_name.to_string());
        }
    }

    // Find which packages are required (in dependencies or devDependencies)
    if let Some(deps) = lock["packages"][""]["dependencies"].as_object() {
        for name in deps.keys() {
            required_packages.insert(name.clone());
        }
    }

    if let Some(deps) = lock["packages"][""]["devDependencies"].as_object() {
        for name in deps.keys() {
            required_packages.insert(name.clone());
        }
    }

    // Find transitive dependencies
    if let Some(packages) = lock["packages"].as_object() {
        for (_, info) in packages {
            if let Some(deps) = info["dependencies"].as_object() {
                for name in deps.keys() {
                    required_packages.insert(name.clone());
                }
            }
        }
    }

    // Orphans = installed but not required by anything
    for pkg in &all_packages {
        if !required_packages.contains(pkg) {
            if let Ok(version) = get_installed_version(pkg) {
                let size = calculate_package_size(pkg).unwrap_or(0);
                orphans.push((pkg.clone(), version, size));
            }
        }
    }

    Ok(orphans)
}

/// Display cleanup results to user
fn display_cleanup_results(
    orphans: &[(String, String, u64)],
    _verbose: bool,
    dry_run: bool,
    force: bool,
    yes: bool,
) -> Result<()> {
    println!(
        "  {} Found {} orphaned package(s):\n",
        "[FOUND]".yellow().bold(),
        orphans.len()
    );

    let mut total_size = 0u64;

    for (pkg, version, size) in orphans {
        total_size += size;
        println!(
            "    {} {}@{} ({})",
            "[ORPHAN]".yellow(),
            pkg.bold(),
            version,
            format_bytes(*size)
        );
    }

    println!();
    println!(
        "  {} Total orphaned disk space: {}",
        "Total:".dimmed(),
        format_bytes(total_size)
    );

    if dry_run {
        println!();
        println!("  {} Dry run - no packages removed", "[DRY RUN]".yellow());
    } else {
        println!();

        if !force {
            let auto_yes = yes || !crate::core::runtime_bin::stdin_is_interactive();
            if auto_yes {
                println!(
                    "  Proceeding with cleanup{}.",
                    if yes {
                        " (--yes)"
                    } else {
                        " (non-interactive)"
                    }
                );
            } else {
                print!("  Remove all orphaned packages? [y/N]: ");
                use std::io::{self, BufRead};
                let stdin = io::stdin();
                let answer = stdin
                    .lock()
                    .lines()
                    .next()
                    .and_then(|l| l.ok())
                    .unwrap_or_default();

                if answer.trim().to_lowercase() != "y" {
                    println!("  Cancelled.");
                    return Ok(());
                }
            }
        }

        // Remove orphans
        let mut success_count = 0;
        for (pkg, _, _) in orphans {
            match npm_uninstall(pkg) {
                Ok(_) => {
                    println!("  {} Removed {}", "[OK]".green(), pkg);
                    success_count += 1;
                }
                Err(e) => {
                    println!("  {} Failed to remove {}: {}", "[ERROR]".red(), pkg, e);
                }
            }
        }

        println!();
        println!(
            "  {} {}/{} orphaned packages removed",
            "[OK]".green(),
            success_count,
            orphans.len()
        );
    }

    println!();
    Ok(())
}

/// Output cleanup results as JSON
fn output_json_cleanup(
    orphans: &[(String, String, u64)],
    dry_run: bool,
    force: bool,
) -> Result<()> {
    use serde_json::json;

    let mut output = json!({
        "mode": "cleanup",
        "dry_run": dry_run,
        "orphans_found": orphans.len()
    });

    let mut orphan_list = Vec::new();
    for (pkg, version, size) in orphans {
        orphan_list.push(json!({
            "name": pkg,
            "version": version,
            "size_bytes": size
        }));
    }

    output["orphans"] = json!(orphan_list);

    let total_size: u64 = orphans.iter().map(|(_, _, size)| size).sum();
    output["total_size_bytes"] = json!(total_size);

    if !dry_run {
        let mut removed = Vec::new();
        for (pkg, _, _) in orphans {
            if force {
                match npm_uninstall(pkg) {
                    Ok(_) => removed.push(pkg.clone()),
                    Err(_) => {}
                }
            }
        }
        output["removed"] = json!(removed);
    }

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// ── Helper Functions ─────────────────────────────────────────────────

/// Remove a package from ven.toml
fn remove_from_ven_toml(package: &str) -> Result<()> {
    use std::fs;

    let cwd = std::env::current_dir()?;

    // Find ven.toml in current directory (don't search parents)
    let ven_toml_path = cwd.join("ven.toml");

    if !ven_toml_path.exists() {
        return Ok(()); // No ven.toml, nothing to update
    }

    // Read and parse the TOML file
    let content = fs::read_to_string(&ven_toml_path)?;
    let mut doc = content.parse::<toml_edit::DocumentMut>()?;

    // Check if package exists in [packages] section
    if let Some(packages) = doc.get_mut("packages") {
        if let Some(packages_table) = packages.as_table_mut() {
            if packages_table.contains_key(package) {
                packages_table.remove(package);

                // Write back to file
                fs::write(&ven_toml_path, doc.to_string())?;
            }
        }
    }

    Ok(())
}

/// Check if a package is installed locally
fn is_package_installed_locally(package: &str) -> bool {
    let pkg_json = std::env::current_dir()
        .unwrap_or_default()
        .join("node_modules")
        .join(package)
        .join("package.json");

    pkg_json.exists()
}

/// Calculate total size of a package directory
fn calculate_package_size(package: &str) -> Result<u64> {
    let pkg_path = std::env::current_dir()?.join("node_modules").join(package);

    if !pkg_path.exists() {
        return Ok(0);
    }

    let mut total_size = 0;

    for entry in std::fs::read_dir(&pkg_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            total_size += entry.metadata()?.len();
        } else if path.is_dir() {
            total_size += calculate_dir_size_recursive(&path)?;
        }
    }

    Ok(total_size)
}

/// Recursively calculate directory size
fn calculate_dir_size_recursive(path: &Path) -> Result<u64> {
    let mut total_size = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            total_size += entry.metadata()?.len();
        } else if path.is_dir() {
            total_size += calculate_dir_size_recursive(&path)?;
        }
    }

    Ok(total_size)
}

/// Get transitive dependencies of a package from lock file
fn get_transitive_dependencies(package: &str) -> Result<Vec<String>> {
    let lock_path = std::env::current_dir()?.join("package-lock.json");

    if !lock_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&lock_path)?;
    let lock: serde_json::Value = serde_json::from_str(&content)?;

    let mut deps = Vec::new();
    let pkg_key = format!("node_modules/{}", package);

    if let Some(deps_obj) = lock["packages"][&pkg_key]["dependencies"].as_object() {
        for dep_name in deps_obj.keys() {
            deps.push(dep_name.clone());
        }
    }

    Ok(deps)
}

/// Format bytes into human-readable string
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
