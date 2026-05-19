//! `core::uninstaller` — plan + execute the full teardown of a ven install.
//!
//! The historical uninstall flow was a 100-line copy-paste PowerShell /
//! shell snippet on the website. It worked but had several long-standing
//! footguns:
//!
//! - It always assumed the runtime data lived under `~/.ven`, so users who
//!   had run `ven path set D:\ven` ended up with an orphaned `D:\ven`
//!   directory after "uninstall".
//! - It never cleared the pointer file at `~/.config/ven/config.toml`.
//! - It never unset the persisted `$VEN_HOME` user env var written by
//!   `ven path set`, so the next install would silently inherit it.
//! - It missed the `config.fish` shell hook and PowerShell `$PROFILE.ps1`
//!   hook, leaving dangling `ven shell hook` lines.
//!
//! This module is the single Rust-side source of truth for what an
//! uninstall touches. The CLI command (`src/cli/uninstall.rs`) and the
//! bundled fallback scripts (`scripts/uninstall.{ps1,sh}`) are both meant
//! to converge on the same outcome.
//!
//! ## Two-phase: `build_plan` → `execute_plan`
//!
//! Splitting plan-construction from execution lets `--dry-run` reuse the
//! exact same discovery logic without touching the filesystem, and gives
//! the test suite a way to exercise the plan logic with mocked roots.
//!
//! ## Scope
//!
//! `UninstallScope` lets a sysadmin run `--system-only` from an elevated
//! shell without nuking the calling user's home dir, or run `--user-only`
//! to leave a shared system install in place. Default is `All`.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::core::user_env;
use crate::core::ven_config;
use crate::core::ven_home::{ven_home_source, HomeSource};

// ─────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────

/// Which install layers to touch. The `Scope` enum from the CLI maps 1:1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UninstallScope {
    /// User-mode install + system-mode install (the default).
    All,
    /// Only touch the user-mode install + user-scope env state.
    UserOnly,
    /// Only touch the system-mode install + system-scope env state.
    SystemOnly,
}

impl UninstallScope {
    fn includes_user(&self) -> bool {
        matches!(self, UninstallScope::All | UninstallScope::UserOnly)
    }
    fn includes_system(&self) -> bool {
        matches!(self, UninstallScope::All | UninstallScope::SystemOnly)
    }
}

/// What `execute_plan` will (or, in `--dry-run`, would) do.
///
/// All optional fields are `None` when that artifact isn't present on
/// this machine. Empty `Vec`s mean "nothing of that class to remove".
#[derive(Debug, Clone)]
pub struct UninstallPlan {
    /// User-mode install root (e.g. `~/.ven` or `%USERPROFILE%\.ven`).
    /// Contains `bin/` AND, by default, the runtime data. Removed wholesale
    /// when the scope includes `User`.
    pub user_install_root: Option<PathBuf>,

    /// System-mode artifacts to remove. On Windows this is typically a
    /// single directory entry (`%ProgramFiles%\ven`). On Unix it's the
    /// individual files at `/usr/local/bin/{ven,ven-launcher,ven-setup}`
    /// plus `/etc/profile.d/ven.sh`. Each entry is removed independently
    /// (file or recursive dir, whichever fits).
    pub system_artifacts: Vec<PathBuf>,

    /// Resolved `$VEN_HOME` (the storage root). May overlap with
    /// `user_install_root` in a default install; in that case the wholesale
    /// removal of `user_install_root` takes care of it and `execute_plan`
    /// skips the redundant pass. After `ven path set D:\ven` the two are
    /// distinct and we remove both.
    pub data_dir: PathBuf,
    /// How `data_dir` was resolved. Drives the "is it safe to delete"
    /// messaging the CLI shows.
    pub data_dir_source: HomeSource,
    /// `true` when `data_dir` sits outside the user install root and needs
    /// its own removal step.
    pub data_dir_is_relocated: bool,

    /// Pointer file at `~/.config/ven/config.toml` (or platform equivalent).
    /// Cleared via [`ven_config::clear_storage_home`].
    pub pointer_file: Option<PathBuf>,

    /// User PATH entries to strip. Used only on Windows (registry edit).
    /// On Unix the rc-file scrubber owns the PATH cleanup.
    pub user_path_entries: Vec<PathBuf>,

    /// System PATH entries to strip. Used only on Windows.
    pub system_path_entries: Vec<PathBuf>,

    /// Names of user-scope env vars to remove (currently just `VEN_HOME`).
    /// Driven through [`user_env::unset_user_env`] so the platform-specific
    /// teardown (PowerShell `[Environment]::SetEnvironmentVariable(_, $null,
    /// 'User')` or `# >>> ven env >>>` block strip) stays in one place.
    pub user_env_vars: Vec<&'static str>,

    /// Unix rc files that may contain ven-managed blocks to strip. Each
    /// file is scrubbed of:
    /// - the `# >>> ven env >>>` … `# <<< ven env <<<` block (VEN_HOME).
    /// - the `# >>> ven-setup PATH >>>` … `# <<< ven-setup PATH <<<` block.
    /// - the `# >>> ven shell hook >>>` … `# <<< ven shell hook <<<` block.
    /// - any orphan unmarked line containing `.ven/bin` (legacy installs).
    pub rc_files_to_clean: Vec<PathBuf>,

    /// `true` when execution will need an admin re-exec to succeed. The
    /// CLI uses this to print a clear message; the actual elevation is
    /// orchestrated at the CLI layer (we don't fork from inside core).
    pub needs_elevation: bool,

    /// Resolved `current_exe()`. The Windows execute path uses this to
    /// rename itself to `*.exe.old` so the install dir is unblocked.
    pub current_exe: Option<PathBuf>,

    pub scope: UninstallScope,
}

