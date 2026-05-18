//! Global ven configuration — the small TOML file that lives next to other
//! per-user app configs (XDG `~/.config/ven/config.toml` on Linux, `%APPDATA%`
//! on Windows, `~/Library/Application Support` on macOS).
//!
//! This is **not** `ven.toml`. `ven.toml` is per-project and pinned in the
//! source tree. This file is per-user and tracks settings that are about *ven
//! itself*, not about any particular project. As of v0.1.6 it only carries the
//! relocated storage root (the "pointer file" for `$VEN_HOME`):
//!
//! ```toml
//! [storage]
//! home = "D:\\ven"
//! set_at = "2026-05-17T10:46:00Z"
//! ```
//!
//! Resolution order for `$VEN_HOME` (see [`crate::core::ven_home`]):
//!
//! 1. `$VEN_HOME` env var
//! 2. `$VEN_STORAGE_PATH` env var (back-compat)
//! 3. `<exe-dir>/.ven` (portable mode)
//! 4. **this file's `[storage].home`** ← inserted by v0.1.6
//! 5. `~/.ven` (default)
//!
//! Per-process env always wins so CI scripts and one-shot overrides keep
//! working. The pointer is for "I moved my data to a different drive once and
//! want ven to remember".

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Subdirectory under the OS config dir where ven keeps its global state.
const APP_DIR: &str = "ven";
/// Filename of the global ven config inside [`APP_DIR`].
const CONFIG_FILE: &str = "config.toml";

/// Schema mirror of the on-disk file. New top-level sections can be added
/// later (telemetry opt-out, default plugins, etc.) without breaking
/// existing config files — every section is optional and defaults to empty.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenGlobalConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Absolute path to the relocated $VEN_HOME. Empty / missing means "no
    /// override; fall through to the next resolver step".
    pub home: PathBuf,
    /// ISO-8601 UTC timestamp of when this entry was written. Audit-only —
    /// the resolver never reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_at: Option<String>,
}

/// Return the absolute path to the global ven config file, or `None` on the
/// (extremely rare) platforms where `dirs::config_dir()` can't resolve a
/// per-user config directory.
///
/// Does **not** check whether the file exists. Callers that need
/// existence-or-default semantics should use [`load`].
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(APP_DIR).join(CONFIG_FILE))
}

/// Read and parse the global config file.
///
/// - Returns `Ok(None)` when the file does not exist (a missing config is the
///   default, not an error).
/// - Returns `Err` when the file exists but cannot be read or parsed, so a
///   typo'd TOML doesn't get silently ignored and trick the user into thinking
///   their pointer was honored.
pub fn load() -> Result<Option<VenGlobalConfig>> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let cfg: VenGlobalConfig = toml::from_str(&bytes)
        .with_context(|| format!("Failed to parse TOML in {}", path.display()))?;
    Ok(Some(cfg))
}

/// Atomically write the global config: serialize to a sibling `.tmp` file
/// and only rename into place once the write is complete. Prevents a crash
/// mid-write from leaving a half-written `config.toml` that the next ven
/// invocation would reject.
pub fn save(cfg: &VenGlobalConfig) -> Result<()> {
    let path = config_path().context("Could not resolve a per-user config directory on this OS")?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Config path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create {}", parent.display()))?;

    let body = toml::to_string_pretty(cfg).context("Failed to serialize ven global config")?;

    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, body).with_context(|| format!("Failed to write {}", tmp.display()))?;
    std::fs::rename(&tmp, &path).with_context(|| {
        format!(
            "Failed to atomically replace {} (left tmp at {})",
            path.display(),
            tmp.display()
        )
    })?;
    Ok(())
}

