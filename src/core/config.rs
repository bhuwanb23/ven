use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct VenConfig {
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub packages: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct RuntimeConfig {
    pub node: String,
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
        
    let config: VenConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML in {:?}", path))?;
        
    Ok(config)
}

/// Maps version strings like "18", "latest", or ">=20" to a semantic version requirement or concrete string
/// Currently a basic implementation that can be expanded later to query actual available versions
pub fn version_spec_resolver(spec: &str) -> String {
    let spec = spec.trim();
    if spec.eq_ignore_ascii_case("latest") {
        return "latest".to_string(); // In a real implementation, this would fetch the actual latest version
    }
    
    // For now, we just pass through the spec, assuming it's either an exact version or semver requirement
    // E.g., "18", "20.11.1", ">=18.0.0"
    spec.to_string()
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
        
        // Missing runtime section which is required
        let toml_content = r#"
[packages]
express = "^4.18.2"
        "#;
        
        file.write_all(toml_content.as_bytes()).unwrap();
        
        let result = parse_ven_toml(&file_path);
        assert!(result.is_err());
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