/// Per-step audit of what `execute_plan` actually did (or, in dry-run mode,
/// stayed empty). Serializable so the CLI can emit it directly under
/// `--json`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UninstallReport {
    pub removed_dirs: Vec<PathBuf>,
    pub removed_files: Vec<PathBuf>,
    pub stripped_path_entries: Vec<String>,
    pub removed_env_vars: Vec<String>,
    /// Things we couldn't remove right now but the OS will clean up later
    /// (e.g. Windows `ven.exe.old` orphan vanishing on reboot).
    pub deferred_actions: Vec<String>,
    /// Soft failures — execute_plan continues past these. Useful when a
    /// single permission denial shouldn't abort an otherwise-successful
    /// teardown.
    pub warnings: Vec<String>,
    /// Hard failures that probably need follow-up (e.g. the running .exe
    /// couldn't be orphaned). Doesn't bail execute_plan on its own — the
    /// CLI inspects this and decides exit code.
    pub errors: Vec<String>,
}

/// Knobs passed to [`execute_plan`].
#[derive(Debug, Clone, Default)]
pub struct ExecuteOptions {
    /// When `true`, don't touch the filesystem or the user environment.
    /// Returns an empty [`UninstallReport`] — the plan itself is the
    /// dry-run payload the caller should print.
    pub dry_run: bool,
}

// ─────────────────────────────────────────────────────────────────────────
// Plan construction
// ─────────────────────────────────────────────────────────────────────────

/// Discover everything `ven uninstall` would touch on this machine. Pure
/// inspection: never writes, never mutates env state.
pub fn build_plan(scope: UninstallScope) -> Result<UninstallPlan> {
    let data_dir_source = ven_home_source();
    let data_dir = data_dir_source.path().to_path_buf();

    let user_install_root = detect_user_install_root();
    let system_artifacts = detect_system_artifacts();

    let data_dir_is_relocated = match &user_install_root {
        // Default ven home (`~/.ven`) IS the user install root — wholesale
        // removal handles it. Anything else is separate disk to clean.
        Some(root) => !same_path(root, &data_dir),
        None => true,
    };

    let pointer_file = ven_config::config_path().filter(|p| p.is_file());

    let user_path_entries: Vec<PathBuf> = if cfg!(target_os = "windows") && scope.includes_user() {
        user_install_root.iter().cloned().collect()
    } else {
        Vec::new()
    };
    let system_path_entries: Vec<PathBuf> =
        if cfg!(target_os = "windows") && scope.includes_system() {
            system_artifacts
                .iter()
                .filter(|p| p.is_dir())
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

    let user_env_vars: Vec<&'static str> = if scope.includes_user() {
        vec!["VEN_HOME"]
    } else {
        Vec::new()
    };

    let rc_files_to_clean = if scope.includes_user() {
        candidate_rc_files()
    } else {
        Vec::new()
    };

    let needs_elevation = scope.includes_system()
        && system_artifacts.iter().any(|p| p.exists())
        && !running_with_privileges();

    let current_exe = std::env::current_exe().ok();

    Ok(UninstallPlan {
        user_install_root: if scope.includes_user() {
            user_install_root
        } else {
            None
        },
        system_artifacts: if scope.includes_system() {
            system_artifacts
        } else {
            Vec::new()
        },
        data_dir,
        data_dir_source,
        data_dir_is_relocated,
        pointer_file: if scope.includes_user() {
            pointer_file
        } else {
            None
        },
        user_path_entries,
        system_path_entries,
        user_env_vars,
        rc_files_to_clean,
        needs_elevation,
        current_exe,
        scope,
    })
}

// ─────────────────────────────────────────────────────────────────────────
// Plan execution
// ─────────────────────────────────────────────────────────────────────────

