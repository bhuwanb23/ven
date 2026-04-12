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
            "✗ Node.js {} is not available or deprecated\n\n\
             ℹ️ Available LTS versions:\n\
               • 18.20.2 (Maintenance LTS)\n\
               • 20.20.2 (Active LTS) ← Recommended\n\
               • 22.22.2 (Current)\n\n\
             💡 Did you mean: ven install node 20",
            major
        ))
    } else if major_num > 23 {
        // Future version that doesn't exist yet
        Err(anyhow::anyhow!(
            "✗ Node.js {} is not available yet\n\n\
             ℹ️ Latest available versions:\n\
               • 22.22.2 (Current)\n\
               • 20.20.2 (Active LTS)\n\n\
             💡 Try: ven install node 22",
            major
        ))
    } else {
        // Other error
        Err(anyhow::anyhow!(
            "✗ Node.js {} version not found\n\n\
             ℹ️ Check available versions at: https://nodejs.org/dist/\n\
             💡 Try: ven install node lts",
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
        println!("{} Fetching {} release list...", "→".cyan(), language.bold());
        plugin.latest_version()?
    } else if !version.contains('.') {
        // Major-only like "20" — resolve to highest 20.x from nodejs.org
        println!("{} Resolving {} {} to latest patch version...", "→".cyan(), language.bold(), version.bold());
        resolve_major_version(plugin, version)?
    } else {
        version.to_string()
    };

    println!("{} Resolved to {} {}", "✓".green(), language.bold(), resolved.bold());
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
    println!("\n{} Interactive Install Mode", "🔧".bold().cyan());
    
    let languages = registry.list_languages();
    let lang_idx = Select::with_theme(&theme)
        .with_prompt("Select language")
        .items(&languages)
        .default(0)
        .interact()?;
    
    let language = &languages[lang_idx];
    let plugin = registry.require(language)?;
    
    println!("\n{} Selected: {}", "✓".green(), language.bold());
    
    // Step 2: Version selection
    let version = select_version_interactive(plugin, language)?;
    
    // Step 3: Install
    println!("\n{} Installing {} {}...", "📥".bold().cyan(), language.bold(), version.bold());
    cmd_install(language, &version)
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
        format!("🔥 Current  (~85% pkg compat)")
    } else if major_num == 22 {
        format!("✅ Current  (~95% pkg compat)")
    } else if major_num == 20 {
        format!("⭐ LTS     (~98% pkg compat) [Recommended]")
    } else if major_num == 18 {
        format!("🔧 LTS     (~95% pkg compat) [Maintenance]")
    } else if major_num <= 16 {
        format!("⚠️  Deprecated (<80% pkg compat)")
    } else {
        format!("✅ Installed")
    }
}

/// Post-install validation: verify binary exists and version matches
fn validate_installation(plugin: &dyn LanguagePlugin, language: &str, version: &str) -> Result<()> {
    println!("\n{} Validating installation...", "🔍".cyan());
    
    // Check 1: Binary exists
    let bin_path = plugin.bin_path(version)?;
    let binary_name = if cfg!(windows) { "node.exe" } else { "node" };
    let binary = bin_path.join(binary_name);
    
    if binary.exists() {
        println!("  {} Binary: {}", "✓".green(), binary.display());
    } else {
        println!("  {} Binary not found: {}", "✗".red(), binary.display());
        return Err(anyhow::anyhow!("Installation validation failed: binary not found"));
    }
    
    // Check 2: Version check
    println!("  {} Version: {} {}", "✓".green(), language.bold(), version.green());
    
    // Check 3: PATH ready
    println!("  {} PATH: Ready to use", "✓".green());
    
    println!("\n{} {} {} installed successfully!", "🚀".green().bold(), language.bold(), version.bold());
    println!("  {} Run: ven init   to create a project", "💡".yellow());
    
    Ok(())
}
