//! Project-local Python virtual environment (`venv` or legacy `.venv`).

use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Default directory name for virtualenv (visible in Explorer; matches common docs).
pub const PROJECT_VENV_DIR: &str = "venv";

/// Older ven releases used `./.venv`; still detected when present.
const LEGACY_VENV_DIR: &str = ".venv";

fn venv_roots_to_check(project_root: &Path) -> [PathBuf; 2] {
    [
        project_root.join(PROJECT_VENV_DIR),
        project_root.join(LEGACY_VENV_DIR),
    ]
}

/// `pyvenv.cfg` parent: `./venv` or legacy `./.venv`.
pub fn local_venv_root(project_root: &Path) -> Option<PathBuf> {
    for base in venv_roots_to_check(project_root) {
        if base.join("pyvenv.cfg").is_file() {
            return Some(base);
        }
    }
    None
}

/// Directory to prepend to `PATH` (`Scripts` on Windows, `bin` on Unix).
pub fn local_venv_bin_dir(project_root: &Path) -> Option<PathBuf> {
    let root = local_venv_root(project_root)?;
    let bin_dir = if cfg!(target_os = "windows") {
        root.join("Scripts")
    } else {
        root.join("bin")
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

/// Creates `./venv` under `project_root` (not `./.venv`).
///
/// Uses `python -m venv` first. Official **Windows embeddable** zips omit the stdlib `venv`
/// package, so that step often fails with `No module named venv`. In that case we install
/// PyPI [`virtualenv`](https://pypi.org/project/virtualenv/) via pip and run it.
pub fn create_local_venv(project_root: &Path, python_executable: &Path) -> Result<PathBuf> {
    let venv = project_root.join(PROJECT_VENV_DIR);
    if venv.join("pyvenv.cfg").is_file() {
        return Ok(venv);
    }
    if venv.exists() {
        std::fs::remove_dir_all(&venv)
            .with_context(|| format!("Could not remove partial {}", venv.display()))?;
    }
    let name = PROJECT_VENV_DIR;

    let mut venv_cmd = Command::new(python_executable);
    venv_cmd.current_dir(project_root).args(["-m", "venv"]);
    #[cfg(target_os = "windows")]
    venv_cmd.arg("--copies");
    venv_cmd.arg(name);

    let try_stdlib_venv = venv_cmd
        .output()
        .with_context(|| format!("Failed to run {:?}", python_executable))?;

    if try_stdlib_venv.status.success() {
        ensure_pyvenv_no_system_site(&venv)?;
        return Ok(venv);
    }

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
        .output()
        .with_context(|| {
            format!(
                "Failed to run pip beside {:?} (need pip + network for virtualenv fallback)",
                python_executable
            )
        })?;
    if !pip_st.status.success() {
        anyhow::bail!(
            "This Python has no stdlib `venv` (common with Windows embeddable builds), \
             and `pip install virtualenv` failed (exit {}). \
             Fix pip/network or use a full Python installer; then run:  {:?} -m venv {}",
            pip_st.status,
            python_executable,
            name
        );
    }

    let mut vx = Command::new(python_executable);
    vx.current_dir(project_root).args(["-m", "virtualenv"]);
    #[cfg(target_os = "windows")]
    vx.arg("--copies");
    vx.arg(name);

    let vx_st = vx
        .output()
        .with_context(|| format!("Failed to run virtualenv via {:?}", python_executable))?;
    if !vx_st.status.success() {
        anyhow::bail!(
            "`{:?} -m virtualenv {name}` exited with {}",
            python_executable,
            vx_st.status
        );
    }

    ensure_pyvenv_no_system_site(&venv)?;
    Ok(venv)
}

/// Force `include-system-site-packages = false` so installs stay under the env only.
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
        if line
            .trim_start()
            .starts_with("include-system-site-packages")
        {
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

fn gitignore_covers_any_venv_line(s: &str) -> bool {
    s.lines().any(|l| {
        let t = l.trim().trim_end_matches('/');
        matches!(
            t,
            "venv"
                | ".venv"
                | "venv/"
                | ".venv/"
                | "**/venv"
                | "**/.venv"
                | "**/venv/"
                | "**/.venv/"
        )
    })
}

/// Appends `venv/` and legacy `.venv/` to `.gitignore` if absent.
pub fn ensure_gitignore_venv(project_root: &Path) -> Result<()> {
    let p = project_root.join(".gitignore");
    if p.exists() {
        let s = fs::read_to_string(&p)?;
        if gitignore_covers_any_venv_line(&s) {
            return Ok(());
        }
        let mut f = OpenOptions::new().append(true).open(&p)?;
        writeln!(
            f,
            "\n# ven (local Python env)\n{}/\n{}/",
            PROJECT_VENV_DIR, LEGACY_VENV_DIR
        )?;
    } else {
        fs::write(&p, format!("{}/\n{}/\n", PROJECT_VENV_DIR, LEGACY_VENV_DIR))?;
    }
    Ok(())
}
