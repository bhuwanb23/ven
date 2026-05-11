//! Unix-specific install logic for `ven-setup` (Linux / macOS).
//!
//! - **User**:   `~/.ven/bin`, PATH block appended to `~/.bashrc` / `~/.zshrc` (and
//!   `~/.profile` as fallback). No sudo. The user must open a new shell (or `exec $SHELL -l`).
//! - **System**: `/usr/local/bin`, PATH ensured via `/etc/profile.d/ven.sh`. Requires
//!   `sudo` -- there is no UAC equivalent; we refuse to proceed unelevated and print
//!   the exact re-invocation hint so the user can rerun under sudo.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::{
    resolve_binary_bytes, run_ven_setup, write_bundled_binary, InstallMode, SetupCli,
    LAUNCHER_EMBEDDED, VEN_EMBEDDED,
};

const VEN_RC_BLOCK_START: &str = "# >>> ven-setup PATH >>>";
const VEN_RC_BLOCK_END: &str = "# <<< ven-setup PATH <<<";

pub fn run(cli: SetupCli, mode: InstallMode) -> Result<()> {
    match mode {
        InstallMode::User => install_user(cli.dry_run),
        InstallMode::System => {
            if !cli.dry_run && !is_root() {
                let exe = std::env::current_exe()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| "ven-setup".to_string());
                anyhow::bail!(
                    "System install requires root. Re-run with:\n    sudo {} --mode system",
                    exe
                );
            }
            install_system(cli.dry_run)
        }
    }
}

// ---------------------------------------------------------------------------
// Install flows
// ---------------------------------------------------------------------------

fn install_user(dry_run: bool) -> Result<()> {
    println!("ven-setup: User Install (no sudo)");
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve user home directory"))?;
    let install_dir = home.join(".ven").join("bin");

    println!("\n[1/4] Extracting and writing binaries");
    let ven_bytes = resolve_binary_bytes("ven", VEN_EMBEDDED)?;
    let launcher_bytes = resolve_binary_bytes("ven-launcher", LAUNCHER_EMBEDDED)?;
    let ven_exe = write_bundled_binary(&install_dir, "ven", &ven_bytes, dry_run)?;
    let _launcher = write_bundled_binary(&install_dir, "ven-launcher", &launcher_bytes, dry_run)?;
    println!(
        "  [OK] Installed to {} (ven {} B + launcher {} B)",
        install_dir.display(),
        ven_bytes.len(),
        launcher_bytes.len()
    );

    println!("\n[2/4] Appending PATH block to shell rc files");
    if !dry_run {
        ensure_user_rc_path(&install_dir)?;
    }
    println!("  [OK] PATH block present in ~/.bashrc / ~/.zshrc (or created ~/.profile)");

    println!("\n[3/4] Installing shell hooks");
    if !dry_run {
        run_ven_setup(&ven_exe)?;
    }
    println!("  [OK] Shell hooks installed");

    println!("\n[4/4] Verifying `ven --version` in a new process");
    if !dry_run {
        let ver = verify_ven_version(&install_dir)?;
        println!("  [OK] {}", ver.trim());
    } else {
        println!("  [OK] dry-run");
    }

    println!("\nDone. Open a new terminal (or `exec $SHELL -l`) and run: ven --version");
    Ok(())
}

