use anyhow::Result;
use colored::Colorize;
use crate::core::packages::*;

/// Remove a package with dependency checking
pub fn cmd_remove(package: &str, force: bool) -> Result<()> {
    if !force {
        let dependents = find_dependents(package)?;
        if !dependents.is_empty() {
            println!(
                "\n  {} {} packages depend on {}:",
                "Warning:".yellow().bold(),
                dependents.len(),
                package.bold()
            );
            for (dep, ver) in &dependents {
                println!(
                    "    {} {}  requires  {}",
                    "•".dimmed(),
                    format!("{} {}", dep, ver).bold(),
                    package
                );
            }
            println!();
            println!("  Removing {} may break these packages.", package);
            print!("  Remove anyway? [y/N]: ");
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let answer = stdin.lock().lines().next()
                .and_then(|l| l.ok())
                .unwrap_or_default();
            if answer.trim().to_lowercase() != "y" {
                println!("  Cancelled.");
                return Ok(());
            }
        }
    }

    npm_uninstall(package)?;
    println!("{} Removed {}", "✓".green(), package.bold());
    Ok(())
}
