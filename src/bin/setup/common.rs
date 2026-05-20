//! Shared CLI, banner, mode prompt, and binary-embedding helpers.

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Embedded binary payloads
// ---------------------------------------------------------------------------
//
// `build.rs` populates `OUT_DIR/{ven,ven-launcher}.bin`. On a clean checkout
// these blobs are zero-length stubs; we fall back to sibling files in that
// case so `cargo run --bin ven-setup` still works during development.
// After `cargo build --release` has produced the real artifacts a second
// build pass will pick them up and embed them properly.

pub const VEN_EMBEDDED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ven.bin"));
pub const LAUNCHER_EMBEDDED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ven-launcher.bin"));

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallMode {
    /// Per-user install. No admin / sudo required.
    User,
    /// Machine-wide install. Windows: UAC. Unix: must run as root / sudo.
    System,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "ven-setup",
    about = "Cross-platform installer for ven (GUI wizard by default; --cli for headless / SSH / CI)",
    long_about = "Installs `ven` and `ven-launcher` by extracting binaries embedded in this installer, \
                  updates PATH (per-user or machine-wide), installs shell hooks, optionally pre-installs \
                  selected runtimes, and verifies `ven --version`. Opens a native GUI wizard by default; \
                  falls back to the CLI flow with --cli, --no-input, or when no display server is reachable."
)]
pub struct SetupCli {
    /// Install mode. Omit to choose interactively (GUI wizard or CLI prompt).
    #[arg(long, value_enum)]
    pub mode: Option<InstallMode>,

    /// Print every step without writing files, modifying the registry / rc files,
    /// running child processes, or requesting elevation.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the interactive prompt; `--mode` must then be supplied. For CI / automation.
    /// Implies `--cli`.
    #[arg(long)]
    pub no_input: bool,

    /// Force the legacy CLI flow even when a graphical session is available.
    /// Useful for SSH, CI, or anywhere a GUI window is undesirable.
    #[arg(long)]
    pub cli: bool,

    /// Override the storage directory ($VEN_HOME) the install will configure.
    /// When unset the default `~/.ven` (or relocated value from a prior `ven path set`) is kept.
    #[arg(long, value_name = "PATH")]
    pub storage_path: Option<PathBuf>,

    /// Pre-install one or more language runtimes after the core install completes.
    /// Comma-separated list of language slugs (e.g. `node,python,go`).
    #[arg(long, value_name = "LANGS", value_delimiter = ',')]
    pub with_runtimes: Vec<String>,

    /// Disable the shell-hook install step (no auto-activation on cd / new terminals).
    #[arg(long)]
    pub no_hook: bool,

    /// Disable the PATH update step (the user takes responsibility for putting
    /// the install dir on PATH themselves).
    #[arg(long)]
    pub no_path: bool,

    /// Resume an install from a TOML config file written by the parent process.
    /// Used by the Windows UAC relaunch / Unix sudo re-invocation flows so the
    /// elevated child preserves the choices the user made in the GUI wizard.
    /// Combined with `--elevated-child` it bypasses the GUI entirely.
    #[arg(long, value_name = "PATH", hide = true)]
    pub resume: Option<PathBuf>,

    /// Internal flag set on the elevated child after a Windows UAC relaunch (loop guard).
    #[arg(long, hide = true)]
    pub elevated_child: bool,
}

// ---------------------------------------------------------------------------
// Banner + mode prompt
// ---------------------------------------------------------------------------

pub fn print_banner(elevated_child: bool) {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("  +-----------------------------------------+");
    println!("  |  Welcome to Ven Installer               |");
    println!("  |  Version {:<31}|", version);
    println!("  +-----------------------------------------+");
    if elevated_child {
        println!("  (elevated child -- system install)");
    }
    println!();
}

pub fn resolve_mode(cli: &SetupCli) -> Result<InstallMode> {
    if let Some(m) = cli.mode {
        return Ok(m);
    }
    if cli.elevated_child {
        anyhow::bail!("--mode is required for the elevated child process");
    }
    if cli.no_input {
        anyhow::bail!("--mode <user|system> is required when --no-input is set");
    }
    prompt_mode_interactive()
}

fn prompt_mode_interactive() -> Result<InstallMode> {
    let theme = ColorfulTheme::default();
    let selection = Select::with_theme(&theme)
        .with_prompt("Select install mode")
        .item("User Install (recommended) -- no admin / sudo, only for you")
        .item("System Install -- admin / sudo required, all users on this machine")
        .default(0)
        .interact()
        .context("Failed to read interactive selection")?;
    Ok(if selection == 0 {
        InstallMode::User
    } else {
        InstallMode::System
    })
}

// ---------------------------------------------------------------------------
// Binary extraction helpers
// ---------------------------------------------------------------------------

/// Resolve the bytes for one of the bundled binaries:
/// 1. If embedded at build time (non-empty), use that payload.
/// 2. Otherwise fall back to a sibling file next to `ven-setup`.
pub fn resolve_binary_bytes(name: &str, embedded: &'static [u8]) -> Result<Vec<u8>> {
    if !embedded.is_empty() {
        return Ok(embedded.to_vec());
    }
    let exe = std::env::current_exe().context("Cannot resolve current executable path")?;
    let src_dir = exe
        .parent()
        .ok_or_else(|| anyhow!("Cannot resolve installer directory"))?;
    let src = src_dir.join(name);
    if !src.is_file() {
        anyhow::bail!(
            "Embedded payload for '{name}' is empty AND no sibling '{name}' was found at {}. \
             For a release build: run `cargo build --release --bin ven --bin ven-launcher` first, \
             then `cargo build --release --bin ven-setup` (two passes so build.rs picks up the artifacts). \
             For dev: place the binary next to ven-setup.",
            src.display()
        );
    }
    fs::read(&src).with_context(|| format!("Failed to read fallback {}", src.display()))
}

/// Write `bytes` to `install_dir/name`, creating parent dirs as needed.
/// On Unix the file is marked `0o755`.
pub fn write_bundled_binary(
    install_dir: &Path,
    name: &str,
    bytes: &[u8],
    dry_run: bool,
) -> Result<PathBuf> {
    let dst = install_dir.join(name);
    if !dry_run {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        fs::write(&dst, bytes).with_context(|| format!("Failed to write {}", dst.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dst)
                .with_context(|| format!("Failed to stat {}", dst.display()))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dst, perms)
                .with_context(|| format!("Failed to chmod {}", dst.display()))?;
        }
    }
    Ok(dst)
}

/// Spawn the freshly-installed `ven` and ask it to install shell hooks.
///
/// Currently unused — `install_steps::step_install_hook` invokes the new
/// `ven` binary directly. Kept as a small wrapper for any caller that
/// wants to re-run `ven setup` post-install (e.g. the GUI's "Install
/// shell hook only" repair path) without duplicating the spawn boilerplate.
#[allow(dead_code)]
pub fn run_ven_setup(ven_exe: &Path) -> Result<()> {
    let status = std::process::Command::new(ven_exe)
        .arg("setup")
        .status()
        .with_context(|| format!("Failed to run {}", ven_exe.display()))?;
    if !status.success() {
        anyhow::bail!("`ven setup` failed");
    }
    Ok(())
}
