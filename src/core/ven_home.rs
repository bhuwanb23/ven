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
//! 4. `~/.ven` — the default for an installed ven on a user's machine.
//!
//! Every consumer of the ven storage root MUST go through [`ven_home`] so the
//! four cases stay coherent. Hardcoding `dirs::home_dir().join(".ven")`
//! anywhere in the codebase silently breaks portable mode.

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
    dirs::home_dir()
        .map(|h| h.join(".ven"))
        .unwrap_or_else(|| PathBuf::from(".ven"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // The resolver reads process-global state (env vars + current exe), so
    // tests that mutate VEN_HOME / VEN_STORAGE_PATH must run serially.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        let _g = ENV_LOCK.lock().unwrap();
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
        let _g = ENV_LOCK.lock().unwrap();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        std::env::set_var("VEN_HOME", "/tmp/explicit-ven");
        std::env::set_var("VEN_STORAGE_PATH", "/tmp/should-be-ignored");
        assert_eq!(ven_home(), PathBuf::from("/tmp/explicit-ven"));
    }

    #[test]
    fn ven_storage_path_used_when_ven_home_unset() {
        let _g = ENV_LOCK.lock().unwrap();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        std::env::set_var("VEN_STORAGE_PATH", "/tmp/legacy-ven");
        assert_eq!(ven_home(), PathBuf::from("/tmp/legacy-ven"));
    }

    #[test]
    fn empty_env_var_is_treated_as_unset() {
        let _g = ENV_LOCK.lock().unwrap();
        let _scrub = EnvGuard::new(&["VEN_HOME", "VEN_STORAGE_PATH"]);

        std::env::set_var("VEN_HOME", "");
        std::env::set_var("VEN_STORAGE_PATH", "/tmp/legacy-via-fallback");
        assert_eq!(ven_home(), PathBuf::from("/tmp/legacy-via-fallback"));
    }

    #[test]
    fn portable_sibling_dir_takes_precedence_over_home_default() {
        let _g = ENV_LOCK.lock().unwrap();
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
}
