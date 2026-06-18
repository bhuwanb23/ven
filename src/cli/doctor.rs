//! `ven doctor` — diagnose multiple installs, PATH shadowing, and upgrade paths.

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

const MIN_SELF_UPDATE_VERSION: &str = "0.1.7";

#[derive(Debug, Clone, Serialize)]
struct InstallEntry {
    path: String,
    version: Option<String>,
    mode: &'static str,
    writable: bool,
    on_path: bool,
    path_index: Option<usize>,
    supports_update: bool,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    running_exe: String,
    running_version: String,
    path_winner: Option<String>,
    path_winner_version: Option<String>,
    entries: Vec<InstallEntry>,
    hints: Vec<String>,
}

pub fn cmd_doctor(json: bool) -> Result<()> {
    let running_exe =
        std::env::current_exe().context("Could not resolve running ven executable")?;
    let running_version = env!("CARGO_PKG_VERSION").to_string();

    let path_dirs = path_directories();
    let mut seen = HashSet::new();
    let mut entries = Vec::new();

    for candidate in candidate_bins() {
        let key = candidate.to_string_lossy().to_lowercase();
        if !seen.insert(key) {
            continue;
        }
        if let Some(entry) = probe_install(&candidate, &path_dirs) {
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let path_winner = find_path_winner();
    let path_winner_version = path_winner.as_ref().and_then(|p| read_ven_version(p).ok());

    let hints = build_hints(
        &running_exe,
        &running_version,
        &entries,
        path_winner.as_deref(),
        path_winner_version.as_deref(),
    );

    if json {
        let report = DoctorReport {
            running_exe: running_exe.display().to_string(),
            running_version: running_version.clone(),
            path_winner: path_winner.map(|p| p.display().to_string()),
            path_winner_version,
            entries,
            hints,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).context("Could not serialize doctor report")?
        );
        return Ok(());
    }

    println!();
    println!("{}", "ven doctor".bold());
    println!(
        "  running : {} ({})",
        running_exe.display(),
        running_version
    );
    if let Some(winner) = path_winner.as_ref() {
        println!(
            "  PATH    : {} {}",
            winner.display(),
            path_winner_version
                .as_deref()
                .map(|v| format!("(ven {v})"))
                .unwrap_or_else(|| "(version unknown)".to_string())
        );
    } else {
        println!("  PATH    : {}", "(no ven found on PATH)".yellow());
    }
    println!();

    if entries.is_empty() {
        println!("  No ven installations found in known locations.");
    } else {
        println!(
            "  {:<44} {:<8} {:<8} {:^6} {:^6} {:^8}",
            "binary", "version", "mode", "PATH", "idx", "update"
        );
        for e in &entries {
            let ver = e.version.as_deref().unwrap_or("?");
            let idx = e
                .path_index
                .map(|i| i.to_string())
                .unwrap_or_else(|| "-".to_string());
            let upd = if e.supports_update { "yes" } else { "no" };
            let on_path = if e.on_path { "yes" } else { "-" };
            println!(
                "  {:<44} {:<8} {:<8} {:^6} {:^6} {:^8}",
                truncate_path(&e.path, 44),
                ver,
                e.mode,
                on_path,
                idx,
                upd
            );
        }
    }

    if !hints.is_empty() {
        println!();
        println!("{}", "Hints:".bold());
        for h in &hints {
            println!("  • {h}");
        }
    }
    println!();
    Ok(())
}

fn candidate_bins() -> Vec<PathBuf> {
    let mut out = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        out.push(exe);
    }

    if let Some(home) = dirs::home_dir() {
        #[cfg(windows)]
        {
            out.push(home.join(".ven").join("bin").join("ven.exe"));
        }
        #[cfg(not(windows))]
        {
            out.push(home.join(".ven").join("bin").join("ven"));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(pf) = std::env::var("ProgramFiles") {
            out.push(PathBuf::from(pf).join("ven").join("bin").join("ven.exe"));
        }
        if let Ok(paths) = which_all_windows("ven.exe") {
            out.extend(paths);
        }
    }

    #[cfg(not(windows))]
    {
        out.push(PathBuf::from("/usr/local/bin/ven"));
        if let Ok(paths) = which_all_unix("ven") {
            out.extend(paths);
        }
    }

    out
}

fn probe_install(bin: &Path, path_dirs: &[PathBuf]) -> Option<InstallEntry> {
    if !bin.is_file() {
        return None;
    }
    let version = read_ven_version(bin).ok();
    let parent = bin.parent()?;
    let mode = classify_install_dir(parent);
    let writable = write_probe_dir(parent);
    let norm_parent = normalize_path(parent);
    let (on_path, path_index) = path_dirs
        .iter()
        .enumerate()
        .find(|(_, d)| normalize_path(d) == norm_parent)
        .map(|(i, _)| (true, Some(i)))
        .unwrap_or((false, None));
    let supports_update = version
        .as_deref()
        .map(version_supports_update)
        .unwrap_or(false);

    Some(InstallEntry {
        path: bin.display().to_string(),
        version,
        mode,
        writable,
        on_path,
        path_index,
        supports_update,
    })
}

fn classify_install_dir(dir: &Path) -> &'static str {
    let s = dir.to_string_lossy().to_lowercase().replace('\\', "/");
    if s.ends_with("/.ven/bin") {
        return "user";
    }
    if s.contains("/program files/ven")
        || s == "/usr/local/bin"
        || s.starts_with("/usr/bin/ven")
        || s == "/usr/bin"
    {
        return "system";
    }
    "portable"
}

