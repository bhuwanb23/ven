use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod packages;
pub use packages::{fetch_npm_info, find_compatible_version, npm_install};

// ── Config structs ──────────────────────────────────────────────────
// These structs map exactly to ven.toml sections.
// #[derive(Deserialize)] means serde can auto-read TOML into them.

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct VenConfig {
    pub runtime:      RuntimeConfig,
    pub packages:     Option<HashMap<String, String>>,
    pub dev_packages: Option<HashMap<String, String>>,
    pub env:          Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct RuntimeConfig {
    pub node: Option<String>,   // "20.11.0", "20", "lts", "latest"
}

// ── Find ven.toml ───────────────────────────────────────────────────
// Walk up directory tree from start_dir until ven.toml found.
// Returns None if no ven.toml exists anywhere above.

pub fn find_ven_toml(start_dir: &Path) -> Option<PathBuf> {
    start_dir
        .ancestors()                    // iterate: dir, parent, grandparent...
        .map(|dir| dir.join("ven.toml"))    // build path: dir/ven.toml
        .find(|path| path.exists())         // return first one that exists
}

// ── Parse ven.toml ──────────────────────────────────────────────────
// Read the file and parse it into a VenConfig struct.
// Returns an error if file is missing or has invalid TOML syntax.

pub fn parse_ven_toml(path: &Path) -> Result<VenConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("Cannot read {}: {}", path.display(), e))?;

    let config: VenConfig = toml::from_str(&content)
        .map_err(|e| anyhow!("Invalid ven.toml at {}: {}", path.display(), e))?;

    Ok(config)
}

// ── Load config for current directory ───────────────────────────────
// Combined helper: find + parse in one call.
// Used by the shell hook and all ven commands.

#[allow(non_snake_case)]
pub fn load_config(dir: &Path) -> Result<Option<VenConfig>> {
    match find_ven_toml(dir) {
        Some(path) => Ok(Some(parse_ven_toml(&path)?)),
        None       => Ok(None),
    }
}

// ── Resolve version alias ───────────────────────────────────────────
// Maps "lts", "latest", "20" to a concrete installed version string.
// Returns the input unchanged if it is already a full version like "20.11.0".

pub fn resolve_node_version(spec: &str, installed: &[String]) -> Result<String> {
    match spec {
        "latest" => {
            installed.iter()
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow!("No Node versions installed. Run: ven install node latest"))
        }
        "lts" => {
            // LTS = even major version numbers (18, 20, 22...)
            installed.iter()
                .filter(|v| is_lts_version(v))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow!("No LTS Node versions installed."))
        }
        spec if !spec.contains('.') => {
            // Major only: "20" → find highest 20.x.x installed
            let major = spec;
            installed.iter()
                .filter(|v| v.starts_with(&format!("{}.", major)))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow!("No Node {} versions installed.", major))
        }
        _ => Ok(spec.to_string()), // already exact: "20.11.0"
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn is_lts_version(version: &str) -> bool {
    // LTS versions have even major numbers: 18.x, 20.x, 22.x
    version.split('.').next()
        .and_then(|major| major.parse::<u32>().ok())
        .map(|n| n % 2 == 0)
        .unwrap_or(false)
}

fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    // Compare "20.11.0" vs "22.3.0" numerically
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|n| n.parse().ok()).collect()
    };
    parse(a).cmp(&parse(b))
}
