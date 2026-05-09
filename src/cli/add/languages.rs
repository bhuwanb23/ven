use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use super::update_ven_toml_packages;

use crate::core::ruby_gems;

pub(super) fn cmd_add_python(package_specs: &[String], dry_run: bool) -> Result<()> {
    let mut installed = Vec::new();
    println!("\n{}", "ven add (python)".bold().cyan());
    println!("  {} {} package(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            let (name, declared) = parse_python_spec(spec);
            println!("  {} {} => {}", "[PREVIEW]".cyan(), name.bold(), declared);
        }
        println!();
        return Ok(());
    }

    for spec in package_specs {
        let (name, declared) = parse_python_spec(spec);
        let python = resolve_python_cmd();
        let status = Command::new(&python)
            .args(["-m", "pip", "install", spec])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!(
                    "  {} {}",
                    "[OK]".green(),
                    format!("Installed {}", spec.bold())
                );
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to install {}", "[ERROR]".red(), spec),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

pub(super) fn cmd_add_go(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (go)".bold().cyan());
    println!("  {} {} module(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            println!("  {} go get {}", "[PREVIEW]".cyan(), spec.bold());
        }
        println!();
        return Ok(());
    }

    ensure_go_mod()?;
    let mut installed = Vec::new();
    for spec in package_specs {
        let status = Command::new("go").args(["get", spec]).status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} {}", "[OK]".green(), format!("Added {}", spec.bold()));
                let (name, declared) = parse_go_spec(spec);
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to add {}", "[ERROR]".red(), spec),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

pub(super) fn cmd_add_rust(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (rust)".bold().cyan());
    println!("  {} {} crate(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            println!("  {} cargo add {}", "[PREVIEW]".cyan(), spec.bold());
        }
        println!();
        return Ok(());
    }

    ensure_cargo_manifest()?;
    let mut installed = Vec::new();
    for spec in package_specs {
        let status = Command::new("cargo").args(["add", spec]).status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} {}", "[OK]".green(), format!("Added {}", spec.bold()));
                let (name, declared) = parse_rust_spec(spec);
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to add {}", "[ERROR]".red(), spec),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }
    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

pub(super) fn cmd_add_java_notice(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (java)".bold().cyan());
    println!("  {} {} item(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Java dependencies are managed by Maven/Gradle.",
            "[INFO]".cyan()
        );
        println!("  {} No changes made.", "[DRY-RUN]".yellow());
        println!();
        return Ok(());
    }
    println!(
        "  {} Java package management is delegated to Maven/Gradle.",
        "[INFO]".cyan()
    );
    println!(
        "  {} Use your build tool (e.g. mvn/gradle) to add dependencies.",
        "[TIP]".cyan()
    );
    println!();
    Ok(())
}

pub(super) fn cmd_add_ruby(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (ruby)".bold().cyan());
    println!("  {} {} gem(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode — no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            let (name, declared) = parse_ruby_gem_spec(spec);
            let decl = declared.as_deref().unwrap_or("*");
            println!("  {} {} => {}", "[PREVIEW]".cyan(), name.bold(), decl);
        }
        println!();
        return Ok(());
    }

    let mut installed = Vec::new();
    for spec in package_specs {
        let (name, _) = parse_ruby_gem_spec(spec);
        let ver = gem_version_arg_from_spec(spec);
        match ruby_gems::gem_install(&name, ver.as_deref()) {
            Ok(()) => {
                println!(
                    "  {} {}",
                    "[OK]".green(),
                    format!("Installed {}", spec.bold())
                );
                let declared = ruby_gems::gem_local_version(&name)?
                    .filter(|v| !v.is_empty())
                    .map(|v| format!(">={v}"))
                    .unwrap_or_else(|| "*".into());
                installed.push((name, declared));
            }
            Err(e) => println!("  {} {} — {}", "[ERROR]".red(), spec, e),
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

/// Version string for `gem install -v`, or omit for latest (None).
fn gem_version_arg_from_spec(spec: &str) -> Option<String> {
    let (_, v_opt) = parse_ruby_gem_spec(spec);
    match v_opt.as_deref() {
        None | Some("*") | Some("latest") | Some("") => None,
        Some(v) => Some(v.to_string()),
    }
}

fn parse_ruby_gem_spec(spec: &str) -> (String, Option<String>) {
    if let Some((name, version)) = spec.rsplit_once('@') {
        if !name.is_empty() && !version.is_empty() {
            return (name.to_string(), Some(version.to_string()));
        }
    }
    (spec.to_string(), None)
}

pub(super) fn cmd_add_deno_notice(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (deno)".bold().cyan());
    println!("  {} {} item(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Deno dependencies are managed by imports/deno.json.",
            "[INFO]".cyan()
        );
        println!("  {} No changes made.", "[DRY-RUN]".yellow());
        println!();
        return Ok(());
    }
    println!(
        "  {} Deno package management is not handled by ven.",
        "[INFO]".cyan()
    );
    println!(
        "  {} Add dependencies via imports or deno.json (and optionally deno.lock).",
        "[TIP]".cyan()
    );
    println!();
    Ok(())
}

pub(super) fn cmd_add_bun(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (bun)".bold().cyan());
    println!("  {} {} package(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            println!("  {} bun add {}", "[PREVIEW]".cyan(), spec.bold());
        }
        println!();
        return Ok(());
    }

    let mut installed = Vec::new();
    for spec in package_specs {
        let status = Command::new("bun").args(["add", spec]).status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} {}", "[OK]".green(), format!("Added {}", spec.bold()));
                let (name, declared) = parse_go_spec(spec);
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to add {}", "[ERROR]".red(), spec),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }
    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

fn ensure_go_mod() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let go_mod = cwd.join("go.mod");
    if go_mod.is_file() {
        return Ok(());
    }
    let module_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("app")
        .to_string();
    let status = Command::new("go")
        .args(["mod", "init", &module_name])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to initialize go.mod (go mod init {})", module_name);
    }
    Ok(())
}

fn parse_go_spec(spec: &str) -> (String, String) {
    if let Some((name, version)) = spec.rsplit_once('@') {
        if !version.is_empty() {
            return (name.to_string(), format!("@{}", version));
        }
    }
    (spec.to_string(), "latest".to_string())
}

fn ensure_cargo_manifest() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cargo_toml = cwd.join("Cargo.toml");
    if cargo_toml.is_file() {
        return Ok(());
    }
    let name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("app");
    let status = Command::new("cargo")
        .args(["init", "--name", name])
        .status()?;
    if !status.success() {
        anyhow::bail!(
            "Failed to initialize Cargo.toml (cargo init --name {})",
            name
        );
    }
    Ok(())
}

fn parse_rust_spec(spec: &str) -> (String, String) {
    if let Some((name, version)) = spec.rsplit_once('@') {
        if !version.is_empty() {
            return (name.to_string(), format!("@{}", version));
        }
    }
    (spec.to_string(), "latest".to_string())
}

fn parse_python_spec(spec: &str) -> (String, String) {
    let ops = ["==", ">=", "<=", "!=", "~=", ">", "<"];
    for op in ops {
        if let Some((name, _)) = spec.split_once(op) {
            return (name.trim().to_string(), spec.trim().to_string());
        }
    }
    (spec.trim().to_string(), "*".to_string())
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