/// Execute the plan, collecting per-step results into [`UninstallReport`].
///
/// In `dry_run` mode this returns an empty report immediately — the plan
/// itself is the dry-run payload, and the CLI prints it as such.
///
/// Execution is best-effort: a single permission denial on one rc file
/// doesn't abort the rest of the teardown. The report's `errors` /
/// `warnings` fields capture problems; the CLI's exit code decision lives
/// at the boundary, not here.
pub fn execute_plan(plan: &UninstallPlan, opts: &ExecuteOptions) -> Result<UninstallReport> {
    let mut report = UninstallReport::default();
    if opts.dry_run {
        return Ok(report);
    }

    // 1. Strip rc-file blocks first. This is reversible by re-adding the
    //    block; doing it before the destructive steps means a user who
    //    Ctrl-C's mid-uninstall still has a clean rc file.
    for rc in &plan.rc_files_to_clean {
        if let Err(e) = scrub_rc_file(rc) {
            report
                .warnings
                .push(format!("Could not scrub {}: {}", rc.display(), e));
        }
    }

    // 2. Remove persisted user env vars (VEN_HOME). Uses the same helper
    //    `ven path set` uses to write them, so the teardown stays symmetric
    //    across platforms.
    for var in &plan.user_env_vars {
        match user_env::unset_user_env(var) {
            Ok(()) => report.removed_env_vars.push((*var).to_string()),
            Err(e) => report.warnings.push(format!("Could not unset ${var}: {e}")),
        }
    }

    // 3. Clear the global pointer file. clear_storage_home() removes the
    //    file entirely when it would be otherwise empty, so we don't have
    //    to special-case "stale config left behind".
    if plan.pointer_file.is_some() {
        match ven_config::clear_storage_home() {
            Ok(()) => {
                if let Some(p) = &plan.pointer_file {
                    report.removed_files.push(p.clone());
                }
            }
            Err(e) => report
                .warnings
                .push(format!("Could not clear pointer file: {e}")),
        }
    }

    // 4. Strip Windows PATH entries (registry edits). Done before deleting
    //    the dirs so a partial failure leaves PATH pointing at something
    //    that still exists.
    #[cfg(target_os = "windows")]
    {
        for entry in &plan.user_path_entries {
            let s = entry.display().to_string();
            match strip_path_entry_windows(&s, false) {
                Ok(true) => report.stripped_path_entries.push(s),
                Ok(false) => {}
                Err(e) => report
                    .warnings
                    .push(format!("Could not strip User PATH entry {s}: {e}")),
            }
        }
        for entry in &plan.system_path_entries {
            let s = entry.display().to_string();
            match strip_path_entry_windows(&s, true) {
                Ok(true) => report.stripped_path_entries.push(s),
                Ok(false) => {}
                Err(e) => report
                    .warnings
                    .push(format!("Could not strip Machine PATH entry {s}: {e}")),
            }
        }
    }

    // 5. If the storage root has been relocated, remove it before the user
    //    install root. Order doesn't matter for correctness, but doing the
    //    "remote" dir first means a permission error there doesn't leave
    //    us with a half-removed install at the default location.
    if plan.data_dir_is_relocated && plan.data_dir.exists() {
        remove_path_best_effort(&plan.data_dir, plan.current_exe.as_deref(), &mut report);
    }

    // 6. Remove the user install root (which includes its bin/ and, in the
    //    default install, the storage data too). On Windows the running
    //    .exe lives inside this tree; we orphan it first.
    if let Some(root) = &plan.user_install_root {
        if root.exists() {
            #[cfg(target_os = "windows")]
            {
                if let Some(exe) = plan.current_exe.as_deref() {
                    if path_starts_with(exe, root) {
                        match self_orphan_windows_exe(exe) {
                            Ok(orphan) => report.deferred_actions.push(format!(
                                "Running executable renamed to {} — Windows will free it on reboot.",
                                orphan.display()
                            )),
                            Err(e) => report.errors.push(format!(
                                "Could not self-orphan {} — install dir cleanup will be partial: {e}",
                                exe.display()
                            )),
                        }
                    }
                }
            }
            remove_path_best_effort(root, plan.current_exe.as_deref(), &mut report);
        }
    }

    // 7. Remove system-mode artifacts (typically requires elevation). Each
    //    entry is removed independently — Unix has a handful of file paths
    //    spread across /usr/local/bin and /etc; Windows has one dir.
    for art in &plan.system_artifacts {
        if !art.exists() {
            continue;
        }
        #[cfg(target_os = "windows")]
        {
            if let Some(exe) = plan.current_exe.as_deref() {
                if path_starts_with(exe, art) {
                    match self_orphan_windows_exe(exe) {
                        Ok(orphan) => report.deferred_actions.push(format!(
                            "Running executable renamed to {} — Windows will free it on reboot.",
                            orphan.display()
                        )),
                        Err(e) => report
                            .errors
                            .push(format!("Could not self-orphan {}: {e}", exe.display())),
                    }
                }
            }
        }
        remove_path_best_effort(art, plan.current_exe.as_deref(), &mut report);
    }

    Ok(report)
}

// ─────────────────────────────────────────────────────────────────────────
// Detection helpers
// ─────────────────────────────────────────────────────────────────────────

/// Where ven gets installed when you run `install.{ps1,sh}` without
/// `--system`. Always `$HOME/.ven` (Unix) or `%USERPROFILE%\.ven`
/// (Windows). Returns `None` if the dir doesn't exist on this machine.
fn detect_user_install_root() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let root = home.join(".ven");
    if root.exists() {
        Some(root)
    } else {
        None
    }
}

/// System-mode artifacts: the files / dirs that `install.{ps1,sh}` places
/// outside `$HOME` when run with elevation. Each path is checked for
/// existence by `build_plan`; we return the canonical candidate list so
/// the CLI can show "would remove this if present" cleanly.
fn detect_system_artifacts() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    #[cfg(target_os = "windows")]
    {
        if let Some(pf) = std::env::var_os("ProgramFiles") {
            let dir = PathBuf::from(pf).join("ven");
            if dir.exists() {
                out.push(dir);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        for stem in ["ven", "ven-launcher", "ven-setup"] {
            let p = PathBuf::from("/usr/local/bin").join(stem);
            if p.exists() {
                out.push(p);
            }
        }
        let profile_d = PathBuf::from("/etc/profile.d/ven.sh");
        if profile_d.exists() {
            out.push(profile_d);
        }
    }
    out
}

/// Rc files we'll scan for the ven-managed blocks. Mirrors the union of
/// what `scripts/install.sh` (`# >>> ven-setup PATH >>>`),
/// `core::user_env` (`# >>> ven env >>>`), and `cli::shell` install
/// touch (`# >>> ven shell hook >>>`).
fn candidate_rc_files() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(home) = dirs::home_dir() {
        for name in [
            ".bashrc",
            ".zshrc",
            ".zprofile",
            ".bash_profile",
            ".profile",
        ] {
            v.push(home.join(name));
        }
        v.push(home.join(".config").join("fish").join("config.fish"));
        // PowerShell user profiles (cross-shell: there's no harm scanning
        // them on Unix — the files simply won't exist). The three paths
        // mirror `shell::windows_powershell_profile_paths` so anything
        // `ven shell install` could have written, this can clean up:
        //   - PowerShell 7+ (the default `pwsh` profile)
        //   - Cursor / VS Code's integrated terminal (loads
        //     `Microsoft.VSCode_profile.ps1` instead of the host default)
        //   - Windows PowerShell 5.1 (legacy `WindowsPowerShell` location)
        // Missing any one of these has been observed to leave the broken
        // `__ven_activate` function firing on every prompt — see the
        // v0.1.7 follow-up that added the VSCode profile in particular.
        v.push(
            home.join("Documents")
                .join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
        );
        v.push(
            home.join("Documents")
                .join("PowerShell")
                .join("Microsoft.VSCode_profile.ps1"),
        );
        v.push(
            home.join("Documents")
                .join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
        );
    }
    v
}

