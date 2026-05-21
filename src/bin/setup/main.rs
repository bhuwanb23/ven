//! `ven-setup` installer entry point.
//!
//! v0.2 dispatches between two front-ends that share one install pipeline:
//!
//! - **GUI wizard** (default, see [`gui`]) — an `eframe::App` that walks the
//!   user through eight screens: Welcome → Mode → Storage → Hook/PATH →
//!   Runtimes → Review → Progress → Done.
//! - **CLI flow** (legacy, see [`windows`] / [`unix`]) — the v0.1.x
//!   `dialoguer`-based prompt + linear install. Used for SSH / CI / headless
//!   contexts and triggered by `--cli`, `--no-input`, or auto-detected when
//!   no display server is reachable.
//!
//! Both front-ends ultimately call into [`install_steps::run`], so the
//! actual install logic exists exactly once.
//!
//! ## Windows subsystem
//!
//! In release builds we link `ven-setup.exe` against the Windows
//! *windows* subsystem (no console allocation) so a double-click does not
//! flash a stray `cmd.exe` window next to the wizard. When the user picks
//! the CLI flow (`--cli`, `--no-input`, or auto-detect on a headless host)
//! we call `windows::attach_parent_console()` *before* any println so
//! `irm | iex`-style invocations still print into the parent PowerShell.
//! Debug builds keep the default console subsystem so `cargo run` and
//! step-through debugging stay ergonomic.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use anyhow::Result;
use clap::Parser;

mod common;
mod install_steps;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(feature = "gui")]
mod gui;

fn main() -> Result<()> {
    let cli = common::SetupCli::parse();

    // Hook up the parent terminal early so any clap parse error or panic
    // before we reach `dispatch_cli` still surfaces in the launching shell.
    // Idempotent: a second `AttachConsole` call inside `dispatch_cli` is a
    // no-op since this process is already attached.
    #[cfg(windows)]
    if cli.cli || cli.no_input || cli.elevated_child {
        windows::attach_parent_console();
    }

    // Resume / elevated-child paths never show UI — they were spawned by
    // a parent that already knows the user's choices.
    let use_cli = should_use_cli(&cli);

    if use_cli {
        common::print_banner(cli.elevated_child);
        let mode = common::resolve_mode(&cli)?;
        return dispatch_cli(cli, mode);
    }

    #[cfg(feature = "gui")]
    {
        match gui::run(cli.clone()) {
            Ok(()) => Ok(()),
            Err(gui::GuiUnavailable) => {
                // The eframe initializer failed (no display server, headless
                // VM, X forwarding broken). Fall back to the CLI flow so a
                // double-click install on a broken display still succeeds
                // — surfacing the dialoguer prompts in the same terminal
                // ven-setup was launched from.
                eprintln!("ven-setup: no GUI session detected, falling back to CLI flow.");
                common::print_banner(cli.elevated_child);
                let mode = common::resolve_mode(&cli)?;
                dispatch_cli(cli, mode)
            }
        }
    }
    #[cfg(not(feature = "gui"))]
    {
        common::print_banner(cli.elevated_child);
        let mode = common::resolve_mode(&cli)?;
        dispatch_cli(cli, mode)
    }
}

/// Decide between the GUI wizard and the legacy CLI flow.
///
/// The CLI flow wins when ANY of the following holds:
/// - `--cli` was passed explicitly.
/// - `--no-input` was passed (CI / automation; would block waiting on the
///   GUI's Next button).
/// - `--elevated-child` is set (the parent already gathered choices; the
///   child resumes from `--resume` and does the install headlessly).
/// - The `gui` feature is compiled out.
/// - Unix and neither `$DISPLAY` nor `$WAYLAND_DISPLAY` are set (e.g. SSH
///   without X forwarding); the GUI couldn't open a window anyway.
// When the `gui` feature is off the early-`return true;` below makes the
// trailing `false` unreachable. That's intentional — the function still
// type-checks and behaves correctly — so we silence the lint just for that
// build configuration.
#[cfg_attr(not(feature = "gui"), allow(unreachable_code))]
fn should_use_cli(cli: &common::SetupCli) -> bool {
    if cli.cli || cli.no_input || cli.elevated_child {
        return true;
    }
    #[cfg(not(feature = "gui"))]
    {
        return true;
    }
    #[cfg(all(unix, feature = "gui"))]
    {
        let has_display =
            std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();
        if !has_display {
            return true;
        }
    }
    false
}

fn dispatch_cli(cli: common::SetupCli, mode: common::InstallMode) -> Result<()> {
    #[cfg(windows)]
    {
        // Re-attach to the parent console so banner / progress lines print
        // into the PowerShell or cmd window the user spawned us from. This
        // is the counterpart to `windows_subsystem = "windows"` above —
        // without it, `ven-setup.exe --cli ...` would silently swallow
        // stdout. No-op when there is no parent console (no harm done).
        windows::attach_parent_console();
        windows::run(cli, mode)
    }
    #[cfg(unix)]
    {
        unix::run(cli, mode)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (cli, mode);
        anyhow::bail!("ven-setup is supported on Windows and Unix-like systems only.");
    }
}