/// Fast path the [`ven_home`] resolver uses: "is there a non-empty
/// `[storage].home` pointer right now?". Returns `None` whenever resolving
/// or reading the file fails, so a corrupt config never crashes the
/// resolver — the worst case degrades to the historic default (`~/.ven`),
/// which is identical to pre-v0.1.6 behavior.
///
/// For callers that actually care about parse errors (the `ven path`
/// command itself), call [`load`] directly.
pub fn pointer_home() -> Option<PathBuf> {
    let cfg = load().ok()??;
    let storage = cfg.storage?;
    if storage.home.as_os_str().is_empty() {
        None
    } else {
        Some(storage.home)
    }
}

/// Set the storage pointer to `home` and record the current time in `set_at`.
/// Preserves any other sections that may exist in the file.
pub fn set_storage_home(home: PathBuf) -> Result<()> {
    let mut cfg = load()?.unwrap_or_default();
    cfg.storage = Some(StorageConfig {
        home,
        set_at: Some(iso8601_utc_now()),
    });
    save(&cfg)
}

/// Remove the storage pointer (`ven path reset`). Preserves any other
/// sections in the file. If the file would become empty, deletes it so a
/// `ven path show` can honestly report "no pointer set".
pub fn clear_storage_home() -> Result<()> {
    let mut cfg = match load()? {
        Some(c) => c,
        None => return Ok(()),
    };
    cfg.storage = None;
    if cfg == VenGlobalConfig::default() {
        if let Some(path) = config_path() {
            if path.is_file() {
                std::fs::remove_file(&path)
                    .with_context(|| format!("Failed to delete {}", path.display()))?;
            }
        }
        return Ok(());
    }
    save(&cfg)
}

/// `YYYY-MM-DDTHH:MM:SSZ` from `SystemTime::now()` without pulling in chrono.
/// Tracks days-from-epoch with the Howard Hinnant civil-day algorithm so we
/// don't drift on leap years. Audit-only: ven never parses this back.
fn iso8601_utc_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let days_since_epoch = (secs / 86_400) as i64;
    let time_of_day = secs % 86_400;
    let hour = (time_of_day / 3600) as u32;
    let minute = ((time_of_day % 3600) / 60) as u32;
    let second = (time_of_day % 60) as u32;

    let (year, month, day) = civil_from_days(days_since_epoch);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

