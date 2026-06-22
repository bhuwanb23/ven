//! Windows-specific primitives for `ven-setup`.
//!
//! Through v0.1.x this module also owned the install orchestration. In v0.2
//! the orchestration moved to [`crate::install_steps`] so the GUI wizard and
//! the CLI share one pipeline; this file now only exposes the *platform*
//! primitives that pipeline calls into:
//!
//! - [`PathScope`] / [`ensure_path_contains`] — HKCU vs HKLM PATH editing
//!   via PowerShell `[Environment]::SetEnvironmentVariable` (correct
//!   `REG_EXPAND_SZ` handling) + `WM_SETTINGCHANGE` broadcast.
//! - [`verify_ven_version`] — child process with merged PATH for the
//!   verify step.
//! - [`is_elevated`] / [`relaunch_elevated_system`] — UAC detection and
//!   the `Start-Process -Verb RunAs` relaunch. v0.2 extends the relaunch
//!   so the GUI can hand the elevated child a TOML resume file with the
//!   user's prior choices.
//! - [`run`] — the legacy CLI entry point. Builds an
//!   [`InstallConfig`](crate::install_steps::InstallConfig) and drives the
//!   shared pipeline with a [`CliSink`](crate::install_steps::CliSink).

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::common::{detect_existing_installs, prompt_existing_install_cli, InstallMode, SetupCli};
use crate::install_steps::{self, CliSink, InstallConfig};

// ---------------------------------------------------------------------------
// Console reattachment (companion to `windows_subsystem = "windows"`)
// ---------------------------------------------------------------------------

/// Attach this process to the parent terminal's console so `println!`
/// output reaches the PowerShell / cmd window the user launched us from.
///
/// A `windows_subsystem = "windows"` binary is started without a console;
/// without this, every `print!` and `eprint!` is silently dropped when
/// the user runs `ven-setup --cli ...` from a terminal. We call
/// `AttachConsole(ATTACH_PARENT_PROCESS)`, which:
///
/// 1. Inherits the parent shell's console for the lifetime of the
///    process, and
/// 2. For a freshly-spawned windows-subsystem child whose std handles
///    were not pre-redirected, transparently re-points
///    `STD_{INPUT,OUTPUT,ERROR}_HANDLE` at `CONIN$` / `CONOUT$`.
///
/// That covers the common case of double-clicking from Explorer (no
/// parent console, no-op) and launching from PowerShell / cmd
/// (parent console adopted, banner + step output flow through).
///
/// We deliberately don't `freopen` or `SetStdHandle` ourselves: every
/// extra knob is a chance to break the rare cases where the parent
/// already piped stdio (e.g. `ven-setup --cli | tee log.txt`), and
/// AttachConsole alone is the pattern used by `pwsh.exe`,
/// `git.exe` 2.x, and the Python launcher.
///
/// Idempotent: a second call is a no-op. Best-effort: if AttachConsole
/// fails (no parent console at all) we leave the streams untouched,
/// which is the GUI default behaviour.
pub fn attach_parent_console() {
    static ATTACHED: AtomicBool = AtomicBool::new(false);
    if ATTACHED.swap(true, Ordering::SeqCst) {
        return;
    }
    // Safety: AttachConsole is a thread-safe Win32 entry point that
    // tolerates "no parent console" as a clean failure (return 0 + last
    // error = ERROR_INVALID_HANDLE). We only call it once per process
    // thanks to the AtomicBool above.
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

// ---------------------------------------------------------------------------
// Zone.Identifier (MoTW) stripping
// ---------------------------------------------------------------------------

/// Remove the `Zone.Identifier` alternate data stream from the current
/// executable. Files downloaded from the internet carry this stream, which
/// causes Windows SmartScreen to prompt "Windows protected your PC".
/// Removing it at startup prevents blocking after the user has already
/// allowed the app once.
///
/// Best-effort: silently ignores all errors (stream absent, no permission,
/// non-NTFS volume, etc.).
pub fn strip_zone_identifier() {
    use windows_sys::Win32::Storage::FileSystem::DeleteFileW;

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    // NTFS ADS path: "C:\path\to\ven-setup.exe:Zone.Identifier"
    let stream_path = format!("{}:Zone.Identifier", exe.display());
    let wide: Vec<u16> = stream_path
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        // DeleteFileW returns non-zero on success. Silently ignore failure
        // (stream missing, not elevated, etc.).
        let _ = DeleteFileW(wide.as_ptr());
    }
}

