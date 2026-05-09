mod display;
mod fetch;
mod select;
mod validate;

use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::plugins::PluginRegistry;

use display::display_version_list;
use fetch::fetch_available_versions;
use fetch::resolve_install_version;
use select::select_from_version_list;
use validate::validate_installation;

// ── ven install <language> <version> ────────────────────────────────────
pub fn cmd_install(language: &str, version: &str) -> Result<()> {
    let registry = PluginRegistry::new();
    let plugin = registry.require(language)?;

    let resolved = resolve_install_version(plugin, language, version)?;

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
    let _plugin = registry.require(language)?;

    println!("\n[OK] Selected: {}", language.bold());

    // Step 2: Version selection (use official remote list for every language)
    let versions = fetch_available_versions(language)?;
    display_version_list(&versions, language)?;
    let version = select_from_version_list(&versions, language)?;

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

    let versions = fetch_available_versions(language)?;

    display_version_list(&versions, language)?;

    let selected_version = select_from_version_list(&versions, language)?;

    println!(
        "\n{} Installing {} {}...",
        "[DOWNLOAD]".cyan().bold(),
        language.bold(),
        selected_version.bold()
    );
    cmd_install(language, &selected_version)
}
