use crate::cli::add::update_ven_toml_packages;
use crate::core::{load_config, packages::*};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
/// Semantic upgrade type classification
#[derive(Debug)]
enum UpgradeType {
    Major,
    Minor,
    Patch,
    New,
    Error,
}

impl UpgradeType {
    fn to_string(&self) -> String {
        match self {
            UpgradeType::Major => "MAJOR".to_string(),
            UpgradeType::Minor => "MINOR".to_string(),
            UpgradeType::Patch => "PATCH".to_string(),
            UpgradeType::New => "NEW".to_string(),
            UpgradeType::Error => "ERROR".to_string(),
        }
    }

    fn colored(&self) -> colored::ColoredString {
        match self {
            UpgradeType::Major => "MAJOR".red().bold(),
            UpgradeType::Minor => "MINOR".yellow().bold(),
            UpgradeType::Patch => "PATCH".green().bold(),
            UpgradeType::New => "NEW".cyan().bold(),
            UpgradeType::Error => "ERROR".red().bold(),
        }
    }
}

/// Upgrade packages with batch support and advanced features
pub fn cmd_upgrade(
    packages: &[String],
    apply: bool,
    dry_run: bool,
    json: bool,
    verbose: bool,
    all: bool,
    force: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;
    let python_mode = !cfg.runtime.python.is_empty() && cfg.runtime.node.is_empty();
    let rust_mode = !cfg.runtime.rust.is_empty()
        && cfg.runtime.node.is_empty()
        && cfg.runtime.python.is_empty()
        && cfg.runtime.go.is_empty();

    // Handle --all flag: get all packages from ven.toml
    let target_packages = if all {
        cfg.packages.keys().cloned().collect::<Vec<String>>()
    } else if packages.is_empty() {
        if json {
            println!("{{\"error\": \"No packages specified\"}}");
        } else {
            println!("\n  {} No packages specified", "[ERROR]".red());
            println!(
                "  {} Usage: ven upgrade <package> [package...] [FLAGS]",
                "[TIP]".cyan()
            );
            println!("  {} Or: ven upgrade --all", "[TIP]".cyan());
            println!();
        }
        return Ok(());
    } else {
        packages.to_vec()
    };

    if python_mode {
        return cmd_upgrade_python(&target_packages, apply, dry_run, json);
    }
    if rust_mode {
        return cmd_upgrade_rust(&target_packages, apply, dry_run, json);
    }

    if json {
        output_json_upgrade(&target_packages, apply, dry_run, verbose)?;
    } else if verbose {
        display_verbose_upgrade(&target_packages, apply, dry_run, force)?;
    } else if dry_run {
        display_dry_run_upgrade(&target_packages, apply)?;
    } else {
        execute_upgrade(&target_packages, apply, force)?;
    }

    Ok(())
}

fn cmd_upgrade_python(packages: &[String], apply: bool, dry_run: bool, json: bool) -> Result<()> {
    let python = resolve_python_cmd();
    let outdated = get_outdated_python_packages(&python)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"python",
                "apply": apply,
                "dry_run": dry_run,
                "packages": packages,
                "outdated": outdated
            }))?
        );
    } else {
        println!("\n  {}", "ven upgrade (python)".bold().cyan());
    }

    for pkg in packages {
        let target = outdated
            .iter()
            .find(|(name, _, _)| name == pkg)
            .map(|(_, current, latest)| (current.clone(), latest.clone()));

        match target {
            Some((current, latest)) => {
                if dry_run || !apply {
                    if !json {
                        println!(
                            "  {} {} {} -> {}",
                            "[UPGRADE]".yellow(),
                            pkg.bold(),
                            current,
                            latest.green()
                        );
                    }
                    continue;
                }
                let status = Command::new(&python)
                    .args(["-m", "pip", "install", "--upgrade", pkg])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        if !json {
                            println!(
                                "  {} Upgraded {} to {}",
                                "[OK]".green(),
                                pkg.bold(),
                                latest.green()
                            );
                        }
                        let _ = update_ven_toml_packages(&[(pkg.clone(), format!(">={}", latest))]);
                    }
                    Ok(_) => {
                        if !json {
                            println!("  {} Failed to upgrade {}", "[ERROR]".red(), pkg);
                        }
                    }
                    Err(e) => {
                        if !json {
                            println!("  {} {}", "[ERROR]".red(), e);
                        }
                    }
                }
            }
            None => {
                if !json {
                    println!("  {} {} is up to date", "✓".green(), pkg.bold());
                }
            }
        }
    }
    if !json {
        println!();
    }
    Ok(())
}