fn write_probe_dir(dir: &Path) -> bool {
    if !dir.exists() {
        return std::fs::create_dir_all(dir).is_ok();
    }
    let probe = dir.join(".ven-doctor-probe");
    match std::fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn path_directories() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p)
                .filter(|d| d.as_os_str().len() > 0)
                .collect()
        })
        .unwrap_or_default()
}

fn find_path_winner() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        which_all_windows("ven.exe")
            .ok()
            .and_then(|v| v.into_iter().next())
    }
    #[cfg(not(windows))]
    {
        which_all_unix("ven")
            .ok()
            .and_then(|v| v.into_iter().next())
    }
}

#[cfg(windows)]
fn which_all_windows(name: &str) -> Result<Vec<PathBuf>> {
    let output = Command::new("where.exe")
        .arg(name)
        .output()
        .context("Failed to run where.exe")?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect())
}

#[cfg(not(windows))]
fn which_all_unix(name: &str) -> Result<Vec<PathBuf>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "command -v {name} 2>/dev/null; which -a {name} 2>/dev/null || true"
        ))
        .output()
        .context("Failed to run which")?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect())
}

fn read_ven_version(bin: &Path) -> Result<String> {
    let output = Command::new(bin)
        .arg("--version")
        .output()
        .with_context(|| format!("Failed to run {} --version", bin.display()))?;
    if !output.status.success() {
        anyhow::bail!("{} --version exited with {}", bin.display(), output.status);
    }
    let line = String::from_utf8_lossy(&output.stdout);
    parse_ven_version_line(&line)
        .ok_or_else(|| anyhow::anyhow!("Could not parse version from: {}", line.trim()))
}

fn parse_ven_version_line(line: &str) -> Option<String> {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("ven ") {
        return Some(rest.split_whitespace().next()?.to_string());
    }
    line.split_whitespace().next().map(|s| s.to_string())
}

fn version_supports_update(version: &str) -> bool {
    let cur = parse_semver_triple(version);
    let min = parse_semver_triple(MIN_SELF_UPDATE_VERSION);
    cur >= min
}

fn parse_semver_triple(s: &str) -> (u32, u32, u32) {
    let mut parts = s.split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts
        .next()
        .and_then(|p| p.parse::<u32>().ok())
        .or_else(|| {
            parts.next().and_then(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            })
        })
        .unwrap_or(0);
    (major, minor, patch)
}

fn normalize_path(p: &Path) -> String {
    p.to_string_lossy().to_lowercase().replace('\\', "/")
}

fn truncate_path(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("...{}", &s[s.len().saturating_sub(max - 3)..])
    }
}

fn build_hints(
    running_exe: &Path,
    running_version: &str,
    entries: &[InstallEntry],
    path_winner: Option<&Path>,
    path_winner_version: Option<&str>,
) -> Vec<String> {
    let mut hints = Vec::new();

    let versions: Vec<_> = entries
        .iter()
        .filter_map(|e| e.version.as_deref())
        .collect();
    let distinct: HashSet<_> = versions.iter().copied().collect();
    if distinct.len() > 1 {
        hints.push(
            "Multiple ven versions are installed. Only the first match on PATH runs when you type `ven`.".into(),
        );
    }

    if let (Some(winner), Some(wv)) = (path_winner, path_winner_version) {
        if !version_supports_update(wv) {
            hints.push(format!(
                "PATH resolves to ven {wv} at {}, which cannot run `ven update` (added in v{MIN_SELF_UPDATE_VERSION}). \
                 Re-install with the install.ps1 / install.sh one-liner or ven-setup, then use `ven update`.",
                winner.display()
            ));
        }
        if wv != running_version {
            hints.push(format!(
                "You invoked {} (ven {}) but `ven` on PATH is {} (ven {}). \
                 Use the full path, fix PATH order, or upgrade the install that PATH uses.",
                running_exe.display(),
                running_version,
                winner.display(),
                wv
            ));
        }
        if let Some(newer) = entries
            .iter()
            .filter(|e| {
                e.version
                    .as_deref()
                    .map(|v| parse_semver_triple(v) > parse_semver_triple(wv))
                    .unwrap_or(false)
            })
            .max_by(|a, b| {
                let va = a.version.as_deref().unwrap_or("0.0.0");
                let vb = b.version.as_deref().unwrap_or("0.0.0");
                parse_semver_triple(va).cmp(&parse_semver_triple(vb))
            })
        {
            hints.push(format!(
                "A newer ven {} exists at {} but PATH still uses {}. \
                 Upgrade that location (e.g. elevated install.ps1 -Mode system with VEN_FORCE_INSTALL=true) \
                 or uninstall the older copy.",
                newer.version.as_deref().unwrap_or("?"),
                newer.path,
                winner.display()
            ));
        }
    }

    let has_system = entries.iter().any(|e| e.mode == "system");
    let has_user = entries.iter().any(|e| e.mode == "user");
    if has_system && has_user {
        hints.push(
            "Both user (~/.ven/bin) and system (Program Files or /usr/local/bin) installs exist. \
             On Windows, Machine PATH is searched before User PATH — system ven wins unless you remove or upgrade it."
                .into(),
        );
    }

    if hints.is_empty() {
        hints.push(
            "No issues detected. Run `ven update --check` to see if a newer release is available."
                .into(),
        );
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ven_version_line_accepts_prefix() {
        assert_eq!(
            parse_ven_version_line("ven 0.2.1\n"),
            Some("0.2.1".to_string())
        );
    }

    #[test]
    fn version_supports_update_cutoff() {
        assert!(!version_supports_update("0.1.6"));
        assert!(version_supports_update("0.1.7"));
        assert!(version_supports_update("0.2.1"));
    }

    #[test]
    fn parse_semver_triple_orders() {
        assert!(parse_semver_triple("0.2.1") > parse_semver_triple("0.1.4"));
    }
}
