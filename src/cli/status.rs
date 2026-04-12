use anyhow::Result;
use colored::Colorize;
use crate::core::load_config;

// ── ven status ────────────────────────────────────────────────────
#[allow(non_snake_case)]
pub fn cmd_status() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let config = load_config(&cwd)?;

    println!("\n  {} {}", "ven status".bold(), cwd.display());

    match config {
        None => {
            println!("  No ven.toml found in this directory tree.");
            println!("  Run: ven init   to create one.");
        }
        Some(cfg) => {
            let node_ver = &cfg.runtime.node;
            println!("  {} {}", "node".bold(), node_ver.green());
            if !cfg.packages.is_empty() {
                println!("  {} {} packages declared", "packages".bold(), cfg.packages.len());
            }
        }
    }
    println!();
    Ok(())
}