fn get_outdated_python_packages(python: &PathBuf) -> Result<Vec<(String, String, String)>> {
    let out = Command::new(python)
        .args(["-m", "pip", "list", "--outdated", "--format=json"])
        .output()?;
    if !out.status.success() {
        return Ok(Vec::new());
    }
    let parsed: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let mut items = Vec::new();
    if let Some(arr) = parsed.as_array() {
        for item in arr {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let version = item
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let latest = item
                .get("latest_version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                items.push((name, version, latest));
            }
        }
    }
    Ok(items)
}

fn resolve_python_cmd() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
            let p = PathBuf::from(venv).join("Scripts").join("python.exe");
            if p.is_file() {
                return p;
            }
        }
        if let Ok(ver) = std::env::var("VEN_PYTHON_VERSION") {
            if let Some(home) = dirs::home_dir() {
                let p = home
                    .join(".ven")
                    .join("python")
                    .join(ver)
                    .join("python.exe");
                if p.is_file() {
                    return p;
                }
            }
        }
        PathBuf::from("python")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("python3")
    }
}

fn cmd_upgrade_rust(packages: &[String], apply: bool, dry_run: bool, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"rust",
                "apply": apply,
                "dry_run": dry_run,
                "packages": packages
            }))?
        );
    } else {
        println!("\n  {}", "ven upgrade (rust)".bold().cyan());
    }

    for pkg in packages {
        if dry_run || !apply {
            if !json {
                println!("  {} cargo update -p {}", "[PREVIEW]".cyan(), pkg.bold());
            }
            continue;
        }
        let status = std::process::Command::new("cargo")
            .args(["update", "-p", pkg])
            .status();
        match status {
            Ok(s) if s.success() => {
                if !json {
                    println!("  {} Updated {}", "[OK]".green(), pkg.bold());
                }
            }
            Ok(_) => {
                if !json {
                    println!("  {} Failed to update {}", "[WARN]".yellow(), pkg);
                }
            }
            Err(e) => {
                if !json {
                    println!("  {} {}", "[ERROR]".red(), e);
                }
            }
        }
    }
    if !json {
        println!();
    }
    Ok(())
}

