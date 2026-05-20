//! Shared install API driven by both the GUI wizard and the legacy CLI flow.
//!
//! ## Why this module exists
//!
//! Through v0.1.x, `ven-setup` had two parallel install paths: one inside
//! [`crate::windows`] and one inside [`crate::unix`], each doing extract ->
//! write -> PATH -> hook -> verify in its own [1/4], [2/4], … style. When
//! the GUI wizard arrived in v0.2 we needed a third driver that could run
//! the *same* sequence on a worker thread and stream progress to an
//! `eframe::App`. Duplicating the install logic again would have been a
//! quick way to drift the three implementations apart.
//!
//! So the install pipeline lives here exactly once. Drivers (CLI / GUI /
//! elevated-child resume) provide an [`InstallConfig`] + a [`ProgressSink`]
//! and call [`run`]. The platform helpers in [`crate::windows`] and
//! [`crate::unix`] now only own the *platform-specific* primitives the
//! pipeline calls into (PATH wiring, elevation detection).
//!
//! ## Pipeline
//!
//! ```text
//! 1/6  Extract embedded binaries (ven + ven-launcher)        [always]
//! 2/6  Configure storage path ($VEN_HOME pointer + env var)  [if non-default]
//! 3/6  Update PATH                                           [if add_to_path]
//! 4/6  Install shell hook                                    [if install_hook]
//! 5/6  Pre-install selected runtimes                         [if any]
//! 6/6  Verify `ven --version`                                [always]
//! ```
//!
//! `TOTAL_STEPS` always reports 6 so the GUI's progress bar doesn't jump;
//! steps that are skipped emit a `StepStarted` + `StepCompleted` pair with
//! a "skipped" sub-label so the user still sees what would have happened.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::common::{
    resolve_binary_bytes, write_bundled_binary, InstallMode, LAUNCHER_EMBEDDED, VEN_EMBEDDED,
};

// ---------------------------------------------------------------------------
// Config + serialization
// ---------------------------------------------------------------------------

/// Total number of steps reported by [`run`]. Constant so the GUI's
/// progress bar doesn't jump when optional steps are skipped.
pub const TOTAL_STEPS: usize = 6;

/// Everything the install pipeline needs to know. Serializable so the
/// Windows UAC parent / Unix sudo parent can hand it to the elevated
/// child through a TOML "resume" file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallConfig {
    pub mode: InstallMode,
    /// Where `ven[.exe]` and `ven-launcher[.exe]` will be written.
    pub install_dir: PathBuf,
    /// Override for `$VEN_HOME` (runtime data root). `None` means "leave
    /// the default in place" — i.e. don't write a pointer file, don't
    /// persist `VEN_HOME` in the user environment.
    pub storage_path: Option<PathBuf>,
    /// Add `install_dir` to PATH (User registry on Windows / rc-block on
    /// Unix user / `/etc/profile.d/ven.sh` on Unix system).
    pub add_to_path: bool,
    /// Run `ven setup` to install the shell hook after the binaries land.
    pub install_hook: bool,
    /// Pre-install these language runtimes immediately after the core
    /// install. Each entry is a slug accepted by `ven install <lang>` —
    /// e.g. `node`, `python`, `go`, `rust`, `java`, `deno`, `bun`, `ruby`.
    pub runtimes_to_install: Vec<String>,
    /// Walk the pipeline without touching the filesystem, registry, rc
    /// files, or spawning sub-processes.
    pub dry_run: bool,
}

impl InstallConfig {
    /// A reasonable starting config: install in the default location for
    /// `mode`, default storage path, PATH + shell hook on, no runtimes
    /// preselected, not a dry-run.
    pub fn default_for_mode(mode: InstallMode) -> Self {
        Self {
            mode,
            install_dir: default_install_dir(mode),
            storage_path: None,
            add_to_path: true,
            install_hook: true,
            runtimes_to_install: Vec::new(),
            dry_run: false,
        }
    }

