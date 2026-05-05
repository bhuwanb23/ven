use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::add::update_ven_toml_packages;

pub(super) fn cmd_upgrade_python(
    packages: &[String],
    apply: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
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

pub(super) fn cmd_upgrade_rust(
    packages: &[String],
    apply: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
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

pub(super) fn cmd_upgrade_java_notice(
    packages: &[String],
    apply: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"java",
                "apply": apply,
                "dry_run": dry_run,
                "packages": packages,
                "message":"Use Maven/Gradle for Java dependency upgrades"
            }))?
        );
        return Ok(());
    }
    println!("\n  {}", "ven upgrade (java)".bold().cyan());
    println!(
        "  {} Java upgrades are managed by Maven/Gradle.",
        "[INFO]".cyan()
    );
    println!("  {} No direct package upgrade performed by ven.", "[TIP]".cyan());
    println!();
    Ok(())
}

pub(super) fn cmd_upgrade_deno_notice(
    packages: &[String],
    apply: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"deno",
                "apply": apply,
                "dry_run": dry_run,
                "packages": packages,
                "message":"Deno manages dependencies via imports/deno.json; ven does not upgrade packages"
            }))?
        );
        return Ok(());
    }
    println!("\n  {}", "ven upgrade (deno)".bold().cyan());
    println!(
        "  {} Deno dependencies are managed by imports/deno.json (and optionally deno.lock).",
        "[INFO]".cyan()
    );
    println!("  {} No direct package upgrade performed by ven.", "[TIP]".cyan());
    println!();
    Ok(())
}
