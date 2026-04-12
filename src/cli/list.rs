use anyhow::Result;
use colored::Colorize;
use crate::plugins::{NodePlugin, LanguagePlugin};

// ── ven list (node) ───────────────────────────────────────────────
pub fn cmd_list(language: Option<&str>) -> Result<()> {
    match language.unwrap_or("node") {
        "node" => {
            let plugin = NodePlugin;
            let versions = plugin.list_installed()?;

            if versions.is_empty() {
                println!("{} No Node versions installed. Run: ven install node latest", "⚠️".yellow());
                return Ok(());
            }

            println!("\n  {}", "node".bold().cyan());
            for v in &versions {
                println!("    {} {}", "•".dimmed(), v);
            }
            println!();
            Ok(())
        }
        other => {
            Err(anyhow::anyhow!("Unknown language: {}. Supported: node", other))
        }
    }
}
