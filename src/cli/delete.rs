//! `ven delete` — remove an installed language runtime.
//!
//! Distinct from `ven remove`, which uninstalls *packages* (npm / pip / cargo /
//! gem / ...). This command deletes a runtime directory under
//! `$VEN_HOME/<language>/<version>/`.
//!
//! Calling conventions:
//!
//! | Invocation                          | Behaviour                                        |
//! |-------------------------------------|--------------------------------------------------|
//! | `ven delete`                        | Wizard: language picker -> version picker        |
//! | `ven delete <language>`             | Skip language picker; show that language's vers. |
//! | `ven delete <language> <version>`   | Skip both pickers; show confirm prompt only      |
//! | `+ -y / --yes`                      | Skip the confirm prompt (CI / scripts)           |
//! | `+ --force`                         | Allow deleting the currently-active runtime      |
//! | `+ --json`                          | Machine-readable result                          |
//!
//! Safety: by default, refuses to delete the runtime currently resolved by the
//! nearest `ven.toml`. Deleting it would silently break the next `cd`
//! activation in that project, so we want an explicit `--force` opt-in.

use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use serde_json::json;

use crate::cli::list::helpers::{
    calculate_dir_size, detect_active_version, format_bytes, get_installation_date,
    get_version_path,
};
use crate::core::find_ven_toml;
use crate::plugins::PluginRegistry;

/// Entry point dispatched from `cli::mod::run`.
pub fn cmd_delete(
    language: Option<String>,
    version: Option<String>,
    yes: bool,
    force: bool,
    json: bool,
) -> Result<()> {
    let registry = PluginRegistry::new();

    // ── Resolve language ────────────────────────────────────────────────
    let language = match language {
        Some(l) => {
            // Validate up-front so a typo doesn't sneak into the wizard.
            registry.require(&l)?;
            l
        }
        None => {
            if json {
                return Err(anyhow!(
                    "`ven delete --json` requires explicit <language> [version] args (no wizard in JSON mode)"
                ));
            }
            pick_language(&registry)?
        }
    };

    // ── Resolve version ─────────────────────────────────────────────────
    let plugin = registry.require(&language)?;
    let installed = plugin.list_installed().unwrap_or_default();

    if installed.is_empty() {
        let msg = format!(
            "No {} versions installed. Nothing to delete.",
            language.bold()
        );
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "noop",
                    "reason": "no_versions_installed",
                    "language": language,
                }))?
            );
        } else {
            println!("\n  {} {}\n", "[INFO]".cyan(), msg);
        }
        return Ok(());
    }

    let version = match version {
        Some(v) => {
            if !installed.iter().any(|installed_v| installed_v == &v) {
                return Err(anyhow!(
                    "{} {} is not installed.\n\n  Installed versions: {}\n  Tip: run `ven list {}` to see them all.",
                    language,
                    v,
                    installed.join(", "),
                    language
                ));
            }
            v
        }
        None => {
            if json {
                return Err(anyhow!(
                    "`ven delete --json` requires an explicit <version> arg (no wizard in JSON mode)"
                ));
            }
            pick_version(&language, &installed)?
        }
    };

    // ── Safety: refuse to delete the active runtime ──────────────────────
    if !force {
        if let Some(active) = detect_active_version(&language)? {
            if active == version {
                let toml_hint = find_ven_toml(&std::env::current_dir()?)
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "the nearest ven.toml".to_string());

                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "status": "refused",
                            "reason": "active_runtime",
                            "language": language,
                            "version": version,
                            "ven_toml": toml_hint,
                            "hint": "pass --force to override",
                        }))?
                    );
                    // Non-zero exit so CI scripts notice.
                    std::process::exit(1);
                }

                return Err(anyhow!(
                    "Cannot delete {} {}: it is the active runtime in {}.\n\n  \
                     Deleting it would break the next `cd` activation in that project.\n  \
                     Pin a different version in ven.toml first, or pass --force to override.",
                    language,
                    version,
                    toml_hint
                ));
            }
        }
    }

    // ── Compute size + confirm ──────────────────────────────────────────
    let path = get_version_path(&language, &version)?;
    if !path.exists() {
        // Stale entry in `list_installed` somehow — surface a useful error.
        return Err(anyhow!(
            "{} {} resolved to {} but that path does not exist on disk.",
            language,
            version,
            path.display()
        ));
    }

    let size = calculate_dir_size(&path).unwrap_or(0);
    let installed_on = get_installation_date(&path);

    let proceed = confirm_deletion(
        &language,
        &version,
        &path,
        size,
        &installed_on,
        force,
        yes,
        json,
    )?;

    if !proceed {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "cancelled",
                    "language": language,
                    "version": version,
                }))?
            );
        } else {
            println!("\n  {} Cancelled.\n", "[INFO]".cyan());
        }
        return Ok(());
    }

    // ── Actually delete ─────────────────────────────────────────────────
    std::fs::remove_dir_all(&path)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "deleted",
                "language": language,
                "version": version,
                "path": path.display().to_string(),
                "freed_bytes": size,
                "freed_human": format_bytes(size),
                "force": force,
            }))?
        );
    } else {
        println!(
            "\n  {} Deleted {} {} ({} freed)\n  {} {}\n",
            "[OK]".green().bold(),
            language.bold(),
            version.bold(),
            format_bytes(size).bold(),
            "[PATH]".dimmed(),
            path.display()
        );
    }

    Ok(())
}

