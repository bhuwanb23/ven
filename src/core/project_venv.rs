//! Project-local `.venv` (standard Python virtual environment).

use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Directory to prepend to `PATH` (`Scripts` on Windows, `bin` on Unix).
pub fn local_venv_bin_dir(project_root: &Path) -> Option<PathBuf> {
    let venv = project_root.join(".venv");
    let cfg = venv.join("pyvenv.cfg");
    if !cfg.is_file() {
        return None;
    }
    let bin_dir = if cfg!(target_os = "windows") {
        venv.join("Scripts")
    } else {
        venv.join("bin")
    };
    let marker = if cfg!(target_os = "windows") {
        bin_dir.join("python.exe")
    } else {
        bin_dir.join("python")
    };
    if marker.is_file() {
        Some(bin_dir)
    } else {
        None
    }
}

/// Reads `version = …` from `pyvenv.cfg` (used for `VEN_PYTHON_VERSION`).
pub fn local_venv_python_version(venv_root: &Path) -> Option<String> {
    let cfg = venv_root.join("pyvenv.cfg");
    let s = std::fs::read_to_string(&cfg).ok()?;
    for line in s.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("version") else {
            continue;
        };
        let Some(ver) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let v = ver.trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

/// Creates `.venv` under `project_root`.
///
/// Uses `python -m venv` first. Official **Windows embeddable** zips omit the stdlib `venv`
/// package, so that step often fails with `No module named venv`. In that case we install
/// PyPI [`virtualenv`](https://pypi.org/project/virtualenv/) via pip and run
/// `python -m virtualenv .venv`, which produces the same layout (`pyvenv.cfg`, `Scripts`/…).
pub fn create_local_venv(project_root: &Path, python_executable: &Path) -> Result<PathBuf> {
    let venv = project_root.join(".venv");
    if venv.join("pyvenv.cfg").is_file() {
        return Ok(venv);
    }
    if venv.exists() {
        std::fs::remove_dir_all(&venv)
            .with_context(|| format!("Could not remove partial {}", venv.display()))?;
    }

    let mut venv_cmd = Command::new(python_executable);
    venv_cmd.current_dir(project_root).args(["-m", "venv"]);
    // Embeddable / some Windows layouts break symlinked venvs; real files isolate site-packages.
    #[cfg(target_os = "windows")]
    venv_cmd.arg("--copies");
    venv_cmd.arg(".venv");

    let try_stdlib_venv = venv_cmd
        .status()
        .with_context(|| format!("Failed to run {:?}", python_executable))?;

    if try_stdlib_venv.success() {
        ensure_pyvenv_no_system_site(&venv)?;
        return Ok(venv);
    }

    // Embed / minimal builds: no stdlib `venv` — bootstrap virtualenv through pip.
    let pip_st = Command::new(python_executable)
        .current_dir(project_root)
        .args([
            "-m",
            "pip",
            "install",
            "-q",
            "--disable-pip-version-check",
            "virtualenv",
        ])
        .status()
        .with_context(|| {
            format!(
                "Failed to run pip beside {:?} (need pip + network for virtualenv fallback)",
                python_executable
            )
        })?;
    if !pip_st.success() {
        anyhow::bail!(
            "This Python has no stdlib `venv` (common with Windows embeddable builds), \
             and `pip install virtualenv` failed (exit {}). \
             Fix pip/network or use a full Python installer; then run:  {:?} -m venv .venv",
            pip_st,
            python_executable
        );
    }

    let mut vx = Command::new(python_executable);
    vx.current_dir(project_root)
        .args(["-m", "virtualenv"]);
    #[cfg(target_os = "windows")]
    vx.arg("--copies");
    vx.arg(".venv");

    let vx_st = vx
        .status()
        .with_context(|| format!("Failed to run virtualenv via {:?}", python_executable))?;
    if !vx_st.success() {
        anyhow::bail!("`{:?} -m virtualenv .venv` exited with {}", python_executable, vx_st);
    }

    ensure_pyvenv_no_system_site(&venv)?;
    Ok(venv)
}

/// Force `include-system-site-packages = false` so installs go to `.venv` only.
fn ensure_pyvenv_no_system_site(venv_root: &Path) -> Result<()> {
    let path = venv_root.join("pyvenv.cfg");
    if !path.is_file() {
        return Ok(());
    }
    let s = fs::read_to_string(&path)?;
    let mut saw_include = false;
    let mut out = String::new();
    let mut changed = false;
    for line in s.lines() {
        if line.trim_start().starts_with("include-system-site-packages") {
            saw_include = true;
            if line.trim() != "include-system-site-packages = false" {
                changed = true;
            }
            out.push_str("include-system-site-packages = false\n");
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !saw_include {
        out.push_str("include-system-site-packages = false\n");
        changed = true;
    }
    if changed {
        fs::write(&path, out)?;
    }
    Ok(())
}

/// Appends `.venv/` to `.gitignore` if not already ignored.
pub fn ensure_gitignore_venv(project_root: &Path) -> Result<()> {
    let p = project_root.join(".gitignore");
    let line = ".venv/";
    if p.exists() {
        let s = std::fs::read_to_string(&p)?;
        if s.lines().any(|l| {
            matches!(
                l.trim(),
                ".venv" | ".venv/" | "**/.venv" | "**/.venv/"
            )
        }) {
            return Ok(());
        }
        let mut f = OpenOptions::new().append(true).open(&p)?;
        writeln!(f, "\n# ven (local Python env)\n{}", line)?;
    } else {
        std::fs::write(&p, format!("{}\n", line))?;
    }
    Ok(())
}
