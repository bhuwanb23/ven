//! `ven uninstall` — full-nuke teardown of the running ven install.
//!
//! Replaces the long copy-paste shell snippet that lived on the install
//! page with a single confirmed, dry-run-capable command. The actual
//! discovery + removal logic lives in [`crate::core::uninstaller`]; this
//! module is the CLI seam — prompts, flag plumbing, JSON output,
//! human-readable formatting.
//!
//! Calling conventions:
//!
//! | Invocation                          | Behaviour                                        |
//! |-------------------------------------|--------------------------------------------------|
//! | `ven uninstall`                     | Interactive: print plan + prompt before nuking   |
//! | `+ -y / --yes`                      | Skip the confirm prompt (CI / scripts)           |
//! | `+ --dry-run`                       | Print the plan; touch nothing                    |
//! | `+ --user-only`                     | Skip the system install layer                    |
//! | `+ --system-only`                   | Skip the user install layer (rare; for admins)   |
//! | `+ --json`                          | Machine-readable result (requires -y / --dry-run)|
//!
//! Safety: unlike `ven delete`, this never auto-confirms on a piped stdin.
//! Uninstall is FAR more destructive (every runtime + cache + state), so
//! the only opt-outs from the prompt are an explicit `-y` or a `--dry-run`.

use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde_json::json;

use crate::core::uninstaller::{
    self, ExecuteOptions, UninstallPlan, UninstallReport, UninstallScope,
};