// ── Interactive helpers ─────────────────────────────────────────────────

fn pick_language(registry: &PluginRegistry) -> Result<String> {
    let theme = ColorfulTheme::default();

    println!("\n{} Delete a language runtime", "[WIZARD]".bold().cyan());

    let all_langs = registry.list_languages();
    // Only offer languages that actually have something installed — otherwise
    // a user picking "ruby" with zero installs has to back out anyway.
    let mut langs_with_installs: Vec<(String, usize)> = Vec::new();
    for lang in &all_langs {
        let count = registry
            .require(lang)
            .ok()
            .and_then(|p| p.list_installed().ok())
            .map(|v| v.len())
            .unwrap_or(0);
        if count > 0 {
            langs_with_installs.push(((*lang).to_string(), count));
        }
    }

    if langs_with_installs.is_empty() {
        return Err(anyhow!(
            "No installed runtimes found in $VEN_HOME. Nothing to delete.\n  Tip: run `ven list` to confirm."
        ));
    }

    let labels: Vec<String> = langs_with_installs
        .iter()
        .map(|(lang, count)| {
            format!(
                "{}  ({} version{} installed)",
                lang,
                count,
                if *count == 1 { "" } else { "s" }
            )
        })
        .collect();

    let idx = Select::with_theme(&theme)
        .with_prompt("Select language")
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(langs_with_installs[idx].0.clone())
}

fn pick_version(language: &str, installed: &[String]) -> Result<String> {
    let theme = ColorfulTheme::default();

    // Annotate each version with size + install date so the user has enough
    // context to pick without going back to `ven list --verbose`.
    let labels: Vec<String> = installed
        .iter()
        .map(|v| {
            let path = get_version_path(language, v).ok();
            let size = path
                .as_ref()
                .map(|p| calculate_dir_size(p).unwrap_or(0))
                .unwrap_or(0);
            let date = path
                .as_ref()
                .map(|p| get_installation_date(p))
                .unwrap_or_else(|| "?".to_string());
            format!("{}  ({} - installed {})", v, format_bytes(size), date)
        })
        .collect();

    let idx = Select::with_theme(&theme)
        .with_prompt(format!("Select {} version to delete", language))
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(installed[idx].clone())
}

fn confirm_deletion(
    language: &str,
    version: &str,
    path: &std::path::Path,
    size: u64,
    installed_on: &str,
    force: bool,
    yes: bool,
    json: bool,
) -> Result<bool> {
    if yes {
        return Ok(true);
    }

    // CI / piped stdin: behave like other commands and auto-confirm.
    if !crate::core::runtime_bin::stdin_is_interactive() {
        if !json {
            println!(
                "\n  {} Auto-confirming (non-interactive stdin).",
                "[INFO]".cyan()
            );
        }
        return Ok(true);
    }

    if json {
        // We don't render a TUI prompt in JSON mode — the caller is supposed
        // to pass -y. If they didn't, treat it as a refused operation.
        return Err(anyhow!(
            "`ven delete --json` requires -y / --yes to confirm (no interactive prompt in JSON mode)"
        ));
    }

    println!(
        "\n  {} About to permanently delete {}",
        "[DELETE]".red().bold(),
        format!("{} {}", language, version).bold()
    );
    println!("    {} {}", "Path:".dimmed(), path.display());
    println!(
        "    {} {} (installed {})",
        "Size:".dimmed(),
        format_bytes(size).bold(),
        installed_on
    );
    if force {
        println!(
            "    {} --force is set: the active-runtime safety check was skipped.",
            "[!]".yellow()
        );
    }
    println!();

    let theme = ColorfulTheme::default();
    let ok = Confirm::with_theme(&theme)
        .with_prompt("Permanently delete this runtime?")
        .default(false)
        .interact()?;

    Ok(ok)
}