/// Show a Win32 message box with the given title and message.
///
/// Used to surface errors when there is no console attached (the common
/// double-click-from-Explorer case).
pub fn show_message_box(title: &str, message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(
            0,
            msg_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

// ---------------------------------------------------------------------------
// CLI driver (legacy + auto-fallback)
// ---------------------------------------------------------------------------

/// CLI entry point: builds an [`InstallConfig`] from `cli` + `mode`, handles
/// the UAC relaunch if needed, then drives the shared install pipeline.
pub fn run(cli: SetupCli, mode: InstallMode) -> Result<()> {
    // Resume-file path: the GUI wrote a TOML with the user's choices and
    // re-spawned us elevated. Load the saved config instead of reading
    // CLI flags so the choices survive UAC.
    let mut cfg = if let Some(resume) = cli.resume.as_deref() {
        InstallConfig::load_from_file(resume).with_context(|| {
            format!(
                "Failed to load resume file at {} (the elevated relaunch handoff is broken)",
                resume.display()
            )
        })?
    } else {
        build_config_from_cli(&cli, mode)
    };

    // Honour explicit CLI overrides even when resuming (e.g. --dry-run).
    if cli.dry_run {
        cfg.dry_run = true;
    }

    // Check for existing installations before proceeding.
    if !cfg.dry_run {
        let existing = detect_existing_installs();
        if !prompt_existing_install_cli(&existing) {
            println!("  Setup cancelled by user.");
            return Ok(());
        }
    }

    if matches!(cfg.mode, InstallMode::System)
        && !cfg.dry_run
        && !cli.elevated_child
        && !is_elevated()?
    {
        relaunch_elevated_system(&cfg)?;
        return Ok(());
    }

    let result = drive_install(&cfg);

    if cli.elevated_child {
        pause_for_user();
    }
    result
}

fn build_config_from_cli(cli: &SetupCli, mode: InstallMode) -> InstallConfig {
    let mut cfg = InstallConfig::default_for_mode(mode);
    cfg.dry_run = cli.dry_run;
    if cli.no_path {
        cfg.add_to_path = false;
    }
    if cli.no_hook {
        cfg.install_hook = false;
    }
    if let Some(p) = cli.storage_path.clone() {
        cfg.storage_path = Some(p);
    }
    if !cli.with_runtimes.is_empty() {
        cfg.runtimes_to_install = cli.with_runtimes.clone();
    }
    cfg
}

fn drive_install(cfg: &InstallConfig) -> Result<()> {
    println!(
        "ven-setup: {} Install ({})",
        match cfg.mode {
            InstallMode::User => "User",
            InstallMode::System => "System",
        },
        if cfg.dry_run { "dry-run" } else { "live" }
    );
    let mut sink = CliSink;
    install_steps::run(cfg, &mut sink).map(|_| ())
}

// ---------------------------------------------------------------------------
// PATH wiring (exposed so install_steps + the GUI elevation code can call in)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum PathScope {
    User,
    Machine,
}

impl PathScope {
    pub fn label(self) -> &'static str {
        match self {
            PathScope::User => "User",
            PathScope::Machine => "Machine",
        }
    }
}

