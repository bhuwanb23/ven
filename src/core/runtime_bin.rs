//! Resolve per-language tool binaries (npm, bun, cargo, go, gem, …) from the
//! current project's `ven.toml` runtime, falling back to a bare command name
//! so PATH-based lookup still works for users who installed a runtime outside
//! of ven.
//!
//! This is the single place where `ven add` / `ven remove` learn _where_ each
//! language's package manager lives, instead of every call site re-deriving
//! `~/.ven/<lang>/<ver>/<bin>` (or worse, hard-coding it).

use std::path::PathBuf;

use crate::core::config::{
    load_config, resolve_bun_version, resolve_deno_version, resolve_go_version,
    resolve_java_version, resolve_node_version, resolve_python_version, resolve_ruby_version,
    resolve_rust_version,
};
use crate::core::project_venv::local_venv_bin_dir;
use crate::plugins::{
    BunPlugin, DenoPlugin, GoPlugin, JavaPlugin, LanguagePlugin, NodePlugin, PythonPlugin,
    RubyPlugin, RustPlugin,
};

/// Resolve `<bin_stem>` for `lang` by walking:
///   1. project venv (`./venv/Scripts`) for python only,
///   2. `ven.toml`'s declared runtime + plugin `bin_path()`,
///   3. fall back to the bare stem (PATH lookup at exec time).
///
/// The returned `PathBuf` is the *exact* path to execute. On Windows we try
/// `.exe`, `.cmd`, `.bat` extensions before the stem itself.
pub fn runtime_tool(lang: &str, bin_stem: &str) -> PathBuf {
    if let Some(p) = resolve_from_project(lang, bin_stem) {
        return p;
    }
    PathBuf::from(bin_stem)
}

fn resolve_from_project(lang: &str, bin_stem: &str) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;

    if lang == "python" {
        if let Some(p) = python_from_local_venv(&cwd, bin_stem) {
            return Some(p);
        }
    }

    let cfg = load_config(&cwd).ok().flatten()?;
    let raw_version = match lang {
        "node" => cfg.runtime.node.clone(),
        "bun" => cfg.runtime.bun.clone(),
        "deno" => cfg.runtime.deno.clone(),
        "python" => cfg.runtime.python.clone(),
        "go" => cfg.runtime.go.clone(),
        "rust" => cfg.runtime.rust.clone(),
        "java" => cfg.runtime.java.clone(),
        "ruby" => cfg.runtime.ruby.clone(),
        _ => return None,
    };
    if raw_version.trim().is_empty() {
        return None;
    }

    let bin_dir = resolve_bin_dir(lang, &raw_version)?;
    find_in_dir(&bin_dir, bin_stem)
}

fn python_from_local_venv(project_root: &std::path::Path, bin_stem: &str) -> Option<PathBuf> {
    let bin = local_venv_bin_dir(project_root)?;
    find_in_dir(&bin, bin_stem)
}

fn resolve_bin_dir(lang: &str, raw_version: &str) -> Option<PathBuf> {
    match lang {
        "node" => {
            let plugin = NodePlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_node_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        "bun" => {
            let plugin = BunPlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_bun_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        "deno" => {
            let plugin = DenoPlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_deno_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        "python" => {
            let plugin = PythonPlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_python_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        "go" => {
            let plugin = GoPlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_go_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        "rust" => {
            let plugin = RustPlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_rust_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        "java" => {
            let plugin = JavaPlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_java_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        "ruby" => {
            let plugin = RubyPlugin;
            let installed = plugin.list_installed().ok()?;
            let resolved = resolve_ruby_version(raw_version, &installed).ok()?;
            plugin.bin_path(&resolved).ok()
        }
        _ => None,
    }
}

/// Look for `<bin_stem>{,.exe,.cmd,.bat,.ps1}` inside `dir` (Windows) or just
/// `<bin_stem>` on Unix. Also probes Python's `Scripts/` sibling (Windows) and
/// Node's lack of a `bin/` subdir (the install root itself holds `npm.cmd`).
fn find_in_dir(dir: &std::path::Path, bin_stem: &str) -> Option<PathBuf> {
    let probes: Vec<PathBuf> = std::iter::once(dir.to_path_buf())
        .chain([
            dir.join("bin"),
            dir.join("Scripts"),
            dir.parent().map(|p| p.join("Scripts")).unwrap_or_default(),
        ])
        .collect();

    for probe in probes {
        if probe.as_os_str().is_empty() {
            continue;
        }
        if !probe.is_dir() {
            continue;
        }
        #[cfg(target_os = "windows")]
        {
            for ext in &["exe", "cmd", "bat", "ps1"] {
                let p = probe.join(format!("{bin_stem}.{ext}"));
                if p.is_file() {
                    return Some(p);
                }
            }
            let bare = probe.join(bin_stem);
            if bare.is_file() {
                return Some(bare);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let p = probe.join(bin_stem);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// True when stdin appears to be an interactive TTY. Used by `ven add` /
/// `ven remove` to skip the y/N confirmation in CI / piped runs.
pub fn stdin_is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}