/// Howard Hinnant's `civil_from_days` — converts days-since-1970-01-01 (proleptic
/// Gregorian) into `(year, month, day)`. Public-domain reference algorithm,
/// chosen because it is leap-year correct without a calendar dep.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────
    // The config-dir-dependent tests below are skipped on Windows because
    // `dirs::config_dir()` on Windows calls the Win32
    // `SHGetKnownFolderPath(FOLDERID_RoamingAppData)` shell API, which does
    // NOT read the `APPDATA` env var — so the `ConfigDirRedirect` mechanism
    // doesn't actually isolate them on Windows runners. Running them there
    // would write into the real `%APPDATA%\ven\config.toml` and race
    // against other tests, polluting the runner's environment and causing
    // mutex-poison cascades.
    //
    // The pointer-file code path itself is platform-agnostic (no
    // `#[cfg(windows)]` in production code) so the Linux/macOS runs of
    // these tests prove the same behavior holds on Windows. Manual end-
    // to-end testing of `ven path set` on Windows covers the real
    // dirs::config_dir() integration.
    //
    // `civil_from_days_matches_known_dates` is pure-math, no env access,
    // so it runs on every platform.
    // ─────────────────────────────────────────────────────────────────────

    // Shared with `ven_home::tests` — see the rationale in `core/mod.rs`.
    // Per-module locks would let tests in the two modules run concurrently
    // and trample each other's $HOME / $XDG_CONFIG_HOME / $APPDATA. On
    // macOS that surfaced as `round_trip_storage_home` panicking with
    // "config should exist after save" — the file did get written, but
    // the other module's Drop restored $HOME before we re-read it, so
    // `dirs::config_dir()` came back pointing at the runner's real home.
    #[cfg(not(target_os = "windows"))]
    use crate::core::lock_test_env as lock_env;

    /// Repoint `dirs::config_dir()` at a fresh tempdir for the duration of
    /// the test, restoring whatever was there before on drop.
    #[cfg(not(target_os = "windows"))]
    struct ConfigDirRedirect {
        _guard: std::sync::MutexGuard<'static, ()>,
        _temp: tempfile::TempDir,
        prev: Vec<(&'static str, Option<String>)>,
    }

    #[cfg(not(target_os = "windows"))]
    impl ConfigDirRedirect {
        fn new() -> Self {
            let guard = lock_env();
            let temp = tempfile::tempdir().expect("tempdir");
            let path = temp.path().to_path_buf();

            // Save current values so Drop can restore them.
            let keys = ["HOME", "XDG_CONFIG_HOME", "APPDATA"];
            let prev: Vec<_> = keys.iter().map(|k| (*k, std::env::var(k).ok())).collect();

            // Linux / fallback macOS: `dirs::config_dir()` honors
            // XDG_CONFIG_HOME first, then $HOME/.config. Setting both pins
            // the resolver to our tempdir regardless of which leg `dirs`
            // takes on this platform.
            std::env::set_var("XDG_CONFIG_HOME", &path);
            std::env::set_var("HOME", &path);
            // Windows: `dirs::config_dir()` ignores APPDATA (uses Known Folders),
            // which is exactly why these tests are #[cfg(not(target_os = "windows"))].
            std::env::set_var("APPDATA", &path);

            Self {
                _guard: guard,
                _temp: temp,
                prev,
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    impl Drop for ConfigDirRedirect {
        fn drop(&mut self) {
            for (k, v) in &self.prev {
                match v {
                    Some(val) => std::env::set_var(k, val),
                    None => std::env::remove_var(k),
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn missing_file_returns_none() {
        let _r = ConfigDirRedirect::new();
        assert!(load().unwrap().is_none());
        assert!(pointer_home().is_none());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn round_trip_storage_home() {
        let _r = ConfigDirRedirect::new();

        let target = PathBuf::from("/tmp/relocated-ven");
        set_storage_home(target.clone()).unwrap();

        let cfg = load().unwrap().expect("config should exist after save");
        let storage = cfg.storage.expect("storage section should be present");
        assert_eq!(storage.home, target);
        assert!(
            storage.set_at.as_deref().is_some_and(|s| s.ends_with('Z')),
            "set_at should be ISO 8601 UTC, got {:?}",
            storage.set_at
        );

        assert_eq!(pointer_home(), Some(target));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn clear_storage_removes_the_file_when_no_other_sections_exist() {
        let _r = ConfigDirRedirect::new();
        set_storage_home(PathBuf::from("/tmp/x")).unwrap();
        assert!(config_path().unwrap().is_file());

        clear_storage_home().unwrap();
        assert!(!config_path().unwrap().is_file());
        assert!(pointer_home().is_none());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn malformed_file_is_an_error_not_silent_none() {
        let _r = ConfigDirRedirect::new();
        let path = config_path().unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"[[not-valid-toml\n").unwrap();

        let err = load().unwrap_err();
        assert!(
            err.to_string().contains("Failed to parse TOML"),
            "expected parse error, got: {}",
            err
        );
        // pointer_home() must NOT propagate the error — it's the
        // resolver fast-path and the worst case must degrade silently
        // to the next resolver step, not crash every ven command.
        assert!(pointer_home().is_none());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn empty_storage_home_is_treated_as_no_pointer() {
        let _r = ConfigDirRedirect::new();
        let path = config_path().unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "[storage]\nhome = \"\"\n").unwrap();

        assert!(pointer_home().is_none());
    }

    #[test]
    fn civil_from_days_matches_known_dates() {
        // 1970-01-01 = day 0
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-01-01 = day 10957 (across two centuries' leap rules)
        assert_eq!(civil_from_days(10_957), (2000, 1, 1));
        // 2024-02-29 = day 19782 (leap year)
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        // 2026-05-17 (today, plan creation date)
        assert_eq!(civil_from_days(20_590), (2026, 5, 17));
    }
}
