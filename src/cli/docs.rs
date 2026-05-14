//! `ven docs` — version-pinned package documentation. See Phase 3 for the
//! full implementation; this module currently delegates to
//! [`crate::core::doc_fetcher`].

use crate::cli::check::primary_runtime_kind;
use crate::core::doc_fetcher::{
    diff_versions, render_doc, resolve_pinned_version, DocOutcome, DocRequest,
};
use crate::core::load_config;
use anyhow::Result;
use colored::Colorize;

pub fn cmd_docs(package: &str, browser: bool, diff: Option<&[String]>, json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;
    let kind = primary_runtime_kind(&cfg);

    if let Some(versions) = diff {
        if versions.len() != 2 {
            anyhow::bail!("--diff expects exactly 2 versions");
        }
        let (a, b) = (versions[0].clone(), versions[1].clone());
        let outcome = diff_versions(&kind, package, &a, &b)?;
        emit(&outcome, json);
        return Ok(());
    }

    let version = resolve_pinned_version(&cwd, &cfg, &kind, package)?.ok_or_else(|| {
        anyhow::anyhow!(
            "Package `{}` not found in ven.lock or ven.toml [packages].",
            package
        )
    })?;

    let req = DocRequest {
        kind: kind.clone(),
        package: package.to_string(),
        version: version.clone(),
        browser,
    };
    let outcome = render_doc(&req)?;
    emit(&outcome, json);
    Ok(())
}

fn emit(outcome: &DocOutcome, json: bool) {
    if json {
        match serde_json::to_string_pretty(outcome) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("  {} {}", "[ERROR]".red(), e),
        }
        return;
    }
    println!(
        "{} {}@{}",
        "Docs".bold().cyan(),
        outcome.package.bold(),
        outcome.version
    );
    if let Some(url) = &outcome.url {
        println!("  {} {}", "URL:".dimmed(), url);
    }
    if let Some(note) = &outcome.note {
        println!("  {} {}", "NOTE:".cyan(), note);
    }
    if outcome.opened_in_browser {
        println!("  {} opened in browser", "[OK]".green());
    }
    if let Some(rendered) = &outcome.rendered {
        println!();
        print!("{}", rendered);
        if !rendered.ends_with('\n') {
            println!();
        }
    }
}
