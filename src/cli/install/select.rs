use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

/// Interactive selection from version list.
pub(super) fn select_from_version_list(versions: &[String], language: &str) -> Result<String> {
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

    items.push("--- Versions ---".to_string()); // Separator
    values.push("".to_string());

    // Add latest 10 versions
    let display_count = std::cmp::min(10, versions.len());
    for (idx, version) in versions.iter().take(display_count).enumerate() {
        let metadata = get_version_metadata_short(version, language);
        items.push(format!("{:2}. {} ({})", idx + 1, version, metadata));
        values.push(version.clone());
    }

    let selection = Select::with_theme(&theme)
        .with_prompt(format!(
            "Select {} version ({} available)",
            language,
            versions.len()
        ))
        .items(&items)
        .default(0)
        .interact()?;

    let selected = &values[selection];
    if selected.is_empty() {
        return Err(anyhow::anyhow!("Please select a valid version"));
    }

    Ok(selected.clone())
}

/// Get short version metadata for display.
fn get_version_metadata_short(version: &str, language: &str) -> String {
    if language == "python" {
        return "CPython".to_string();
    } else if language == "go" {
        return "Go".to_string();
    } else if language == "rust" {
        return "Rust".to_string();
    } else if language == "java" {
        return "OpenJDK".to_string();
    } else if language == "deno" {
        return "Deno".to_string();
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
