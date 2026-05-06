//! Shell detection for spawning terminals (launcher).

use std::path::Path;
use std::process::Command;

/// Kind of interactive shell chosen for spawning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellKind {
    PowerShell,
    Cmd,
    Bash,
    Zsh,
    /// Unrecognized `$SHELL` or other shell path.
    Other(String),
}

impl std::fmt::Display for ShellKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellKind::PowerShell => write!(f, "PowerShell"),
            ShellKind::Cmd => write!(f, "Cmd"),
            ShellKind::Bash => write!(f, "Bash"),
            ShellKind::Zsh => write!(f, "Zsh"),
            ShellKind::Other(path) => {
                if let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) {
                    write!(f, "{name}")
                } else {
                    write!(f, "{path}")
                }
            }
        }
    }
}

/// Decide which shell the launcher should use for new terminal sessions.
pub fn detect_shell() -> ShellKind {
    #[cfg(windows)]
    {
        windows_detect_shell()
    }
    #[cfg(not(windows))]
    {
        unix_detect_shell()
    }
}

#[cfg(windows)]
fn windows_detect_shell() -> ShellKind {
    if powershell_on_path() || powershell_default_exists() {
        return ShellKind::PowerShell;
    }
    ShellKind::Cmd
}

#[cfg(windows)]
fn powershell_default_exists() -> bool {
    powershell_exe_paths().iter().any(|p| Path::new(p).is_file())
}

#[cfg(windows)]
fn powershell_exe_paths() -> &'static [&'static str] {
    &[
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        r"C:\Windows\SysWOW64\WindowsPowerShell\v1.0\powershell.exe",
    ]
}

#[cfg(windows)]
fn powershell_on_path() -> bool {
    let out = Command::new("where")
        .args(["powershell"])
        .output();
    matches!(out, Ok(o) if o.status.success())
}

#[cfg(not(windows))]
fn unix_detect_shell() -> ShellKind {
    let shell = match std::env::var("SHELL") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => return ShellKind::Bash,
    };

    classify_unix_shell(&shell).unwrap_or(ShellKind::Other(shell))
}

#[cfg(not(windows))]
fn classify_unix_shell(path: &str) -> Option<ShellKind> {
    let lower = path.to_ascii_lowercase();
    if lower.contains("zsh") {
        return Some(ShellKind::Zsh);
    }
    if lower.contains("bash") {
        return Some(ShellKind::Bash);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(windows))]
    #[test]
    fn unix_classify_bash_zsh() {
        assert_eq!(classify_unix_shell("/bin/bash"), Some(ShellKind::Bash));
        assert_eq!(
            classify_unix_shell("/usr/local/bin/zsh"),
            Some(ShellKind::Zsh)
        );
    }

    #[test]
    fn display_names() {
        assert_eq!(format!("{}", ShellKind::PowerShell), "PowerShell");
        assert_eq!(format!("{}", ShellKind::Cmd), "Cmd");
        assert_eq!(format!("{}", ShellKind::Bash), "Bash");
        assert_eq!(format!("{}", ShellKind::Zsh), "Zsh");
        assert_eq!(format!("{}", ShellKind::Other("/bin/fish".into())), "fish");
    }
}