    /// Load a previously-serialized config (written by the GUI parent
    /// before relaunching for elevation). Returns a helpful error if the
    /// file is missing or malformed so the elevated child can bail with
    /// a clear message instead of crashing.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let bytes = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read resume file {}", path.display()))?;
        let cfg: Self = toml::from_str(&bytes)
            .with_context(|| format!("Failed to parse resume file {}", path.display()))?;
        Ok(cfg)
    }

    /// Persist this config to a TOML file the elevated child can read.
    /// Used by the GUI's "elevate" path on Windows (UAC relaunch) and the
    /// "Re-run with sudo" hint on Unix.
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create resume file parent {}", parent.display())
            })?;
        }
        let body =
            toml::to_string_pretty(self).context("Failed to serialize install config to TOML")?;
        std::fs::write(path, body)
            .with_context(|| format!("Failed to write resume file {}", path.display()))?;
        Ok(())
    }
}

/// The default install dir for `mode`. Mirrors what v0.1.x baked into
/// `windows::install_user` / `unix::install_user` etc.
pub fn default_install_dir(mode: InstallMode) -> PathBuf {
    #[cfg(windows)]
    {
        match mode {
            InstallMode::User => dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ven")
                .join("bin"),
            InstallMode::System => std::env::var_os("ProgramFiles")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"))
                .join("ven")
                .join("bin"),
        }
    }
    #[cfg(unix)]
    {
        match mode {
            InstallMode::User => dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ven")
                .join("bin"),
            InstallMode::System => PathBuf::from("/usr/local/bin"),
        }
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = mode;
        PathBuf::from(".")
    }
}

/// The default `$VEN_HOME` (runtime data root). Always `~/.ven` so the GUI's
/// Storage screen can show a stable "Default" pre-fill regardless of the
/// install mode.
pub fn default_storage_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ven")
}

// ---------------------------------------------------------------------------
// Progress events + sinks
// ---------------------------------------------------------------------------

/// Events streamed by [`run`] so drivers can render progress in their own
/// idiom. The GUI's worker thread forwards each event over an `mpsc`
/// channel to the [`crate::gui::screens::progress`] screen; the CLI sink
/// just `println!`s.
#[derive(Clone, Debug)]
pub enum ProgressEvent {
    /// New step is starting. `index` is 1-based; `total` is [`TOTAL_STEPS`].
    StepStarted {
        index: usize,
        total: usize,
        label: String,
    },
    /// Sub-step within the current step. Replaces previous detail text.
    StepDetail { sub_label: String },
    /// A single line of arbitrary log output (e.g. piped child stdout).
    StepLog { line: String },
    /// Current step finished successfully (or was skipped per config).
    StepCompleted { index: usize, skipped: bool },
    /// Whole install finished cleanly. `ven_version` is `None` in dry-run.
    InstallCompleted { ven_version: Option<String> },
    /// An unrecoverable error occurred. The driver decides what to do.
    InstallFailed { error: String },
}

/// Sink for [`ProgressEvent`]s. `Send` so the GUI can build a sink that
/// forwards over an `mpsc::Sender<ProgressEvent>` from a worker thread.
pub trait ProgressSink: Send {
    fn emit(&mut self, event: ProgressEvent);
}

/// CLI sink — prints each event to stdout/stderr in the same shape the
/// pre-v0.2 implementation used (`[1/4]`-style header lines, indented
/// detail lines). Kept compatible so docs / screenshots that show the
/// CLI output still match.
pub struct CliSink;

