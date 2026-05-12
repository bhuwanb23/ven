use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::core::deno_imports::{self, DenoManifest};
use crate::core::gemfile::Gemfile;
use crate::core::java_manifest::{self, JavaCoord};
use crate::core::requirements::Requirements;
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
                println!(
                    "  {} Would gem uninstall {}",
                    "[PREVIEW]".cyan(),
                    pkg.bold()
                );
            }
            println!();
        }
        return Ok(());
    }

    let cwd = std::env::current_dir()?;
    let gemfile_path = Gemfile::path_for(&cwd);
    let use_bundler = gemfile_path.is_file() && which_bundle();

    let mut removed: Vec<String> = Vec::new();
    for pkg in packages {
        let ok = if use_bundler {
            run_bundle_remove(pkg)
        } else {
            ruby_gems::gem_uninstall_all(pkg).map(|_| ())
        };
        match ok {
            Ok(()) => {
                println!("  {} Removed {}", "[OK]".green(), pkg.bold());
                removed.push(pkg.clone());
                let _ = remove_from_ven_toml(pkg);
                if !use_bundler {
                    if let Ok(mut gf) = Gemfile::load_or_default(&cwd) {
                        if gf.exists() && gf.remove(pkg) {
                            let _ = gf.write();
                        }
                    }
                }
            }
            Err(_) => println!(
                "  {} Failed to remove {} (maybe not installed)",
                "[WARN]".yellow(),
                pkg
            ),
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

fn which_bundle() -> bool {
    Command::new("bundle")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_bundle_remove(name: &str) -> Result<()> {
    let status = Command::new("bundle")
        .args(["remove", name])
        .status()
        .map_err(|e| anyhow::anyhow!("bundle remove failed to start: {e}"))?;
    if !status.success() {
        anyhow::bail!("bundle remove exit code {:?}", status.code());
    }
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
    if !removed.is_empty() {
        sync_requirements_after_remove(&removed)?;
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

fn sync_requirements_after_remove(names: &[String]) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mut req = Requirements::load_or_empty(&cwd)?;
    if !req.exists() {
        return Ok(());
    }
    let mut any = false;
    for name in names {
        if req.remove(name) {
            any = true;
        }
    }
    if any {
        req.write()?;
        println!(
            "  {} {}",
            "[REQ]".cyan(),
            "Synced requirements.txt".dimmed()
        );
    }
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

pub(super) fn cmd_remove_java(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let project = match java_manifest::detect(&cwd) {
        Some(p) => p,
        None => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "mode":"java",
                        "error":"No pom.xml or build.gradle(.kts) found"
                    }))?
                );
            } else {
                println!("\n  {}", "ven remove (java)".bold().cyan());
                println!(
                    "  {} No pom.xml or build.gradle(.kts) found.",
                    "[ERROR]".red()
                );
                println!();
            }
            return Ok(());
        }
    };

    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "mode":"java_dry_run",
                    "tool": format!("{:?}", project.tool),
                    "manifest": project.manifest.to_string_lossy(),
                    "packages": packages
                }))?
            );
        } else {
            println!("\n  {}", "ven remove (java)".bold().cyan());
            for pkg in packages {
                println!("  {} Would remove {}", "[PREVIEW]".cyan(), pkg.bold());
            }
            println!();
        }
        return Ok(());
    }

    let mut removed = Vec::new();
    if !json {
        println!("\n  {}", "ven remove (java)".bold().cyan());
    }
    for spec in packages {
        let coord = match JavaCoord::parse(spec) {
            Ok(c) => c,
            Err(e) => {
                if !json {
                    println!("  {} {}: {}", "[ERROR]".red(), spec, e);
                }
                continue;
            }
        };
        match java_manifest::remove(&project, &coord) {
            Ok(true) => {
                if !json {
                    println!("  {} Removed {}", "[OK]".green(), spec.bold());
                }
                removed.push(spec.clone());
                let _ = remove_from_ven_toml(&coord.ven_toml_key());
            }
            Ok(false) => {
                if !json {
                    println!(
                        "  {} {} not present in {}",
                        "[WARN]".yellow(),
                        spec,
                        project.manifest.display()
                    );
                }
            }
            Err(e) => {
                if !json {
                    println!("  {} {}: {}", "[ERROR]".red(), spec, e);
                }
            }
        }
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"java",
                "removed": removed
            }))?
        );
    }
    println!();
    Ok(())
}

pub(super) fn cmd_remove_deno(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "mode":"deno_dry_run",
                    "packages": packages
                }))?
            );
        } else {
            println!("\n  {}", "ven remove (deno)".bold().cyan());
            for pkg in packages {
                println!("  {} Would remove import {}", "[PREVIEW]".cyan(), pkg.bold());
            }
            println!();
        }
        return Ok(());
    }

    // Try `deno remove` first (1.42+). If it fails or absent, fall back.
    let mut removed = Vec::new();
    let used_deno = matches!(deno_imports::try_deno_remove(packages), Ok(true));
    if used_deno {
        for p in packages {
            removed.push(p.clone());
            let _ = remove_from_ven_toml(p);
        }
        if !json {
            println!(
                "\n  {} {} `deno remove` succeeded ({} item(s))",
                "ven remove (deno)".bold().cyan(),
                "[OK]".green(),
                packages.len()
            );
        }
    } else {
        let mut manifest = DenoManifest::load_or_create(&cwd)?;
        for spec in packages {
            let (key, _) = deno_imports::parse_spec(spec)
                .unwrap_or_else(|_| (spec.clone(), spec.clone()));
            if manifest.remove_import(&key) {
                removed.push(key.clone());
                let _ = remove_from_ven_toml(&key);
                if !json {
                    println!("  {} Removed import {}", "[OK]".green(), key.bold());
                }
            } else if !json {
                println!(
                    "  {} {} not in deno.json imports",
                    "[WARN]".yellow(),
                    key
                );
            }
        }
        manifest.write()?;
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"deno",
                "removed": removed
            }))?
        );
    }
    println!();
    Ok(())
}

pub(super) fn cmd_remove_go(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
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
                    "mode":"go_dry_run",
                    "packages": packages
                }))?
            );
        } else {
            println!(
                "\n  {} {}",
                "ven remove".bold().cyan(),
                "[GO DRY RUN]".yellow()
            );
            for pkg in packages {
                println!("  {} go get {}@none", "[PREVIEW]".cyan(), pkg.bold());
            }
            println!();
        }
        return Ok(());
    }
    let mut removed: Vec<String> = Vec::new();
    for pkg in packages {
        let arg = format!("{}@none", pkg);
        let status = Command::new("go").args(["get", &arg]).status();
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
    // `go mod tidy` cleans go.sum and indirect entries.
    let _ = Command::new("go").args(["mod", "tidy"]).status();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode":"go",
                "removed": removed
            }))?
        );
    }
    println!();
    Ok(())
}

pub(super) fn cmd_remove_bun(packages: &[String], dry_run: bool, json: bool) -> Result<()> {
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
                    "mode":"bun_dry_run",
                    "packages": packages
                }))?
            );
        } else {
            println!(
                "\n  {} {}",
                "ven remove".bold().cyan(),
                "[BUN DRY RUN]".yellow()
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
        let status = Command::new("bun").args(["remove", pkg]).status();
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
                "mode":"bun",
                "removed": removed
            }))?
        );
    }
    println!();
    Ok(())
}
