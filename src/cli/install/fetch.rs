use anyhow::Result;
use colored::Colorize;

use crate::core::bun_install::{fetch_bun_release_versions, resolve_bun_version_spec};
use crate::core::deno_install::{fetch_deno_release_versions, resolve_deno_version_spec};
use crate::core::go_install::{fetch_go_release_versions, resolve_go_version_spec};
use crate::core::java_install::{fetch_java_release_versions, resolve_java_version_spec};
use crate::core::python_install::{fetch_python_release_versions, resolve_python_version_spec};
use crate::core::ruby_install::{fetch_ruby_release_versions, resolve_ruby_version_spec};
use crate::core::rust_install::{fetch_rust_release_versions, resolve_rust_version_spec};
use crate::plugins::LanguagePlugin;

/// Resolve a major version like "20" to the latest 20.x.x by fetching nodejs.org release list
fn resolve_major_version(_plugin: &dyn LanguagePlugin, major: &str) -> Result<String> {
    let response = reqwest::blocking::get("https://nodejs.org/dist/index.json")
        .map_err(|e| anyhow::anyhow!("Cannot reach nodejs.org: {}", e))?;
    let releases: Vec<serde_json::Value> = response.json()?;

    // Find highest version with this major number
    for release in &releases {
        if let Some(ver) = release.get("version").and_then(|v| v.as_str()) {
            let ver_clean = ver.trim_start_matches('v');
            let release_major = ver_clean.split('.').next().unwrap_or("0");
            if release_major == major {
                return Ok(ver_clean.to_string()); // releases are sorted newest first
            }
        }
    }

    // Version not found - provide helpful suggestions
    let major_num: u32 = major.parse().unwrap_or(0);

    if major_num > 0 && major_num < 18 {
        Err(anyhow::anyhow!(
            "[ERROR] Node.js {} is not available or deprecated\n\n\
             [INFO] Available LTS versions:\n\
               - 18.20.2 (Maintenance LTS)\n\
               - 20.20.2 (Active LTS) <- Recommended\n\
               - 22.22.2 (Current)\n\n\
             [TIP] Try: ven install node 20",
            major
        ))
    } else if major_num > 23 {
        Err(anyhow::anyhow!(
            "[ERROR] Node.js {} is not available yet\n\n\
             [INFO] Latest available versions:\n\
               - 22.22.2 (Current)\n\
               - 20.20.2 (Active LTS)\n\n\
             [TIP] Try: ven install node 22",
            major
        ))
    } else {
        Err(anyhow::anyhow!(
            "[ERROR] Node.js {} version not found\n\n\
             [INFO] Check available versions at: https://nodejs.org/dist/\n\
             [TIP] Try: ven install node lts",
            major
        ))
    }
}

pub(super) fn resolve_install_version(
    plugin: &dyn LanguagePlugin,
    language: &str,
    version: &str,
) -> Result<String> {
    let resolved = if language == "python" {
        println!("{} Resolving Python from python.org...", "[FETCH]".cyan());
        let avail = fetch_python_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Python releases: {}", e))?;
        resolve_python_version_spec(version, &avail)?
    } else if language == "deno" {
        println!("{} Resolving Deno from GitHub...", "[FETCH]".cyan());
        let avail = fetch_deno_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Deno releases: {}", e))?;
        resolve_deno_version_spec(version, &avail)?
    } else if language == "go" {
        println!("{} Resolving Go from go.dev...", "[FETCH]".cyan());
        let avail = fetch_go_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Go releases: {}", e))?;
        resolve_go_version_spec(version, &avail)?
    } else if language == "rust" {
        println!("{} Resolving Rust releases...", "[FETCH]".cyan());
        let avail = fetch_rust_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Rust releases: {}", e))?;
        resolve_rust_version_spec(version, &avail)?
    } else if language == "java" {
        println!("{} Resolving Java from Adoptium...", "[FETCH]".cyan());
        let avail = fetch_java_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Java releases: {}", e))?;
        resolve_java_version_spec(version, &avail)?
    } else if language == "ruby" {
        println!("{} Resolving Ruby release...", "[FETCH]".cyan());
        let avail = fetch_ruby_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Ruby releases: {}", e))?;
        resolve_ruby_version_spec(version, &avail)?
    } else if language == "bun" {
        println!("{} Resolving Bun release...", "[FETCH]".cyan());
        let avail = fetch_bun_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Bun releases: {}", e))?;
        resolve_bun_version_spec(version, &avail)?
    } else if version == "lts" || version == "latest" {
        println!(
            "{} Fetching {} release list...",
            "[FETCH]".cyan(),
            language.bold()
        );
        plugin.latest_version()?
    } else if !version.contains('.') {
        println!(
            "{} Resolving {} {} to latest patch version...",
            "[RESOLVE]".cyan(),
            language.bold(),
            version.bold()
        );
        resolve_major_version(plugin, version)?
    } else {
        // Exact version with dot - validate it exists in available releases
        let avail = fetch_available_versions(language)?;
        if avail.iter().any(|v| v == version) {
            version.to_string()
        } else {
            let sample: Vec<&str> = avail.iter().take(5).map(|s| s.as_str()).collect();
            return Err(anyhow::anyhow!(
                "{} {} version {} not found.\n\n  Available versions (sample): {}\n  [TIP] Run: ven install {} latest",
                "[ERROR]".red(),
                language.bold(),
                version.bold(),
                sample.join(", "),
                language
            ));
        }
    };
    Ok(resolved)
}

/// Fetch available versions from official source
pub(super) fn fetch_available_versions(language: &str) -> Result<Vec<String>> {
    if language == "node" {
        let response = reqwest::blocking::get("https://nodejs.org/dist/index.json")
            .map_err(|e| anyhow::anyhow!("Cannot reach nodejs.org: {}", e))?;
        let releases: Vec<serde_json::Value> = response.json()?;

        let versions: Vec<String> = releases
            .iter()
            .filter_map(|r| r.get("version").and_then(|v| v.as_str()))
            .map(|v| v.trim_start_matches('v').to_string())
            .collect();

        Ok(versions)
    } else if language == "python" {
        fetch_python_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Python releases: {}", e))
    } else if language == "go" {
        fetch_go_release_versions().map_err(|e| anyhow::anyhow!("Cannot list Go releases: {}", e))
    } else if language == "rust" {
        fetch_rust_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Rust releases: {}", e))
    } else if language == "java" {
        fetch_java_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Java releases: {}", e))
    } else if language == "deno" {
        fetch_deno_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Deno releases: {}", e))
    } else if language == "ruby" {
        fetch_ruby_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Ruby releases: {}", e))
    } else if language == "bun" {
        fetch_bun_release_versions().map_err(|e| anyhow::anyhow!("Cannot list Bun releases: {}", e))
    } else {
        Err(anyhow::anyhow!(
            "Version listing not yet supported for {}",
            language
        ))
    }
}
