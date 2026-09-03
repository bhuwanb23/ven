//! `ven set global <language> [version]` — make an *installed* runtime
//! globally available from every new shell by persisting its bin dir on
//! the User PATH. No admin rights needed (Windows: HKCU\\Environment PATH;
//! Unix: a fenced `>>> ven global PATH >>>` block in the rc files).
//!
//! The version must be one that's already installed under `$VEN_HOME` —
//! this command never downloads anything. Omit the version to pick from
//! the installed ones interactively.
//!
//! Subcommands:
//!
//! | Invocation                              | Behaviour                                        |
//! |-----------------------------------------|--------------------------------------------------|
//! | `ven set global`                        | List current ven-managed global PATH entries     |
//! | `ven set global <lang>`                 | Pick an installed version interactively, set it  |
//! | `ven set global <lang> <ver>`           | Resolve `<ver>` against installed, set it        |
//! | `ven set global <lang> --unset`         | Remove every global PATH entry for `<lang>`      |
//! | `ven set global <lang> <ver> --unset`   | Remove that version's global PATH entry          |
//! | `ven set global <lang> ... --json`      | Machine-readable output                          |

use anyhow::{anyhow, Result};
use clap::Subcommand;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::PathBuf;

use crate::core::user_env;
use crate::core::ven_home::ven_home;
use crate::plugins::PluginRegistry;

// ─────────────────────────────────────────────────────────────────────────
// Subcommand surface (parsed by clap in src/cli/mod.rs)
// ─────────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum SetCmd {
    /// Make an installed runtime globally available on the User PATH
    /// (installed versions only; no admin rights needed).
    ///
    /// With no arguments, lists the current ven-managed global entries.
    /// Pass a language to set (interactive version picker when omitted)
    /// or `--unset` to remove it.
    Global {
        /// Language to make global (e.g. node, python, rust). Omit to list.
        language: Option<String>,
        /// Installed version to make global. Omit to pick interactively.
        version: Option<String>,
        /// Remove the language's global PATH entry instead of setting it.
        #[arg(long)]
        unset: bool,
        /// Machine-readable output.
        #[arg(long)]
        json: bool,
    },
}

// ─────────────────────────────────────────────────────────────────────────
// Dispatch
// ─────────────────────────────────────────────────────────────────────────

pub fn cmd_set(cmd: Option<SetCmd>) -> Result<()> {
    match cmd.unwrap_or(SetCmd::Global {
        language: None,
        version: None,
        unset: false,
        json: false,
    }) {
        SetCmd::Global {
            language,
            version,
            unset,
            json,
        } => cmd_global(language, version, unset, json),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// `ven set global`
// ─────────────────────────────────────────────────────────────────────────

fn cmd_global(
    language: Option<String>,
    version: Option<String>,
    unset: bool,
    json: bool,
) -> Result<()> {
    let Some(lang) = language else {
        return list_globals(json);
    };

    let registry = PluginRegistry::new();
    let plugin = registry.require(&lang)?;
    let installed = plugin.list_installed()?;
    if installed.is_empty() {
        return Err(anyhow!(
            "No {} versions are installed under ven. Run `ven install {}` first.",
            lang.bold(),
            lang.bold()
        ));
    }

    if unset {
        return unset_global(&lang, version.as_deref(), plugin, &installed, json);
    }

    // Resolve the version: explicit spec (checked against installed) or
    // an interactive picker over installed versions.
    let resolved = match version {
        Some(spec) => resolve_installed(&lang, &spec, &installed)
            .map_err(|e| anyhow!("{e}\n  Tip: pick a version from `ven list {}`.", lang))?,
        None => {
            if installed.len() == 1 {
                installed[0].clone()
            } else {
                pick_installed_version(&lang, &installed)?
            }
        }
    };

    let bin = plugin
        .bin_path(&resolved)
        .map_err(|e| anyhow!("Could not resolve {} {} bin dir: {e}", lang, resolved))?;
    if !bin.is_dir() {
        return Err(anyhow!(
            "Expected {} {} binaries at {} but that directory doesn't exist.",
            lang,
            resolved,
            bin.display()
        ));
    }

    let added = user_env::add_global_path(&bin)?;
    let label = format!("{} {}", lang, resolved);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": if added { "set" } else { "already_set" },
                "language": lang,
                "version": resolved,
                "bin": bin.display().to_string(),
                "scope": "user",
            }))?
        );
        return Ok(());
    }

    println!();
    if added {
        println!(
            "  {} {} is now globally available (User PATH).",
            "[OK]".green().bold(),
            label.bold()
        );
    } else {
        println!(
            "  {} {} was already on the User PATH.",
            "[i]".dimmed(),
            label.bold()
        );
    }
    println!(
        "  {} {}\n",
        "[PATH]".dimmed(),
        bin.display().to_string().cyan()
    );
    println!(
        "  To use it in THIS terminal right now, run:\n\n    {}",
        session_eval_hint(&bin).yellow()
    );
    println!("  New terminals will pick it up automatically (restart your shell if needed).\n");
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// `ven set global <lang> --unset`
// ─────────────────────────────────────────────────────────────────────────

