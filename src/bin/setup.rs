//! `ven-setup` installer (Windows user install, no admin).
//!
//! Plan 2 / Option A:
//! 1) Copy `ven.exe` + `ven-launcher.exe` to `%USERPROFILE%\\.ven\\bin`
//! 2) Add that path to HKCU user PATH and broadcast WM_SETTINGCHANGE
//! 3) Run `ven setup` to install shell hooks
//! 4) Verify `ven --version` in a new child process

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    name = "ven-setup",
    about = "Windows user-mode installer for ven (no admin)",
    long_about = "Installs ven binaries to %USERPROFILE%\\.ven\\bin, updates user PATH, installs shell hooks, and verifies `ven --version`."
)]
struct SetupCli {
    /// Print actions without modifying the machine.
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = SetupCli::parse();

    #[cfg(not(windows))]
    {
        anyhow::bail!("ven-setup is currently supported on Windows only.");
    }

    #[cfg(windows)]
    {
        run_windows_user_install(cli.dry_run)
    }
}

#[cfg(windows)]
fn run_windows_user_install(dry_run: bool) -> Result<()> {
    println!("ven-setup (User Install, no admin)");

    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve user home directory"))?;
    let install_dir = home.join(".ven").join("bin");
    if !dry_run {
        fs::create_dir_all(&install_dir)
            .with_context(|| format!("Failed to create {}", install_dir.display()))?;
    }

    let exe = std::env::current_exe().context("Cannot resolve current executable path")?;
    let src_dir = exe
        .parent()
        .ok_or_else(|| anyhow!("Cannot resolve installer directory"))?;

    println!("\n[1/4] Copying binaries");
    let ven_exe = copy_required_binary(src_dir, &install_dir, "ven.exe", dry_run)?;
    let _launcher_exe = copy_required_binary(src_dir, &install_dir, "ven-launcher.exe", dry_run)?;
    println!("  [OK] Installed to {}", install_dir.display());

    println!("\n[2/4] Updating user PATH (HKCU)");
    if !dry_run {
        ensure_user_path_contains(&install_dir)?;
    }
    println!("  [OK] User PATH contains {}", install_dir.display());

    println!("\n[3/4] Installing shell hooks");
    if !dry_run {
        run_ven_setup(&ven_exe)?;
    }
    println!("  [OK] Shell hook setup executed");

    println!("\n[4/4] Verifying `ven --version` in new process");
    if !dry_run {
        let ver = verify_ven_version(&install_dir)?;
        println!("  [OK] {}", ver.trim());
    } else {
        println!("  [OK] dry-run");
    }

    println!("\nDone. Open a new terminal and run: ven --version");
    Ok(())
}

#[cfg(windows)]
fn copy_required_binary(src_dir: &Path, install_dir: &Path, name: &str, dry_run: bool) -> Result<PathBuf> {
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
fn ensure_user_path_contains(path_entry: &Path) -> Result<()> {
    let entry = path_entry.to_string_lossy().to_string();
    let entry_ps = entry.replace('\'', "''");

    // Update HKCU user PATH and broadcast WM_SETTINGCHANGE in one script.
    let script = format!(
        r#"$target = '{entry_ps}'
$current = [Environment]::GetEnvironmentVariable('Path', 'User')
if ([string]::IsNullOrWhiteSpace($current)) {{
  $new = $target
}} elseif ($current -split ';' | Where-Object {{ $_.Trim().ToLowerInvariant() -eq $target.ToLowerInvariant() }}) {{
  $new = $current
}} else {{
  $new = $current.TrimEnd(';') + ';' + $target
}}
[Environment]::SetEnvironmentVariable('Path', $new, 'User')

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Native {{
  [DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
  public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
}}
'@
$HWND_BROADCAST = [IntPtr]0xffff
$WM_SETTINGCHANGE = 0x001A
[UIntPtr]$result = [UIntPtr]::Zero
[Win32.Native]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result) | Out-Null"#,
    );

    let status = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .context("Failed to run PowerShell for PATH update")?;
    if !status.success() {
        anyhow::bail!("Failed to update user PATH in HKCU");
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

#[cfg(windows)]
fn verify_ven_version(install_dir: &Path) -> Result<String> {
    // Simulate a fresh terminal by running a new shell process with updated PATH.
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

