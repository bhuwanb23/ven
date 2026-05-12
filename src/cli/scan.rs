//! `ven scan` — source-tree scanners. Today: ghost dependency detection.

use crate::cli::add::update_ven_toml_packages;
use crate::cli::check::primary_runtime_kind;
use crate::core::ghost_scanner::{scan_project, GhostReport};
use crate::core::load_config;
use anyhow::Result;
use colored::Colorize;

pub fn cmd_scan(ghosts: bool, fix: bool, json: bool) -> Result<()> {
    // Today the only sub-scanner is `--ghosts`. If no flag was passed, run it
    // by default — that lets `ven scan` work as a friendly shorthand.
    let _ = ghosts;

    let cwd = std::env::current_dir()?;
    let cfg = load_config(&cwd)?
        .ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;
    let kind = primary_runtime_kind(&cfg);
    let report = scan_project(&cwd, &cfg, kind)?;

    if fix && report.has_ghosts() {
        let entries: Vec<(String, String)> = report
            .ghosts
            .iter()
            .map(|g| (g.name.clone(), "latest".to_string()))
            .collect();
        update_ven_toml_packages(&entries)?;
    }

    if json {
        let actionable = report.ghosts.len();
        let out = serde_json::json!({
            "project": cwd.to_string_lossy(),
            "ecosystem": report.ecosystem,
            "files_scanned": report.files_scanned,
            "ghosts": report.ghosts,
            "actionable": actionable,
            "fix_applied": fix && actionable > 0,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        if actionable > 0 && !fix {
            anyhow::bail!("ven scan --ghosts: {} ghost(s) found", actionable);
        }
        return Ok(());
    }

    print_ghosts_human(&report, fix);
    if report.has_ghosts() && !fix {
        anyhow::bail!(
            "ven scan --ghosts: {} ghost(s) found (re-run with --fix to add them)",
            report.ghosts.len()
        );
    }
    Ok(())
}

fn print_ghosts_human(report: &GhostReport, fix: bool) {
    println!(
        "{} {}",
        "Ghost dependency scan".bold().cyan(),
        format!("({})", report.ecosystem).dimmed()
    );
    println!(
        "  {} {} file(s) scanned",
        "[INFO]".cyan(),
        report.files_scanned
    );
    if !report.has_ghosts() {
        println!(
            "  {} No ghosts — every imported name is declared.",
            "[OK]".green()
        );
        return;
    }
    println!(
        "  {} {} undeclared import(s):",
        "[GHOST]".yellow().bold(),
        report.ghosts.len()
    );
    for g in &report.ghosts {
        println!(
            "    - {} ({}× — first seen in {})",
            g.name.bold(),
            g.occurrences,
            g.first_seen_in.dimmed()
        );
    }
    if fix {
        println!(
            "  {} Added {} ghost(s) to ven.toml [packages] (spec = \"latest\").",
            "[FIX]".green(),
            report.ghosts.len()
        );
    } else {
        println!(
            "  {} re-run with `--fix` to add them to ven.toml [packages]",
            "[TIP]".cyan()
        );
    }
}