fn unset_global(
    lang: &str,
    version: Option<&str>,
    plugin: &dyn crate::plugins::LanguagePlugin,
    installed: &[String],
    json: bool,
) -> Result<()> {
    // Collect the bin dirs we manage for this language (all installed
    // versions, or just the one requested).
    let mut targets: Vec<PathBuf> = Vec::new();
    match version {
        Some(spec) => {
            let resolved = resolve_installed(lang, spec, installed)
                .map_err(|e| anyhow!("{e}\n  Tip: pick a version from `ven list {}`.", lang))?;
            targets.push(plugin.bin_path(&resolved)?);
        }
        None => {
            for v in installed {
                if let Ok(p) = plugin.bin_path(v) {
                    targets.push(p);
                }
            }
        }
    }

    let mut removed_any = false;
    for bin in &targets {
        if user_env::remove_global_path(bin)? {
            removed_any = true;
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": if removed_any { "unset" } else { "not_set" },
                "language": lang,
                "version": version.unwrap_or("*"),
                "removed": targets.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            }))?
        );
        return Ok(());
    }

    println!();
    if removed_any {
        println!(
            "  {} {} removed from the User PATH.",
            "[OK]".green().bold(),
            format!("{} {}", lang, version.unwrap_or("(all versions)")).bold()
        );
    } else {
        println!(
            "  {} No global PATH entry found for {}.",
            "[i]".dimmed(),
            lang.bold()
        );
    }
    println!();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// `ven set global` — list current globals
// ─────────────────────────────────────────────────────────────────────────

fn list_globals(json: bool) -> Result<()> {
    let entries = user_env::list_global_paths()?;
    let home = ven_home();

    if json {
        let items: Vec<serde_json::Value> = entries
            .iter()
            .map(|p| {
                let (lang, ver) = classify_entry(p, &home);
                serde_json::json!({
                    "path": p.display().to_string(),
                    "language": lang,
                    "version": ver,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "global": items }))?
        );
        return Ok(());
    }

    println!();
    if entries.is_empty() {
        println!(
            "  {} No global runtimes set.\n  Use `ven set global <language> [version]` to make an installed runtime available everywhere.\n",
            "[INFO]".cyan()
        );
        return Ok(());
    }

    println!(
        "  {} Global PATH entries (User scope):",
        "Global".bold().cyan()
    );
    for p in &entries {
        let (lang, ver) = classify_entry(p, &home);
        let tag = match (lang, ver) {
            (Some(l), Some(v)) => format!("{} {}  ", l.cyan(), v.green()),
            _ => String::new(),
        };
        println!("    {}{}", tag, p.display().to_string().dimmed());
    }
    println!(
        "  {} Use `ven set global <language> --unset` to remove one.\n",
        "[hint]".dimmed()
    );
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────

/// Map a global PATH entry back to `(language, version)` when it lives
/// under the ven home (`<home>/<lang>/<version>/bin`).
fn classify_entry(
    entry: &std::path::Path,
    home: &std::path::Path,
) -> (Option<String>, Option<String>) {
    // <home>/<lang>/<version>/bin
    let mut comps = entry.components().rev();
    let _bin = comps.next();
    let version = comps
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string());
    let lang = comps
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string());
    let rest: PathBuf = comps.rev().collect();
    if rest == *home && lang.is_some() {
        (lang, version)
    } else {
        (None, None)
    }
}

/// Shell snippet that prepends `bin` to PATH in the current session.
fn session_eval_hint(bin: &std::path::Path) -> String {
    #[cfg(windows)]
    {
        format!("$env:Path = \"{}\" + ';' + $env:Path", bin.display())
    }
    #[cfg(not(windows))]
    {
        format!("export PATH=\"{}$PATH\"", bin.display())
    }
}

/// Resolve a version spec against the *installed* versions for `lang`,
/// using the same per-language resolvers as `ven status`.
fn resolve_installed(lang: &str, spec: &str, installed: &[String]) -> Result<String> {
    use crate::core::{
        resolve_bun_version, resolve_deno_version, resolve_go_version, resolve_java_version,
        resolve_node_version, resolve_php_version, resolve_python_version, resolve_ruby_version,
        resolve_rust_version,
    };
    match lang {
        "node" => resolve_node_version(spec, installed),
        "python" => resolve_python_version(spec, installed),
        "go" => resolve_go_version(spec, installed),
        "rust" => resolve_rust_version(spec, installed),
        "java" => resolve_java_version(spec, installed),
        "ruby" => resolve_ruby_version(spec, installed),
        "bun" => resolve_bun_version(spec, installed),
        "deno" => resolve_deno_version(spec, installed),
        "php" => resolve_php_version(spec, installed),
        _ => Err(anyhow!("No version resolver registered for {lang}")),
    }
}

fn pick_installed_version(language: &str, installed: &[String]) -> Result<String> {
    let theme = ColorfulTheme::default();
    let labels: Vec<String> = installed
        .iter()
        .map(|v| format!("{}  (installed)", v))
        .collect();
    let idx = Select::with_theme(&theme)
        .with_prompt(format!("Select {} version to make global", language))
        .items(&labels)
        .default(0)
        .interact()?;
    Ok(installed[idx].clone())
}
