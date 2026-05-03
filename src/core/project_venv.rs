//! Project-local `.venv` (standard Python virtual environment).

use anyhow::{Context, Result};
use std::fs::OpenOptions;
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

/// Creates `.venv` under `project_root` using `python_executable -m venv .venv`.
pub fn create_local_venv(project_root: &Path, python_executable: &Path) -> Result<PathBuf> {
    let venv = project_root.join(".venv");
    if venv.join("pyvenv.cfg").is_file() {
        return Ok(venv);
    }
    if venv.exists() {
        std::fs::remove_dir_all(&venv)
            .with_context(|| format!("Could not remove partial {}", venv.display()))?;
    }
    let st = Command::new(python_executable)
        .current_dir(project_root)
        .args(["-m", "venv", ".venv"])
        .status()
        .with_context(|| format!("Failed to run {:?}", python_executable))?;
    if !st.success() {
        anyhow::bail!(
            "{:?} -m venv .venv exited with {}",
            python_executable,
            st
        );
    }
    Ok(venv)
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
