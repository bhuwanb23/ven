use crate::core::{find_ven_toml, parse_ven_toml, resolve_node_version};
use crate::plugins::PluginRegistry;
use anyhow::Result;
use colored::Colorize;
use serde::Serialize;

// ── ven list [language] ───────────────────────────────────────────────
pub fn cmd_list(language: Option<&str>, verbose: bool, json: bool) -> Result<()> {
    let lang = language.unwrap_or("node");
    let registry = PluginRegistry::new();
    let plugin = registry.require(lang)?;

    // Get installed versions
    let versions = plugin.list_installed()?;

    if versions.is_empty() {
        if json {
            // Output empty JSON array
            println!("[]");
        } else {
            println!(
                "{} No {} versions installed. Run: ven install {} latest",
                "[WARN]".yellow(),
                lang.bold(),
                lang.bold()
            );
        }
        return Ok(());
    }

    // Detect active version from ven.toml
    let active_version = detect_active_version(lang)?;

    if json {
        // JSON output mode
        output_json(lang, &versions, &active_version, verbose)?;
    } else if verbose {
        // Verbose mode with disk size and dates
        display_verbose_mode(lang, &versions, &active_version)?;
    } else {
        // Normal mode with metadata
        display_versions_with_metadata(lang, &versions, &active_version)?;
    }

    Ok(())
}

/// Detect which version is currently active (from ven.toml)
fn detect_active_version(language: &str) -> Result<Option<String>> {
    // Only support node for now
    if language != "node" {
        return Ok(None);
    }

    // Find ven.toml in current directory
    let current_dir = std::env::current_dir()?;
    let toml_path = match find_ven_toml(&current_dir) {
        Some(p) => p,
        None => return Ok(None), // No ven.toml found
    };

    // Parse config
    let config = parse_ven_toml(&toml_path)?;
    let node_spec = &config.runtime.node;

    if node_spec.is_empty() {
        return Ok(None);
    }

    // Get installed versions and resolve
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;
    let installed = plugin.list_installed().unwrap_or_default();

    // Resolve version spec to actual version
    match resolve_node_version(node_spec, &installed) {
        Ok(resolved) => Ok(Some(resolved)),
        Err(_) => Ok(None), // If resolution fails, no active version
    }
}

