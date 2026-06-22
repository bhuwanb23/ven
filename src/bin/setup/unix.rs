//! Unix-specific primitives for `ven-setup` (Linux / macOS).
//!
//! Through v0.1.x this module owned the install orchestration. v0.2 lifted
//! the orchestration into [`crate::install_steps`] so the GUI wizard and
//! the CLI share one pipeline; this file now exposes only the platform
//! primitives:
//!
//! - [`is_root`] — sudo detection (avoids a `libc` dep by shelling to `id -u`).
//! - [`ensure_user_rc_path`] — appends the `# >>> ven-setup PATH >>>` block
//!   to `~/.bashrc` / `~/.zshrc`, falling back to creating `~/.profile`.
//! - [`ensure_etc_profile_d_path`] — writes the system-wide
//!   `/etc/profile.d/ven.sh` PATH guard.
//! - [`verify_ven_version`] — child process with merged PATH for the
//!   verify step.
//! - [`run`] — the legacy CLI entry point. Builds an
//!   [`InstallConfig`](crate::install_steps::InstallConfig) and drives the
//!   shared pipeline.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::{detect_existing_installs, prompt_existing_install_cli, InstallMode, SetupCli};
use crate::install_steps::{self, CliSink, InstallConfig};
use ven::shell::shell_escape_posix;

const VEN_RC_BLOCK_START: &str = "# >>> ven-setup PATH >>>";
const VEN_RC_BLOCK_END: &str = "# <<< ven-setup PATH <<<";

// ---------------------------------------------------------------------------
// Backup helpers
// ---------------------------------------------------------------------------

/// Create a `.bak` copy of `path` before modifying it. Returns the backup
/// path on success, or an error if the backup could not be created.
fn backup_file(path: &Path) -> Result<PathBuf> {
    let backup = path.with_extension(
        path.extension()
            .map(|e| format!("{}.bak", e.to_string_lossy()))
            .unwrap_or_else(|| "bak".to_string()),
    );
    fs::copy(path, &backup).with_context(|| {
        format!(
            "Failed to create backup {} -> {}",
            path.display(),
            backup.display()
        )
    })?;
    eprintln!("  Backup created: {}", backup.display());
    Ok(backup)
}

/// Attempt to restore `path` from `backup`. Logs a warning on failure
/// but does not propagate — there's nothing useful to do if the restore
/// itself fails.
fn try_restore(path: &Path, backup: &Path) {
    if let Err(e) = fs::copy(backup, path) {
        eprintln!(
            "  [WARN] Could not restore {} from {}: {}",
            path.display(),
            backup.display(),
            e
        );
    } else {
        eprintln!(
            "  Restored {} from backup {}",
            path.display(),
            backup.display()
        );
    }
}

// ---------------------------------------------------------------------------
// CLI driver
// ---------------------------------------------------------------------------

pub fn run(cli: SetupCli, mode: InstallMode) -> Result<()> {
    // If the user is resuming from a sudo-relaunched parent, the previous
    // process serialized their wizard / CLI choices into a TOML and passed
    // `--resume <path>`. Honour that so a sudo re-invocation doesn't lose
    // the storage path, runtime selection, etc.
    let mut cfg = if let Some(resume) = cli.resume.as_deref() {
        InstallConfig::load_from_file(resume).with_context(|| {
            format!(
                "Failed to load resume file at {} (the sudo relaunch handoff is broken)",
                resume.display()
            )
        })?
    } else {
        build_config_from_cli(&cli, mode)
    };

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

    if matches!(cfg.mode, InstallMode::System) && !cfg.dry_run && !is_root() {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "ven-setup".to_string());
        // Stash the config so a sudo re-invocation with --resume picks it
        // up. We swallow the write error: even without the resume file
        // the bare command works, just with default settings.
        let resume_hint = match resume_file_path() {
            Ok(p) => cfg.save_to_file(&p).ok().map(|_| p),
            Err(_) => None,
        };
        let resume_arg = resume_hint
            .as_ref()
            .map(|p| format!(" --resume '{}'", p.display()))
            .unwrap_or_default();
        anyhow::bail!(
            "System install requires root. Re-run with:\n    sudo {exe} --mode system{resume_arg}"
        );
    }

    println!(
        "ven-setup: {} Install ({})",
        match cfg.mode {
            InstallMode::User => "User",
            InstallMode::System => "System",
        },
        if cfg.dry_run { "dry-run" } else { "live" }
    );

    let mut sink = CliSink;
    install_steps::run(&cfg, &mut sink).map(|_| ())
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

// ---------------------------------------------------------------------------
// Elevation
// ---------------------------------------------------------------------------

/// `true` when the current process is running as root. Avoids a `libc`
/// dependency by shelling out to `id -u` — same trick as
/// `core::uninstaller::running_with_privileges` uses elsewhere.
pub fn is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == "0")
        .unwrap_or(false)
}

/// Where the GUI wizard (and the CLI's "re-run with sudo" hint) stashes
/// the TOML resume file. `$TMPDIR/ven-setup-resume.toml` so it survives
/// across a sudo re-invocation in the same shell.
pub fn resume_file_path() -> Result<PathBuf> {
    Ok(std::env::temp_dir().join("ven-setup-resume.toml"))
}

