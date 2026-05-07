use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::core::ruby_gems;

use super::remove_from_ven_toml;

pub(super) fn cmd_remove_ruby(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
    if packages.is_empty() {
        if json {
            println!("{{\"error\":\"No packages specified\"}}");
        } else {
            println!("  {} No packages specified", "[ERROR]".red());
        }
        return Ok(());
    }
    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "mode":"ruby_dry_run",
                    "packages": packages
                }))?
            );
        } else {
            println!(
                "\n  {} {}",
                "ven remove".bold().cyan(),
                "[RUBY DRY RUN]".yellow()
            );
            for pkg in packages {
                println!("  {} Would gem uninstall {}", "[PREVIEW]".cyan(), pkg.bold());
            }
            println!();
        }
        return Ok(());
    }

    let mut removed: Vec<String> = Vec::new();
    for pkg in packages {
        match ruby_gems::gem_uninstall_all(pkg) {
            Ok(()) => {
                println!("  {} Removed {}", "[OK]".green(), pkg.bold());
                removed.push(pkg.clone());
                let _ = remove_from_ven_toml(pkg);
            }
            Err(_) => println!("  {} Failed to remove {} (maybe not installed)", "[WARN]".yellow(), pkg),
        }
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"ruby",
                "removed": removed
            }))?
        );
    }
    println!();
    Ok(())
}

pub(super) fn cmd_remove_python(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
    if packages.is_empty() {
        if json {
            println!("{{\"error\":\"No packages specified\"}}");
        } else {
            println!("  {} No packages specified", "[ERROR]".red());
        }
        return Ok(());
    }
    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "mode":"python_dry_run",
                    "packages": packages
                }))?
            );
        } else {
            println!(
                "\n  {} {}",
                "ven remove".bold().cyan(),
                "[PYTHON DRY RUN]".yellow()
            );
            for pkg in packages {
                println!("  {} Would remove {}", "[PREVIEW]".cyan(), pkg.bold());
            }
            println!();
        }
        return Ok(());
    }

    let python = resolve_python_cmd();
    let mut removed: Vec<String> = Vec::new();
    for pkg in packages {
        let status = Command::new(&python)
            .args(["-m", "pip", "uninstall", "-y", pkg])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} Removed {}", "[OK]".green(), pkg.bold());
                removed.push(pkg.clone());
                let _ = remove_from_ven_toml(pkg);
            }
            Ok(_) => println!("  {} Failed to remove {}", "[WARN]".yellow(), pkg),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"python",
                "removed": removed
            }))?
        );
    }
    println!();
    Ok(())
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

pub(super) fn cmd_remove_rust(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
    if packages.is_empty() {
        if json {
            println!("{{\"error\":\"No packages specified\"}}");
        } else {
            println!("  {} No packages specified", "[ERROR]".red());
        }
        return Ok(());
    }
    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "mode":"rust_dry_run",
                    "packages": packages
                }))?
            );
        } else {
            println!(
                "\n  {} {}",
                "ven remove".bold().cyan(),
                "[RUST DRY RUN]".yellow()
            );
            for pkg in packages {
                println!("  {} Would remove {}", "[PREVIEW]".cyan(), pkg.bold());
            }
            println!();
        }
        return Ok(());
    }
    let mut removed: Vec<String> = Vec::new();
    for pkg in packages {
        let status = std::process::Command::new("cargo")
            .args(["remove", pkg])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} Removed {}", "[OK]".green(), pkg.bold());
                removed.push(pkg.clone());
                let _ = remove_from_ven_toml(pkg);
            }
            Ok(_) => println!("  {} Failed to remove {}", "[WARN]".yellow(), pkg),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"rust",
                "removed": removed
            }))?
        );
    }
    println!();
    Ok(())
}

pub(super) fn cmd_remove_java_notice(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"java",
                "dry_run": dry_run,
                "packages": packages,
                "message":"Use Maven/Gradle to manage Java dependencies"
            }))?
        );
        return Ok(());
    }
    println!("\n  {}", "ven remove (java)".bold().cyan());
    println!(
        "  {} Java dependency removal is managed by Maven/Gradle.",
        "[INFO]".cyan()
    );
    println!("  {} No direct removal performed by ven.", "[TIP]".cyan());
    println!();
    Ok(())
}

pub(super) fn cmd_remove_deno_notice(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"deno",
                "dry_run": dry_run,
                "packages": packages,
                "message":"Deno manages dependencies via imports/deno.json; ven does not remove packages"
            }))?
        );
        return Ok(());
    }
    println!("\n  {}", "ven remove (deno)".bold().cyan());
    println!(
        "  {} Deno dependencies are managed by imports/deno.json (and optionally deno.lock).",
        "[INFO]".cyan()
    );
    println!("  {} No direct removal performed by ven.", "[TIP]".cyan());
    println!();
    Ok(())
}