/// Execute batch upgrade with preview/apply support
fn execute_upgrade(packages: &[String], apply: bool, force: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    let mut up_to_date_count = 0;
    let mut upgraded_count = 0;
    let mut failed_count = 0;
    let mut results: Vec<(String, String, String, bool)> = Vec::new();

    for package in packages {
        println!("{} Processing {}...", "→".cyan(), package.bold());

        match process_single_upgrade(package, &node_version, apply, force) {
            Ok((current, latest, upgraded)) => {
                if current == latest {
                    up_to_date_count += 1;
                    println!(
                        "  {} {} is already up to date ({})",
                        "✓".green(),
                        package.bold(),
                        latest
                    );
                    results.push((package.clone(), current, latest, true));
                } else if upgraded {
                    upgraded_count += 1;
                    results.push((package.clone(), current, latest, true));
                } else {
                    // Preview mode, not applied
                    results.push((package.clone(), current, latest, false));
                }
            }
            Err(e) => {
                println!("  {} {}", "[ERROR]".red(), e.to_string().red());
                failed_count += 1;
                results.push((package.clone(), "error".to_string(), String::new(), false));
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
        "Up to date:".dimmed(),
        up_to_date_count.to_string().green()
    );

    if apply {
        println!(
            "    {} {}",
            "Upgraded:".dimmed(),
            upgraded_count.to_string().cyan()
        );
    } else {
        println!(
            "    {} {}",
            "Available upgrades:".dimmed(),
            (packages.len() - up_to_date_count).to_string().yellow()
        );
    }

    if failed_count > 0 {
        println!(
            "    {} {}",
            "Failed:".dimmed(),
            failed_count.to_string().red()
        );
        println!();
        for (name, current, _, success) in &results {
            if !success && current == "error" {
                println!("    {} {}", "[FAIL]".red(), name);
            }
        }
    }

    if !apply && (packages.len() - up_to_date_count) > 0 {
        println!();
        println!(
            "  {} Run  {}  to apply upgrades",
            "[TIP]".cyan(),
            "ven upgrade --all --apply".bold()
        );
    }

    println!();
    Ok(())
}

/// Process a single package upgrade
fn process_single_upgrade(
    package: &str,
    node_version: &str,
    apply: bool,
    force: bool,
) -> Result<(String, String, bool)> {
    // Get currently installed version
    let current_ver = get_installed_version(package).unwrap_or_else(|_| "unknown".to_string());

    // Fetch latest compatible version
    let info = fetch_npm_info(package)?;
    let latest = find_compatible_version(&info, node_version)
        .ok_or_else(|| anyhow::anyhow!("No compatible version found"))?;

    // Check if upgrade is needed
    if current_ver == latest {
        return Ok((current_ver, latest, false));
    }

    // Determine upgrade type
    let upgrade_type = classify_upgrade(&current_ver, &latest);

    if current_ver != "unknown" {
        println!(
            "  {} {}  →  {}  ({})",
            package.bold(),
            current_ver.dimmed(),
            latest.green(),
            upgrade_type.colored()
        );
    } else {
        println!(
            "  {}  →  {}  ({})",
            package.bold(),
            latest.green(),
            upgrade_type.colored()
        );
    }

    // Apply upgrade if requested
    if apply {
        if !force && is_major_upgrade(&current_ver, &latest) {
            print!(
                "\n  {} Major upgrade detected. Continue? [y/N]: ",
                "[!]".yellow().bold()
            );
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let answer = stdin
                .lock()
                .lines()
                .next()
                .and_then(|l| l.ok())
                .unwrap_or_default();

            if answer.trim().to_lowercase() != "y" {
                println!("  Skipped {}.", package);
                return Ok((current_ver, latest, false));
            }
            println!();
        }

        npm_install(package, &latest)?;

        let packages_vec = [(package.to_string(), latest.clone())];
        update_ven_toml_packages(&packages_vec)?;

        println!("  {} Upgraded to {}", "✓".green(), latest);
        return Ok((current_ver, latest, true));
    }

    Ok((current_ver, latest, false))
}

/// Display dry-run preview without executing
fn display_dry_run_upgrade(packages: &[String], apply: bool) -> Result<()> {
    println!(
        "\n  {} {}",
        "ven upgrade".bold().cyan(),
        "[DRY RUN]".yellow()
    );
    println!(
        "  {} Preview mode - no changes will be made\n",
        "[INFO]".cyan()
    );

    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    let mut up_to_date = Vec::new();
    let mut can_upgrade = Vec::new();
    let mut failed = Vec::new();

    for package in packages {
        let current_ver = get_installed_version(package).unwrap_or_else(|_| "unknown".to_string());

        match fetch_npm_info(package) {
            Ok(info) => {
                if let Some(latest) = find_compatible_version(&info, &node_version) {
                    if current_ver == latest {
                        up_to_date.push((package.to_string(), latest));
                    } else {
                        let upgrade_type = classify_upgrade(&current_ver, &latest);
                        can_upgrade.push((package.to_string(), current_ver, latest, upgrade_type));
                    }
                } else {
                    failed.push((package.to_string(), "No compatible version".to_string()));
                }
            }
            Err(e) => {
                failed.push((package.to_string(), e.to_string()));
            }
        }
    }

    // Display can upgrade
    if !can_upgrade.is_empty() {
        println!("  {} Packages that can be upgraded:", "→".green().bold());
        for (pkg, _current, latest, upgrade_type) in &can_upgrade {
            println!(
                "    {} {} → {} ({})",
                "↑".yellow(),
                pkg.bold(),
                latest.green(),
                upgrade_type.colored()
            );
        }
    }

    // Display up to date
    if !up_to_date.is_empty() {
        println!();
        println!("  {} Packages already up to date:", "✓".green().bold());
        for (pkg, version) in &up_to_date {
            println!("    {} {} ({})", "✓".green(), pkg, version.dimmed());
        }
    }

    // Display failed
    if !failed.is_empty() {
        println!();
        println!("  {} Packages with errors:", "✗".red().bold());
        for (pkg, error) in &failed {
            println!("    {} {} ({})", "✗".red(), pkg, error);
        }
    }

    // Summary
    println!();
    println!("  {}", "Summary".bold().cyan());
    println!(
        "    {} {} package(s) analyzed",
        "Total:".dimmed(),
        packages.len()
    );
    println!(
        "    {} {} can be upgraded",
        "Upgrade:".dimmed(),
        can_upgrade.len().to_string().yellow()
    );
    println!(
        "    {} {} already up to date",
        "Current:".dimmed(),
        up_to_date.len().to_string().green()
    );

    if !failed.is_empty() {
        println!(
            "    {} {} had errors",
            "Failed:".dimmed(),
            failed.len().to_string().red()
        );
    }

    if !can_upgrade.is_empty() && !apply {
        println!();
        println!(
            "  {} Run  {}  to apply",
            "[TIP]".cyan(),
            "ven upgrade --apply".bold()
        );
    }

    println!();
    Ok(())
}

/// Display verbose mode with detailed analysis
fn display_verbose_upgrade(
    packages: &[String],
    apply: bool,
    dry_run: bool,
    _force: bool,
) -> Result<()> {
    println!(
        "\n  {} {}",
        "ven upgrade".bold().cyan(),
        "[VERBOSE]".yellow()
    );
    println!(
        "  {} Analyzing {} package(s)\n",
        "[INFO]".cyan(),
        packages.len()
    );

    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    let mut upgrade_details = Vec::new();

    for package in packages {
        println!("{} Analyzing {}...", "→".cyan(), package.bold());

        let current_ver = get_installed_version(package).unwrap_or_else(|_| "unknown".to_string());

        match fetch_npm_info(package) {
            Ok(info) => {
                if let Some(latest) = find_compatible_version(&info, &node_version) {
                    let upgrade_type = classify_upgrade(&current_ver, &latest);

                    println!(
                        "  {} Current version: {}",
                        "Current:".dimmed(),
                        if current_ver == "unknown" {
                            "not installed".to_string()
                        } else {
                            current_ver.clone()
                        }
                    );
                    println!("  {} Latest compatible: {}", "Latest:".dimmed(), latest);
                    println!(
                        "  {} Upgrade type: {}",
                        "Type:".dimmed(),
                        upgrade_type.colored()
                    );
                    println!(
                        "  {} Node {} {}",
                        "Compatibility:".dimmed(),
                        node_version,
                        "✓".green()
                    );

                    // Calculate disk space if installed
                    if current_ver != "unknown" {
                        let current_size = calculate_package_size(package).unwrap_or(0);
                        println!(
                            "  {} Current size: {}",
                            "Disk Usage:".dimmed(),
                            format_bytes(current_size)
                        );
                    }

                    // Show changelog hint
                    if current_ver != latest && current_ver != "unknown" {
                        let changelog_url =
                            format!("https://npmjs.com/package/{}/v/{}", package, latest);
                        println!("  {} {}", "Changelog:".dimmed(), changelog_url);
                    }

                    if dry_run {
                        println!("  {} Would upgrade (dry run)", "[DRY RUN]".yellow());
                    } else if apply {
                        println!("  {} Ready to upgrade", "[READY]".green());
                    } else {
                        println!("  {} Preview mode", "[PREVIEW]".cyan());
                    }

                    upgrade_details.push((
                        package.to_string(),
                        current_ver,
                        latest,
                        upgrade_type,
                        true,
                    ));
                } else {
                    println!("  {} No compatible version found", "[ERROR]".red());
                    upgrade_details.push((
                        package.to_string(),
                        current_ver,
                        "N/A".to_string(),
                        UpgradeType::Error,
                        false,
                    ));
                }
            }
            Err(e) => {
                println!("  {} {}", "[ERROR]".red(), e.to_string());
                upgrade_details.push((
                    package.to_string(),
                    current_ver,
                    "error".to_string(),
                    UpgradeType::Error,
                    false,
                ));
            }
        }

        println!();
    }

    // Total impact summary
    println!("  {}", "Total Impact".bold().underline());
    println!(
        "    {} {} package(s) analyzed",
        "Analyzed:".dimmed(),
        packages.len().to_string().cyan()
    );

    let can_upgrade = upgrade_details
        .iter()
        .filter(|(_, c, l, _, _)| c != l && c != "error" && c != "unknown")
        .count();
    println!(
        "    {} {} package(s) can be upgraded",
        "Available:".dimmed(),
        can_upgrade.to_string().yellow()
    );

    println!();
    Ok(())
}

/// Output JSON format for scripting
fn output_json_upgrade(
    packages: &[String],
    apply: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    use serde_json::json;

    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    let mut output = json!({
        "mode": if dry_run { "dry_run" } else if apply { "apply" } else { "preview" },
        "node_version": node_version,
        "packages_requested": packages
    });

    let mut results = Vec::new();

    for package in packages {
        let current_ver = get_installed_version(package).unwrap_or_else(|_| "unknown".to_string());

        let mut pkg_info = json!({
            "name": package,
            "current_version": current_ver
        });

        match fetch_npm_info(package) {
            Ok(info) => {
                if let Some(latest) = find_compatible_version(&info, &node_version) {
                    let upgrade_type = classify_upgrade(&current_ver, &latest);

                    pkg_info["latest_version"] = json!(latest);
                    pkg_info["upgrade_type"] = json!(upgrade_type.to_string());
                    pkg_info["upgrade_available"] = json!(current_ver != latest);

                    if verbose {
                        let changelog_url =
                            format!("https://npmjs.com/package/{}/v/{}", package, latest);
                        pkg_info["changelog_url"] = json!(changelog_url);
                    }

                    if apply && current_ver != latest && !dry_run {
                        match npm_install(package, &latest) {
                            Ok(_) => {
                                pkg_info["status"] = json!("upgraded");
                                pkg_info["success"] = json!(true);

                                let packages_vec = [(package.to_string(), latest.clone())];
                                let _ = update_ven_toml_packages(&packages_vec);
                            }
                            Err(e) => {
                                pkg_info["status"] = json!("failed");
                                pkg_info["error"] = json!(e.to_string());
                                pkg_info["success"] = json!(false);
                            }
                        }
                    } else {
                        pkg_info["status"] = json!(if current_ver == latest {
                            "up_to_date"
                        } else {
                            "available"
                        });
                        pkg_info["success"] = json!(null);
                    }
                } else {
                    pkg_info["status"] = json!("no_compatible_version");
                    pkg_info["success"] = json!(false);
                }
            }
            Err(e) => {
                pkg_info["status"] = json!("error");
                pkg_info["error"] = json!(e.to_string());
                pkg_info["success"] = json!(false);
            }
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
        "upgraded": success_count,
        "available": results.iter().filter(|r| r["upgrade_available"].as_bool().unwrap_or(false)).count()
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// ── Helper Functions ─────────────────────────────────────────────────

/// Classify the type of upgrade based on version difference
fn classify_upgrade(current: &str, latest: &str) -> UpgradeType {
    if current == "unknown" {
        return UpgradeType::New;
    }

    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    if current_parts.len() < 3 || latest_parts.len() < 3 {
        return UpgradeType::Error;
    }

    if current_parts[0] != latest_parts[0] {
        UpgradeType::Major
    } else if current_parts[1] != latest_parts[1] {
        UpgradeType::Minor
    } else {
        UpgradeType::Patch
    }
}

/// Check if upgrade is a major version change
fn is_major_upgrade(current: &str, latest: &str) -> bool {
    if current == "unknown" {
        return false;
    }

    let current_major = current
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let latest_major = latest
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    current_major != latest_major
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
