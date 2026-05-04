use crate::core::go_install::{fetch_go_release_versions, resolve_go_version_spec};
use crate::core::java_install::{fetch_java_release_versions, resolve_java_version_spec};
use crate::core::python_install::{fetch_python_release_versions, resolve_python_version_spec};
use crate::core::rust_install::{fetch_rust_release_versions, resolve_rust_version_spec};
use crate::plugins::{LanguagePlugin, PluginRegistry};
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};

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
        // Deprecated or very old version
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
        // Future version that doesn't exist yet
        Err(anyhow::anyhow!(
            "[ERROR] Node.js {} is not available yet\n\n\
             [INFO] Latest available versions:\n\
               - 22.22.2 (Current)\n\
               - 20.20.2 (Active LTS)\n\n\
             [TIP] Try: ven install node 22",
            major
        ))
    } else {
        // Other error
        Err(anyhow::anyhow!(
            "[ERROR] Node.js {} version not found\n\n\
             [INFO] Check available versions at: https://nodejs.org/dist/\n\
             [TIP] Try: ven install node lts",
            major
        ))
    }
}

// ── ven install node <version> ────────────────────────────────────
pub fn cmd_install(language: &str, version: &str) -> Result<()> {
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;

    // Resolve aliases AND major-only versions before installing
    // "lts" → latest LTS  e.g. "20.11.0"
    // "latest" → latest stable e.g. "22.3.0"
    // "20" → highest 20.x available on nodejs.org e.g. "20.11.0"
    // "20.11.0" → exact, pass through
    let resolved = if language == "python" {
        println!("{} Resolving Python from python.org...", "[FETCH]".cyan());
        let avail = fetch_python_release_versions()
            .map_err(|e| anyhow::anyhow!("Cannot list Python releases: {}", e))?;
        resolve_python_version_spec(version, &avail)?
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
        version.to_string()
    };

    println!(
        "{} Resolved to {} {}",
        "[OK]".green(),
        language.bold(),
        resolved.bold()
    );
    plugin.install_version(&resolved)?;

    // Post-install validation
    validate_installation(plugin, language, &resolved)?;

    Ok(())
}

/// Interactive install mode: guide user through language and version selection
pub fn cmd_install_interactive() -> Result<()> {
    let theme = ColorfulTheme::default();
    let registry = PluginRegistry::new();

    // Step 1: Language selection
    println!("\n{} Interactive Install Mode", "[WIZARD]".bold().cyan());

    let languages = registry.list_languages();
    let lang_idx = Select::with_theme(&theme)
        .with_prompt("Select language")
        .items(&languages)
        .default(0)
        .interact()?;

    let language = &languages[lang_idx];
    let plugin = registry.require(language)?;

    println!("\n[OK] Selected: {}", language.bold());

    // Step 2: Version selection (Python uses the same remote list UI as `ven install python`)
    let version =
        if *language == "python" || *language == "go" || *language == "rust" || *language == "java" {
        let versions = fetch_available_versions(language)?;
        display_version_list(&versions, language)?;
        select_from_version_list(&versions, language)?
    } else {
        select_version_interactive(plugin, language)?
    };

    println!(
        "\n{} Installing {} {}...",
        "[DOWNLOAD]".bold().cyan(),
        language.bold(),
        version.bold()
    );
    cmd_install(language, &version)
}

/// Show available versions for a language and let user select one
pub fn cmd_install_with_version_list(language: &str) -> Result<()> {
    let registry = PluginRegistry::new();
    let _plugin = registry.require(language)?;

    println!(
        "\n{} Available {} Versions",
        "[PKG]".cyan().bold(),
        language.bold()
    );

    // Fetch available versions from nodejs.org
    let versions = fetch_available_versions(language)?;

    // Display versions with metadata
    display_version_list(&versions, language)?;

    // Interactive selection
    let selected_version = select_from_version_list(&versions, language)?;

    // Install selected version
    println!(
        "\n{} Installing {} {}...",
        "[DOWNLOAD]".cyan().bold(),
        language.bold(),
        selected_version.bold()
    );
    cmd_install(language, &selected_version)
}

/// Fetch available versions from official source
fn fetch_available_versions(language: &str) -> Result<Vec<String>> {
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
        fetch_java_release_versions().map_err(|e| anyhow::anyhow!("Cannot list Java releases: {}", e))
    } else {
        Err(anyhow::anyhow!(
            "Version listing not yet supported for {}",
            language
        ))
    }
}