/// Get version status based on major version number
fn get_version_status(version: &str) -> (&'static str, &'static str) {
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

/// Display versions with metadata and active indicator
fn display_versions_with_metadata(
    language: &str,
    versions: &[String],
    active_version: &Option<String>,
) -> Result<()> {
    let count = versions.len();

    println!(
        "\n  {} ({} versions installed)",
        language.bold().cyan(),
        count.to_string().bold()
    );
    println!();

    for version in versions {
        let (status, description) = get_version_status(version);

        // Determine marker
        let is_active = active_version.as_ref() == Some(version);
        let marker = if is_active {
            "▸".bold().green()
        } else {
            "•".dimmed()
        };

        // Determine status color and tag
        let status_tag = match status {
            "LTS" => format!("[LTS] ⭐"),
            "CURRENT" => format!("[CURRENT]"),
            "DEPRECATED" => format!("[DEPRECATED]"),
            _ => format!("[{}] ", status),
        };

        // Print version line
        if is_active {
            println!(
                "    {} {}  {} {}",
                marker,
                version.bold().green(),
                status_tag,
                format!("- {}", description).dimmed()
            );
        } else {
            println!(
                "    {} {}  {} {}",
                marker,
                version,
                status_tag,
                format!("- {}", description).dimmed()
            );
        }
    }

    // Show helpful tips
    println!();
    if let Some(active) = active_version {
        println!(
            "  {} Currently active: {}",
            "[ACTIVE]".green().bold(),
            active.bold()
        );
    }

    // Check for deprecated versions
    let deprecated_count = versions
        .iter()
        .filter(|v| get_version_status(v).0 == "DEPRECATED")
        .count();

    if deprecated_count > 0 {
        println!(
            "  {} {} deprecated version(s) - consider removing to free space",
            "[TIP]".yellow(),
            deprecated_count
        );
    }

    println!();
    Ok(())
}

/// Calculate directory size recursively
fn calculate_dir_size(path: &std::path::Path) -> Result<u64> {
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

/// Format bytes to human-readable format
fn format_bytes(bytes: u64) -> String {
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

/// Get installation date from directory metadata
fn get_installation_date(version_path: &std::path::Path) -> String {
    if let Ok(metadata) = std::fs::metadata(version_path) {
        if let Ok(created) = metadata.created() {
            // Convert to seconds since epoch and format manually
            let duration = created
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = duration.as_secs();

            // Simple date calculation (approximate, but good enough)
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

/// Get version storage path
fn get_version_path(language: &str, version: &str) -> Result<std::path::PathBuf> {
    let storage_root = std::env::var("VEN_STORAGE_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("Cannot find home directory")
                .join(".ven")
        });

    Ok(storage_root.join(language).join(version))
}

/// Verbose mode: show disk size and installation dates
fn display_verbose_mode(
    language: &str,
    versions: &[String],
    active_version: &Option<String>,
) -> Result<()> {
    let count = versions.len();
    let mut total_size: u64 = 0;

    println!(
        "\n  {} ({} versions installed)",
        language.bold().cyan(),
        count.to_string().bold()
    );
    println!();

    for version in versions {
        let (status, _description) = get_version_status(version);
        let version_path = get_version_path(language, version)?;

        // Calculate size
        let size = calculate_dir_size(&version_path).unwrap_or(0);
        total_size += size;

        // Get installation date
        let install_date = get_installation_date(&version_path);

        // Determine marker
        let is_active = active_version.as_ref() == Some(version);
        let marker = if is_active {
            "▸".bold().green()
        } else {
            "•".dimmed()
        };

        // Status tag
        let status_tag = match status {
            "LTS" => format!("[LTS] ⭐"),
            "CURRENT" => format!("[CURRENT]"),
            "DEPRECATED" => format!("[DEPRECATED]"),
            _ => format!("[{}] ", status),
        };

        // Print verbose line
        if is_active {
            println!(
                "    {} {}  {}  {}  Installed: {}",
                marker,
                version.bold().green(),
                status_tag,
                format_bytes(size).bold().cyan(),
                install_date
            );
        } else {
            println!(
                "    {} {}  {}  {}  Installed: {}",
                marker,
                version,
                status_tag,
                format_bytes(size),
                install_date
            );
        }
    }

    // Show summary
    println!();
    if let Some(active) = active_version {
        println!(
            "  {} Currently active: {}",
            "[ACTIVE]".green().bold(),
            active.bold()
        );
    }
    println!(
        "  {} Total disk space: {}",
        "[DISK]".cyan().bold(),
        format_bytes(total_size).bold()
    );

    // Check for deprecated versions
    let deprecated_count = versions
        .iter()
        .filter(|v| get_version_status(v).0 == "DEPRECATED")
        .count();

    if deprecated_count > 0 {
        println!(
            "  {} {} deprecated version(s) - consider removing to free space",
            "[TIP]".yellow(),
            deprecated_count
        );
    }

    println!();
    Ok(())
}

/// JSON data structure for output
#[derive(Serialize)]
struct VersionInfo {
    version: String,
    status: String,
    description: String,
    is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_human: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    installed_date: Option<String>,
}

#[derive(Serialize)]
struct ListOutput {
    language: String,
    count: usize,
    active_version: Option<String>,
    total_size_bytes: Option<u64>,
    total_size_human: Option<String>,
    versions: Vec<VersionInfo>,
}

/// JSON output mode for scripting
fn output_json(
    language: &str,
    versions: &[String],
    active_version: &Option<String>,
    verbose: bool,
) -> Result<()> {
    let mut version_infos = Vec::new();
    let mut total_size: u64 = 0;

    for version in versions {
        let (status, description) = get_version_status(version);
        let is_active = active_version.as_ref() == Some(version);

        let mut info = VersionInfo {
            version: version.clone(),
            status: status.to_string(),
            description: description.to_string(),
            is_active,
            size_bytes: None,
            size_human: None,
            installed_date: None,
        };

        if verbose {
            let version_path = get_version_path(language, version)?;
            let size = calculate_dir_size(&version_path).unwrap_or(0);
            total_size += size;

            info.size_bytes = Some(size);
            info.size_human = Some(format_bytes(size));
            info.installed_date = Some(get_installation_date(&version_path));
        }

        version_infos.push(info);
    }

    let output = ListOutput {
        language: language.to_string(),
        count: versions.len(),
        active_version: active_version.clone(),
        total_size_bytes: if verbose { Some(total_size) } else { None },
        total_size_human: if verbose {
            Some(format_bytes(total_size))
        } else {
            None
        },
        versions: version_infos,
    };

    let json_string = serde_json::to_string_pretty(&output)?;
    println!("{}", json_string);

    Ok(())
}
