use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub struct VenConfig {
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub packages: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub venv: VenVenvConfig,
}

/// Optional `[venv]` block (legacy). Hooks prepend `./venv` when it exists; `auto_path` is unused
/// but kept so existing `ven.toml` files deserialize unchanged.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct VenVenvConfig {
    #[serde(default = "default_venv_auto_path")]
    pub auto_path: bool,
}

fn default_venv_auto_path() -> bool {
    true
}

impl Default for VenVenvConfig {
    fn default() -> Self {
        Self {
            auto_path: default_venv_auto_path(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub node: String,
    /// Python interpreter version (Windows embeddable install under ~/.ven/python)
    #[serde(default)]
    pub python: String,
    /// Go toolchain version (installed under ~/.ven/go/<version>)
    #[serde(default)]
    pub go: String,
    /// Rust toolchain version (installed under ~/.ven/rust/<version>)
    #[serde(default)]
    pub rust: String,
}

/// Walks up the directory tree to find the nearest `ven.toml` file.
pub fn find_ven_toml(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = start_dir;

    loop {
        let potential_file = current_dir.join("ven.toml");
        if potential_file.is_file() {
            return Some(potential_file);
        }

        // Move up to the parent directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break, // Reached the root directory
        }
    }

    None
}

/// Parses the `ven.toml` file at the given path into a `VenConfig` struct.
pub fn parse_ven_toml(path: &Path) -> Result<VenConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;

    let config: VenConfig =
        toml::from_str(&content).with_context(|| format!("Failed to parse TOML in {:?}", path))?;

    Ok(config)
}

/// Load config for current directory - combined helper: find + parse in one call.
#[allow(non_snake_case)]
pub fn load_config(dir: &Path) -> Result<Option<VenConfig>> {
    match find_ven_toml(dir) {
        Some(path) => Ok(Some(parse_ven_toml(&path)?)),
        None => Ok(None),
    }
}

/// Maps version strings like "18", "latest", or ">=20" to a semantic version requirement or concrete string
/// Currently a basic implementation that can be expanded later to query actual available versions
#[allow(dead_code)]
pub fn version_spec_resolver(spec: &str) -> String {
    let spec = spec.trim();
    if spec.eq_ignore_ascii_case("latest") {
        return "latest".to_string(); // In a real implementation, this would fetch the actual latest version
    }

    // For now, we just pass through the spec, assuming it's either an exact version or semver requirement
    // E.g., "18", "20.11.1", ">=18.0.0"
    spec.to_string()
}

/// Resolve version alias to concrete version string
pub fn resolve_node_version(spec: &str, installed: &[String]) -> Result<String> {
    match spec {
        "latest" => installed
            .iter()
            .max_by(|a, b| version_cmp(a, b))
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("No Node versions installed. Run: ven install node latest")
            }),
        "lts" => {
            // LTS = even major version numbers (18, 20, 22...)
            installed
                .iter()
                .filter(|v| is_lts_version(v))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No LTS Node versions installed."))
        }
        spec if !spec.contains('.') => {
            // Major only: "20" → find highest 20.x.x installed
            let major = spec;
            installed
                .iter()
                .filter(|v| v.starts_with(&format!("{}.", major)))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Node {} versions installed.", major))
        }
        _ => Ok(spec.to_string()), // already exact: "20.11.0"
    }
}

/// Resolve ven.toml `runtime.python` against versions installed under ~/.ven/python
pub fn resolve_python_version(spec: &str, installed: &[String]) -> Result<String> {
    let spec = spec.trim();
    match spec {
        "latest" => installed
            .iter()
            .max_by(|a, b| version_cmp(a, b))
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("No Python versions installed. Run: ven install python latest")
            }),
        _ if !spec.contains('.') => {
            let prefix = format!("{}.", spec);
            installed
                .iter()
                .filter(|v| v.starts_with(&prefix))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Python {}.x.y installed.", spec))
        }
        _ if spec.matches('.').count() == 1 => {
            let prefix = format!("{}.", spec);
            installed
                .iter()
                .filter(|v| v.starts_with(&prefix))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Python {}.z installed.", spec))
        }
        _ => Ok(spec.to_string()),
    }
}

/// Resolve ven.toml `runtime.go` against versions installed under ~/.ven/go
pub fn resolve_go_version(spec: &str, installed: &[String]) -> Result<String> {
    let spec = spec.trim().trim_start_matches("go");
    match spec {
        "latest" => installed
            .iter()
            .max_by(|a, b| version_cmp(a, b))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No Go versions installed. Run: ven install go latest")),
        _ if !spec.contains('.') => {
            let prefix = format!("{}.", spec);
            installed
                .iter()
                .filter(|v| v.starts_with(&prefix))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Go {}.x installed.", spec))
        }
        _ if spec.matches('.').count() == 1 => {
            let prefix = format!("{}.", spec);
            installed
                .iter()
                .filter(|v| v.starts_with(&prefix))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Go {}.z installed.", spec))
        }
        _ => Ok(spec.to_string()),
    }
}

