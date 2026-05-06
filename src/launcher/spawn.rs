//! Spawn an interactive shell with the same env as `ven shell activate` (Phase 4).

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::launcher::{detect_shell, ShellKind};
use crate::launcher::env::apply_activation_env;
use crate::shell::{path_for_env_value, resolve_activation_environment, ActivationResolve};

/// Open a **new terminal** whose cwd is the `ven.toml` project root and env matches `ven shell activate`.
pub fn spawn_project_shell(project_hint: &Path) -> Result<()> {
    match resolve_activation_environment(project_hint)? {
        ActivationResolve::NoToml => {
            let start = path_for_env_value(project_hint);
            anyhow::bail!(
                "ven-launcher: no ven.toml found when searching upward from \"{start}\".\n\
                 Try from the project folder, or pass a path explicitly, e.g.\n\
                   ven-launcher\n\
                   ven-launcher D:\\projects\\myapp\n\
                   ven-launcher .\\example"
            )
        }
        ActivationResolve::MissingToolchain {
            language,
            install_with,
        } => {
            let start = path_for_env_value(project_hint);
            anyhow::bail!(
                "ven-launcher: '{}' ({}) is not installed for this machine (near \"{}\").\n\
                 Hint: ven install {} {}",
                language,
                install_with,
                start,
                language,
                install_with
            )
        }
        ActivationResolve::Ready(parts) => {
            let cwd = std::fs::canonicalize(&parts.project_root)
                .unwrap_or_else(|_| parts.project_root.clone());

            let kind = detect_shell();
            spawn_for_shell(kind, &parts, &cwd)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn spawn_for_shell(
    kind: ShellKind,
    parts: &crate::shell::ActivationParts,
    cwd: &Path,
) -> Result<()> {
    use std::os::windows::process::CommandExt;

    const CREATE_NEW_CONSOLE: u32 = 0x00000010;

    match kind {
        ShellKind::PowerShell => {
            let loc = path_for_env_value(cwd);
            let cmdline = format!(
                "Set-Location -LiteralPath '{}'",
                loc.replace('\'', "''")
            );
            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoExit", "-NoLogo", "-Command", &cmdline])
                .current_dir(cwd)
                .creation_flags(CREATE_NEW_CONSOLE);
            apply_activation_env(&mut cmd, parts);
            cmd.spawn()
                .context("failed to start PowerShell (try: powershell.exe on PATH)")?;
        }
        ShellKind::Cmd | ShellKind::Bash | ShellKind::Zsh | ShellKind::Other(_) => {
            // Cmd is the reliable “new console + stay open” path on Windows.
            let comspec = std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".into());
            let cd_arg = format!("cd /d {}", cmd_quoted_path(cwd));
            let mut cmd = Command::new(comspec);
            cmd.args(["/K", &cd_arg])
                .current_dir(cwd)
                .creation_flags(CREATE_NEW_CONSOLE);
            apply_activation_env(&mut cmd, parts);
            cmd.spawn().context("failed to start cmd.exe")?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn cmd_quoted_path(p: &Path) -> String {
    let s = path_for_env_value(p);
    format!("\"{}\"", s.replace('\"', "\"\""))
}

#[cfg(not(windows))]
fn spawn_for_shell(
    kind: ShellKind,
    parts: &crate::shell::ActivationParts,
    cwd: &Path,
) -> Result<()> {
    let cd_line = format!("cd {} || exit 1\n", bash_single_quoted(&path_for_env_value(cwd)));

    match kind {
        ShellKind::Bash | ShellKind::Cmd => {
            let mut tmp = tempfile::Builder::new()
                .prefix("ven-launcher-")
                .suffix("-init.bash")
                .tempfile()
                .context("temp init file")?;
            tmp.write_all(cd_line.as_bytes())
                .context("write bash init")?;
            tmp.flush().ok();

            let path = tmp.into_temp_path();
            let retained = path
                .keep()
                .map_err(|e| anyhow::anyhow!("persist bash init file: {}", e))?;

            let mut cmd = Command::new("bash");
            cmd.args([
                "--init-file",
                retained.to_str().ok_or_else(|| {
                    anyhow::anyhow!("non-UTF8 bash init path {}", retained.display())
                })?,
                "-i",
            ])
            .current_dir(cwd);
            apply_activation_env(&mut cmd, parts);
            cmd.spawn().context("failed to start bash")?;
        }
        ShellKind::Zsh => {
            let mut tmp = tempfile::Builder::new()
                .prefix("ven-launcher-")
                .suffix("-init.zsh")
                .tempfile()
                .context("temp init file")?;
            tmp.write_all(cd_line.as_bytes())
                .context("write zsh init")?;
            tmp.flush().ok();

            let path = tmp.into_temp_path();
            let retained = path
                .keep()
                .map_err(|e| anyhow::anyhow!("persist zsh init file: {}", e))?;

            let mut cmd = Command::new("zsh");
            cmd.args([
                "--init-file",
                retained.to_str().ok_or_else(|| {
                    anyhow::anyhow!("non-UTF8 zsh init path {}", retained.display())
                })?,
                "-i",
            ])
            .current_dir(cwd);
            apply_activation_env(&mut cmd, parts);
            cmd.spawn().context("failed to start zsh")?;
        }
        ShellKind::PowerShell => {
            let mut cmd = Command::new("pwsh");
            cmd.args(["-NoExit", "-NoLogo"]).current_dir(cwd);
            apply_activation_env(&mut cmd, parts);
            let _ = cmd.spawn().or_else(|_| {
                let mut c = Command::new("powershell");
                c.args(["-NoExit", "-NoLogo"])
                    .current_dir(cwd);
                apply_activation_env(&mut c, parts);
                c.spawn()
            })
            .context("failed to start pwsh/powershell")?;
        }
        ShellKind::Other(exe) => {
            if Path::new(&exe).is_file() {
                let mut cmd = Command::new(&exe);
                cmd.arg("-i").current_dir(cwd);
                apply_activation_env(&mut cmd, parts);
                cmd.spawn()
                    .with_context(|| format!("failed to start {}", exe))?;
            } else {
                anyhow::bail!("SHELL is set to '{exe}' but that file is missing; use bash or zsh.");
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn bash_single_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}
