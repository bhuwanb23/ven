//! Welcome banner and active-runtime summary for `ven-launcher`.

use std::io::Write;

use crate::launcher::quote::bash_single_quoted;
use crate::shell::{path_for_env_value, ActivationParts};

#[derive(Clone, Copy, Debug)]
pub enum GreetingStyle {
    Unicode,
    Ascii,
}

const INNER_WIDTH: usize = 54;

/// Shared top banner so launcher looks the same regardless of project detection state.
pub fn generic_header_lines(style: GreetingStyle) -> Vec<String> {
    let mut lines = Vec::new();
    match style {
        GreetingStyle::Unicode => {
            let rule = "═".repeat(INNER_WIDTH);
            lines.push(format!("╔{rule}╗"));
            lines.push(format!(
                "║{}║",
                pad_inner("  VEN LAUNCHER  •  Environment Ready", INNER_WIDTH)
            ));
            lines.push(format!(
                "║{}║",
                pad_inner("  Fast project shell with managed runtimes", INNER_WIDTH)
            ));
            lines.push(format!("╚{rule}╝"));
        }
        GreetingStyle::Ascii => {
            let rule = "-".repeat(INNER_WIDTH);
            lines.push(format!("+{rule}+"));
            lines.push(format!(
                "|{}|",
                pad_inner("  VEN LAUNCHER - Environment Ready", INNER_WIDTH)
            ));
            lines.push(format!(
                "|{}|",
                pad_inner("  Fast project shell with managed runtimes", INNER_WIDTH)
            ));
            lines.push(format!("+{rule}+"));
        }
    }
    lines
}

/// Lines to print (ends with one blank line for spacing).
pub fn greeting_lines(parts: &ActivationParts, style: GreetingStyle) -> Vec<String> {
    let mut lines = generic_header_lines(style);

    lines.push(String::new());

    let name = parts
        .project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    lines.push(format!("Project: {name}"));
    lines.push(format!(
        "Location: {}",
        path_for_env_value(&parts.project_root)
    ));
    lines.push(String::new());
    lines.push("Active runtimes".to_string());
    lines.push("-".repeat(26));

    push_runtime(
        &mut lines,
        "Node",
        "JS runtime",
        parts.node_resolved.as_ref(),
        style,
    );
    push_runtime(
        &mut lines,
        "Python",
        "Scripting runtime",
        parts.python_resolved.as_ref(),
        style,
    );
    push_runtime(
        &mut lines,
        "Go",
        "Toolchain",
        parts.go_resolved.as_ref(),
        style,
    );
    push_runtime(
        &mut lines,
        "Rust",
        "Toolchain",
        parts.rust_resolved.as_ref(),
        style,
    );
    push_runtime(
        &mut lines,
        "Java",
        "JDK",
        parts.java_resolved.as_ref(),
        style,
    );
    push_runtime(
        &mut lines,
        "Deno",
        "Runtime",
        parts.deno_resolved.as_ref(),
        style,
    );
    push_runtime(
        &mut lines,
        "Ruby",
        "MRI",
        parts.ruby_resolved.as_ref(),
        style,
    );

    lines.push(String::new());
    lines.push("Quick start".to_string());
    lines.push("-".repeat(26));
    match style {
        GreetingStyle::Unicode => {
            lines.push("  • node --version".to_string());
            lines.push("  • python --version".to_string());
            lines.push("  • ruby --version".to_string());
            lines.push("  • ven status --verbose".to_string());
        }
        GreetingStyle::Ascii => {
            lines.push("  - node --version".to_string());
            lines.push("  - python --version".to_string());
            lines.push("  - ruby --version".to_string());
            lines.push("  - ven status --verbose".to_string());
        }
    }

    lines.push(String::new());
    lines
}

fn pad_inner(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n >= width {
        s.chars().take(width).collect()
    } else {
        format!("{}{}", s, " ".repeat(width - n))
    }
}

fn push_runtime(
    lines: &mut Vec<String>,
    label: &str,
    desc: &str,
    v: Option<&String>,
    style: GreetingStyle,
) {
    let bullet = match style {
        GreetingStyle::Unicode => "  ▸",
        GreetingStyle::Ascii => "  >",
    };
    if let Some(s) = v {
        let t = s.trim();
        if !t.is_empty() {
            lines.push(format!("{bullet} {label:<8} {t:<12} ({desc})"));
        }
    }
}

pub fn write_greeting_to_stdout(parts: &ActivationParts) {
    for line in greeting_lines(parts, GreetingStyle::Unicode) {
        println!("{}", line);
    }
}

/// Bash/zsh `--init-file` prelude: print banner via `printf`, then caller appends `cd`.
pub fn write_posix_printf_greeting(
    dest: &mut impl Write,
    parts: &ActivationParts,
) -> std::io::Result<()> {
    let fmt_q = bash_single_quoted("%s\n");
    for line in greeting_lines(parts, GreetingStyle::Unicode) {
        writeln!(dest, "printf {} {}", fmt_q, bash_single_quoted(&line))?;
    }
    writeln!(dest, "printf {} {}", fmt_q, bash_single_quoted(""))?;
    Ok(())
}

/// PowerShell profile via `-File` (UTF-8): banner + `Set-Location`.
pub fn write_powershell_profile_init(
    dest: &mut impl Write,
    parts: &ActivationParts,
    cwd_lit: &str,
) -> std::io::Result<()> {
    writeln!(dest, "Write-Host ''")?;
    for line in greeting_lines(parts, GreetingStyle::Unicode) {
        writeln!(dest, "Write-Host '{}'", line.replace('\'', "''"))?;
    }
    writeln!(dest, "Write-Host ''")?;
    writeln!(
        dest,
        "Set-Location -LiteralPath '{}'",
        cwd_lit.replace('\'', "''")
    )?;
    Ok(())
}

/// Windows `cmd.exe` ASCII banner + `cd /d` (`^|` so `echo` does not treat `|` as a pipe).
pub fn write_cmd_autorun(
    dest: &mut impl Write,
    parts: &ActivationParts,
    cwd_cmd: &str,
) -> std::io::Result<()> {
    writeln!(dest, "@echo off")?;
    writeln!(dest, "")?;
    for line in greeting_lines(parts, GreetingStyle::Ascii) {
        let safe = line.replace('|', "^|");
        writeln!(dest, "echo {safe}")?;
    }
    writeln!(dest, "")?;
    writeln!(dest, "cd /d {cwd_cmd}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ActivationParts;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn banner_lists_project_and_node() {
        let parts = ActivationParts {
            project_root: PathBuf::from("my-app"),
            prepend_dirs: vec![],
            node_bin_for_path: None,
            node_resolved: Some("20.20.2".into()),
            python_resolved: Some("3.11.5".into()),
            go_resolved: None,
            go_root_for_env: None,
            rust_resolved: None,
            rust_root_for_env: None,
            java_resolved: None,
            java_home_for_env: None,
            deno_resolved: None,
            bun_resolved: None,
            ruby_resolved: None,
            ruby_gem_home_for_env: None,
            php_resolved: None,
            php_root_for_env: None,
            virtual_env_root: None,
            toml_normalized: String::new(),
            ven_user_env: HashMap::new(),
        };
        let text = greeting_lines(&parts, GreetingStyle::Unicode).join("\n");
        assert!(text.contains("VEN LAUNCHER"));
        assert!(text.contains("Project: my-app"));
        assert!(text.contains("Node"));
        assert!(text.contains("20.20.2"));
        assert!(text.contains("Python"));
        assert!(text.contains("3.11.5"));
    }
}