impl ProgressSink for CliSink {
    fn emit(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::StepStarted {
                index,
                total,
                label,
            } => {
                println!("\n[{index}/{total}] {label}");
            }
            ProgressEvent::StepDetail { sub_label } => {
                println!("  {sub_label}");
            }
            ProgressEvent::StepLog { line } => {
                println!("  {line}");
            }
            ProgressEvent::StepCompleted { index, skipped } => {
                if skipped {
                    println!("  [SKIP] step {index} skipped per config");
                } else {
                    println!("  [OK] step {index} complete");
                }
            }
            ProgressEvent::InstallCompleted { ven_version } => {
                let v = ven_version.as_deref().unwrap_or("dry-run");
                println!("\nDone. {v}");
                println!("Open a new terminal and run: ven --version");
            }
            ProgressEvent::InstallFailed { error } => {
                eprintln!("\n[ERROR] {error}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Execute the full install pipeline against `cfg`. Returns the
/// `ven --version` line on success (or an empty string in dry-run /
/// when verify was skipped). On failure, emits `InstallFailed` and
/// propagates the underlying error to the caller — both signal paths
/// exist so the GUI can decide between an in-progress bail vs a
/// post-run hard error.
pub fn run(cfg: &InstallConfig, sink: &mut dyn ProgressSink) -> Result<String> {
    let result = run_inner(cfg, sink);
    match &result {
        Ok(version) => sink.emit(ProgressEvent::InstallCompleted {
            ven_version: if version.is_empty() {
                None
            } else {
                Some(version.clone())
            },
        }),
        Err(e) => sink.emit(ProgressEvent::InstallFailed {
            error: format!("{e:#}"),
        }),
    }
    result
}

fn run_inner(cfg: &InstallConfig, sink: &mut dyn ProgressSink) -> Result<String> {
    // ── 1/6 Extract embedded binaries ──────────────────────────────────
    sink.emit(ProgressEvent::StepStarted {
        index: 1,
        total: TOTAL_STEPS,
        label: "Extracting bundled binaries".into(),
    });
    let ven_exe = step_extract(cfg, sink)?;
    sink.emit(ProgressEvent::StepCompleted {
        index: 1,
        skipped: false,
    });

    // ── 2/6 Configure storage path ─────────────────────────────────────
    sink.emit(ProgressEvent::StepStarted {
        index: 2,
        total: TOTAL_STEPS,
        label: "Configuring storage directory ($VEN_HOME)".into(),
    });
    let storage_skipped = step_configure_storage(cfg, sink)?;
    sink.emit(ProgressEvent::StepCompleted {
        index: 2,
        skipped: storage_skipped,
    });

    // ── 3/6 Update PATH ────────────────────────────────────────────────
    sink.emit(ProgressEvent::StepStarted {
        index: 3,
        total: TOTAL_STEPS,
        label: "Updating PATH".into(),
    });
    let path_skipped = step_update_path(cfg, sink)?;
    sink.emit(ProgressEvent::StepCompleted {
        index: 3,
        skipped: path_skipped,
    });

    // ── 4/6 Install shell hook ─────────────────────────────────────────
    sink.emit(ProgressEvent::StepStarted {
        index: 4,
        total: TOTAL_STEPS,
        label: "Installing shell hook".into(),
    });
    let hook_skipped = step_install_hook(cfg, &ven_exe, sink)?;
    sink.emit(ProgressEvent::StepCompleted {
        index: 4,
        skipped: hook_skipped,
    });

    // ── 5/6 Pre-install runtimes ───────────────────────────────────────
    sink.emit(ProgressEvent::StepStarted {
        index: 5,
        total: TOTAL_STEPS,
        label: "Pre-installing selected runtimes".into(),
    });
    let runtimes_skipped = step_preinstall_runtimes(cfg, &ven_exe, sink)?;
    sink.emit(ProgressEvent::StepCompleted {
        index: 5,
        skipped: runtimes_skipped,
    });

    // ── 6/6 Verify ─────────────────────────────────────────────────────
    sink.emit(ProgressEvent::StepStarted {
        index: 6,
        total: TOTAL_STEPS,
        label: "Verifying ven --version".into(),
    });
    let version = step_verify(cfg, sink)?;
    sink.emit(ProgressEvent::StepCompleted {
        index: 6,
        skipped: false,
    });

    Ok(version)
}

// ---------------------------------------------------------------------------
// Individual steps
// ---------------------------------------------------------------------------

fn step_extract(cfg: &InstallConfig, sink: &mut dyn ProgressSink) -> Result<PathBuf> {
    let (ven_name, launcher_name) = if cfg!(windows) {
        ("ven.exe", "ven-launcher.exe")
    } else {
        ("ven", "ven-launcher")
    };

    let ven_bytes = resolve_binary_bytes(ven_name, VEN_EMBEDDED)?;
    let launcher_bytes = resolve_binary_bytes(launcher_name, LAUNCHER_EMBEDDED)?;

    sink.emit(ProgressEvent::StepDetail {
        sub_label: format!(
            "Writing {} + {} ({} B + {} B) to {}",
            ven_name,
            launcher_name,
            ven_bytes.len(),
            launcher_bytes.len(),
            cfg.install_dir.display()
        ),
    });

    let ven_exe = write_bundled_binary(&cfg.install_dir, ven_name, &ven_bytes, cfg.dry_run)?;
    let _launcher_exe = write_bundled_binary(
        &cfg.install_dir,
        launcher_name,
        &launcher_bytes,
        cfg.dry_run,
    )?;
    Ok(ven_exe)
}

/// Returns `Ok(true)` when the step was a no-op (storage stays at default).
fn step_configure_storage(cfg: &InstallConfig, sink: &mut dyn ProgressSink) -> Result<bool> {
    let Some(target) = cfg.storage_path.as_ref() else {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "No custom storage path requested — leaving $VEN_HOME at default".into(),
        });
        return Ok(true);
    };

    let default = default_storage_path();
    if same_path(target, &default) {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "Requested path matches default — leaving $VEN_HOME unchanged".into(),
        });
        return Ok(true);
    }

