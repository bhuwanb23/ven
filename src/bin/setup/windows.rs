//! Windows-specific install logic for `ven-setup`.
//!
//! - **User**:   `%USERPROFILE%\.ven\bin`, HKCU `Path`, no UAC.
//! - **System**: `%ProgramFiles%\ven\bin`, HKLM Machine `Path`, requires elevation;
//!   the non-elevated parent relaunches itself via `Start-Process -Verb RunAs` and exits.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::{
    resolve_binary_bytes, run_ven_setup, write_bundled_binary, InstallMode, SetupCli,
    LAUNCHER_EMBEDDED, VEN_EMBEDDED,
};

pub fn run(cli: SetupCli, mode: InstallMode) -> Result<()> {
    match mode {
        InstallMode::User => install_user(cli.dry_run),
        InstallMode::System => {
            // Real system installs require admin; dry-runs intentionally skip elevation.
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

// ---------------------------------------------------------------------------
// Install flows
// ---------------------------------------------------------------------------

fn install_user(dry_run: bool) -> Result<()> {
    println!("ven-setup: User Install (no admin)");
    let install_dir = ven::core::ven_home::ven_home().join("bin");
    do_install(&install_dir, PathScope::User, dry_run)
}

fn install_system(dry_run: bool) -> Result<()> {
    println!("ven-setup: System Install (admin)");
    let program_files = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    let install_dir = program_files.join("ven").join("bin");
    do_install(&install_dir, PathScope::Machine, dry_run)
}

#[derive(Clone, Copy, Debug)]
enum PathScope {
    User,
    Machine,
}

impl PathScope {
    fn label(self) -> &'static str {
        match self {
            PathScope::User => "User",
            PathScope::Machine => "Machine",
        }
    }
}

fn do_install(install_dir: &Path, scope: PathScope, dry_run: bool) -> Result<()> {
    println!("\n[1/4] Extracting and writing binaries");
    let ven_bytes = resolve_binary_bytes("ven.exe", VEN_EMBEDDED)?;
    let launcher_bytes = resolve_binary_bytes("ven-launcher.exe", LAUNCHER_EMBEDDED)?;
    let ven_exe = write_bundled_binary(install_dir, "ven.exe", &ven_bytes, dry_run)?;
    let _launcher_exe =
        write_bundled_binary(install_dir, "ven-launcher.exe", &launcher_bytes, dry_run)?;
    println!(
        "  [OK] Installed to {} (ven {} B + launcher {} B)",
        install_dir.display(),
        ven_bytes.len(),
        launcher_bytes.len()
    );

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
// PATH + broadcast
// ---------------------------------------------------------------------------

/// Append `path_entry` to the given PATH scope (HKCU or HKLM Machine) if missing,
/// then broadcast `WM_SETTINGCHANGE` so already-running shells pick up the change.
///
/// We delegate the registry write to PowerShell's `[Environment]::SetEnvironmentVariable`
/// because it correctly handles the `REG_EXPAND_SZ` vs `REG_SZ` distinction; manual
/// `RegSetValueEx` calls can corrupt the system `Path` if the value type is wrong.
fn ensure_path_contains(path_entry: &Path, scope: PathScope) -> Result<()> {
    let entry = path_entry.to_string_lossy().to_string();
    let entry_ps = entry.replace('\'', "''");
    let scope_ps = scope.label();

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

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

/// Spawn `cmd /C ven --version` with `PATH = {install_dir};{current_PATH}` so the
/// check matches a brand-new terminal even before the broadcast reaches this tree.
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
// Elevation (UAC)
// ---------------------------------------------------------------------------

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

/// Re-spawn `ven-setup.exe` with `Start-Process -Verb RunAs`, passing
/// `--mode system --elevated-child` so the elevated child skips prompting
/// and cannot recurse into another UAC relaunch.
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

fn pause_for_user() {
    use std::io::{stdin, BufRead, Write};
    print!("\nPress Enter to close this window...");
    let _ = std::io::stdout().flush();
    let mut buf = String::new();
    let _ = stdin().lock().read_line(&mut buf);
}
