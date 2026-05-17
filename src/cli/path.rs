//! `ven path` — manage where ven stores its data on disk.
//!
//! When the user's C: drive fills up (or the project's `~/.ven` is sitting
//! on a roaming profile they want off), this command relocates ven's
//! storage root to a new location, optionally physically moving the
//! existing runtimes / cache / lockfile state with it.
//!
//! Subcommands:
//!
//! | Invocation                                            | Behaviour                                            |
//! |-------------------------------------------------------|------------------------------------------------------|
//! | `ven path`                                            | Alias for `ven path show`                            |
//! | `ven path show [--json]`                              | Print resolved $VEN_HOME, the resolver source, size, free space |
//! | `ven path set <dir>`                                  | Wizard: ask whether to move existing data            |
//! | `ven path set <dir> --move`                           | Skip the wizard; move the existing data              |
//! | `ven path set <dir> --no-move` / `--pointer-only`     | Skip the wizard; write the pointer only              |
//! | `ven path set <dir> -y / --yes`                       | Default to `--move` without prompting (CI / scripts) |
//! | `ven path set <dir> --json`                           | Machine-readable output (requires `--move` / `--no-move` / `--pointer-only`) |
//! | `ven path reset`                                      | Clear the pointer; ven home reverts to ~/.ven        |
//!
//! See [`crate::core::ven_home`] for how the resolved storage root is
//! actually picked up, and [`crate::core::storage_move`] for the
//! cross-drive-safe move algorithm.

use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use serde_json::json;
use std::path::{Path, PathBuf};

use crate::core::storage_move::{self, MoveOptions, MoveReport, SourceSize};
use crate::core::user_env;
use crate::core::ven_config;
use crate::core::ven_home::{ven_home, ven_home_source, HomeSource};
use crate::plugins::PluginRegistry;

// ─────────────────────────────────────────────────────────────────────────
// Subcommand surface (parsed by clap in src/cli/mod.rs)
// ─────────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum PathCmd {
    /// Print resolved $VEN_HOME, the resolver source, size, and free disk.
    Show {
        /// JSON output for scripting / CI.
        #[arg(long)]
        json: bool,
    },
    /// Relocate $VEN_HOME to a different directory.
    ///
    /// Interactive prompt by default. Pass `--move`, `--no-move`, or
    /// `--pointer-only` to skip the prompt. `--json` requires an explicit
    /// flag (no prompts in JSON mode).
    Set {
        /// New $VEN_HOME directory.
        target: PathBuf,
        /// Physically move existing data to the new location.
        #[arg(long, group = "move_choice")]
        r#move: bool,
        /// Update the pointer only; leave existing data where it is.
        #[arg(long, group = "move_choice")]
        no_move: bool,
        /// Alias for `--no-move`.
        #[arg(long, group = "move_choice")]
        pointer_only: bool,
        /// Skip the confirmation prompt (defaults to `--move`).
        #[arg(short = 'y', long)]
        yes: bool,
        /// Force a stale `.ven-move.lock` to be ignored (use only if you're
        /// sure no other ven process is in the middle of a relocation).
        #[arg(long)]
        force_unlock: bool,
        /// Machine-readable output.
        #[arg(long)]
        json: bool,
    },
    /// Clear the pointer; ven home reverts to ~/.ven.
    Reset {
        /// Move data from the current pointer location back to ~/.ven.
        #[arg(long, group = "move_choice")]
        r#move: bool,
        /// Just clear the pointer; leave the current data in place.
        #[arg(long, group = "move_choice")]
        no_move: bool,
        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
        /// Machine-readable output.
        #[arg(long)]
        json: bool,
    },
}

// ─────────────────────────────────────────────────────────────────────────
// Dispatch
// ─────────────────────────────────────────────────────────────────────────

