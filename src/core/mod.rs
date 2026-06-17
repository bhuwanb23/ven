pub mod bun_install;
pub mod config;
pub mod deno_imports;
pub mod deno_install;
pub mod doc_fetcher;
pub mod download;
pub mod endoflife;
pub mod extract;
pub mod gemfile;
pub mod ghost_scanner;
pub mod go_install;
pub mod integrity;
pub mod java_install;
pub mod java_manifest;
pub mod npm_registry;
pub mod osv;
pub mod packages;
pub mod project_venv;
pub mod python_install;
pub mod requirements;
pub mod ruby_gems;
pub mod ruby_install;
pub mod runtime_bin;
pub mod rust_install;
pub mod security;
pub mod storage_move;
pub mod uninstaller;
pub mod user_env;
pub mod utils;
pub mod ven_config;
pub mod ven_home;

#[allow(unused_imports)] // crate root re-exports for `ven` as a library
pub use bun_install::{install_bun as install_bun_native, BunDownloader};
pub use config::{
    find_ven_toml, load_config, parse_ven_toml, resolve_bun_version, resolve_deno_version,
    resolve_go_version, resolve_java_version, resolve_node_version, resolve_python_version,
    resolve_ruby_version, resolve_rust_version,
};
#[allow(unused_imports)] // crate root re-exports for `ven` as a library
pub use deno_install::{install_deno as install_deno_native, DenoDownloader};
pub use download::NodeDownloader;
pub use extract::install_node as install_node_native;
#[allow(unused_imports)] // crate root re-exports for `ven` as a library
pub use go_install::{install_go as install_go_native, GoDownloader};
#[allow(unused_imports)] // crate root re-exports for `ven` as a library
pub use java_install::{install_java as install_java_native, JavaDownloader};
#[allow(unused_imports)] // crate root re-exports for `ven` as a library
pub use python_install::{install_python as install_python_native, PythonDownloader};
#[allow(unused_imports)]
pub use ruby_install::{
    fetch_ruby_release_versions, install_ruby as install_ruby_native, resolve_ruby_version_spec,
    RubyDownloader,
};
#[allow(unused_imports)] // crate root re-exports for `ven` as a library
pub use rust_install::{install_rust as install_rust_native, RustDownloader};
pub use security::SecurityScanner;
pub use ven_home::ven_home;

/// Drive a `Future` to completion, reusing an existing Tokio runtime when
/// one is already active on the current thread and spinning up a fresh one
/// otherwise.  This is the single canonical blocking bridge used across the
/// crate (CLI helpers, dependency intelligence, security scanning, etc.).
pub fn block_on_async<F: std::future::Future>(f: F) -> anyhow::Result<F::Output> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        Ok(tokio::task::block_in_place(|| handle.block_on(f)))
    } else {
        Ok(tokio::runtime::Runtime::new()?.block_on(f))
    }
}

// Note: All implementation functions have been moved to config.rs and packages.rs
// This file now only serves as a module export hub

// ─────────────────────────────────────────────────────────────────────────────
// Crate-wide test-only env mutex.
//
// Several submodules (`ven_home::tests`, `ven_config::tests`) need to mutate
// process-global env state: $HOME / $XDG_CONFIG_HOME / $APPDATA to redirect
// `dirs::config_dir()`, and $VEN_HOME / $VEN_STORAGE_PATH to drive the
// resolver. Because these are process-global, those tests must be serialized.
//
// If each submodule owned its own static `Mutex<()>`, tests in the two
// modules would still race each other in parallel — they'd hold different
// locks. On macOS this raced visibly: `dirs::config_dir()` there reads only
// `$HOME` (XDG is ignored on Apple platforms), so a `ven_home::tests` Drop
// restoring `$HOME` mid-flight in another test would silently re-point
// `config_dir()` at the runner's real home and the assertion would explode.
//
// One lock for the whole crate, in one place, fixes that.
//
// All env-mutating tests in this crate must `let _g = lock_test_env();` at
// the top of the test before touching any of the relevant env vars.
#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Acquire `TEST_ENV_LOCK` in a poison-resilient way. If a previous test
/// panicked while holding the lock, take ownership of the inner guard
/// rather than re-panicking on every subsequent test — that cascades a
/// single root-cause failure into N spurious failures and hides the
/// actual one.
#[cfg(test)]
pub(crate) fn lock_test_env() -> std::sync::MutexGuard<'static, ()> {
    TEST_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner())
}
