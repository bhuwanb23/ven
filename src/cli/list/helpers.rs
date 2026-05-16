use crate::core::{
    find_ven_toml, parse_ven_toml, resolve_go_version, resolve_java_version, resolve_node_version,
    resolve_python_version, resolve_rust_version,
};
use crate::plugins::PluginRegistry;
use anyhow::Result;

pub(crate) fn detect_active_version(language: &str) -> Result<Option<String>> {
    let current_dir = std::env::current_dir()?;
    let toml_path = match find_ven_toml(&current_dir) {
        Some(p) => p,
        None => return Ok(None),
    };

    let config = parse_ven_toml(&toml_path)?;
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;
    let installed = plugin.list_installed().unwrap_or_default();

    match language {
        "node" => {
            let spec = config.runtime.node.trim();
            if spec.is_empty() {
                return Ok(None);
            }
            match resolve_node_version(spec, &installed) {
                Ok(resolved) => Ok(Some(resolved)),
                Err(_) => Ok(None),
            }
        }
        "python" => {
            let spec = config.runtime.python.trim();
            if spec.is_empty() {
                return Ok(None);
            }
            match resolve_python_version(spec, &installed) {
                Ok(resolved) => Ok(Some(resolved)),
                Err(_) => Ok(None),
            }
        }
        "go" => {
            let spec = config.runtime.go.trim();
            if spec.is_empty() {
                return Ok(None);
            }
            match resolve_go_version(spec, &installed) {
                Ok(resolved) => Ok(Some(resolved)),
                Err(_) => Ok(None),
            }
        }
        "rust" => {
            let spec = config.runtime.rust.trim();
            if spec.is_empty() {
                return Ok(None);
            }
            match resolve_rust_version(spec, &installed) {
                Ok(resolved) => Ok(Some(resolved)),
                Err(_) => Ok(None),
            }
        }
        "java" => {
            let spec = config.runtime.java.trim();
            if spec.is_empty() {
                return Ok(None);
            }
            match resolve_java_version(spec, &installed) {
                Ok(resolved) => Ok(Some(resolved)),
                Err(_) => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

pub(crate) fn get_version_status(language: &str, version: &str) -> (&'static str, &'static str) {
    if language == "python" {
        return get_python_version_status(version);
    }
    if language == "go" {
        return get_go_version_status(version);
    }
    if language == "rust" {
        return ("STABLE", "Rust stable release");
    }
    if language == "java" {
        return ("LTS", "OpenJDK release");
    }

    let major = version.split('.').next().unwrap_or("0");
    let major_num: u32 = major.parse().unwrap_or(0);

    match major_num {
        0..=14 => ("DEPRECATED", "End-of-life"),
        15..=16 => ("DEPRECATED", "Maintenance ended"),
        18 => ("LTS", "Active LTS"),
        20 => ("LTS", "Active LTS (Recommended)"),
        21 => ("DEPRECATED", "Maintenance ended"),
        22 => ("CURRENT", "Active development"),
        23..=99 => ("CURRENT", "Latest stable"),
        _ => ("UNKNOWN", "Unknown status"),
    }
}

fn get_go_version_status(version: &str) -> (&'static str, &'static str) {
    let minor = version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    match minor {
        0..=18 => ("MAINT", "Older supported line"),
        19..=21 => ("STABLE", "Stable release line"),
        22..=99 => ("CURRENT", "Latest stable line"),
        _ => ("GO", "Go toolchain"),
    }
}

fn get_python_version_status(version: &str) -> (&'static str, &'static str) {
    let minor = version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    match minor {
        0..=9 => ("EOL", "End-of-life"),
        10..=11 => ("SECURITY", "Security-fixes-only"),
        12 => ("STABLE", "Current stable"),
        13..=99 => ("CURRENT", "Latest stable line"),
        _ => ("PY", "Python release"),
    }
}

pub(crate) fn calculate_dir_size(path: &std::path::Path) -> Result<u64> {
    let mut total_size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                total_size += entry.metadata()?.len();
            } else if path.is_dir() {
                total_size += calculate_dir_size(&path)?;
            }
        }
    }
    Ok(total_size)
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub(crate) fn get_installation_date(version_path: &std::path::Path) -> String {
    if let Ok(metadata) = std::fs::metadata(version_path) {
        if let Ok(created) = metadata.created() {
            let duration = created
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = duration.as_secs();
            let days = secs / 86400;
            let year = 1970 + (days / 365);
            let remaining_days = days % 365;
            let month = (remaining_days / 30) + 1;
            let day = (remaining_days % 30) + 1;
            return format!("{:04}-{:02}-{:02}", year, month, day);
        }
    }
    "Unknown".to_string()
}

pub(crate) fn get_version_path(language: &str, version: &str) -> Result<std::path::PathBuf> {
    Ok(crate::core::ven_home::ven_home()
        .join(language)
        .join(version))
}
