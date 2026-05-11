//! `ven-setup` installer for Windows.
//!
//! Modes:
//! - **User**   (default, no admin): copies binaries to `%USERPROFILE%\.ven\bin`,
//!   updates HKCU `Path`, broadcasts `WM_SETTINGCHANGE`, runs `ven setup`, verifies.
//! - **System** (requires admin):    copies binaries to `%ProgramFiles%\ven\bin`,
//!   updates HKLM Machine `Path`, broadcasts `WM_SETTINGCHANGE`, runs `ven setup`, verifies.
//!
//! System mode triggers a UAC prompt and relaunches itself elevated when needed.

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
#[cfg(windows)]
use dialoguer::theme::ColorfulTheme;
#[cfg(windows)]
use dialoguer::Select;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum InstallMode {
    /// Per-user install under `%USERPROFILE%\.ven\bin` (no admin required).
    User,
    /// Machine-wide install under `%ProgramFiles%\ven\bin` (requires admin / UAC).
    System,
}

#[derive(Parser, Debug)]
#[command(
    name = "ven-setup",
    about = "Windows installer for ven (user-mode, no admin; or system-wide with UAC)",
    long_about = "Installs ven binaries, updates PATH (User or Machine scope), installs shell hooks, and verifies `ven --version`."
)]
struct SetupCli {
    /// Install mode. Omit to choose interactively (1 = User, 2 = System).
    #[arg(long, value_enum)]
    mode: Option<InstallMode>,

    /// Print actions without touching the file system, registry, or running child processes.
    #[arg(long)]
    dry_run: bool,

    /// Skip the interactive mode prompt; `--mode` must then be supplied.
    #[arg(long)]
    no_input: bool,

    /// Internal flag set on the elevated child after a UAC relaunch (prevents loops).
    #[arg(long, hide = true)]
    elevated_child: bool,
}

fn main() -> Result<()> {
    let cli = SetupCli::parse();

    #[cfg(not(windows))]
    {
        let _ = cli;
        anyhow::bail!("ven-setup is currently supported on Windows only.");
    }

    #[cfg(windows)]
    {
        run_windows(cli)
    }
}

// ---------------------------------------------------------------------------
// Top-level Windows flow
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn run_windows(cli: SetupCli) -> Result<()> {
    print_banner(cli.elevated_child);

    let mode = resolve_mode(&cli)?;

    match mode {
        InstallMode::User => install_user(cli.dry_run),
        InstallMode::System => {
            // Real system installs require admin; relaunch via UAC if needed.
            // Dry-runs do not modify anything and intentionally skip elevation.
            if !cli.dry_run && !cli.elevated_child && !is_elevated()? {
                return relaunch_elevated_system();
            }
            let result = install_system(cli.dry_run);
            if cli.elevated_child {
                pause_for_user();
            }
            result
        }
    }
}

#[cfg(windows)]
fn print_banner(elevated_child: bool) {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("  +-----------------------------------------+");
    println!("  |  Welcome to Ven Installer               |");
    println!("  |  Version {:<31}|", version);
    println!("  +-----------------------------------------+");
    if elevated_child {
        println!("  (elevated child — system install)");
    }
    println!();
}

// ---------------------------------------------------------------------------
// Mode selection
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn resolve_mode(cli: &SetupCli) -> Result<InstallMode> {
    if let Some(m) = cli.mode {
        return Ok(m);
    }
    if cli.elevated_child {
        // Elevated child must be invoked with explicit --mode; bail loudly to avoid loops.
        anyhow::bail!("--mode is required for the elevated child process");
    }
    if cli.no_input {
        anyhow::bail!("--mode <user|system> is required when --no-input is set");
    }
    prompt_mode_interactive()
}

