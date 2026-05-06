//! `ven status` — project/runtime/package summary (basic, `--verbose`, `--json`).
mod basic;
mod helpers;
mod json;
mod verbose;

use crate::core::{find_ven_toml, parse_ven_toml};
use anyhow::Result;
use colored::Colorize;
pub fn cmd_status(json: bool, verbose: bool, fix: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Find ven.toml
    let toml_path = match find_ven_toml(&cwd) {
        Some(p) => p,
        None => {
            if json {
                println!("{{\"error\": \"No ven.toml found\"}}");
            } else {
                println!("\n  {} {}", "ven status".bold(), cwd.display());
                println!(
                    "  {} No ven.toml found in this directory tree.",
                    "[WARN]".yellow()
                );
                println!("  {} Run: ven init   to create one.", "[TIP]".cyan());
                println!();
            }
            return Ok(());
        }
    };

    let config = parse_ven_toml(&toml_path)?;

    if json {
        json::output_json_status(&cwd, &toml_path, &config, verbose)?;
    } else if verbose {
        verbose::display_verbose_status(&cwd, &toml_path, &config, fix)?;
    } else {
        basic::display_basic_status(&cwd, &toml_path, &config)?;
    }

    Ok(())
}
