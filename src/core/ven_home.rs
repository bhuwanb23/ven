//! Single source of truth for the ven storage root (a.k.a. `VEN_HOME`).
//!
//! Resolution order (most → least specific):
//!
//! 1. `$VEN_HOME` environment variable, if set and non-empty. Explicit user
//!    override; trumps everything else.
//! 2. `$VEN_STORAGE_PATH` environment variable, if set and non-empty.
//!    Back-compat with the convention used by some early modules
//!    (`python_install`, `osv`, `endoflife`, `doc_fetcher`,
//!    `intelligence::store`).
//! 3. `<dir-of-current-exe>/.ven` if that directory exists. This is what
//!    enables "USB-stick portable" mode: drop the launcher anywhere, create
//!    a sibling `.ven/` folder, and ven keeps every runtime / cache / lock
//!    state inside the bundle without touching `$HOME`.
//! 4. Pointer in the global ven config file ([`ven_config::pointer_home`]).
//!    This is what `ven path set <dir>` writes (since v0.1.6) so a user who
//!    relocates `~/.ven` to a different drive doesn't have to also set an
//!    env var by hand.
//! 5. `~/.ven` — the default for an installed ven on a user's machine.
//!
//! Every consumer of the ven storage root MUST go through [`ven_home`] so the
//! five cases stay coherent. Hardcoding `dirs::home_dir().join(".ven")`
//! anywhere in the codebase silently breaks portable mode and the pointer.

use crate::core::ven_config;
use std::path::PathBuf;

/// Resolve the active ven storage root for this process.
///
/// See the module docs for the full precedence rules. Falls back to a literal
/// `.ven` in the working directory only if `dirs::home_dir()` returns `None`,
/// which is essentially never on a real OS.
pub fn ven_home() -> PathBuf {
    if let Ok(p) = std::env::var("VEN_HOME") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(p) = std::env::var("VEN_STORAGE_PATH") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let portable = dir.join(".ven");
            if portable.is_dir() {
                return portable;
            }
        }
    }
    if let Some(pointer) = ven_config::pointer_home() {
        return pointer;
    }
    dirs::home_dir()
        .map(|h| h.join(".ven"))
        .unwrap_or_else(|| PathBuf::from(".ven"))
}

/// Discriminated description of which resolver step produced the current
/// `ven_home()` value. Used by `ven path show` to explain to the user which
/// knob is currently in effect, and by `ven path set` to warn when an env
/// var will shadow the pointer they're about to write.
///
/// Resolver step is computed identically to [`ven_home`] — same precedence,
/// same emptiness rules — so the two cannot disagree. If they ever do, treat
/// it as a bug in this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeSource {
    /// `$VEN_HOME` env var.
    EnvVenHome(PathBuf),
    /// `$VEN_STORAGE_PATH` env var (back-compat).
    EnvVenStoragePath(PathBuf),
    /// `<exe-dir>/.ven` exists alongside the launcher (portable mode).
    PortableSibling(PathBuf),
    /// Pointer in `~/.config/ven/config.toml` `[storage].home`.
    Pointer(PathBuf),
    /// Default — no override active.
    Default(PathBuf),
}

impl HomeSource {
    pub fn path(&self) -> &std::path::Path {
        match self {
            HomeSource::EnvVenHome(p)
            | HomeSource::EnvVenStoragePath(p)
            | HomeSource::PortableSibling(p)
            | HomeSource::Pointer(p)
            | HomeSource::Default(p) => p,
        }
    }

    /// Short, machine-readable identifier (used in `--json` output and the
    /// "Source: ..." line of `ven path show`).
    pub fn kind(&self) -> &'static str {
        match self {
            HomeSource::EnvVenHome(_) => "env:VEN_HOME",
            HomeSource::EnvVenStoragePath(_) => "env:VEN_STORAGE_PATH",
            HomeSource::PortableSibling(_) => "portable",
            HomeSource::Pointer(_) => "pointer",
            HomeSource::Default(_) => "default",
        }
    }
}