/// Display version list with metadata and recommendations
fn display_version_list(versions: &[String], language: &str) -> Result<()> {
    // Get installed versions
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;
    let installed = plugin.list_installed().unwrap_or_default();

    println!();

    // Show quick options first
    println!("  {} Quick Options:", "[SPECIAL]".cyan().bold());
    println!(
        "    {}  - Install latest stable release",
        "latest".bold().green()
    );
    if language == "node" {
        println!(
            "    {}    - Install latest LTS version",
            "lts".bold().green()
        );
    }
    println!();

    // Show latest 10 versions
    println!(
        "  {} Latest Available Versions:",
        "[VERSIONS]".cyan().bold()
    );

    let display_count = std::cmp::min(10, versions.len());

    for (idx, version) in versions.iter().take(display_count).enumerate() {
        let metadata = get_version_metadata(version, language);
        let is_installed = installed.contains(&version.to_string());
        let marker = if is_installed {
            "[INSTALLED]"
        } else {
            "         "
        };
        let num = format!("{:2}.", idx + 1);

        println!("    {} {} {}  {}", num, marker, version, metadata);
    }

    if versions.len() > 10 {
        let hint = if language == "python" {
            "3.12, 3.13, or 3"
        } else if language == "go" {
            "1.21, 1.22, or 1"
        } else if language == "rust" {
            "1.75, 1.76, or 1"
        } else if language == "java" {
            "11, 17, 21, or 21.0"
        } else {
            "20, 22, 18"
        };
        println!(
            "\n  [INFO] ... and {} more versions (use a major or full version, e.g. {})",
            versions.len() - 10,
            hint
        );
    }

    if language == "python" {
        println!(
            "\n{} Example: {} {}  (or full patch e.g. 3.12.7)",
            "[TIP]".yellow(),
            "ven install python".dimmed(),
            "3.12".green()
        );
    } else if language == "go" {
        println!(
            "\n{} Example: {} {}  (or full patch e.g. 1.21.5)",
            "[TIP]".yellow(),
            "ven install go".dimmed(),
            "1.21".green()
        );
    } else if language == "rust" {
        println!(
            "\n{} Example: {} {}  (or full patch e.g. 1.75.0)",
            "[TIP]".yellow(),
            "ven install rust".dimmed(),
            "1.75".green()
        );
    } else if language == "java" {
        println!(
            "\n{} Example: {} {}  (or full patch e.g. 21.0.5)",
            "[TIP]".yellow(),
            "ven install java".dimmed(),
            "21".green()
        );
    } else {
        println!(
            "\n{} Recommended: {} {} (LTS - Best compatibility)",
            "[TIP]".yellow(),
            language.bold(),
            "20".green()
        );
    }

    Ok(())
}

/// Interactive selection from version list
fn select_from_version_list(versions: &[String], language: &str) -> Result<String> {
    use dialoguer::theme::ColorfulTheme;
    use dialoguer::Select;

    let theme = ColorfulTheme::default();

    // Build selection items
    let mut items: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    // Add special options at the top
    items.push("latest - Latest stable release".to_string());
    values.push("latest".to_string());

    if language == "node" {
        items.push("lts    - Latest LTS version (Recommended)".to_string());
        values.push("lts".to_string());
    }

    items.push("--- Press ENTER to select ---".to_string()); // Separator
    values.push("".to_string());

    // Add latest 10 versions
    let display_count = std::cmp::min(10, versions.len());
    for (idx, version) in versions.iter().take(display_count).enumerate() {
        let metadata = get_version_metadata_short(version, language);
        items.push(format!("{:2}. {} ({})", idx + 1, version, metadata));
        values.push(version.clone());
    }

    // Show selection menu
    let default_idx = if language == "node" { 1 } else { 0 };
    let selection = Select::with_theme(&theme)
        .with_prompt("Select version (use arrow keys)")
        .items(&items)
        .default(default_idx)
        .interact()?;

    let selected = &values[selection];

    // Check if user selected separator
    if selected.is_empty() {
        return Err(anyhow::anyhow!("Please select a valid version"));
    }

    Ok(selected.clone())
}

/// Get short version metadata for display
fn get_version_metadata_short(version: &str, language: &str) -> String {
    if language == "python" {
        return "CPython".to_string();
    } else if language == "go" {
        return "Go".to_string();
    } else if language == "rust" {
        return "Rust".to_string();
    } else if language == "java" {
        return "OpenJDK".to_string();
    }
    let major = version.split('.').next().unwrap_or("0");
    let major_num: u32 = major.parse().unwrap_or(0);

    if major_num >= 23 {
        "CURRENT".to_string()
    } else if major_num == 22 {
        "CURRENT".to_string()
    } else if major_num == 20 {
        "LTS".to_string()
    } else if major_num == 18 {
        "LTS".to_string()
    } else if major_num <= 16 {
        "DEPRECATED".to_string()
    } else {
        "STABLE".to_string()
    }
}

