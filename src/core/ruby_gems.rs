//! Helpers for invoking `gem` against the activated Ruby (`GEM_HOME` / PATH).

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

/// `gem list -e` regex for an exact gem name.
pub fn gem_list_pattern(name: &str) -> String {
    let mut s = String::from("^");
    for c in name.chars() {
        match c {
            '^' | '$' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                s.push('\\');
                s.push(c);
            }
            _ => s.push(c),
        }
    }
    s.push('$');
    s
}

fn gem_program() -> PathBuf {
    PathBuf::from("gem")
}

/// First installed version for this gem in the current environment (default or only).
pub fn gem_local_version(name: &str) -> Result<Option<String>> {
    let exe = gem_program();
    let pat = gem_list_pattern(name);
    let out = Command::new(&exe)
        .args(["list", "-e", &pat])
        .output()
        .with_context(|| format!("failed to spawn {:?}", exe.display()))?;

    if !out.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8_lossy(&out.stdout);
    parse_gem_list_first_version(name, &text)
}

fn parse_gem_list_first_version(name: &str, stdout: &str) -> Result<Option<String>> {
    let prefix = format!("{} (", name);
    for raw in stdout.lines() {
        let line = raw.trim_start();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let inner = rest.trim_end_matches(')').trim();
            let part = inner.strip_prefix("default:").unwrap_or(inner).trim();
            let first = part.split(',').next().unwrap_or(part).trim();
            if !first.is_empty() {
                return Ok(Some(first.to_string()));
            }
        }
    }
    Ok(None)
}

#[derive(Debug, Deserialize)]
struct RubygemsGemInfo {
    version: String,
}

/// Latest release version from rubygems.org (simple JSON).
pub fn rubygems_latest_version(name: &str) -> Result<String> {
    let url = format!("https://rubygems.org/api/v1/gems/{name}.json");
    let v: RubygemsGemInfo = Client::new()
        .get(&url)
        .header("User-Agent", "ven/0.1 (https://github.com/)")
        .send()
        .with_context(|| format!("GET {}", url))?
        .error_for_status()
        .with_context(|| format!("rubygems.org: gem {:?}", name))?
        .json()
        .with_context(|| format!("parse rubygems JSON for {:?}", name))?;
    Ok(v.version.trim().to_string())
}

pub fn gem_install(name: &str, version: Option<&str>) -> Result<()> {
    let exe = gem_program();
    let mut cmd = Command::new(&exe);
    cmd.arg("install").arg("--no-document").arg(name);
    if let Some(v) = version {
        if !v.is_empty() && v != "latest" && v != "*" {
            cmd.args(["-v", v]);
        }
    }
    let st = cmd
        .status()
        .with_context(|| format!("failed to spawn {:?}", exe.display()))?;
    if !st.success() {
        anyhow::bail!("gem install {} failed (exit {:?})", name, st.code());
    }
    Ok(())
}

/// Uninstall all local versions of the gem (non-interactive).
pub fn gem_uninstall_all(name: &str) -> Result<()> {
    let exe = gem_program();
    let st = Command::new(&exe)
        .args(["uninstall", name, "-aIx"])
        .status()
        .with_context(|| format!("failed to spawn {:?}", exe.display()))?;
    if !st.success() {
        anyhow::bail!("gem uninstall {} failed (exit {:?})", name, st.code());
    }
    Ok(())
}
