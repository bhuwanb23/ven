mod helpers;

use crate::plugins::PluginRegistry;
use anyhow::Result;
use colored::Colorize;
use helpers::{
    calculate_dir_size, detect_active_version, format_bytes, get_installation_date,
    get_version_path, get_version_status,
};
use serde::Serialize;

// ── ven list [language] ───────────────────────────────────────────────
pub fn cmd_list(language: Option<&str>, verbose: bool, json: bool) -> Result<()> {
    let registry = PluginRegistry::new();
    match language {
        Some(lang) => list_single_language(&registry, lang, verbose, json),
        None => list_all_languages(&registry, verbose, json),
    }
}

fn list_single_language(
    registry: &PluginRegistry,
    lang: &str,
    verbose: bool,
    json: bool,
) -> Result<()> {
    let plugin = registry.require(lang)?;
    let versions = plugin.list_installed()?;

    if versions.is_empty() {
        if json {
            let output = build_list_output(lang, &versions, &None, verbose)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
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

    let active_version = detect_active_version(lang)?;

    if json {
        let output = build_list_output(lang, &versions, &active_version, verbose)?;
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if verbose {
        display_verbose_mode(lang, &versions, &active_version)?;
    } else {
        display_versions_with_metadata(lang, &versions, &active_version)?;
    }

    Ok(())
}

fn list_all_languages(registry: &PluginRegistry, verbose: bool, json: bool) -> Result<()> {
    let langs = registry.list_languages();
    if json {
        let mut map = serde_json::Map::new();
        for lang in &langs {
            let plugin = registry.require(lang)?;
            let versions = plugin.list_installed().unwrap_or_default();
            let active_version = detect_active_version(lang)?;
            let output = build_list_output(lang, &versions, &active_version, verbose)?;
            map.insert((*lang).to_string(), serde_json::to_value(&output)?);
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Object(map))?
        );
        return Ok(());
    }

    let n = langs.len();
    for (i, lang) in langs.iter().enumerate() {
        let plugin = registry.require(lang)?;
        let versions = plugin.list_installed().unwrap_or_default();
        let active_version = detect_active_version(lang)?;

        if versions.is_empty() {
            println!(
                "\n  {} {}",
                lang.bold().cyan(),
                "(no versions installed)".dimmed()
            );
            println!(
                "    {} {}",
                "→".dimmed(),
                format!("ven install {} latest", lang).dimmed()
            );
        } else if verbose {
            display_verbose_mode(lang, &versions, &active_version)?;
        } else {
            display_versions_with_metadata(lang, &versions, &active_version)?;
        }

        if i + 1 < n {
            println!();
        }
    }

    Ok(())
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
        let (status, description) = get_version_status(language, version);

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

    let old_count = versions
        .iter()
        .filter(|v| matches!(get_version_status(language, v).0, "DEPRECATED" | "EOL"))
        .count();

    if old_count > 0 {
        println!(
            "  {} {} old / end-of-life version(s) — consider upgrading or removing",
            "[TIP]".yellow(),
            old_count
        );
    }

    println!();
    Ok(())
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
        let (status, _description) = get_version_status(language, version);
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

    let old_count = versions
        .iter()
        .filter(|v| matches!(get_version_status(language, v).0, "DEPRECATED" | "EOL"))
        .count();

    if old_count > 0 {
        println!(
            "  {} {} old / end-of-life version(s) — consider upgrading or removing",
            "[TIP]".yellow(),
            old_count
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

/// Build JSON-serializable list payload (single language).
fn build_list_output(
    language: &str,
    versions: &[String],
    active_version: &Option<String>,
    verbose: bool,
) -> Result<ListOutput> {
    let mut version_infos = Vec::new();
    let mut total_size: u64 = 0;

    for version in versions {
        let (status, description) = get_version_status(language, version);
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

    Ok(ListOutput {
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
    })
}
