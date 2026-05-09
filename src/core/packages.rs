use anyhow::{anyhow, Result};
use colored::Colorize;
use std::process::Command;

// ── Run npm install ──────────────────────────────────────────────────

pub fn npm_install(package: &str, version: &str) -> Result<()> {
    let pkg_spec = format!("{}@{}", package, version);
    println!("{} Installing {}...", "[DOWNLOAD]".cyan(), pkg_spec.bold());

    // On Windows, use npm.cmd instead of npm
    let npm_cmd = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    let status = Command::new(npm_cmd)
        .args(["install", &pkg_spec])
        .status()
        .map_err(|_| anyhow!("npm not found. Is Node installed and active?"))?;

    if !status.success() {
        return Err(anyhow!("npm install failed for {}", pkg_spec));
    }

    println!("{} Installed {}", "[OK]".green(), pkg_spec.bold());
    Ok(())
}

// ── Check what depends on a package ─────────────────────────────────

pub fn find_dependents(package: &str) -> Result<Vec<(String, String)>> {
    // Read node_modules/.package-lock.json or package-lock.json
    // to find which installed packages require this one
    let lock_path = std::env::current_dir()?.join("package-lock.json");

    if !lock_path.exists() {
        return Ok(vec![]); // no lock file = cannot check
    }

    let content = std::fs::read_to_string(&lock_path)?;
    let lock: serde_json::Value = serde_json::from_str(&content)?;

    let mut dependents = Vec::new();

    // Walk packages in lock file, check their dependencies
    if let Some(packages) = lock["packages"].as_object() {
        for (name, info) in packages {
            if name.is_empty() {
                continue;
            } // skip root
            if let Some(deps) = info["dependencies"].as_object() {
                if deps.contains_key(package) {
                    let clean_name = name.trim_start_matches("node_modules/");
                    let version = info["version"].as_str().unwrap_or("").to_string();
                    dependents.push((clean_name.to_string(), version));
                }
            }
        }
    }

    Ok(dependents)
}

// ── Run npm uninstall ────────────────────────────────────────────────

pub fn npm_uninstall(package: &str) -> Result<()> {
    // Try to find npm in .ven directory first, then system PATH
    let npm_path = if let Ok(ven_node) = std::env::var("VEN_NODE_VERSION") {
        format!("C:\\Users\\Bhuwan\\.ven\\node\\{}\\npm.cmd", ven_node)
    } else {
        "npm".to_string()
    };

    let status = if std::path::Path::new(&npm_path).exists() {
        Command::new(&npm_path)
            .args(["uninstall", package])
            .status()
            .map_err(|e| anyhow!("npm execution failed: {}", e))?
    } else {
        Command::new("npm")
            .args(["uninstall", package])
            .status()
            .map_err(|_| anyhow!("npm not found"))?
    };

    if !status.success() {
        return Err(anyhow!("npm uninstall failed for {}", package));
    }
    Ok(())
}

// ── Get installed version from node_modules ─────────────────────────

pub fn get_installed_version(package: &str) -> Result<String> {
    let pkg_json = std::env::current_dir()?
        .join("node_modules")
        .join(package)
        .join("package.json");

    let content = std::fs::read_to_string(&pkg_json)?;
    let v: serde_json::Value = serde_json::from_str(&content)?;

    v["version"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Cannot read version"))
}

// ── Fetch changelog / release notes ─────────────────────────────────

#[allow(dead_code)]
pub fn fetch_release_notes(package: &str, _from_ver: &str, to_ver: &str) -> String {
    // Fetch from npm registry's "release" or from GitHub releases API
    // For Phase 1: use npm registry "description" field as a fallback
    // Full changelog parsing comes in Phase 2
    format!(
        "See full changelog: npmjs.com/package/{}/v/{}",
        package, to_ver
    )
}

// ── Tests ────────────────────────────────────────────────────────────