/// Append `path_entry` to the given PATH scope (HKCU or HKLM Machine) if
/// missing, then broadcast `WM_SETTINGCHANGE` so already-running shells
/// pick up the change.
///
/// We delegate to PowerShell's `[Environment]::SetEnvironmentVariable`
/// because it handles the `REG_EXPAND_SZ` vs `REG_SZ` distinction
/// correctly — manual `RegSetValueEx` writes have historically corrupted
/// the system PATH on locked-down hosts.
pub fn ensure_path_contains(path_entry: &Path, scope: PathScope) -> Result<()> {
    let entry = path_entry.to_string_lossy().to_string();
    let entry_ps = entry.replace('\'', "''");
    let scope_ps = scope.label();

    // Read the current PATH before modification so we can restore on failure.
    let current_path = read_registry_path(scope)?;

    let script = format!(
        r#"$target = '{entry_ps}'
$scope = '{scope_ps}'
$current = [Environment]::GetEnvironmentVariable('Path', $scope)
if ([string]::IsNullOrWhiteSpace($current)) {{
  $new = $target
}} elseif ($current -split ';' | Where-Object {{ $_.Trim().ToLowerInvariant() -eq $target.ToLowerInvariant() }}) {{
  $new = $current
}} else {{
  $new = $current.TrimEnd(';') + ';' + $target
}}
[Environment]::SetEnvironmentVariable('Path', $new, $scope)

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
namespace Win32 {{
  public static class Native {{
    [DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern IntPtr SendMessageTimeout(
      IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
      uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
  }}
}}
'@
$HWND_BROADCAST = [IntPtr]0xffff
$WM_SETTINGCHANGE = 0x001A
[UIntPtr]$result = [UIntPtr]::Zero
[Win32.Native]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result) | Out-Null"#,
    );

    let status = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .status()
        .context("Failed to run PowerShell for PATH update")?;
    if !status.success() {
        // Attempt to restore the original PATH on failure.
        if let Err(restore_err) = write_registry_path(scope, &current_path) {
            eprintln!(
                "  [WARN] Failed to restore {} PATH: {}",
                scope.label(),
                restore_err
            );
        } else {
            eprintln!("  Restored {} PATH to previous value", scope.label());
        }
        anyhow::bail!("Failed to update {} PATH", scope.label());
    }
    Ok(())
}

/// Read the current PATH for the given scope from the Windows Registry.
fn read_registry_path(scope: PathScope) -> Result<String> {
    let scope_ps = scope.label();
    let script = format!(r#"[Environment]::GetEnvironmentVariable('Path', '{scope_ps}')"#,);
    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .output()
        .with_context(|| format!("Failed to read {} PATH", scope.label()))?;
    if !output.status.success() {
        anyhow::bail!(
            "Failed to read {} PATH: {}",
            scope.label(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Write a value back to the PATH in the given registry scope.
fn write_registry_path(scope: PathScope, value: &str) -> Result<()> {
    let scope_ps = scope.label();
    let value_ps = value.replace('\'', "''");
    let script =
        format!(r#"[Environment]::SetEnvironmentVariable('Path', '{value_ps}', '{scope_ps}')"#,);
    let status = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .status()
        .with_context(|| format!("Failed to restore {} PATH", scope.label()))?;
    if !status.success() {
        anyhow::bail!("PowerShell command to restore PATH failed");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

/// Spawn `cmd /C ven --version` with `PATH = {install_dir};{current_PATH}`
/// so the check matches a brand-new terminal even before the broadcast
/// reaches this process tree.
pub fn verify_ven_version(install_dir: &Path) -> Result<String> {
    let base = std::env::var("PATH").unwrap_or_default();
    let merged = format!("{};{}", install_dir.to_string_lossy(), base);

    let output = Command::new("cmd.exe")
        .args(["/C", "ven --version"])
        .env("PATH", merged)
        .output()
        .context("Failed to spawn verification process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Verification failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// Elevation (UAC)
// ---------------------------------------------------------------------------

/// Return `Ok(true)` when the current process is running with administrator
/// privileges. Probes via PowerShell's `WindowsPrincipal` to avoid pulling
/// in `winapi` / `windows-rs` just for this check.
pub fn is_elevated() -> Result<bool> {
    let script = r#"([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)"#;
    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .context("Failed to check elevation status")?;
    if !output.status.success() {
        anyhow::bail!(
            "Elevation check failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().eq_ignore_ascii_case("True"))
}

/// Re-spawn `ven-setup.exe` with `Start-Process -Verb RunAs`. The elevated
/// child receives `--elevated-child --resume <path>` where `<path>` is a
/// TOML file containing every choice the user made in the wizard (or the
/// CLI flags they passed) — so UAC doesn't lose state.
pub fn relaunch_elevated_system(cfg: &InstallConfig) -> Result<()> {
    let exe = std::env::current_exe().context("Cannot resolve current executable path")?;

    let resume_path = resume_file_path()?;
    cfg.save_to_file(&resume_path)
        .with_context(|| format!("Failed to write resume file {}", resume_path.display()))?;

    let exe_ps = exe.to_string_lossy().replace('\'', "''");
    let resume_ps = resume_path.to_string_lossy().replace('\'', "''");

    println!("\nSystem install requires administrator privileges.");
    println!("Approve the UAC prompt to continue. A new elevated window will open.");
    println!("  Resume file: {}", resume_path.display());

    let script = format!(
        r#"Start-Process -FilePath '{exe_ps}' -ArgumentList '--mode','system','--elevated-child','--resume','{resume_ps}' -Verb RunAs"#,
    );

    let status = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .status()
        .context("Failed to spawn elevated installer")?;

    if !status.success() {
        anyhow::bail!("UAC elevation was declined or failed");
    }

    println!("\nElevated installer launched. You can close this window.");
    Ok(())
}

/// Where the parent stashes the TOML resume file so the elevated child can
/// pick it up. `%TEMP%\ven-setup-resume.toml` — overwritten on every run.
pub fn resume_file_path() -> Result<PathBuf> {
    let dir = std::env::temp_dir();
    Ok(dir.join("ven-setup-resume.toml"))
}

fn pause_for_user() {
    use std::io::{stdin, BufRead, Write};
    print!("\nPress Enter to close this window...");
    let _ = std::io::stdout().flush();
    let mut buf = String::new();
    let _ = stdin().lock().read_line(&mut buf);
}
