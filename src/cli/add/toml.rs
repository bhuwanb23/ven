use anyhow::Result;
use colored::Colorize;
use toml_edit::{value, DocumentMut};

/// Update ven.toml with multiple packages using proper TOML parsing.
pub fn update_ven_toml_packages(packages: &[(String, String)]) -> Result<()> {
    use crate::core::find_ven_toml;

    let cwd = std::env::current_dir()?;
    let toml_path =
        find_ven_toml(&cwd).ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    let content = std::fs::read_to_string(&toml_path)?;
    let mut doc: DocumentMut = content
        .parse::<DocumentMut>()
        .map_err(|e| anyhow::anyhow!("Failed to parse ven.toml: {}", e))?;

    if !doc.contains_key("packages") {
        doc["packages"] = toml_edit::table();
    }

    let packages_table = doc["packages"]
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("Failed to access [packages] table"))?;

    for (pkg_name, version) in packages {
        let action = if packages_table.contains_key(pkg_name) {
            "Updated"
        } else {
            "Added"
        };

        packages_table.insert(pkg_name, value(version));

        println!(
            "  {} {} {} = \"{}\"",
            "[TOML]".cyan(),
            action,
            pkg_name,
            version
        );
    }

    std::fs::write(&toml_path, doc.to_string())?;
    println!(
        "  {} ven.toml updated with {} package(s)",
        "[OK]".green(),
        packages.len()
    );

    Ok(())
}
