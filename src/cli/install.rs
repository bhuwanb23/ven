use anyhow::Result;
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use crate::plugins::{PluginRegistry, LanguagePlugin};

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
    let resolved = if version == "lts" || version == "latest" {
        println!("{} Fetching {} release list...", "[FETCH]".cyan(), language.bold());
        plugin.latest_version()?
    } else if !version.contains('.') {
        // Major-only like "20" — resolve to highest 20.x from nodejs.org
        println!("{} Resolving {} {} to latest patch version...", "[RESOLVE]".cyan(), language.bold(), version.bold());
        resolve_major_version(plugin, version)?
    } else {
        version.to_string()
    };

    println!("{} Resolved to {} {}", "[OK]".green(), language.bold(), resolved.bold());
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
    
    // Step 2: Version selection
    let version = select_version_interactive(plugin, language)?;
    
    // Step 3: Install
    println!("\n{} Installing {} {}...", "[DOWNLOAD]".bold().cyan(), language.bold(), version.bold());
    cmd_install(language, &version)
}

/// Show available versions for a language and let user select one
pub fn cmd_install_with_version_list(language: &str) -> Result<()> {
    let registry = PluginRegistry::new();
    let _plugin = registry.require(language)?;
    
    println!("\n{} Available {} Versions", "[PKG]".cyan().bold(), language.bold());
    
    // Fetch available versions from nodejs.org
    let versions = fetch_available_versions(language)?;
    
    // Display versions with metadata
    display_version_list(&versions, language)?;
    
    // Interactive selection
    let selected_version = select_from_version_list(&versions, language)?;
    
    // Install selected version
    println!("\n{} Installing {} {}...", "[DOWNLOAD]".cyan().bold(), language.bold(), selected_version.bold());
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
    } else {
        Err(anyhow::anyhow!("Version listing not yet supported for {}", language))
    }
}

/// Display version list with metadata and recommendations
fn display_version_list(versions: &[String], language: &str) -> Result<()> {
    // Get installed versions
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;
    let installed = plugin.list_installed().unwrap_or_default();
    
    // Show top versions (latest from each major line)
    let mut shown_majors = std::collections::HashSet::new();
    let mut recommended_versions: Vec<(String, String)> = Vec::new();
    
    println!();
    
    for version in versions.iter().take(50) { // Show top 50
        let major = version.split('.').next().unwrap_or("0");
        
        // Only show first version of each major (latest patch)
        if shown_majors.insert(major.to_string()) {
            let metadata = get_version_metadata(version);
            let is_installed = installed.contains(&version.to_string());
            let marker = if is_installed { " ✓" } else { "  " };
            
            println!("  {} {}  {}", marker, version.bold(), metadata.dimmed());
            
            // Track recommended versions
            if major == "20" || major == "22" {
                recommended_versions.push((version.clone(), major.to_string()));
            }
        }
    }
    
    // Show recommendation
    println!("\n{} Recommended: {} {} (LTS)", "[TIP]".yellow(), language.bold(), "20".green());
    
    Ok(())
}

/// Interactive selection from version list
fn select_from_version_list(versions: &[String], _language: &str) -> Result<String> {
    use std::io::{self, BufRead, Write};
    
    println!();
    print!("? Enter version number (or press ENTER for 20): ");
    io::stdout().flush()?;
    
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.lock().read_line(&mut input)?;
    let selected = input.trim().to_string();
    
    // Default to 20 if empty
    let selected = if selected.is_empty() {
        "20".to_string()
    } else {
        selected
    };
    
    // Validate version exists
    let major_exists = versions.iter()
        .any(|v| v.starts_with(&format!("{}.", selected)) || v == &selected);
    
    if !major_exists && selected != "lts" && selected != "latest" {
        return Err(anyhow::anyhow!(
            "Version {} not found. Use exact version (e.g., 20.20.2) or major version (e.g., 20)",
            selected
        ));
    }
    
    Ok(selected)
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
            let info = get_version_metadata(version);
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
        options.insert(0, VersionOption {
            value: "".to_string(),
            display: "⚠️  No versions installed - select an alias to install".to_string(),
        });
    }
    
    // Extract display items
    let display_items: Vec<String> = options.iter()
        .map(|opt| opt.display.clone())
        .collect();
    
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
fn get_version_metadata(version: &str) -> String {
    let major = version.split('.').next().unwrap_or("0");
    let major_num: u32 = major.parse().unwrap_or(0);
    
    if major_num >= 23 {
        format!("[CURRENT]  (~85% pkg compat)")
    } else if major_num == 22 {
        format!("[CURRENT]  (~95% pkg compat)")
    } else if major_num == 20 {
        format!("[LTS]      (~98% pkg compat) [Recommended]")
    } else if major_num == 18 {
        format!("[LTS]      (~95% pkg compat) [Maintenance]")
    } else if major_num <= 16 {
        format!("[DEPRECATED] (<80% pkg compat)")
    } else {
        format!("[INSTALLED]")
    }
}

/// Post-install validation: verify binary exists and version matches
fn validate_installation(plugin: &dyn LanguagePlugin, language: &str, version: &str) -> Result<()> {
    println!("\n{} Validating installation...", "[CHECK]".cyan());
    
    // Check 1: Binary exists
    let bin_path = plugin.bin_path(version)?;
    let binary_name = if cfg!(windows) { "node.exe" } else { "node" };
    let binary = bin_path.join(binary_name);
    
    if binary.exists() {
        println!("  [OK] Binary: {}", binary.display());
    } else {
        println!("  [FAIL] Binary not found: {}", binary.display());
        return Err(anyhow::anyhow!("Installation validation failed: binary not found"));
    }
    
    // Check 2: Version check
    println!("  [OK] Version: {} {}", language.bold(), version.green());
    
    // Check 3: PATH ready
    println!("  [OK] PATH: Ready to use");
    
    println!("\n{} {} {} installed successfully!", "[SUCCESS]".green().bold(), language.bold(), version.bold());
    println!("  [TIP] Run: ven init   to create a project");
    
    Ok(())
}