// ---------------------------------------------------------------------------
// PATH wiring (exposed for install_steps to call into)
// ---------------------------------------------------------------------------

/// Append the `# >>> ven-setup PATH >>>` … `# <<< ven-setup PATH <<<`
/// block to whichever of `~/.bashrc`, `~/.zshrc`, `~/.profile` exists.
/// Creates `~/.profile` when none are present so a fresh shell still
/// picks up the install dir.
pub fn ensure_user_rc_path(install_dir: &Path) -> Result<()> {
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
    let escaped = shell_escape_posix(&install_dir.display().to_string());
    format!(
        "{start}\nexport PATH={escaped}:\"$PATH\"\n{end}",
        start = VEN_RC_BLOCK_START,
        escaped = escaped,
        end = VEN_RC_BLOCK_END,
    )
}

fn append_block_if_missing(rc: &Path, block: &str) -> Result<()> {
    let existing =
        fs::read_to_string(rc).with_context(|| format!("Failed to read {}", rc.display()))?;
    if existing.contains(VEN_RC_BLOCK_START) {
        return Ok(());
    }

    // Backup before modification so a midway failure doesn't corrupt the
    // shell profile.
    let backup = backup_file(rc)?;

    let result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(rc)
            .with_context(|| format!("Failed to open {}", rc.display()))?;
        if !existing.ends_with('\n') {
            writeln!(file)?;
        }
        writeln!(file, "{}", block)?;
        Ok(())
    })();

    if let Err(ref e) = result {
        eprintln!(
            "  [ERROR] Failed to modify {}: {}. Restoring from backup.",
            rc.display(),
            e
        );
        try_restore(rc, &backup);
    }

    result
}

/// Write `/etc/profile.d/ven.sh` so the install dir is on PATH for every
/// login shell, system-wide. Idempotent: the script uses a `case` check
/// so sourcing it twice doesn't duplicate the PATH entry.
pub fn ensure_etc_profile_d_path(install_dir: &Path) -> Result<()> {
    let profile_d = Path::new("/etc/profile.d");
    fs::create_dir_all(profile_d).context("Failed to ensure /etc/profile.d exists")?;
    let script = profile_d.join("ven.sh");
    let escaped = shell_escape_posix(&install_dir.display().to_string());
    let content = format!(
        "#!/bin/sh\n# Installed by ven-setup\n__VEN_INSTALL_DIR={escaped}\ncase \":$PATH:\" in\n  *\":$__VEN_INSTALL_DIR:\"*) ;;\n  *) export PATH=\"$__VEN_INSTALL_DIR:$PATH\" ;;\nesac\n",
        escaped = escaped,
    );

    // Backup the existing file if present, then write the new content.
    if script.is_file() {
        backup_file(&script)?;
    }

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

pub fn verify_ven_version(install_dir: &Path) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn render_rc_block_safe_path() {
        let dir = PathBuf::from("/usr/local/bin");
        let block = render_rc_block(&dir);
        assert!(block.contains("export PATH='/usr/local/bin':\"$PATH\""));
        assert!(block.contains(VEN_RC_BLOCK_START));
        assert!(block.contains(VEN_RC_BLOCK_END));
    }

    #[test]
    fn render_rc_block_injection_attempt() {
        let dir = PathBuf::from(r#"x":$(malicious):"#);
        let block = render_rc_block(&dir);
        assert!(block.contains("'x\":$(malicious):'"));
        assert!(!block.contains("$(malicious)"));
    }

    #[test]
    fn render_rc_block_single_quote_in_path() {
        let dir = PathBuf::from("/path/with'quote");
        let block = render_rc_block(&dir);
        assert!(block.contains("'\\''"));
        assert!(!block.contains("export PATH=\"/path/with'quote"));
    }

    #[test]
    fn render_rc_block_dollar_sign_in_path() {
        let dir = PathBuf::from("/path/$HOME/bin");
        let block = render_rc_block(&dir);
        assert!(block.contains("'$HOME'"));
    }

    #[test]
    fn render_rc_block_backtick_in_path() {
        let dir = PathBuf::from("/path/`cmd`/bin");
        let block = render_rc_block(&dir);
        assert!(block.contains("'`cmd`'"));
    }

    #[test]
    fn ensure_etc_profile_d_escaping() {
        let dir = PathBuf::from(r#"x":$(malicious):"#);
        let escaped = shell_escape_posix(&dir.display().to_string());
        let content = format!(
            "#!/bin/sh\n# Installed by ven-setup\n__VEN_INSTALL_DIR={escaped}\ncase \":$PATH:\" in\n  *\":$__VEN_INSTALL_DIR:\"*) ;;\n  *) export PATH=\"$__VEN_INSTALL_DIR:$PATH\" ;;\nesac\n",
            escaped = escaped,
        );
        assert!(content.contains("__VEN_INSTALL_DIR='x\":$(malicious):'"));
        assert!(!content.contains("$(malicious)"));
    }
}