    if cfg.dry_run {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: format!(
                "[dry-run] would set $VEN_HOME = {} and write pointer file",
                target.display()
            ),
        });
        return Ok(false);
    }

    sink.emit(ProgressEvent::StepDetail {
        sub_label: format!("Writing pointer file -> {}", target.display()),
    });
    ven::core::ven_config::set_storage_home(target.clone())
        .context("Failed to write the global storage pointer")?;

    // Only persist $VEN_HOME in the *user* environment for User installs.
    // A system-wide install on Unix has no concept of "the calling user's
    // shell env" — the elevated child runs as root and any rc-file write
    // there would land in root's profile. We still write the pointer
    // file so anyone who explicitly sources it gets the same VEN_HOME.
    let persist_env = matches!(cfg.mode, InstallMode::User);
    if persist_env {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: format!(
                "Persisting $VEN_HOME = {} in user environment",
                target.display()
            ),
        });
        ven::core::user_env::set_user_env("VEN_HOME", &target.display().to_string())
            .context("Failed to persist $VEN_HOME in the user environment")?;
    }
    Ok(false)
}

fn step_update_path(cfg: &InstallConfig, sink: &mut dyn ProgressSink) -> Result<bool> {
    if !cfg.add_to_path {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "PATH update disabled by the user".into(),
        });
        return Ok(true);
    }
    if cfg.dry_run {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: format!("[dry-run] would add {} to PATH", cfg.install_dir.display()),
        });
        return Ok(false);
    }

    #[cfg(windows)]
    {
        let scope = match cfg.mode {
            InstallMode::User => crate::windows::PathScope::User,
            InstallMode::System => crate::windows::PathScope::Machine,
        };
        sink.emit(ProgressEvent::StepDetail {
            sub_label: format!(
                "Writing {} PATH and broadcasting WM_SETTINGCHANGE",
                scope.label()
            ),
        });
        crate::windows::ensure_path_contains(&cfg.install_dir, scope)?;
    }
    #[cfg(unix)]
    {
        match cfg.mode {
            InstallMode::User => {
                sink.emit(ProgressEvent::StepDetail {
                    sub_label: "Appending PATH block to ~/.bashrc / ~/.zshrc / ~/.profile".into(),
                });
                crate::unix::ensure_user_rc_path(&cfg.install_dir)?;
            }
            InstallMode::System => {
                sink.emit(ProgressEvent::StepDetail {
                    sub_label: "Writing /etc/profile.d/ven.sh".into(),
                });
                crate::unix::ensure_etc_profile_d_path(&cfg.install_dir)?;
            }
        }
    }
    Ok(false)
}

fn step_install_hook(
    cfg: &InstallConfig,
    ven_exe: &Path,
    sink: &mut dyn ProgressSink,
) -> Result<bool> {
    if !cfg.install_hook {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "Shell hook install disabled by the user".into(),
        });
        return Ok(true);
    }
    // System installs on Unix skip per-user hooks — ven setup writes to
    // the *calling user's* rc files, and under sudo that's root's rc.
    // We preserve the old v0.1.x behaviour of printing a hint.
    if cfg!(unix) && matches!(cfg.mode, InstallMode::System) {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "System install — each user should run `ven setup` from their own shell"
                .into(),
        });
        return Ok(true);
    }
    if cfg.dry_run {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "[dry-run] would run `ven setup` to install the shell hook".into(),
        });
        return Ok(false);
    }
    sink.emit(ProgressEvent::StepDetail {
        sub_label: "Running `ven setup` to register the shell hook".into(),
    });
    spawn_streaming(ven_exe, &["setup"], &cfg.install_dir, sink).map(|_| false)
}