// ─────────────────────────────────────────────────────────────────────────
// Rc-file scrubber
// ─────────────────────────────────────────────────────────────────────────

const KNOWN_BLOCKS: &[(&str, &str)] = &[
    ("# >>> ven env >>>", "# <<< ven env <<<"),
    ("# >>> ven-setup PATH >>>", "# <<< ven-setup PATH <<<"),
    ("# >>> ven shell hook >>>", "# <<< ven shell hook <<<"),
];

/// Head markers for the `ven shell install` hook block. Unlike the fenced
/// `KNOWN_BLOCKS` pairs above, the installer in `src/cli/shell.rs` writes
/// these without any closing marker — the hook is just appended to the
/// end of the profile. So the scrubber's strategy is "trim from the
/// earliest head-marker line to end-of-file".
///
/// Listing all three hook flavors AND the wrapper banner means we catch:
///   - profiles touched by `ven shell install` (which prefixes the wrapper
///     banner before the body), and
///   - profiles a user wired up by piping `ven shell hook <shell>` >> rc
///     (no banner, hook body starts directly).
const HOOK_HEAD_MARKERS: &[&str] = &[
    "# ven shell hook - Auto-loads on terminal start",
    "# ven shell hook (bash/zsh)",
    "# ven shell hook (fish)",
    "# ven shell hook (PowerShell)",
];

/// Soft cap on how much trailing content the hook scrub is willing to
/// drop. The hook itself is ~1–2 KB across all three shells; anything
/// past this threshold almost certainly means the user appended their own
/// content after the hook, so we'd rather leave the broken hook in place
/// (and warn) than silently nuke their custom rc additions.
const HOOK_TRIM_BUDGET: usize = 16 * 1024;