fn install_system(dry_run: bool) -> Result<()> {
    println!("ven-setup: System Install (root)");
    let install_dir = PathBuf::from("/usr/local/bin");

    println!("\n[1/4] Extracting and writing binaries");
    let ven_bytes = resolve_binary_bytes("ven", VEN_EMBEDDED)?;
    let launcher_bytes = resolve_binary_bytes("ven-launcher", LAUNCHER_EMBEDDED)?;
    let ven_exe = write_bundled_binary(&install_dir, "ven", &ven_bytes, dry_run)?;
    let _launcher = write_bundled_binary(&install_dir, "ven-launcher", &launcher_bytes, dry_run)?;
    println!(
        "  [OK] Installed to {} (ven {} B + launcher {} B)",
        install_dir.display(),
        ven_bytes.len(),
        launcher_bytes.len()
    );

    println!("\n[2/4] Ensuring /usr/local/bin on system PATH (/etc/profile.d/ven.sh)");
    if !dry_run {
        ensure_etc_profile_d_path(&install_dir)?;
    }
    println!("  [OK] /etc/profile.d/ven.sh present");

    println!("\n[3/4] Skipping per-user shell hooks (system install)");
    println!("  [HINT] Each user should run: ven setup");

    println!("\n[4/4] Verifying `ven --version` in a new process");
    if !dry_run {
        let ver = verify_ven_version(&install_dir)?;
        println!("  [OK] {}", ver.trim());
    } else {
        println!("  [OK] dry-run");
    }

    println!("\nDone. Open a new terminal and run: ven --version");
    let _ = ven_exe;
    Ok(())
}

// ---------------------------------------------------------------------------
// Elevation
// ---------------------------------------------------------------------------

/// Return true when running as root. Avoids a libc dep by shelling out to `id -u`.
fn is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == "0")
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// PATH wiring
// ---------------------------------------------------------------------------

fn ensure_user_rc_path(install_dir: &Path) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve home"))?;
    let candidates = [
        home.join(".bashrc"),
        home.join(".zshrc"),
        home.join(".profile"),
    ];
    let block = render_rc_block(install_dir);
    let mut wrote_any = false;
    for rc in &candidates {
        if rc.is_file() {
            append_block_if_missing(rc, &block)?;
            wrote_any = true;
        }
    }
    if !wrote_any {
        let rc = home.join(".profile");
        fs::write(&rc, format!("{}\n", block))
            .with_context(|| format!("Failed to write {}", rc.display()))?;
    }
    Ok(())
}

fn render_rc_block(install_dir: &Path) -> String {
    format!(
        "{start}\nexport PATH=\"{dir}:$PATH\"\n{end}",
        start = VEN_RC_BLOCK_START,
        dir = install_dir.display(),
        end = VEN_RC_BLOCK_END,
    )
}

fn append_block_if_missing(rc: &Path, block: &str) -> Result<()> {
    let existing =
        fs::read_to_string(rc).with_context(|| format!("Failed to read {}", rc.display()))?;
    if existing.contains(VEN_RC_BLOCK_START) {
        return Ok(());
    }
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(rc)
        .with_context(|| format!("Failed to open {}", rc.display()))?;
    if !existing.ends_with('\n') {
        writeln!(file)?;
    }
    writeln!(file, "{}", block)?;
    Ok(())
}

fn ensure_etc_profile_d_path(install_dir: &Path) -> Result<()> {
    let profile_d = Path::new("/etc/profile.d");
    fs::create_dir_all(profile_d).context("Failed to ensure /etc/profile.d exists")?;
    let script = profile_d.join("ven.sh");
    let content = format!(
        "#!/bin/sh\n# Installed by ven-setup\ncase \":$PATH:\" in\n  *\":{dir}:\"*) ;;\n  *) export PATH=\"{dir}:$PATH\" ;;\nesac\n",
        dir = install_dir.display(),
    );
    fs::write(&script, content).with_context(|| format!("Failed to write {}", script.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script)
            .with_context(|| format!("Failed to stat {}", script.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms)
            .with_context(|| format!("Failed to chmod {}", script.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

fn verify_ven_version(install_dir: &Path) -> Result<String> {
    let base = std::env::var("PATH").unwrap_or_default();
    let merged = format!("{}:{}", install_dir.display(), base);
    let output = Command::new("sh")
        .args(["-c", "ven --version"])
        .env("PATH", merged)
        .output()
        .context("Failed to spawn verification process")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Verification failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