/// Resolve ven.toml `runtime.rust` against versions installed under ~/.ven/rust
pub fn resolve_rust_version(spec: &str, installed: &[String]) -> Result<String> {
    let spec = spec.trim().trim_start_matches('v');
    match spec {
        "latest" | "stable" => installed
            .iter()
            .max_by(|a, b| version_cmp(a, b))
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("No Rust versions installed. Run: ven install rust latest")
            }),
        _ if !spec.contains('.') => {
            let prefix = format!("{}.", spec);
            installed
                .iter()
                .filter(|v| v.starts_with(&prefix))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Rust {}.x installed.", spec))
        }
        _ if spec.matches('.').count() == 1 => {
            let prefix = format!("{}.", spec);
            installed
                .iter()
                .filter(|v| v.starts_with(&prefix))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Rust {}.z installed.", spec))
        }
        _ => Ok(spec.to_string()),
    }
}

fn is_lts_version(version: &str) -> bool {
    // LTS versions have even major numbers: 18.x, 20.x, 22.x
    version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .map(|n| n % 2 == 0)
        .unwrap_or(false)
}

fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    // Compare "20.11.0" vs "22.3.0" numerically
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|n| n.parse().ok()).collect() };
    parse(a).cmp(&parse(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_valid_ven_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("ven.toml");
        let mut file = File::create(&file_path).unwrap();

        let toml_content = r#"
[runtime]
node = "20.11.1"

[packages]
express = "^4.18.2"
react = "18.2.0"

[env]
NODE_ENV = "development"
PORT = "3000"
        "#;

        file.write_all(toml_content.as_bytes()).unwrap();

        let config = parse_ven_toml(&file_path).unwrap();

        assert_eq!(config.runtime.node, "20.11.1");
        assert_eq!(config.packages.get("express").unwrap(), "^4.18.2");
        assert_eq!(config.packages.get("react").unwrap(), "18.2.0");
        assert_eq!(config.env.get("NODE_ENV").unwrap(), "development");
        assert_eq!(config.env.get("PORT").unwrap(), "3000");
    }

    #[test]
    fn test_parse_missing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent.toml");

        let result = parse_ven_toml(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("ven.toml");
        let mut file = File::create(&file_path).unwrap();
        // Broken TOML syntax (parse must fail)
        file.write_all(b"[[[not-valid-toml.\n").unwrap();

        let result = parse_ven_toml(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_venv_defaults_auto_path_true() {
        let cfg: VenConfig = toml::from_str(
            r#"[runtime]
node = "18"
"#,
        )
        .unwrap();
        assert!(cfg.venv.auto_path);
    }

    #[test]
    fn test_venv_explicit_auto_path_false() {
        let cfg: VenConfig = toml::from_str(
            r#"[runtime]
python = "3.12.0"

[venv]
auto_path = false
"#,
        )
        .unwrap();
        assert!(!cfg.venv.auto_path);
    }

    #[test]
    fn test_parse_packages_only_ven_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("ven.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(
            br#"[packages]
express = "^4.18.2"
"#,
        )
        .unwrap();

        let config = parse_ven_toml(&file_path).unwrap();
        assert!(config.runtime.node.is_empty());
        assert_eq!(
            config.packages.get("express").map(String::as_str),
            Some("^4.18.2")
        );
    }

    #[test]
    fn test_find_ven_toml() {
        let dir = tempdir().unwrap();
        let root_path = dir.path();

        // Create a nested directory structure: root/a/b/c
        let nested_dir = root_path.join("a").join("b").join("c");
        fs::create_dir_all(&nested_dir).unwrap();

        // Create ven.toml in root/a
        let toml_dir = root_path.join("a");
        let toml_path = toml_dir.join("ven.toml");
        File::create(&toml_path).unwrap();

        // Test finding from root/a/b/c (should find in root/a)
        let found_path = find_ven_toml(&nested_dir).unwrap();
        assert_eq!(found_path, toml_path);

        // Test finding from root/a (should find in root/a)
        let found_path_direct = find_ven_toml(&toml_dir).unwrap();
        assert_eq!(found_path_direct, toml_path);

        // Test finding from root (should not find anything)
        let not_found = find_ven_toml(root_path);
        assert!(not_found.is_none());
    }
}