#[cfg(windows)]
fn prompt_mode_interactive() -> Result<InstallMode> {
    let theme = ColorfulTheme::default();
    let selection = Select::with_theme(&theme)
        .with_prompt("Select install mode")
        .item("User Install (recommended) — no admin, only for you")
        .item("System Install — requires admin, all users on this machine")
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
// Install flows
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn install_user(dry_run: bool) -> Result<()> {
    println!("ven-setup: User Install (no admin)");
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve user home directory"))?;
    let install_dir = home.join(".ven").join("bin");
    do_install(&install_dir, PathScope::User, dry_run)
}

#[cfg(windows)]
fn install_system(dry_run: bool) -> Result<()> {
    println!("ven-setup: System Install (admin)");
    let program_files = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    let install_dir = program_files.join("ven").join("bin");
    do_install(&install_dir, PathScope::Machine, dry_run)
}

#[cfg(windows)]
fn do_install(install_dir: &Path, scope: PathScope, dry_run: bool) -> Result<()> {
    if !dry_run {
        fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create {}", install_dir.display()))?;
    }

    let exe = std::env::current_exe().context("Cannot resolve current executable path")?;
    let src_dir = exe
        .parent()
        .ok_or_else(|| anyhow!("Cannot resolve installer directory"))?;

    println!("\n[1/4] Copying binaries");
    let ven_exe = copy_required_binary(src_dir, install_dir, "ven.exe", dry_run)?;
    let _launcher_exe = copy_required_binary(src_dir, install_dir, "ven-launcher.exe", dry_run)?;
    println!("  [OK] Installed to {}", install_dir.display());

    println!("\n[2/4] Updating {} PATH", scope.label());
    if !dry_run {
        ensure_path_contains(install_dir, scope)?;
    }
    println!(
        "  [OK] {} PATH contains {}",
        scope.label(),
        install_dir.display()
    );

    println!("\n[3/4] Installing shell hooks");
    if !dry_run {
        run_ven_setup(&ven_exe)?;
    }
    println!("  [OK] Shell hook setup executed");

    println!("\n[4/4] Verifying `ven --version` in a new process");
    if !dry_run {
        let ver = verify_ven_version(install_dir)?;
        println!("  [OK] {}", ver.trim());
    } else {
        println!("  [OK] dry-run");
    }

    println!("\nDone. Open a new terminal and run: ven --version");
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared helpers (binaries, PATH, broadcast, verify)
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn copy_required_binary(
    src_dir: &Path,
    install_dir: &Path,
    name: &str,
    dry_run: bool,
) -> Result<PathBuf> {
    let src = src_dir.join(name);
    if !src.is_file() {
        anyhow::bail!(
            "Required binary '{}' was not found next to ven-setup.exe at {}",
            name,
            src.display()
        );
    }
    let dst = install_dir.join(name);
    if !dry_run {
        fs::copy(&src, &dst)
            .with_context(|| format!("Failed to copy {} -> {}", src.display(), dst.display()))?;
    }
    Ok(dst)
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug)]
enum PathScope {
    User,
    Machine,
}

#[cfg(windows)]
impl PathScope {
    fn label(self) -> &'static str {
        match self {
            PathScope::User => "User",
            PathScope::Machine => "Machine",
        }
    }
}

/// Append `path_entry` to the given PATH scope (HKCU or HKLM Machine) if missing,
/// then broadcast `WM_SETTINGCHANGE` so already-open Explorer/process trees see it.
#[cfg(windows)]
fn ensure_path_contains(path_entry: &Path, scope: PathScope) -> Result<()> {
    let entry = path_entry.to_string_lossy().to_string();
    let entry_ps = entry.replace('\'', "''");
    let scope_ps = scope.label();

    // PowerShell is used to (a) read & write the correct registry hive via
    // [Environment]::SetEnvironmentVariable, which avoids REG_MULTI_SZ pitfalls,
    // and (b) broadcast WM_SETTINGCHANGE via P/Invoke so the change propagates
    // to running shells without requiring a sign-out.
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
        anyhow::bail!("Failed to update {} PATH", scope.label());
    }
    Ok(())
}

#[cfg(windows)]
fn run_ven_setup(ven_exe: &Path) -> Result<()> {
    let status = Command::new(ven_exe)
        .arg("setup")
        .status()
        .with_context(|| format!("Failed to run {}", ven_exe.display()))?;
    if !status.success() {
        anyhow::bail!("`ven setup` failed");
    }
    Ok(())
}

/// Spawn `cmd /C ven --version` with `PATH = {install_dir};{current_PATH}` so the
/// check matches what a brand-new terminal will see, even if the broadcast hasn't
/// yet reached this process tree.
#[cfg(windows)]
fn verify_ven_version(install_dir: &Path) -> Result<String> {
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
// Elevation (UAC) handling
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn is_elevated() -> Result<bool> {
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

/// Re-spawn the current `ven-setup.exe` with `Start-Process -Verb RunAs`, passing
/// `--mode system --elevated-child` so the elevated process skips the prompt and
/// does not recurse into another UAC relaunch.
#[cfg(windows)]
fn relaunch_elevated_system() -> Result<()> {
    let exe = std::env::current_exe().context("Cannot resolve current executable path")?;
    let exe_ps = exe.to_string_lossy().replace('\'', "''");

    println!("\nSystem install requires administrator privileges.");
    println!("Approve the UAC prompt to continue. A new elevated window will open.");

    let script = format!(
        r#"Start-Process -FilePath '{exe_ps}' -ArgumentList '--mode','system','--elevated-child' -Verb RunAs"#,
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

#[cfg(windows)]
fn pause_for_user() {
    use std::io::{stdin, BufRead, Write};
    print!("\nPress Enter to close this window...");
    let _ = std::io::stdout().flush();
    let mut buf = String::new();
    let _ = stdin().lock().read_line(&mut buf);
}