/// Entry point dispatched from `cli::mod::run`.
pub fn cmd_uninstall(
    yes: bool,
    json: bool,
    dry_run: bool,
    user_only: bool,
    system_only: bool,
) -> Result<()> {
    if user_only && system_only {
        return Err(anyhow!(
            "--user-only and --system-only are mutually exclusive."
        ));
    }
    let scope = if user_only {
        UninstallScope::UserOnly
    } else if system_only {
        UninstallScope::SystemOnly
    } else {
        UninstallScope::All
    };

    // JSON-mode guardrails:
    //   - JSON without -y AND without --dry-run is ambiguous (would the
    //     command execute or just plan?). Refuse explicitly so CI scripts
    //     don't accidentally erase user installs.
    if json && !yes && !dry_run {
        return Err(anyhow!(
            "`ven uninstall --json` requires either --dry-run (plan only) or -y / --yes (execute)."
        ));
    }

    let plan = uninstaller::build_plan(scope)?;

    if json {
        return emit_json(&plan, dry_run, yes);
    }

    print_header(dry_run);
    print_plan(&plan);

    // Bail early when there's literally nothing to do.
    if is_plan_empty(&plan) {
        println!();
        println!(
            "  {} Nothing to uninstall — ven doesn't appear to be installed for this user.",
            "[i]".cyan()
        );
        println!(
            "    Tip: run `{}` to double-check the storage root.",
            "ven path show".bold()
        );
        return Ok(());
    }

    // Elevation gate. We don't fork sudo / UAC from here — the user is
    // expected to re-run from an elevated shell, same convention as the
    // bundled `ven-uninstall.{ps1,sh}` fallback scripts.
    if plan.needs_elevation && !dry_run {
        println!();
        println!(
            "  {} this would touch a system install at {}",
            "[!]".yellow(),
            plan.system_artifacts
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        if cfg!(target_os = "windows") {
            println!(
                "    Re-run from an elevated PowerShell ({})",
                "Start PowerShell -> Run as Administrator".dimmed()
            );
        } else {
            println!(
                "    Re-run with sudo: {}",
                "sudo ven uninstall".bold()
            );
        }
        println!(
            "    Or pass {} to skip the system layer for now.",
            "--user-only".bold()
        );
        return Err(anyhow!(
            "Insufficient privileges for the system install layer."
        ));
    }

    if dry_run {
        println!();
        println!(
            "  {} dry-run finished — no files were touched.",
            "[DRY-RUN]".cyan().bold()
        );
        println!(
            "    Re-run with {} to actually uninstall.",
            "-y / --yes".bold()
        );
        return Ok(());
    }

    if !yes && !confirm_destructive_action(&plan)? {
        println!();
        println!("  {} Cancelled.", "[i]".cyan());
        return Ok(());
    }

    let report = uninstaller::execute_plan(&plan, &ExecuteOptions { dry_run: false })?;

    print_report(&report);

    if !report.errors.is_empty() {
        return Err(anyhow!(
            "Uninstall completed with {} error(s). See report above; you may need to delete the remaining files manually.",
            report.errors.len()
        ));
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// Heuristics
// ─────────────────────────────────────────────────────────────────────────

fn is_plan_empty(plan: &UninstallPlan) -> bool {
    plan.user_install_root.is_none()
        && plan.system_artifacts.is_empty()
        && plan.pointer_file.is_none()
        // A relocated data_dir with nothing else still counts as "something
        // to do" — but only if it actually exists on disk.
        && !(plan.data_dir_is_relocated && plan.data_dir.exists())
}

// ─────────────────────────────────────────────────────────────────────────
// Human output
// ─────────────────────────────────────────────────────────────────────────

fn print_header(dry_run: bool) {
    println!();
    if dry_run {
        println!("{} ven uninstall (dry-run)", "[DRY-RUN]".cyan().bold());
        println!(
            "  {} Nothing will be removed. This is a plan-only run.",
            "[i]".cyan()
        );
    } else {
        println!("{} ven uninstall", "[WIZARD]".bold().cyan());
        println!(
            "  {} This will permanently remove ven, every installed runtime,",
            "[!]".yellow()
        );
        println!("       all cache + state, and the ven PATH entries.");
    }
}

fn print_plan(plan: &UninstallPlan) {
    println!();
    println!("  {}", "Will remove:".bold());

    if let Some(root) = &plan.user_install_root {
        println!(
            "    • user install root  {}",
            root.display().to_string().bold()
        );
    } else {
        println!("    • user install root  {}", "(not present)".dimmed());
    }

    if plan.data_dir_is_relocated && plan.data_dir.exists() {
        println!(
            "    • runtime data dir   {}  {}",
            plan.data_dir.display().to_string().bold(),
            format!("[relocated via {}]", plan.data_dir_source.kind()).dimmed()
        );
    } else if !plan.data_dir_is_relocated {
        println!(
            "    • runtime data dir   {}",
            "(folded into install root above)".dimmed()
        );
    }

    if plan.system_artifacts.is_empty() {
        println!("    • system install     {}", "(not present)".dimmed());
    } else {
        let entries = plan
            .system_artifacts
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("    • system install     {}", entries.bold());
    }

    if let Some(p) = &plan.pointer_file {
        println!(
            "    • pointer file       {}",
            p.display().to_string().bold()
        );
    }
    if !plan.user_env_vars.is_empty() {
        let vars = plan
            .user_env_vars
            .iter()
            .map(|v| format!("${v}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("    • user env vars      {}", vars.bold());
    }
    if !plan.user_path_entries.is_empty() {
        let entries = plan
            .user_path_entries
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("    • user PATH entries  {}", entries.bold());
    }
    if !plan.system_path_entries.is_empty() {
        let entries = plan
            .system_path_entries
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "    • system PATH entries {}",
            entries.bold()
        );
    }
    let rc_present: Vec<_> = plan
        .rc_files_to_clean
        .iter()
        .filter(|p| p.is_file())
        .collect();
    if !rc_present.is_empty() {
        println!(
            "    • shell rc files     {} ven-managed blocks will be removed",
            format!("({})", rc_present.len()).bold()
        );
        for rc in &rc_present {
            println!("                         {}", rc.display().to_string().dimmed());
        }
    }

    println!();
    println!(
        "  {} scope: {}",
        "[i]".cyan(),
        format!("{:?}", plan.scope).to_lowercase()
    );
    if let Some(exe) = &plan.current_exe {
        println!("  {} running exe: {}", "[i]".cyan(), exe.display());
    }
}

fn print_report(report: &UninstallReport) {
    println!();
    println!("{}", "Uninstalled.".green().bold());

    if !report.removed_dirs.is_empty() {
        println!(
            "  {} {} dir(s) removed",
            "[OK]".green(),
            report.removed_dirs.len()
        );
    }
    if !report.removed_files.is_empty() {
        println!(
            "  {} {} file(s) removed",
            "[OK]".green(),
            report.removed_files.len()
        );
    }
    if !report.removed_env_vars.is_empty() {
        println!(
            "  {} user env var(s) cleared: {}",
            "[OK]".green(),
            report.removed_env_vars.join(", ")
        );
    }
    if !report.stripped_path_entries.is_empty() {
        println!(
            "  {} PATH entry/entries stripped: {}",
            "[OK]".green(),
            report.stripped_path_entries.join(", ")
        );
    }
    for note in &report.deferred_actions {
        println!("  {} {}", "[i]".cyan(), note);
    }
    for w in &report.warnings {
        println!("  {} {}", "[!]".yellow(), w);
    }
    for e in &report.errors {
        println!("  {} {}", "[!]".red(), e);
    }

    println!();
    println!(
        "  Open a new terminal and run {} to confirm.",
        "ven --version".bold()
    );
    println!(
        "  (Expected: {} — the command should no longer be found.)",
        "command not found".dimmed()
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Confirmation prompt
// ─────────────────────────────────────────────────────────────────────────

fn confirm_destructive_action(plan: &UninstallPlan) -> Result<bool> {
    if !crate::core::runtime_bin::stdin_is_interactive() {
        // Uninstall is unique in the CLI: it does NOT auto-confirm on a
        // piped stdin the way `delete` does, because the blast radius is
        // an order of magnitude larger.
        return Err(anyhow!(
            "Refusing to uninstall on a non-interactive shell. Pass -y / --yes if this is intentional."
        ));
    }

    println!();
    println!(
        "  {} About to permanently delete the entire ven install:",
        "[DELETE]".red().bold()
    );
    if let Some(root) = &plan.user_install_root {
        println!("    {}", root.display());
    }
    if plan.data_dir_is_relocated && plan.data_dir.exists() {
        println!("    {}", plan.data_dir.display());
    }
    for art in &plan.system_artifacts {
        println!("    {}", art.display());
    }
    println!();

    let theme = ColorfulTheme::default();
    let ok = Confirm::with_theme(&theme)
        .with_prompt("Permanently remove ven and all installed runtimes?")
        .default(false)
        .interact()?;
    Ok(ok)
}

// ─────────────────────────────────────────────────────────────────────────
// JSON output
// ─────────────────────────────────────────────────────────────────────────

fn emit_json(plan: &UninstallPlan, dry_run: bool, yes: bool) -> Result<()> {
    let plan_json = plan_to_json(plan);

    if dry_run {
        let payload = json!({
            "status": "dry-run",
            "scope": format!("{:?}", plan.scope).to_lowercase(),
            "needs_elevation": plan.needs_elevation,
            "plan": plan_json,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    // Real execute path with --json -y.
    if plan.needs_elevation {
        let payload = json!({
            "status": "needs_elevation",
            "scope": format!("{:?}", plan.scope).to_lowercase(),
            "plan": plan_json,
            "hint": if cfg!(target_os = "windows") {
                "Re-run from an elevated PowerShell."
            } else {
                "Re-run with sudo."
            },
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        std::process::exit(1);
    }

    if is_plan_empty(plan) {
        let payload = json!({
            "status": "noop",
            "reason": "nothing_installed",
            "scope": format!("{:?}", plan.scope).to_lowercase(),
            "plan": plan_json,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    // Caller passed -y, so we proceed.
    let _ = yes;
    let report = uninstaller::execute_plan(plan, &ExecuteOptions { dry_run: false })?;
    let payload = json!({
        "status": if report.errors.is_empty() { "ok" } else { "partial" },
        "scope": format!("{:?}", plan.scope).to_lowercase(),
        "plan": plan_json,
        "report": report,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    if !report.errors.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

fn plan_to_json(plan: &UninstallPlan) -> serde_json::Value {
    json!({
        "user_install_root": plan.user_install_root.as_ref().map(|p| p.display().to_string()),
        "system_artifacts": plan.system_artifacts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "data_dir": plan.data_dir.display().to_string(),
        "data_dir_source": plan.data_dir_source.kind(),
        "data_dir_is_relocated": plan.data_dir_is_relocated,
        "pointer_file": plan.pointer_file.as_ref().map(|p| p.display().to_string()),
        "user_path_entries": plan.user_path_entries.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "system_path_entries": plan.system_path_entries.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "user_env_vars": plan.user_env_vars,
        "rc_files_to_clean": plan.rc_files_to_clean.iter()
            .filter(|p| p.is_file())
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>(),
        "needs_elevation": plan.needs_elevation,
        "current_exe": plan.current_exe.as_ref().map(|p| p.display().to_string()),
    })
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lock_test_env as lock_env;

    /// Drop-guard that scrubs the named env vars to a known starting
    /// state and restores them on Drop. Same shape used by
    /// `core::ven_home::tests`.
    struct EnvGuard {
        keys: Vec<&'static str>,
        prev: Vec<(&'static str, Option<String>)>,
    }
    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            let prev = keys
                .iter()
                .map(|k| (*k, std::env::var(k).ok()))
                .collect::<Vec<_>>();
            for k in keys {
                std::env::remove_var(k);
            }
            Self {
                keys: keys.to_vec(),
                prev,
            }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for k in &self.keys {
                std::env::remove_var(k);
            }
            for (k, v) in &self.prev {
                if let Some(val) = v {
                    std::env::set_var(k, val);
                }
            }
        }
    }

    #[test]
    fn json_mode_without_yes_or_dry_run_errors() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        let err = cmd_uninstall(false, true, false, false, false)
            .expect_err("plain --json must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("--dry-run") && msg.contains("-y"),
            "error should mention both opt-ins; got: {msg}"
        );
    }

    #[test]
    fn user_only_and_system_only_are_mutually_exclusive() {
        let err = cmd_uninstall(true, false, false, true, true)
            .expect_err("conflicting scope flags must be rejected");
        assert!(err.to_string().contains("mutually exclusive"));
    }
}