/// Same precedence as [`ven_home`], but also tells you *why*.
pub fn ven_home_source() -> HomeSource {
    if let Ok(p) = std::env::var("VEN_HOME") {
        if !p.is_empty() {
            return HomeSource::EnvVenHome(PathBuf::from(p));
        }
    }
    if let Ok(p) = std::env::var("VEN_STORAGE_PATH") {
        if !p.is_empty() {
            return HomeSource::EnvVenStoragePath(PathBuf::from(p));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let portable = dir.join(".ven");
            if portable.is_dir() {
                return HomeSource::PortableSibling(portable);
            }
        }
    }
    if let Some(pointer) = ven_config::pointer_home() {
        return HomeSource::Pointer(pointer);
    }
    let default = dirs::home_dir()
        .map(|h| h.join(".ven"))
        .unwrap_or_else(|| PathBuf::from(".ven"));
    HomeSource::Default(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Shared with `ven_config::tests` — see the rationale in `core/mod.rs`.
    // A per-module lock raced visibly on macOS, where `dirs::config_dir()`
    // is steered only by `$HOME`, because both modules' Drop impls were
    // restoring env state while the other's test was reading it.
    use crate::core::lock_test_env as lock_env;

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
    fn defaults_to_home_dot_ven_when_no_env_set() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        let resolved = ven_home();
        let expected = dirs::home_dir()
            .map(|h| h.join(".ven"))
            .unwrap_or_else(|| PathBuf::from(".ven"));

        // The sibling-`.ven` clause may match in unusual test runners (where
        // the test binary sits next to a `.ven/` from prior runs). In that
        // case the resolved path will be exe-dir/.ven instead, which is also
        // a valid outcome for the "no env vars" leg.
        let exe_sibling = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.join(".ven")));
        assert!(
            resolved == expected || Some(&resolved) == exe_sibling.as_ref(),
            "expected {expected:?} or {exe_sibling:?}, got {resolved:?}",
        );
    }

    #[test]
    fn ven_home_env_var_wins_outright() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        std::env::set_var("VEN_HOME", "/tmp/explicit-ven");
        std::env::set_var("VEN_STORAGE_PATH", "/tmp/should-be-ignored");
        assert_eq!(ven_home(), PathBuf::from("/tmp/explicit-ven"));
    }

    #[test]
    fn ven_storage_path_used_when_ven_home_unset() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        std::env::set_var("VEN_STORAGE_PATH", "/tmp/legacy-ven");
        assert_eq!(ven_home(), PathBuf::from("/tmp/legacy-ven"));
    }

    #[test]
    fn empty_env_var_is_treated_as_unset() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        std::env::set_var("VEN_HOME", "");
        std::env::set_var("VEN_STORAGE_PATH", "/tmp/legacy-via-fallback");
        assert_eq!(ven_home(), PathBuf::from("/tmp/legacy-via-fallback"));
    }

    #[test]
    fn portable_sibling_dir_takes_precedence_over_home_default() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(_) => return, // Skip on platforms where current_exe is unreliable.
        };
        let exe_dir = match exe.parent() {
            Some(d) => d.to_path_buf(),
            None => return,
        };
        let portable = exe_dir.join(".ven");
        let created_for_test = !portable.exists();
        if created_for_test {
            if std::fs::create_dir_all(&portable).is_err() {
                return; // Read-only sandbox, skip.
            }
        }

        let resolved = ven_home();
        assert_eq!(resolved, portable);

        if created_for_test {
            let _ = std::fs::remove_dir(&portable);
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pointer-file precedence tests (introduced in v0.1.6).
    //
    // These tests redirect `dirs::config_dir()` at a tempdir by mutating
    // HOME / XDG_CONFIG_HOME / APPDATA. Because that's process-global state
    // shared with `ven_config::tests`, both modules acquire the same
    // crate-wide lock from `core::lock_test_env()`.
    //
    // **Skipped on Windows.** `dirs::config_dir()` on Windows calls the
    // Win32 `SHGetKnownFolderPath(FOLDERID_RoamingAppData)` shell API,
    // which does NOT read the `APPDATA` env var — so this redirection
    // doesn't actually isolate the test. Running these on a Windows CI
    // runner writes into the real `%APPDATA%\ven\config.toml` and races
    // against other tests, which is both a correctness hazard and a
    // pollution hazard. The pointer-file code path itself is platform-
    // agnostic (no `#[cfg(windows)]` in `ven_config.rs`), so the Linux
    // and macOS runs of these tests prove the same behavior holds on
    // Windows. Manual end-to-end testing via `ven path set` on Windows
    // exercises the dirs::config_dir() integration directly.
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(not(target_os = "windows"))]
    struct ConfigDirRedirect {
        _temp: tempfile::TempDir,
        prev: Vec<(&'static str, Option<String>)>,
    }

    #[cfg(not(target_os = "windows"))]
    impl ConfigDirRedirect {
        fn new() -> Self {
            let temp = tempfile::tempdir().expect("tempdir");
            let path = temp.path().to_path_buf();
            let keys = ["HOME", "XDG_CONFIG_HOME", "APPDATA"];
            let prev: Vec<_> = keys.iter().map(|k| (*k, std::env::var(k).ok())).collect();
            std::env::set_var("XDG_CONFIG_HOME", &path);
            std::env::set_var("HOME", &path);
            std::env::set_var("APPDATA", &path);
            Self { _temp: temp, prev }
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
    fn pointer_file_overrides_default_home() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);
        let _redir = ConfigDirRedirect::new();

        let target = PathBuf::from("/tmp/relocated-ven");
        crate::core::ven_config::set_storage_home(target.clone()).unwrap();

        // The default ($HOME/.ven) is whatever ConfigDirRedirect put us in,
        // and that absolutely is NOT `target`, so this assertion is meaningful.
        let resolved = ven_home();
        // The resolver may still pick the portable sibling if a `.ven/` happens
        // to sit next to the test binary on the runner. Accept either pointer
        // or that sibling — both prove the default fallback was not taken.
        let exe_sibling = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.join(".ven")));
        assert!(
            resolved == target || Some(&resolved) == exe_sibling.as_ref(),
            "expected pointer {target:?} or portable {exe_sibling:?}, got {resolved:?}",
        );

        // Clean up so we don't pollute other tests' tempdirs.
        let _ = crate::core::ven_config::clear_storage_home();
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn env_var_overrides_pointer_file() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);
        let _redir = ConfigDirRedirect::new();

        crate::core::ven_config::set_storage_home(PathBuf::from("/tmp/pointer-says")).unwrap();
        std::env::set_var("VEN_HOME", "/tmp/env-wins");

        assert_eq!(ven_home(), PathBuf::from("/tmp/env-wins"));

        let _ = crate::core::ven_config::clear_storage_home();
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn home_source_reports_correct_kind() {
        let _g = lock_env();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);
        let _redir = ConfigDirRedirect::new();

        // No overrides at all — default.
        assert_eq!(ven_home_source().kind(), "default");

        // Pointer beats default.
        crate::core::ven_config::set_storage_home(PathBuf::from("/tmp/p")).unwrap();
        // Skip assertion if a portable sibling exists next to the test binary,
        // because then "portable" rightfully outranks "pointer" and the
        // resolver's behavior is correct — we just can't observe "pointer"
        // from this environment.
        let portable_present = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.join(".ven").is_dir()))
            .unwrap_or(false);
        if !portable_present {
            assert_eq!(ven_home_source().kind(), "pointer");
        }

        // Env beats pointer.
        std::env::set_var("VEN_HOME", "/tmp/e");
        assert_eq!(ven_home_source().kind(), "env:VEN_HOME");

        let _ = crate::core::ven_config::clear_storage_home();
    }
}