pub fn cmd_path(cmd: Option<PathCmd>) -> Result<()> {
    match cmd.unwrap_or(PathCmd::Show { json: false }) {
        PathCmd::Show { json } => cmd_show(json),
        PathCmd::Set {
            target,
            r#move,
            no_move,
            pointer_only,
            yes,
            force_unlock,
            json,
        } => cmd_set(target, r#move, no_move || pointer_only, yes, force_unlock, json),
        PathCmd::Reset {
            r#move,
            no_move,
            yes,
            json,
        } => cmd_reset(r#move, no_move, yes, json),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// `ven path show`
// ─────────────────────────────────────────────────────────────────────────

fn cmd_show(json: bool) -> Result<()> {
    let source = ven_home_source();
    let home = source.path().to_path_buf();
    let size = if home.is_dir() {
        storage_move::measure_source(&home).unwrap_or(SourceSize { bytes: 0, files: 0 })
    } else {
        SourceSize { bytes: 0, files: 0 }
    };
    let registry = PluginRegistry::new();
    let languages_installed = count_languages_installed(&registry);

    // Pointer info — what the file says vs. what env says vs. what we
    // actually resolved.
    let pointer = ven_config::pointer_home();
    let env_home = std::env::var("VEN_HOME").ok().filter(|s| !s.is_empty());
    let env_storage = std::env::var("VEN_STORAGE_PATH")
        .ok()
        .filter(|s| !s.is_empty());

    if json {
        let payload = json!({
            "home": home.display().to_string(),
            "source": source.kind(),
            "size_bytes": size.bytes,
            "size_human": human_bytes(size.bytes),
            "languages_installed": languages_installed,
            "pointer": pointer.as_ref().map(|p| p.display().to_string()),
            "env_VEN_HOME": env_home,
            "env_VEN_STORAGE_PATH": env_storage,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!();
    println!(
        "  {} {}",
        "Storage root:".bold(),
        home.display().to_string().cyan()
    );
    println!("  {} {}", "Source:      ".bold(), describe_source(&source));
    println!(
        "  {} {} across {} language(s)",
        "Size:        ".bold(),
        human_bytes(size.bytes).bold(),
        languages_installed
    );
    if let Some(p) = &pointer {
        println!(
            "  {} {}",
            "Pointer:     ".bold(),
            p.display().to_string().dimmed()
        );
        if let Some(env) = &env_home {
            if Path::new(env) != p.as_path() {
                println!(
                    "  {} $VEN_HOME = {} (shadows the pointer; clear it or run `ven path set {}` to align)",
                    "[!]".yellow(),
                    env.dimmed(),
                    p.display()
                );
            }
        }
    }
    if let Some(env) = env_home {
        if pointer.is_none() {
            println!("  {} $VEN_HOME = {}", "Env:         ".bold(), env.dimmed());
        }
    }
    if let Some(env) = env_storage {
        println!(
            "  {} $VEN_STORAGE_PATH = {} (deprecated; use VEN_HOME)",
            "Env:         ".bold(),
            env.dimmed()
        );
    }
    println!();
    Ok(())
}

fn describe_source(source: &HomeSource) -> String {
    match source {
        HomeSource::EnvVenHome(_) => "env var $VEN_HOME (highest precedence)".to_string(),
        HomeSource::EnvVenStoragePath(_) => {
            "env var $VEN_STORAGE_PATH (back-compat)".to_string()
        }
        HomeSource::PortableSibling(p) => format!(
            "portable: .ven/ next to {}",
            p.parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<launcher>".to_string())
        ),
        HomeSource::Pointer(_) => {
            let cfg_path = ven_config::config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<config>".to_string());
            format!("pointer file ({})", cfg_path)
        }
        HomeSource::Default(_) => "default ($HOME/.ven — no override set)".to_string(),
    }
}

fn count_languages_installed(registry: &PluginRegistry) -> usize {
    registry
        .list_languages()
        .into_iter()
        .filter(|lang| {
            registry
                .require(lang)
                .ok()
                .and_then(|p| p.list_installed().ok())
                .map(|v| !v.is_empty())
                .unwrap_or(false)
        })
        .count()
}

// ─────────────────────────────────────────────────────────────────────────
// `ven path set`
// ─────────────────────────────────────────────────────────────────────────

fn cmd_set(
    raw_target: PathBuf,
    flag_move: bool,
    flag_no_move: bool,
    yes: bool,
    force_unlock: bool,
    json: bool,
) -> Result<()> {
    let target = canonicalize_target(&raw_target)?;
    let source = ven_home();
    let source_info = ven_home_source();
    let size = storage_move::measure_source(&source).unwrap_or(SourceSize { bytes: 0, files: 0 });

    // Decide the move policy.
    let decision = decide_move(flag_move, flag_no_move, yes, json, &source, &target, size)?;

    if matches!(decision, MoveDecision::Cancelled) {
        return emit_cancelled(json, &source, &target);
    }

    // ── Phase 1: relocate data (if requested) ──────────────────────────
    let move_report = if matches!(decision, MoveDecision::Move) {
        let opts = MoveOptions {
            progress: !json,
            force_unlock,
            ..MoveOptions::default()
        };
        Some(storage_move::move_storage(&source, &target, &opts)?)
    } else {
        // Pointer-only: target must exist (or be creatable) but we don't
        // touch the source.
        if !target.exists() {
            std::fs::create_dir_all(&target)
                .with_context(|| format!("Failed to create {}", target.display()))?;
        }
        None
    };

    // ── Phase 2: write the pointer (source of truth for ven itself) ────
    ven_config::set_storage_home(target.clone())
        .context("Pointer write failed AFTER data move — the data is at the new location but ven doesn't know. Re-run `ven path set <target> --pointer-only` to align.")?;

    // ── Phase 3: persist VEN_HOME in user env (best-effort) ────────────
    let env_warning = match user_env::set_user_env("VEN_HOME", &target.to_string_lossy()) {
        Ok(()) => None,
        Err(e) => Some(format!("{}", e)),
    };

    // ── Emit result ─────────────────────────────────────────────────────
    if json {
        let mut out = json!({
            "status": "ok",
            "from": source.display().to_string(),
            "from_source": source_info.kind(),
            "to": target.display().to_string(),
            "pointer": ven_config::config_path()
                .map(|p| p.display().to_string()),
            "moved": move_report.is_some(),
        });
        if let Some(r) = &move_report {
            out["bytes_moved"] = json!(r.bytes_moved);
            out["files_moved"] = json!(r.files_moved);
            out["used_fast_path"] = json!(r.used_fast_path);
        }
        if let Some(w) = &env_warning {
            out["env_warning"] = json!(w);
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!();
    if let Some(r) = &move_report {
        println!(
            "  {} Moved {} ({} file{}): {} -> {}",
            "[OK]".green().bold(),
            human_bytes(r.bytes_moved).bold(),
            r.files_moved,
            if r.files_moved == 1 { "" } else { "s" },
            source.display(),
            target.display().to_string().cyan(),
        );
        if !r.used_fast_path {
            println!(
                "       {} cross-device copy (target is on a different drive than the source)",
                "[i]".dimmed(),
            );
        }
    } else {
        println!(
            "  {} Pointer updated; existing data left at {}",
            "[OK]".green().bold(),
            source.display()
        );
    }
    if let Some(p) = ven_config::config_path() {
        println!(
            "  {} Pointer: {}",
            "[OK]".green().bold(),
            p.display().to_string().dimmed()
        );
    }
    if let Some(w) = env_warning {
        println!(
            "  {} Could not persist VEN_HOME in your user environment: {}",
            "[WARN]".yellow().bold(),
            w
        );
        println!(
            "       This is non-fatal — ven itself reads the pointer file, but new shells won't see VEN_HOME automatically. You can `setx VEN_HOME \"{}\"` (Windows) or add `export VEN_HOME=\"{}\"` to your rc file by hand.",
            target.display(),
            target.display()
        );
    } else {
        println!(
            "  {} VEN_HOME persisted in your User environment (restart your shell to see it in new sessions)",
            "[OK]".green().bold()
        );
    }
    println!();
    Ok(())
}

#[derive(Debug)]
enum MoveDecision {
    Move,
    PointerOnly,
    Cancelled,
}

fn decide_move(
    flag_move: bool,
    flag_no_move: bool,
    yes: bool,
    json: bool,
    source: &Path,
    target: &Path,
    size: SourceSize,
) -> Result<MoveDecision> {
    // Explicit flag wins.
    if flag_move {
        return Ok(MoveDecision::Move);
    }
    if flag_no_move {
        return Ok(MoveDecision::PointerOnly);
    }
    if json {
        return Err(anyhow!(
            "`ven path set --json` requires --move, --no-move, or --pointer-only (no interactive prompts in JSON mode)."
        ));
    }
    if yes {
        return Ok(MoveDecision::Move);
    }
    if !crate::core::runtime_bin::stdin_is_interactive() {
        // CI / piped stdin: default to move so scripts don't half-configure.
        return Ok(MoveDecision::Move);
    }
    // Empty source = nothing to move. Quietly skip the prompt.
    if size.files == 0 {
        return Ok(MoveDecision::PointerOnly);
    }

    println!();
    println!(
        "  {} {}",
        "[ven path]".bold().cyan(),
        "Relocate ven storage root".bold()
    );
    println!("    {} {}", "From:".dimmed(), source.display());
    println!(
        "    {} {}",
        "To:  ".dimmed(),
        target.display().to_string().cyan()
    );
    println!(
        "    {} {} across {} file(s)",
        "Data:".dimmed(),
        human_bytes(size.bytes).bold(),
        size.files
    );
    println!();

    let theme = ColorfulTheme::default();
    let idx = Select::with_theme(&theme)
        .with_prompt("What should happen to the existing data?")
        .items(&[
            "Move it to the new location (recommended)",
            "Leave it where it is; just update the pointer",
            "Cancel",
        ])
        .default(0)
        .interact()?;
    Ok(match idx {
        0 => MoveDecision::Move,
        1 => MoveDecision::PointerOnly,
        _ => MoveDecision::Cancelled,
    })
}

fn emit_cancelled(json: bool, source: &Path, target: &Path) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "cancelled",
                "from": source.display().to_string(),
                "to": target.display().to_string(),
            }))?
        );
    } else {
        println!("\n  {} Cancelled.\n", "[INFO]".cyan());
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// `ven path reset`
// ─────────────────────────────────────────────────────────────────────────

fn cmd_reset(flag_move: bool, flag_no_move: bool, yes: bool, json: bool) -> Result<()> {
    let source = ven_home();
    let source_info = ven_home_source();

    // If no pointer is set, nothing to do.
    if !matches!(source_info, HomeSource::Pointer(_)) {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "noop",
                    "reason": "no_pointer_set",
                    "home": source.display().to_string(),
                    "source": source_info.kind(),
                }))?
            );
        } else {
            println!(
                "\n  {} No pointer to clear — current ven home {} is set by `{}`, not by `ven path set`.\n",
                "[INFO]".cyan(),
                source.display(),
                source_info.kind()
            );
        }
        return Ok(());
    }

    // Default target = ~/.ven, the historic location.
    let default_home = dirs::home_dir()
        .ok_or_else(|| anyhow!("Cannot resolve $HOME"))?
        .join(".ven");
    let size = storage_move::measure_source(&source).unwrap_or(SourceSize { bytes: 0, files: 0 });

    let move_decision = if flag_move {
        MoveDecision::Move
    } else if flag_no_move {
        MoveDecision::PointerOnly
    } else if json {
        return Err(anyhow!(
            "`ven path reset --json` requires --move or --no-move (no interactive prompts in JSON mode)."
        ));
    } else if yes || !crate::core::runtime_bin::stdin_is_interactive() {
        MoveDecision::Move
    } else if size.files == 0 {
        MoveDecision::PointerOnly
    } else {
        // Reuse the same prompt machinery as `set` so the UX stays consistent.
        decide_move(false, false, false, false, &source, &default_home, size)?
    };

    if matches!(move_decision, MoveDecision::Cancelled) {
        return emit_cancelled(json, &source, &default_home);
    }

    let move_report = if matches!(move_decision, MoveDecision::Move) && source != default_home {
        let opts = MoveOptions {
            progress: !json,
            ..MoveOptions::default()
        };
        Some(storage_move::move_storage(&source, &default_home, &opts)?)
    } else {
        None
    };

    ven_config::clear_storage_home().context("Failed to clear ven global config")?;

    let env_warning = match user_env::unset_user_env("VEN_HOME") {
        Ok(()) => None,
        Err(e) => Some(format!("{}", e)),
    };

    if json {
        let mut out = json!({
            "status": "reset",
            "from": source.display().to_string(),
            "to": default_home.display().to_string(),
            "moved": move_report.is_some(),
        });
        if let Some(r) = &move_report {
            out["bytes_moved"] = json!(r.bytes_moved);
            out["files_moved"] = json!(r.files_moved);
        }
        if let Some(w) = &env_warning {
            out["env_warning"] = json!(w);
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!();
    println!(
        "  {} ven storage root reverted to default ({})",
        "[OK]".green().bold(),
        default_home.display().to_string().cyan()
    );
    if let Some(r) = &move_report {
        println!(
            "  {} Moved {} from {}",
            "[OK]".green().bold(),
            human_bytes(r.bytes_moved).bold(),
            source.display()
        );
    } else if source != default_home {
        println!(
            "  {} Pointer cleared; existing data left at {}",
            "[i]".dimmed(),
            source.display()
        );
    }
    if let Some(w) = env_warning {
        println!(
            "  {} Could not clear VEN_HOME from your user environment: {}",
            "[WARN]".yellow().bold(),
            w
        );
    } else {
        println!(
            "  {} VEN_HOME removed from your User environment",
            "[OK]".green().bold()
        );
    }
    println!();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────

fn canonicalize_target(raw: &Path) -> Result<PathBuf> {
    // Absolutize. `canonicalize` would fail on non-existing paths (which is
    // exactly the case for "I want to relocate ven to D:\\ven that doesn't
    // exist yet"), so we just join against cwd if it's relative.
    let abs = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        std::env::current_dir()?.join(raw)
    };
    // Trim a trailing slash to make display + path-prefix checks robust.
    let normalized = abs
        .components()
        .collect::<PathBuf>();
    if normalized.as_os_str().is_empty() {
        return Err(anyhow!("Target path is empty after normalization"));
    }
    Ok(normalized)
}

fn human_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_formats_the_useful_thresholds() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1536), "1.5 KB");
        assert_eq!(human_bytes(5 * 1024 * 1024), "5.0 MB");
        // 1.50 GB
        assert!(human_bytes(1_610_612_736).starts_with("1.50"));
    }

    #[test]
    fn canonicalize_makes_relative_paths_absolute() {
        let result = canonicalize_target(Path::new("subdir")).unwrap();
        assert!(result.is_absolute(), "got: {}", result.display());
        assert!(result.ends_with("subdir"));
    }

    #[test]
    fn canonicalize_keeps_absolute_paths_untouched() {
        #[cfg(windows)]
        let target = Path::new(r"D:\ven");
        #[cfg(unix)]
        let target = Path::new("/tmp/ven");
        let result = canonicalize_target(target).unwrap();
        assert_eq!(result, target);
    }
}
