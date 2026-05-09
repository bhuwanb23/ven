//! Spawn an interactive shell with the same env as `ven shell activate` (Phase 4).

use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::launcher::env::apply_activation_env;
#[cfg(windows)]
use crate::launcher::greeting::{
    generic_header_lines, greeting_lines, write_cmd_autorun, GreetingStyle,
};
#[cfg(not(windows))]
use crate::launcher::greeting::{
    generic_header_lines, greeting_lines, write_posix_printf_greeting, GreetingStyle,
};
#[cfg(not(windows))]
use crate::launcher::quote::bash_single_quoted;
use crate::launcher::{detect_shell, ShellKind};
use crate::shell::{path_for_env_value, resolve_activation_environment, ActivationResolve};

/// Open a **new terminal** whose cwd is the `ven.toml` project root and env matches `ven shell activate`.
pub fn spawn_project_shell(project_hint: &Path) -> Result<()> {
    match resolve_activation_environment(project_hint)? {
        ActivationResolve::NoToml => {
            let start = path_for_env_value(project_hint);
            let kind = detect_shell();
            spawn_without_project(kind, project_hint, &start)?;
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

fn missing_toml_message(start: &str, style: GreetingStyle) -> Vec<String> {
    let mut lines = generic_header_lines(style);
    lines.push(String::new());
    lines.push("Project status".to_string());
    lines.push("--------------------------".to_string());
    let marker = match style {
        GreetingStyle::Unicode => "  ▸",
        GreetingStyle::Ascii => "  >",
    };
    lines.push(format!(
        "{marker} No ven.toml detected in this folder tree."
    ));
    lines.push(String::new());
    lines.extend(vec![
        format!("Search started from: {start}"),
        "Tip: run from your project folder or pass a project path.".to_string(),
        "Examples:".to_string(),
        "  ven-launcher ./example".to_string(),
        "  ven-launcher path/to/myapp".to_string(),
    ]);
    lines
}

#[cfg(windows)]
fn spawn_without_project(kind: ShellKind, cwd: &Path, start: &str) -> Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x00000010;

    match kind {
        ShellKind::PowerShell => {
            let lines = missing_toml_message(start, GreetingStyle::Unicode);
            let mut ps_cmds = Vec::new();
            ps_cmds.push("Write-Host ''".to_string());
            for line in lines {
                ps_cmds.push(format!("Write-Host '{}'", line.replace('\'', "''")));
            }
            ps_cmds.push("Write-Host ''".to_string());
            ps_cmds.push(format!(
                "Set-Location -LiteralPath '{}'",
                path_for_env_value(cwd).replace('\'', "''")
            ));
            let cmdline = ps_cmds.join("; ");

            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoExit", "-NoLogo", "-Command", &cmdline])
                .current_dir(cwd)
                .creation_flags(CREATE_NEW_CONSOLE);
            cmd.spawn()
                .context("failed to open PowerShell for no-ven.toml message")?;
        }
        _ => {
            let lines = missing_toml_message(start, GreetingStyle::Ascii);
            let mut bat = tempfile::Builder::new()
                .prefix("ven-launcher-missing-")
                .suffix(".cmd")
                .tempfile()
                .context("temp cmd missing-toml script")?;
            writeln!(bat, "@echo off").ok();
            writeln!(bat).ok();
            for line in lines {
                writeln!(bat, "echo {}", line.replace('|', "^|")).ok();
            }
            writeln!(bat).ok();
            writeln!(bat, "cd /d {}", cmd_quoted_path(cwd)).ok();
            bat.flush().ok();
            let kept = bat
                .into_temp_path()
                .keep()
                .map_err(|e| anyhow::anyhow!("persist missing-toml cmd script: {}", e))?;
            let comspec = std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".into());
            let mut cmd = Command::new(comspec);
            cmd.args(["/K", kept.to_string_lossy().as_ref()])
                .current_dir(cwd)
                .creation_flags(CREATE_NEW_CONSOLE);
            cmd.spawn()
                .context("failed to open cmd for no-ven.toml message")?;
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn spawn_without_project(kind: ShellKind, cwd: &Path, start: &str) -> Result<()> {
    let mut init = tempfile::Builder::new()
        .prefix("ven-launcher-missing-")
        .suffix(".sh")
        .tempfile()
        .context("temp missing-toml shell script")?;
    let lines = missing_toml_message(start, GreetingStyle::Unicode);
    for line in lines {
        writeln!(
            init,
            "printf {} {}",
            bash_single_quoted("%s\n"),
            bash_single_quoted(&line)
        )
        .ok();
    }
    writeln!(init).ok();
    writeln!(
        init,
        "cd {} || true",
        bash_single_quoted(&path_for_env_value(cwd))
    )
    .ok();
    init.flush().ok();
    let kept = init
        .into_temp_path()
        .keep()
        .map_err(|e| anyhow::anyhow!("persist missing-toml init script: {}", e))?;

    match kind {
        ShellKind::Zsh => {
            Command::new("zsh")
                .args(["--init-file", kept.to_string_lossy().as_ref(), "-i"])
                .current_dir(cwd)
                .spawn()
                .context("failed to open zsh for no-ven.toml message")?;
        }
        _ => {
            Command::new("bash")
                .args(["--init-file", kept.to_string_lossy().as_ref(), "-i"])
                .current_dir(cwd)
                .spawn()
                .context("failed to open bash for no-ven.toml message")?;
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
            let cmdline = powershell_inline_command(parts, cwd);

            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoExit", "-NoLogo", "-Command", &cmdline])
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
                kept.to_str()
                    .ok_or_else(|| anyhow::anyhow!("cmd script path is not valid UTF-8"))?,
            ])
            .current_dir(cwd)
            .creation_flags(CREATE_NEW_CONSOLE);
            apply_activation_env(&mut cmd, parts);
            cmd.spawn().context("failed to start cmd.exe")?;
        }
    }
    Ok(())
}

fn powershell_inline_command(parts: &crate::shell::ActivationParts, cwd: &Path) -> String {
    let mut commands = Vec::new();
    commands.push("Write-Host ''".to_string());
    for line in greeting_lines(parts, GreetingStyle::Unicode) {
        commands.push(format!("Write-Host '{}'", line.replace('\'', "''")));
    }
    commands.push("Write-Host ''".to_string());
    commands.push(format!(
        "Set-Location -LiteralPath '{}'",
        path_for_env_value(cwd).replace('\'', "''")
    ));
    commands.join("; ")
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
    let cd_line = format!(
        "cd {} || exit 1\n",
        bash_single_quoted(&path_for_env_value(cwd))
    );

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
            let cmdline = powershell_inline_command(parts, cwd);

            let mut run = |program: &str| -> Result<()> {
                let mut cmd = Command::new(program);
                cmd.args(["-NoExit", "-NoLogo", "-Command", &cmdline])
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