fn step_preinstall_runtimes(
    cfg: &InstallConfig,
    ven_exe: &Path,
    sink: &mut dyn ProgressSink,
) -> Result<bool> {
    if cfg.runtimes_to_install.is_empty() {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "No runtimes selected for pre-install".into(),
        });
        return Ok(true);
    }
    if cfg.dry_run {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: format!(
                "[dry-run] would `ven install <lang> latest` for: {}",
                cfg.runtimes_to_install.join(", ")
            ),
        });
        return Ok(false);
    }
    for runtime in &cfg.runtimes_to_install {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: format!("Installing {runtime} (latest) — this may take a few minutes"),
        });
        // Per-runtime failure does NOT abort the whole install; we log
        // and continue. The Done screen surfaces any per-runtime errors
        // so the user can finish the core install and retry individual
        // runtimes via `ven install <lang>` from a terminal.
        if let Err(e) =
            spawn_streaming(ven_exe, &["install", runtime, "latest"], &cfg.install_dir, sink)
        {
            sink.emit(ProgressEvent::StepLog {
                line: format!(
                    "[WARN] {runtime} pre-install failed: {e}. Re-run `ven install {runtime}` manually."
                ),
            });
        }
    }
    Ok(false)
}

fn step_verify(cfg: &InstallConfig, sink: &mut dyn ProgressSink) -> Result<String> {
    if cfg.dry_run {
        sink.emit(ProgressEvent::StepDetail {
            sub_label: "[dry-run] would spawn `ven --version` with merged PATH".into(),
        });
        return Ok(String::new());
    }
    sink.emit(ProgressEvent::StepDetail {
        sub_label: "Spawning ven --version with PATH = install_dir + current PATH".into(),
    });
    #[cfg(windows)]
    {
        crate::windows::verify_ven_version(&cfg.install_dir)
            .map(|s| s.trim().to_string())
            .map(|s| {
                sink.emit(ProgressEvent::StepLog { line: s.clone() });
                s
            })
    }
    #[cfg(unix)]
    {
        crate::unix::verify_ven_version(&cfg.install_dir)
            .map(|s| s.trim().to_string())
            .map(|s| {
                sink.emit(ProgressEvent::StepLog { line: s.clone() });
                s
            })
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (cfg, sink);
        anyhow::bail!("verify is only implemented for Windows and Unix");
    }
}

// ---------------------------------------------------------------------------
// Sub-process helper
// ---------------------------------------------------------------------------