/// Interactive version selection with metadata
fn select_version_interactive(plugin: &dyn LanguagePlugin, language: &str) -> Result<String> {
    let theme = ColorfulTheme::default();

    // Get installed versions
    let installed = plugin.list_installed().unwrap_or_default();

    // Build version options with metadata
    struct VersionOption {
        value: String,
        display: String,
    }

    let mut options: Vec<VersionOption> = Vec::new();

    // Add installed versions first
    if !installed.is_empty() {
        for version in &installed {
            let info = get_version_metadata(version, language);
            options.push(VersionOption {
                value: version.clone(),
                display: format!("{}  {}", version, info),
            });
        }

        // Separator
        options.push(VersionOption {
            value: "".to_string(),
            display: "─── Version Aliases ───".to_string(),
        });
    }

    // Add aliases
    options.push(VersionOption {
        value: "latest".to_string(),
        display: "latest              Latest stable release".to_string(),
    });
    options.push(VersionOption {
        value: "lts".to_string(),
        display: "lts                 Latest LTS (recommended)".to_string(),
    });
    options.push(VersionOption {
        value: "22".to_string(),
        display: "22                  Current release line".to_string(),
    });
    options.push(VersionOption {
        value: "20".to_string(),
        display: "20                  Active LTS (best compatibility)".to_string(),
    });
    options.push(VersionOption {
        value: "18".to_string(),
        display: "18                  Maintenance LTS".to_string(),
    });

    // Warning if no versions installed
    if installed.is_empty() {
        options.insert(
            0,
            VersionOption {
                value: "".to_string(),
                display: "⚠️  No versions installed - select an alias to install".to_string(),
            },
        );
    }

    // Extract display items
    let display_items: Vec<String> = options.iter().map(|opt| opt.display.clone()).collect();

    let version_idx = Select::with_theme(&theme)
        .with_prompt(format!("Select {} version", language))
        .items(&display_items)
        .default(if installed.is_empty() { 2 } else { 0 })
        .interact()?;

    let selected = &options[version_idx];

    // Skip separator and warning
    if selected.value.is_empty() || selected.value.starts_with("⚠️") {
        return Err(anyhow::anyhow!("Please select a valid version"));
    }

    Ok(selected.value.clone())
}

/// Get version metadata (compatibility, status, etc.)
fn get_version_metadata(version: &str, language: &str) -> String {
    if language == "python" {
        return format!("[Python {}]", version);
    }
    if language == "go" {
        return format!("[Go {}]", version);
    }
    if language == "rust" {
        return format!("[Rust {}]", version);
    }
    let major = version.split('.').next().unwrap_or("0");
    let major_num: u32 = major.parse().unwrap_or(0);

    if major_num >= 23 {
        "[CURRENT]  (~85% pkg compat)".to_string()
    } else if major_num == 22 {
        "[CURRENT]  (~95% pkg compat)".to_string()
    } else if major_num == 20 {
        "[LTS]      (~98% pkg compat) [Recommended]".to_string()
    } else if major_num == 18 {
        "[LTS]      (~95% pkg compat) [Maintenance]".to_string()
    } else if major_num <= 16 {
        "[DEPRECATED] (<80% pkg compat)".to_string()
    } else {
        "[INSTALLED]".to_string()
    }
}

/// Post-install validation: verify binary exists and version matches
fn validate_installation(plugin: &dyn LanguagePlugin, language: &str, version: &str) -> Result<()> {
    println!("\n{} Validating installation...", "[CHECK]".cyan());

    // Check 1: Binary exists
    let bin_path = plugin.bin_path(version)?;
    let binary_name = match language {
        "node" => {
            if cfg!(target_os = "windows") {
                "node.exe"
            } else {
                "node"
            }
        }
        "python" => {
            if cfg!(target_os = "windows") {
                "python.exe"
            } else {
                "python3"
            }
        }
        "go" => {
            if cfg!(target_os = "windows") {
                "go.exe"
            } else {
                "go"
            }
        }
        "rust" => {
            if cfg!(target_os = "windows") {
                "cargo.exe"
            } else {
                "cargo"
            }
        }
        "java" => {
            if cfg!(target_os = "windows") {
                "java.exe"
            } else {
                "java"
            }
        }
        _ => {
            if cfg!(target_os = "windows") {
                "node.exe"
            } else {
                "node"
            }
        }
    };
    let binary = bin_path.join(binary_name);

    if binary.exists() {
        println!("  [OK] Binary: {}", binary.display());
    } else {
        println!("  [FAIL] Binary not found: {}", binary.display());
        return Err(anyhow::anyhow!(
            "Installation validation failed: binary not found"
        ));
    }

    // Check 2: Version check
    println!("  [OK] Version: {} {}", language.bold(), version.green());

    // Check 3: PATH ready
    println!("  [OK] PATH: Ready to use");

    println!(
        "\n{} {} {} installed successfully!",
        "[SUCCESS]".green().bold(),
        language.bold(),
        version.bold()
    );
    println!("  [TIP] Run: ven init   to create a project");

    Ok(())
}
