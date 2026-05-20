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
        let has_display = std::env::var_os("DISPLAY").is_some()
            || std::env::var_os("WAYLAND_DISPLAY").is_some();
        if !has_display {
            return true;
        }
    }
    false
}

fn dispatch_cli(cli: common::SetupCli, mode: common::InstallMode) -> Result<()> {
    #[cfg(windows)]
    {
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