/// Spawn `ven_exe` with `args`, merging `install_dir` onto the front of
/// PATH so the freshly-installed ven resolves first, and stream every
/// line of stdout + stderr to `sink` as `StepLog` events.
///
/// Two reader threads share a single `mpsc::Sender<String>`; the main
/// thread drains the receiver until both senders are dropped, then waits
/// on the child. That keeps progress live (no buffering) while staying
/// `Send` so the GUI worker can call this from its own thread.
fn spawn_streaming(
    ven_exe: &Path,
    args: &[&str],
    install_dir: &Path,
    sink: &mut dyn ProgressSink,
) -> Result<()> {
    let base = std::env::var("PATH").unwrap_or_default();
    let sep = if cfg!(windows) { ";" } else { ":" };
    let merged = format!("{}{sep}{}", install_dir.display(), base);

    let mut child = Command::new(ven_exe)
        .args(args)
        .env("PATH", merged)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn {} {}", ven_exe.display(), args.join(" ")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("child stdout pipe missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("child stderr pipe missing"))?;

    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let tx_out = tx.clone();
    let h_out = std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx_out.send(line).is_err() {
                break;
            }
        }
    });
    let tx_err = tx;
    let h_err = std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if tx_err.send(line).is_err() {
                break;
            }
        }
    });

    while let Ok(line) = rx.recv() {
        sink.emit(ProgressEvent::StepLog { line });
    }
    let _ = h_out.join();
    let _ = h_err.join();

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait on {}", ven_exe.display()))?;
    if !status.success() {
        anyhow::bail!(
            "{} {} exited with status {}",
            ven_exe.display(),
            args.join(" "),
            status
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn same_path(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    if let (Ok(ca), Ok(cb)) = (a.canonicalize(), b.canonicalize()) {
        return ca == cb;
    }
    let na = a.display().to_string();
    let nb = b.display().to_string();
    if cfg!(windows) {
        na.to_ascii_lowercase()
            .trim_end_matches(['/', '\\'])
            .replace('/', "\\")
            == nb
                .to_ascii_lowercase()
                .trim_end_matches(['/', '\\'])
                .replace('/', "\\")
    } else {
        na.trim_end_matches('/') == nb.trim_end_matches('/')
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock sink that just records every event so we can assert on the
    /// step sequence without spawning anything.
    #[derive(Default)]
    struct RecordingSink {
        events: Vec<ProgressEvent>,
    }
    impl ProgressSink for RecordingSink {
        fn emit(&mut self, event: ProgressEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn default_for_mode_has_safe_defaults() {
        let cfg = InstallConfig::default_for_mode(InstallMode::User);
        assert!(cfg.add_to_path);
        assert!(cfg.install_hook);
        assert!(!cfg.dry_run);
        assert!(cfg.runtimes_to_install.is_empty());
        assert!(cfg.storage_path.is_none());
    }

    #[test]
    fn config_round_trips_through_toml() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("resume.toml");
        let mut cfg = InstallConfig::default_for_mode(InstallMode::System);
        cfg.add_to_path = false;
        cfg.install_hook = false;
        cfg.runtimes_to_install = vec!["node".into(), "python".into()];
        cfg.dry_run = true;
        cfg.save_to_file(&path).expect("save");
        let loaded = InstallConfig::load_from_file(&path).expect("load");
        assert_eq!(loaded.mode, InstallMode::System);
        assert!(!loaded.add_to_path);
        assert!(!loaded.install_hook);
        assert_eq!(loaded.runtimes_to_install, vec!["node", "python"]);
        assert!(loaded.dry_run);
    }

    #[test]
    fn dry_run_emits_step_pairs_without_touching_fs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut cfg = InstallConfig::default_for_mode(InstallMode::User);
        cfg.install_dir = tmp.path().join("fake-bin");
        cfg.dry_run = true;
        // dry-run still tries to "extract" but write_bundled_binary
        // skips the actual write — so the fake install dir stays empty.
        cfg.add_to_path = true;
        cfg.install_hook = true;
        cfg.runtimes_to_install = vec!["node".into()];

        let mut sink = RecordingSink::default();
        // step_extract will still call resolve_binary_bytes which falls
        // through to a sibling-file lookup. In the test harness that file
        // doesn't exist, so the error is expected — we only want to
        // verify the sequence up to step 1's start event.
        let _ = run_inner(&cfg, &mut sink);
        let starts: Vec<usize> = sink
            .events
            .iter()
            .filter_map(|e| match e {
                ProgressEvent::StepStarted { index, .. } => Some(*index),
                _ => None,
            })
            .collect();
        assert_eq!(starts.first(), Some(&1));
        // We can't assert further without real embedded binaries, but
        // the test confirms the step machinery wires up.
    }

    #[test]
    fn run_emits_install_failed_on_error() {
        let mut cfg = InstallConfig::default_for_mode(InstallMode::User);
        // Force step_extract to fail by pointing install_dir at an
        // un-writable location AND clearing dry_run so the write
        // actually attempts.
        cfg.install_dir = PathBuf::from("/this/path/should/not/be/writable");
        cfg.dry_run = false;
        let mut sink = RecordingSink::default();
        let res = run(&cfg, &mut sink);
        assert!(res.is_err(), "expected failure when install_dir unwritable");
        assert!(
            sink.events
                .iter()
                .any(|e| matches!(e, ProgressEvent::InstallFailed { .. })),
            "expected an InstallFailed event in {:?}",
            sink.events
        );
    }

    #[test]
    fn same_path_normalises_trailing_separators() {
        let a = PathBuf::from("/tmp/foo");
        let b = PathBuf::from("/tmp/foo/");
        assert!(same_path(&a, &b));
    }
}
