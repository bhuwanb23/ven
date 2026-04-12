use anyhow::{Result, anyhow};
use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

// ── npm registry response types ──────────────────────────────────────

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct NpmPackageInfo {
    pub name: String,
    #[serde(rename = "dist-tags", default)]
    pub dist_tags: HashMap<String, String>,
    #[serde(default)]
    pub versions: HashMap<String, NpmVersionInfo>,
}

#[derive(Deserialize, Debug)]
pub struct NpmVersionInfo {
    #[serde(default)]
    pub engines: Option<HashMap<String, String>>,
}

// ── Fetch package info from npm registry ─────────────────────────────

pub fn fetch_npm_info(package: &str) -> Result<NpmPackageInfo> {
    let url = format!("https://registry.npmjs.org/{}", package);

    // Use reqwest blocking client (simpler for now, async in Phase 2)
    let response = reqwest::blocking::get(&url)
        .map_err(|e| anyhow!("Cannot reach npm registry: {}", e))?;

    if response.status().as_u16() == 404 {
        return Err(anyhow!("Package '{}' not found on npm", package));
    }

    // Get response as text first for better error messages
    let status = response.status();
    let text = response.text()
        .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

    // Try to parse JSON
    let info: NpmPackageInfo = serde_json::from_str(&text)
        .map_err(|e| {
            // Show first 300 chars of response on error
            let preview = if text.len() > 300 {
                format!("{}...", &text[..300])
            } else {
                text.clone()
            };
            anyhow!("Failed to parse npm response: {}\nStatus: {}\nResponse preview: {}", e, status, preview)
        })?;

    Ok(info)
}

// ── Find best compatible version ─────────────────────────────────────
// Given current Node version and npm package info,
// returns the highest compatible version of the package.

pub fn find_compatible_version(
    info: &NpmPackageInfo,
    node_version: &str,
) -> Option<String> {
    // Try latest first
    if let Some(latest) = info.dist_tags.get("latest") {
        if is_compatible(info, latest, node_version) {
            return Some(latest.clone());
        }
    }

    // Fall back: sort all versions desc, pick highest compatible
    let mut versions: Vec<&String> = info.versions.keys().collect();
    versions.sort_by(|a, b| semver_cmp(b, a)); // desc

    versions.into_iter()
        .find(|v| is_compatible(info, v, node_version))
        .cloned()
}

fn is_compatible(info: &NpmPackageInfo, pkg_ver: &str, node_ver: &str) -> bool {
    let Some(ver_info) = info.versions.get(pkg_ver) else {
        return false;
    };
    let Some(engines) = &ver_info.engines else {
        return true; // no engine constraint = compatible with all Node versions
    };
    let Some(node_req) = engines.get("node") else {
        return true;
    };
    // Simplified check: semver satisfies
    // For Phase 1 just check if major version satisfies
    node_version_satisfies(node_ver, node_req)
}

fn node_version_satisfies(node_ver: &str, requirement: &str) -> bool {
    // Parse major from node version: "20.11.0" → 20
    let node_major = node_ver.split('.').next()
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(0);

    // Handle common requirement formats:
    // ">= 0.10.0"  ">= 14"  "^18"  "*"
    let req = requirement.trim();
    if req == "*" || req.is_empty() { return true; }

    // Extract minimum version from requirement
    let min_ver_str: String = req.chars()
        .skip_while(|c| !c.is_ascii_digit())
        .collect();

    let min_major = min_ver_str.split('.').next()
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(0);

    node_major >= min_major
}

fn semver_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|n| {
            n.chars().take_while(|c| c.is_ascii_digit())
             .collect::<String>().parse().ok()
        }).collect()
    };
    parse(a).cmp(&parse(b))
}

// ── Run npm install ──────────────────────────────────────────────────

pub fn npm_install(package: &str, version: &str) -> Result<()> {
    let pkg_spec = format!("{}@{}", package, version);
    println!("{} Installing {}...", "[DOWNLOAD]".cyan(), pkg_spec.bold());

    let status = Command::new("npm")
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
            if name.is_empty() { continue; } // skip root
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
    let status = Command::new("npm")
        .args(["uninstall", package])
        .status()
        .map_err(|_| anyhow!("npm not found"))?;

    if !status.success() {
        return Err(anyhow!("npm uninstall failed for {}", package));
    }
    Ok(())
}

// ── Get installed version from node_modules ─────────────────────────

pub fn get_installed_version(package: &str) -> Result<String> {
    let pkg_json = std::env::current_dir()?
        .join("node_modules").join(package).join("package.json");
    
    let content = std::fs::read_to_string(&pkg_json)?;
    let v: serde_json::Value = serde_json::from_str(&content)?;
    
    v["version"].as_str().map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Cannot read version"))
}

// ── Fetch changelog / release notes ─────────────────────────────────

pub fn fetch_release_notes(package: &str, _from_ver: &str, to_ver: &str) -> String {
    // Fetch from npm registry's "release" or from GitHub releases API
    // For Phase 1: use npm registry "description" field as a fallback
    // Full changelog parsing comes in Phase 2
    format!("See full changelog: npmjs.com/package/{}/v/{}", package, to_ver)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_version_satisfies_basic() {
        // Test major version checking
        assert!(node_version_satisfies("20.11.0", ">= 0.10.0"));
        assert!(node_version_satisfies("18.0.0", ">= 14"));
        assert!(node_version_satisfies("22.0.0", ">= 18"));
        
        // Test incompatible versions
        assert!(!node_version_satisfies("16.0.0", ">= 18"));
        assert!(!node_version_satisfies("14.0.0", ">= 20"));
        
        // Test wildcard
        assert!(node_version_satisfies("20.0.0", "*"));
        assert!(node_version_satisfies("20.0.0", ""));
    }

    #[test]
    fn test_semver_cmp() {
        use std::cmp::Ordering;
        
        assert_eq!(semver_cmp("4.18.2", "4.18.1"), Ordering::Greater);
        assert_eq!(semver_cmp("4.18.0", "4.17.9"), Ordering::Greater);
        assert_eq!(semver_cmp("5.0.0", "4.99.99"), Ordering::Greater);
        assert_eq!(semver_cmp("4.18.2", "4.18.2"), Ordering::Equal);
        assert_eq!(semver_cmp("4.18.1", "4.18.2"), Ordering::Less);
    }
}
