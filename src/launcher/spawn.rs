//! Spawn an interactive shell with the same env as `ven shell activate` (Phase 4).

use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

#[cfg(windows)]
use crate::launcher::greeting::{write_cmd_autorun, write_powershell_profile_init};
#[cfg(not(windows))]
use crate::launcher::greeting::{write_posix_printf_greeting, write_powershell_profile_init};
#[cfg(not(windows))]
use crate::launcher::quote::bash_single_quoted;
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
                 Try from the project folder, or pass a path explicitly, for example:\n\
                   ven-launcher\n\
                   ven-launcher path/to/myapp\n\
                   ven-launcher ./example"
            )
        }
        ActivationResolve::MissingToolchain {
            language,
            install_with,
        } => {
            let start = path_for_env_value(project_hint);
            anyhow::bail!(
                "ven-launcher: required runtime is not installed for this machine.\n\
                 • Language: {language}\n\
                 • Requested in ven.toml: {install_with}\n\
                 • Search started from: {start}\n\
                 Install it, then retry:\n\
                   ven install {language} {install_with}"
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
            let mut ps1 = tempfile::Builder::new()
                .prefix("ven-launcher-")
                .suffix(".ps1")
                .tempfile()
                .context("temp PowerShell profile")?;
            let loc = path_for_env_value(cwd);
            write_powershell_profile_init(&mut ps1, parts, &loc).context("write PowerShell profile")?;
            ps1.flush().ok();
            let kept = ps1
                .into_temp_path()
                .keep()
                .map_err(|e| anyhow::anyhow!("persist PowerShell profile: {}", e))?;

            let mut cmd = Command::new("powershell.exe");
            cmd.args([
                "-NoExit",
                "-NoLogo",
                "-File",
                kept.to_str().ok_or_else(|| {
                    anyhow::anyhow!("PowerShell profile path is not valid UTF-8")
                })?,
            ])
            .current_dir(cwd)
            .creation_flags(CREATE_NEW_CONSOLE);
            apply_activation_env(&mut cmd, parts);
            cmd.spawn()
                .context("failed to start PowerShell (try: powershell.exe on PATH)")?;
        }
        ShellKind::Cmd | ShellKind::Bash | ShellKind::Zsh | ShellKind::Other(_) => {
            let cwd_q = cmd_quoted_path(cwd);
            let mut bat = tempfile::Builder::new()
                .prefix("ven-launcher-")
                .suffix(".cmd")
                .tempfile()
                .context("temp cmd autorun")?;
            write_cmd_autorun(&mut bat, parts, &cwd_q).context("write cmd autorun")?;
            bat.flush().ok();
            let kept = bat
                .into_temp_path()
                .keep()
                .map_err(|e| anyhow::anyhow!("persist cmd script: {}", e))?;

            let comspec = std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".into());
            let mut cmd = Command::new(comspec);
            cmd.args([
                "/K",
                kept.to_str().ok_or_else(|| {
                    anyhow::anyhow!("cmd script path is not valid UTF-8")
                })?,
            ])
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
            write_posix_printf_greeting(&mut tmp, parts).context("write bash greeting")?;
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
            write_posix_printf_greeting(&mut tmp, parts).context("write zsh greeting")?;
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
            let mut ps1 = tempfile::Builder::new()
                .prefix("ven-launcher-")
                .suffix(".ps1")
                .tempfile()
                .context("temp PowerShell profile")?;
            let loc = path_for_env_value(cwd);
            write_powershell_profile_init(&mut ps1, parts, &loc).context("write PowerShell profile")?;
            ps1.flush().ok();
            let kept = ps1
                .into_temp_path()
                .keep()
                .map_err(|e| anyhow::anyhow!("persist PowerShell profile: {}", e))?;

            let mut run = |program: &str| -> Result<()> {
                let mut cmd = Command::new(program);
                cmd.args([
                    "-NoExit",
                    "-NoLogo",
                    "-File",
                    kept.to_str().ok_or_else(|| {
                        anyhow::anyhow!("PowerShell profile path is not valid UTF-8")
                    })?,
                ])
                .current_dir(cwd);
                apply_activation_env(&mut cmd, parts);
                cmd.spawn().context(format!("failed to start {program}"))?;
                Ok(())
            };

            match run("pwsh") {
                Ok(()) => {}
                Err(_) => run("powershell")?,
            }
        }
        ShellKind::Other(exe) => {
            if Path::new(&exe).is_file() {
                let mut cmd = Command::new(&exe);
                cmd.arg("-i").current_dir(cwd);
                apply_activation_env(&mut cmd, parts);
                cmd.spawn()
                    .with_context(|| format!("failed to start {}", exe))?;
            } else {
                anyhow::bail!(
                    "ven-launcher: SHELL is set to '{}' but that program was not found.\n\
                     Set SHELL to bash or zsh, or run ven-launcher from PowerShell/cmd on Windows.",
                    exe
                );
            }
        }
    }
    Ok(())
}