/// Read `rc`, strip every ven-managed block AND any unmarked line
/// containing `.ven/bin` (legacy installs that pre-date the fenced-block
/// markers), and write back if anything changed. No-op if the file
/// doesn't exist or has no ven-related content.
fn scrub_rc_file(rc: &Path) -> Result<bool> {
    if !rc.is_file() {
        return Ok(false);
    }
    let original =
        std::fs::read_to_string(rc).with_context(|| format!("Failed to read {}", rc.display()))?;
    let scrubbed = scrub_rc_content(&original);
    if scrubbed != original {
        std::fs::write(rc, scrubbed)
            .with_context(|| format!("Failed to write {}", rc.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Pure-string version of [`scrub_rc_file`] — broken out for unit tests
/// so we can prove that "preserve everything except ven blocks" holds
/// without needing a real tempdir.
fn scrub_rc_content(input: &str) -> String {
    let mut out = input.to_string();

    for (start, end) in KNOWN_BLOCKS {
        loop {
            let Some(s) = out.find(start) else {
                break;
            };
            let Some(rel) = out[s..].find(end) else {
                break;
            };
            let e = s + rel + end.len();
            let mut tail = e;
            // Swallow exactly one trailing newline so we don't leave a blank
            // line behind the removed block.
            if out[tail..].starts_with('\n') {
                tail += 1;
            }
            out = format!("{}{}", &out[..s], &out[tail..]);
        }
    }

    // Strip the unfenced `ven shell hook` block written by `ven shell
    // install` (and by `ven shell hook <shell> >> ~/.bashrc`-style manual
    // setups). Has to run AFTER the fenced-block pass so we don't
    // double-trim — but BEFORE the orphan-line filter, which would
    // otherwise spuriously strip `__VEN_BIN="…/.ven/bin/ven"` lines
    // mid-hook and leave a broken body.
    out = trim_shell_hook_block(&out);

    // Strip any orphan line that still references `.ven/bin` (legacy
    // installs whose PATH lines were never wrapped in a marker block, or
    // bare lines a user added by hand). Keep everything else.
    let filtered: String = out
        .lines()
        .filter(|line| !is_orphan_ven_path_line(line))
        .collect::<Vec<_>>()
        .join("\n");

    // Preserve the trailing newline if the original had one — most shells
    // grumble about rc files missing a final newline.
    let mut result = filtered;
    if out.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// If `content` contains any `# ven shell hook …` head marker, return
/// `content` trimmed from the start of that line to end-of-file. The
/// installer always appends the hook at EOF, so trim-to-EOF is the right
/// inverse — and it's the only way to handle a block that has no closing
/// fence.
///
/// Bails out (returns the input unchanged) when the trim would exceed
/// [`HOOK_TRIM_BUDGET`]; that almost always means the user has appended
/// their own content after the hook, and silently nuking it would be
/// worse than leaving the dead hook in place for a follow-up `ven
/// uninstall` to log a warning about.
fn trim_shell_hook_block(content: &str) -> String {
    let mut earliest: Option<usize> = None;
    for m in HOOK_HEAD_MARKERS {
        if let Some(i) = content.find(m) {
            earliest = match earliest {
                Some(prev) => Some(prev.min(i)),
                None => Some(i),
            };
        }
    }
    let Some(mut start) = earliest else {
        return content.to_string();
    };
    let bytes = content.as_bytes();
    // Walk back to the start of the line that holds the marker so we
    // don't leave a half-stripped line behind.
    while start > 0 && bytes[start - 1] != b'\n' {
        start -= 1;
    }
    // Also eat exactly one blank line immediately above — the installer
    // prefixes `"\n"` before the wrapper banner, and leaving a trailing
    // blank line in an otherwise-clean rc file is a common diff-noise
    // complaint we've heard from users since v0.1.5.
    if start >= 2 && &content[start - 2..start] == "\n\n" {
        start -= 1;
    }
    if content.len() - start > HOOK_TRIM_BUDGET {
        return content.to_string();
    }
    content[..start].to_string()
}

/// `true` for lines like `export PATH="$HOME/.ven/bin:$PATH"` that
/// reference the user install dir directly. We deliberately match on
/// `.ven/bin` (a stable suffix) rather than `$HOME/.ven` because users
/// often expand `$HOME` to a literal `/home/foo/.ven/bin` in scripts.
fn is_orphan_ven_path_line(line: &str) -> bool {
    let l = line.trim();
    if l.is_empty() || l.starts_with('#') {
        return false;
    }
    // Only strip PATH-mutating lines. A user-authored comment that mentions
    // `.ven/bin` shouldn't disappear silently.
    let looks_like_path_export =
        l.contains("PATH") && (l.starts_with("export") || l.starts_with("set "));
    looks_like_path_export && l.contains(".ven/bin")
}

// ─────────────────────────────────────────────────────────────────────────
// Filesystem helpers
// ─────────────────────────────────────────────────────────────────────────

/// Best-effort `rm -rf path`, with a Windows-aware fallback for "the
/// running executable is inside this tree": delete everything we CAN
/// remove and report the orphan as a deferred action. `current_exe` is
/// passed so we can recognise the running .exe.old that we just renamed
/// in step 6/7 of `execute_plan` (it'll be in this tree).
fn remove_path_best_effort(path: &Path, current_exe: Option<&Path>, report: &mut UninstallReport) {
    if !path.exists() {
        return;
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            report
                .errors
                .push(format!("Could not stat {}: {e}", path.display()));
            return;
        }
    };

    if metadata.is_file() || metadata.file_type().is_symlink() {
        match std::fs::remove_file(path) {
            Ok(()) => report.removed_files.push(path.to_path_buf()),
            Err(e) => report
                .errors
                .push(format!("Could not remove {}: {e}", path.display())),
        }
        return;
    }

    // Directory: try the fast path first.
    match std::fs::remove_dir_all(path) {
        Ok(()) => {
            report.removed_dirs.push(path.to_path_buf());
            return;
        }
        Err(e) => {
            // On Windows, a locked .exe.old (the orphan we just created)
            // will keep us from clearing the dir. Fall through to
            // per-entry removal so we get partial cleanup.
            #[cfg(not(target_os = "windows"))]
            {
                report
                    .errors
                    .push(format!("Could not remove dir {}: {e}", path.display()));
                return;
            }
            #[cfg(target_os = "windows")]
            {
                let _ = e;
            }
        }
    }

    // Per-entry walk: remove what we can, skip locked .exe.old files,
    // record the rest as a deferred action.
    #[cfg(target_os = "windows")]
    {
        let mut skipped_orphans: Vec<PathBuf> = Vec::new();
        let walker = walkdir::WalkDir::new(path).contents_first(true);
        for entry in walker.into_iter().filter_map(|r| r.ok()) {
            let p = entry.path();
            if is_skip_orphan(p, current_exe) {
                skipped_orphans.push(p.to_path_buf());
                continue;
            }
            let ft = entry.file_type();
            let res = if ft.is_dir() {
                std::fs::remove_dir(p)
            } else {
                std::fs::remove_file(p)
            };
            match res {
                Ok(()) => {
                    if ft.is_dir() {
                        report.removed_dirs.push(p.to_path_buf());
                    } else {
                        report.removed_files.push(p.to_path_buf());
                    }
                }
                Err(e) => {
                    // A dir that still contains a skipped orphan can't be
                    // removed; that's expected, not an error.
                    let still_contains_orphan = skipped_orphans.iter().any(|o| o.starts_with(p));
                    if !still_contains_orphan {
                        report
                            .errors
                            .push(format!("Could not remove {}: {e}", p.display()));
                    }
                }
            }
        }
        if !skipped_orphans.is_empty() {
            report.deferred_actions.push(format!(
                "{} orphan file(s) under {} are still locked by the running ven process; \
                 they will vanish on reboot, after which {} can be removed by hand.",
                skipped_orphans.len(),
                path.display(),
                path.display()
            ));
        }
    }

    // Belt-and-braces on non-Windows: we already returned above on success.
    #[cfg(not(target_os = "windows"))]
    {
        let _ = current_exe;
    }
}

#[cfg(target_os = "windows")]
fn is_skip_orphan(p: &Path, current_exe: Option<&Path>) -> bool {
    // The actual orphan we created via self_orphan_windows_exe().
    if let Some(exe) = current_exe {
        let stale = exe.with_extension("exe.old");
        if same_path(p, &stale) {
            return true;
        }
    }
    // And any other *.exe.old leftover from a previous install/uninstall
    // attempt that's still locked.
    if let Some(ext) = p.extension() {
        if ext.eq_ignore_ascii_case("old") {
            if let Some(stem) = p.file_stem() {
                let s = stem.to_string_lossy().to_lowercase();
                if s.ends_with(".exe") {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn is_skip_orphan(_p: &Path, _current_exe: Option<&Path>) -> bool {
    false
}

/// Rename the running .exe to `*.exe.old`. Windows allows the rename even
/// while the file is in use; the rename frees the *original* path so a
/// subsequent install can write a new ven.exe there, and the .exe.old
/// vanishes on reboot.
///
/// Returns the path the file was renamed to.
///
/// Extracted from `src/cli/update.rs::replace_in_place` so `update` and
/// `uninstall` share one implementation — if you patch one, audit the other.
#[cfg(target_os = "windows")]
pub fn self_orphan_windows_exe(exe: &Path) -> Result<PathBuf> {
    let stale = exe.with_extension("exe.old");
    let _ = std::fs::remove_file(&stale);
    std::fs::rename(exe, &stale).with_context(|| {
        format!(
            "Could not move {} -> {} (is another ven process running?)",
            exe.display(),
            stale.display()
        )
    })?;
    Ok(stale)
}

/// No-op on Unix — the inode keeps the running process alive after unlink
/// so we don't need the orphan dance. Defined so the signature stays
/// callable from cross-platform code.
#[cfg(not(target_os = "windows"))]
pub fn self_orphan_windows_exe(exe: &Path) -> Result<PathBuf> {
    Ok(exe.to_path_buf())
}

// ─────────────────────────────────────────────────────────────────────────
// Windows PATH editing
// ─────────────────────────────────────────────────────────────────────────

/// Remove a single entry from the User or Machine PATH on Windows. Returns
/// `Ok(true)` when the entry was found and stripped, `Ok(false)` when it
/// wasn't present (no-op). Uses the same `WM_SETTINGCHANGE` broadcast
/// pattern as [`crate::core::user_env`] so already-open Explorer / shells
/// pick up the change.
#[cfg(target_os = "windows")]
fn strip_path_entry_windows(entry: &str, machine_scope: bool) -> Result<bool> {
    let scope = if machine_scope { "Machine" } else { "User" };
    let entry_ps = entry.replace('\'', "''");
    let script = format!(
        r#"$scope = '{scope}'
$entry = '{entry_ps}'
$current = [Environment]::GetEnvironmentVariable('PATH', $scope)
if (-not $current) {{ Write-Output 'NOOP'; exit 0 }}
$parts = $current -split ';' | Where-Object {{ $_ -ne '' }}
$kept = $parts | Where-Object {{ $_.TrimEnd('\\') -ne $entry.TrimEnd('\\') }}
if ($kept.Count -eq $parts.Count) {{
    Write-Output 'NOOP'
    exit 0
}}
$new = ($kept -join ';')
[Environment]::SetEnvironmentVariable('PATH', $new, $scope)

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
namespace Win32VenUninstall {{
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
[Win32VenUninstall.Native]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result) | Out-Null
Write-Output 'STRIPPED'"#
    );

    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .output()
        .context("Failed to spawn powershell.exe")?;
    if !output.status.success() {
        anyhow::bail!(
            "PowerShell PATH edit exited with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains("STRIPPED"))
}

// ─────────────────────────────────────────────────────────────────────────
// Privilege detection
// ─────────────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn running_with_privileges() -> bool {
    // Cheap probe: try writing to `%ProgramFiles%`. If that succeeds we're
    // either Admin or the dir already happens to be writable to us, which
    // for the purpose of `needs_elevation` is the same answer.
    let Some(pf) = std::env::var_os("ProgramFiles") else {
        return false;
    };
    let probe = PathBuf::from(pf).join(".ven-uninstall-probe");
    match std::fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn running_with_privileges() -> bool {
    // Pulling in libc just for `geteuid` is heavy for this one call; a
    // write probe answers the only question we actually have — "can we
    // touch the system install dir?" — and is correct for sudo, doas, and
    // a passwordless root shell alike.
    let probe = PathBuf::from("/usr/local/bin/.ven-uninstall-probe");
    match std::fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Path helpers
// ─────────────────────────────────────────────────────────────────────────

/// `Path` equality that tolerates trailing separators and case differences
/// on Windows. We use `canonicalize` only when both paths exist; otherwise
/// fall back to a normalized string compare.
fn same_path(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    if let (Ok(ca), Ok(cb)) = (a.canonicalize(), b.canonicalize()) {
        return ca == cb;
    }
    let na = normalize_for_compare(a);
    let nb = normalize_for_compare(b);
    na == nb
}

fn normalize_for_compare(p: &Path) -> String {
    let mut s = p.display().to_string();
    if cfg!(target_os = "windows") {
        s = s.replace('/', "\\").to_lowercase();
        while s.ends_with('\\') {
            s.pop();
        }
    } else {
        while s.ends_with('/') {
            s.pop();
        }
    }
    s
}

/// `child` lives inside `parent` (or equals it).
fn path_starts_with(child: &Path, parent: &Path) -> bool {
    if child == parent {
        return true;
    }
    if let (Ok(cc), Ok(cp)) = (child.canonicalize(), parent.canonicalize()) {
        return cc.starts_with(&cp);
    }
    let nc = normalize_for_compare(child);
    let np = normalize_for_compare(parent);
    if cfg!(target_os = "windows") {
        nc == np || nc.starts_with(&format!("{np}\\"))
    } else {
        nc == np || nc.starts_with(&format!("{np}/"))
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lock_test_env as lock_env;

    /// Mutex-protected env scrubber used by tests that mutate process-wide
    /// env state — same convention as `ven_home::tests` and
    /// `ven_config::tests`.
    struct EnvGuard {
        keys: Vec<&'static str>,
        prev: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            let prev = keys
                .iter()
                .map(|k| (*k, std::env::var(k).ok()))
                .collect::<Vec<_>>();
            for k in keys {
                std::env::remove_var(k);
            }
            Self {
                keys: keys.to_vec(),
                prev,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for k in &self.keys {
                std::env::remove_var(k);
            }
            for (k, v) in &self.prev {
                if let Some(val) = v {
                    std::env::set_var(k, val);
                }
            }
        }
    }

    #[test]
    fn scrub_strips_ven_env_block() {
        let block = "# >>> ven env >>>\nexport VEN_HOME=\"/tmp/x\"\n# <<< ven env <<<";
        let input = format!("# user content\n{block}\nalias l='ls'\n");
        let out = scrub_rc_content(&input);
        assert!(out.contains("# user content"));
        assert!(out.contains("alias l='ls'"));
        assert!(!out.contains("VEN_HOME"));
        assert!(!out.contains("ven env"));
    }

    #[test]
    fn scrub_strips_setup_path_block() {
        let block =
            "# >>> ven-setup PATH >>>\nexport PATH=\"$HOME/.ven/bin:$PATH\"\n# <<< ven-setup PATH <<<";
        let input = format!("export EDITOR=nano\n{block}\n");
        let out = scrub_rc_content(&input);
        assert!(out.contains("EDITOR=nano"));
        assert!(!out.contains("ven-setup PATH"));
        assert!(!out.contains(".ven/bin"));
    }

    #[test]
    fn scrub_strips_orphan_ven_path_line() {
        let input = "alias k='kubectl'\nexport PATH=\"$HOME/.ven/bin:$PATH\"\n# keep me\n";
        let out = scrub_rc_content(input);
        assert!(out.contains("alias k='kubectl'"));
        assert!(out.contains("# keep me"));
        assert!(!out.contains(".ven/bin"));
    }

    #[test]
    fn scrub_preserves_comment_mentioning_ven_bin() {
        // A user-written comment that *mentions* .ven/bin must NOT vanish —
        // only PATH-mutating lines do.
        let input = "# remember: .ven/bin is the install dir\nexport EDITOR=vim\n";
        let out = scrub_rc_content(input);
        assert_eq!(out, input, "comment should be preserved verbatim");
    }

    #[test]
    fn scrub_handles_no_matches_as_noop() {
        let input = "alias l='ls -la'\nexport EDITOR=vim\n";
        let out = scrub_rc_content(input);
        assert_eq!(out, input);
    }

    #[test]
    fn scrub_strips_multiple_blocks_in_one_pass() {
        let env_block = "# >>> ven env >>>\nexport VEN_HOME=\"/x\"\n# <<< ven env <<<";
        let path_block =
            "# >>> ven-setup PATH >>>\nexport PATH=\"$HOME/.ven/bin:$PATH\"\n# <<< ven-setup PATH <<<";
        let input = format!("{env_block}\n# middle\n{path_block}\n");
        let out = scrub_rc_content(&input);
        assert!(!out.contains("ven env"));
        assert!(!out.contains("ven-setup PATH"));
        assert!(out.contains("# middle"));
    }

    #[test]
    fn scrub_strips_powershell_hook_block_without_closing_marker() {
        // Reproduces the v0.1.7 leak: `ven shell install` on Windows writes
        // the hook with the `# ven shell hook (PowerShell)` marker and no
        // closing fence. Pre-fix, the scrubber missed it entirely and the
        // dead `__ven_activate` kept spamming `Write-Warning` on every
        // PowerShell prompt after `ven uninstall`.
        let prior = "Set-Alias g git\n$env:EDITOR = 'nvim'\n";
        let hook = "\n# ven shell hook - Auto-loads on terminal start\n\
                    \n# ven shell hook (PowerShell) - Auto-switches runtimes on cd / Set-Location\n\
                    if (-not $global:VEN_ORIGINAL_PATH) { $global:VEN_ORIGINAL_PATH = $env:PATH }\n\
                    $global:VEN_BIN = \"C:\\Users\\me\\.ven\\bin\\ven.exe\"\n\
                    function global:__ven_activate { Write-Warning 'ven: gone' }\n\
                    function global:prompt { __ven_activate; '> ' }\n";
        let input = format!("{prior}{hook}");
        let out = scrub_rc_content(&input);
        assert!(out.contains("Set-Alias g git"), "user content preserved");
        assert!(out.contains("EDITOR = 'nvim'"), "user content preserved");
        assert!(
            !out.contains("ven shell hook"),
            "hook marker fully stripped"
        );
        assert!(
            !out.contains("__ven_activate"),
            "hook body fully stripped"
        );
        assert!(
            !out.contains("VEN_ORIGINAL_PATH"),
            "hook body fully stripped"
        );
    }

    #[test]
    fn scrub_strips_bash_hook_block_without_closing_marker() {
        let prior = "alias ll='ls -lah'\nexport EDITOR=vim\n";
        let hook = "\n# ven shell hook - Auto-loads on terminal start\n\
                    \n# ven shell hook (bash/zsh) - Auto-switches runtimes on cd\n\
                    __VEN_ORIGINAL_PATH=\"$PATH\"\n\
                    __VEN_BIN=\"$HOME/.ven/bin/ven\"\n\
                    __ven_activate() { :; }\n\
                    cd() { builtin cd \"$@\" && __ven_activate; }\n\
                    __ven_activate\n";
        let input = format!("{prior}{hook}");
        let out = scrub_rc_content(&input);
        assert!(out.contains("alias ll='ls -lah'"));
        assert!(out.contains("EDITOR=vim"));
        assert!(!out.contains("ven shell hook"));
        assert!(!out.contains("__ven_activate"));
        assert!(
            !out.contains("__VEN_BIN"),
            "the embedded $HOME/.ven/bin reference must go with the hook"
        );
    }

    #[test]
    fn scrub_preserves_content_after_oversize_hook_match() {
        // Defensive: if some plugin appends >16 KB after the hook marker
        // we'd rather keep the broken hook around (and let a future
        // uninstall pass log a warning) than silently nuke the user's
        // custom content. Build a payload past the budget with a marker
        // near the start, and confirm nothing was trimmed.
        let mut input = String::from("# ven shell hook (bash/zsh)\nbody\n");
        input.push_str(&"x".repeat(HOOK_TRIM_BUDGET + 16));
        let out = scrub_rc_content(&input);
        assert!(
            out.contains("ven shell hook"),
            "oversize trim must be a no-op so we don't shred custom content"
        );
        assert_eq!(out.len(), input.len(), "no bytes should have been removed");
    }

    #[test]
    fn scope_includes_logic() {
        assert!(UninstallScope::All.includes_user());
        assert!(UninstallScope::All.includes_system());
        assert!(UninstallScope::UserOnly.includes_user());
        assert!(!UninstallScope::UserOnly.includes_system());
        assert!(!UninstallScope::SystemOnly.includes_user());
        assert!(UninstallScope::SystemOnly.includes_system());
    }

    #[test]
    fn build_plan_honors_relocated_ven_home() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        std::env::set_var("VEN_HOME", "/tmp/relocated-ven-uninstaller-test");
        let plan = build_plan(UninstallScope::All).expect("plan should build");
        assert_eq!(
            plan.data_dir,
            PathBuf::from("/tmp/relocated-ven-uninstaller-test")
        );
        assert!(plan.data_dir_is_relocated, "should detect relocation");
        assert_eq!(plan.data_dir_source.kind(), "env:VEN_HOME");
    }

    #[test]
    fn execute_plan_dry_run_does_not_touch_filesystem() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        // Seed a fake "user install" tree so we know it's not removed.
        let temp = tempfile::tempdir().expect("tempdir");
        let fake_root = temp.path().join(".ven-fake-uninstall");
        std::fs::create_dir_all(fake_root.join("bin")).expect("mkdir");
        std::fs::write(fake_root.join("bin").join("ven"), b"#!/bin/sh\necho fake")
            .expect("write fake ven");

        // Build a custom plan pointing at the fake root.
        let plan = UninstallPlan {
            user_install_root: Some(fake_root.clone()),
            system_artifacts: Vec::new(),
            data_dir: fake_root.clone(),
            data_dir_source: HomeSource::Default(fake_root.clone()),
            data_dir_is_relocated: false,
            pointer_file: None,
            user_path_entries: Vec::new(),
            system_path_entries: Vec::new(),
            user_env_vars: Vec::new(),
            rc_files_to_clean: Vec::new(),
            needs_elevation: false,
            current_exe: None,
            scope: UninstallScope::UserOnly,
        };

        let report =
            execute_plan(&plan, &ExecuteOptions { dry_run: true }).expect("dry-run should succeed");

        assert!(
            report.removed_dirs.is_empty(),
            "dry-run must not remove dirs"
        );
        assert!(
            report.removed_files.is_empty(),
            "dry-run must not remove files"
        );
        assert!(
            report.removed_env_vars.is_empty(),
            "dry-run must not unset env"
        );
        assert!(
            fake_root.exists(),
            "fake install tree must still exist after dry-run"
        );
        assert!(
            fake_root.join("bin").join("ven").exists(),
            "fake ven binary must still exist after dry-run"
        );
    }

    #[test]
    fn execute_plan_removes_user_install_tree() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        let temp = tempfile::tempdir().expect("tempdir");
        let fake_root = temp.path().join(".ven-real-uninstall");
        std::fs::create_dir_all(fake_root.join("bin")).expect("mkdir");
        std::fs::write(fake_root.join("bin").join("ven"), b"fake").expect("write");
        std::fs::create_dir_all(fake_root.join("node").join("20.0.0")).expect("mkdir");

        let plan = UninstallPlan {
            user_install_root: Some(fake_root.clone()),
            system_artifacts: Vec::new(),
            data_dir: fake_root.clone(),
            data_dir_source: HomeSource::Default(fake_root.clone()),
            data_dir_is_relocated: false,
            pointer_file: None,
            user_path_entries: Vec::new(),
            system_path_entries: Vec::new(),
            user_env_vars: Vec::new(),
            rc_files_to_clean: Vec::new(),
            needs_elevation: false,
            current_exe: None,
            scope: UninstallScope::UserOnly,
        };

        let report =
            execute_plan(&plan, &ExecuteOptions::default()).expect("execute should succeed");
        assert!(!fake_root.exists(), "fake root should be gone");
        assert_eq!(report.errors.len(), 0, "no errors expected, got {report:?}");
    }

    #[test]
    fn scrub_rc_file_writes_back_only_on_change() {
        let temp = tempfile::tempdir().expect("tempdir");
        let rc = temp.path().join(".bashrc");

        std::fs::write(&rc, "alias l='ls'\n").expect("write");
        let mtime_before = std::fs::metadata(&rc)
            .and_then(|m| m.modified())
            .expect("mtime");

        // No ven content → no rewrite.
        assert!(!scrub_rc_file(&rc).unwrap());
        let after_noop = std::fs::read_to_string(&rc).unwrap();
        assert_eq!(after_noop, "alias l='ls'\n");

        // Add a ven block → rewrite.
        std::fs::write(
            &rc,
            "alias l='ls'\n# >>> ven env >>>\nexport VEN_HOME=/x\n# <<< ven env <<<\n",
        )
        .expect("write");
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(scrub_rc_file(&rc).unwrap());
        let after_change = std::fs::read_to_string(&rc).unwrap();
        assert!(after_change.contains("alias l='ls'"));
        assert!(!after_change.contains("ven env"));
        // The mtime check is defensive — we don't strictly assert it
        // changed (filesystems with second-resolution may not budge) but
        // we DID verify content changed above.
        let _ = mtime_before;
    }

    #[test]
    fn same_path_handles_trailing_separators() {
        let a = PathBuf::from("/tmp/foo");
        let b = PathBuf::from("/tmp/foo/");
        assert!(same_path(&a, &b));
    }

    #[test]
    fn path_starts_with_detects_nested_paths() {
        let parent = PathBuf::from("/tmp/foo");
        let child = PathBuf::from("/tmp/foo/bar/baz");
        assert!(path_starts_with(&child, &parent));
        assert!(path_starts_with(&parent, &parent));
        assert!(!path_starts_with(&parent, &child));
        assert!(!path_starts_with(
            &PathBuf::from("/tmp/foobar"),
            &PathBuf::from("/tmp/foo")
        ));
    }
}
