//! Resolve `ven.toml` runtimes into PATH prepends and toolchain env vars.
//! Shared by `ven shell activate` and `ven-launcher`.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::{
    find_ven_toml, parse_ven_toml, project_venv, resolve_deno_version, resolve_go_version,
    resolve_java_version, resolve_node_version, resolve_python_version, resolve_rust_version,
};
use crate::plugins::{
    DenoPlugin, GoPlugin, JavaPlugin, LanguagePlugin, NodePlugin, PythonPlugin, RustPlugin,
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

/// Resolved toolchain layout from a project directory (same inputs as shell activation).
#[derive(Debug, Clone)]
pub struct ActivationParts {
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
    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
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

    if node_spec.is_empty()
        && python_spec.is_empty()
        && go_spec.is_empty()
        && rust_spec.is_empty()
        && java_spec.is_empty()
        && deno_spec.is_empty()
    {
        anyhow::bail!(
            "ven.toml [runtime]: set `node` and/or `python` and/or `go` and/or `rust` and/or `java` and/or `deno`"
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
                                let installed =
                                    PythonPlugin.list_installed().unwrap_or_default();
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

    let toml_normalized = if cfg!(target_os = "windows") {
        toml_absolute.replace('/', "\\")
    } else {
        toml_absolute.replace('\\', "/")
    };

    Ok(ActivationResolve::Ready(ActivationParts {
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
            out.push_str(&format!("$env:VEN_NODE_VERSION = \"{}\"\n", v));
        }
        if let Some(ref v) = parts.python_resolved {
            out.push_str(&format!("$env:VEN_PYTHON_VERSION = \"{}\"\n", v));
        }
        if let Some(ref v) = parts.go_resolved {
            out.push_str(&format!("$env:VEN_GO_VERSION = \"{}\"\n", v));
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
            out.push_str(&format!("$env:VEN_RUST_VERSION = \"{}\"\n", v));
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
            out.push_str(&format!("$env:VEN_JAVA_VERSION = \"{}\"\n", v));
        }
        if let Some(ref home) = parts.java_home_for_env {
            out.push_str(&format!(
                "$env:JAVA_HOME = \"{}\"\n",
                path_for_env_value(home)
            ));
        }
        if let Some(ref v) = parts.deno_resolved {
            out.push_str(&format!("$env:VEN_DENO_VERSION = \"{}\"\n", v));
        }
        if let Some(ref vr) = parts.virtual_env_root {
            out.push_str(&format!(
                "$env:VIRTUAL_ENV = \"{}\"\n",
                path_for_env_value(vr)
            ));
        }
        out.push_str(&format!(
            "$env:VEN_TOML = \"{}\"\n",
            parts.toml_normalized
        ));
        for (key, val) in &parts.ven_user_env {
            out.push_str(&format!("$env:{} = \"{}\"\n", key, val));
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
            out.push_str(&format!("export VEN_NODE_VERSION=\"{}\"\n", v));
        }
        if let Some(ref v) = parts.python_resolved {
            out.push_str(&format!("export VEN_PYTHON_VERSION=\"{}\"\n", v));
        }
        if let Some(ref v) = parts.go_resolved {
            out.push_str(&format!("export VEN_GO_VERSION=\"{}\"\n", v));
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
            out.push_str(&format!("export VEN_RUST_VERSION=\"{}\"\n", v));
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
            out.push_str(&format!("export VEN_JAVA_VERSION=\"{}\"\n", v));
        }
        if let Some(ref home) = parts.java_home_for_env {
            out.push_str(&format!(
                "export JAVA_HOME=\"{}\"\n",
                path_for_env_value(home)
            ));
        }
        if let Some(ref v) = parts.deno_resolved {
            out.push_str(&format!("export VEN_DENO_VERSION=\"{}\"\n", v));
        }
        if let Some(ref vr) = parts.virtual_env_root {
            out.push_str(&format!(
                "export VIRTUAL_ENV=\"{}\"\n",
                path_for_env_value(vr)
            ));
        }
        out.push_str(&format!(
            "export VEN_TOML=\"{}\"\n",
            parts.toml_normalized
        ));
        for (key, val) in &parts.ven_user_env {
            out.push_str(&format!("export {}=\"{}\"\n", key, val));
        }
        out
    };

    exports
}
