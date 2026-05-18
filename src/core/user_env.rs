//! Persist a user-scoped environment variable across shell sessions.
//!
//! `ven path set <dir>` uses this to write `VEN_HOME` somewhere the OS will
//! re-export into every future terminal, so external tools (npm, pip,
//! editor terminals, third-party shells) inherit the relocated storage
//! root without ven having to be in the loop. The pointer file in
//! `~/.config/ven/config.toml` is ven's source of truth; this is purely a
//! convenience for the rest of the user's environment.
//!
//! Failures here are non-fatal — the caller surfaces them as warnings, not
//! errors. The pointer file is enough on its own for ven itself.
//!
//! ## Platforms
//!
//! - **Windows**: `[Environment]::SetEnvironmentVariable($name, $value, 'User')`
//!   via PowerShell, plus a `WM_SETTINGCHANGE` broadcast so already-running
//!   Explorer / shells refresh their env block. Same pattern as the
//!   `ensure_path_contains` helper in `src/bin/setup/windows.rs` — we keep
//!   the two implementations independent because they live in different
//!   binaries (`ven` vs. `ven-setup`).
//!
//! - **Unix**: append/replace a single `export NAME="value"` line inside a
//!   `# >>> ven env >>>` / `# <<< ven env <<<` block in every rc file that
//!   actually exists in `$HOME` (`.bashrc`, `.zshrc`, `.profile`, fish
//!   `config.fish`). Same fenced-block convention `ven setup` uses for the
//!   shell hook, so the user has one place to look when something feels off.

use anyhow::{anyhow, Context, Result};

// `Path` and the rc-file block markers are only consumed by the Unix branch.
// On Windows persistence runs through the registry, which doesn't read or
// write the rc files at all.
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::Path;

/// Marker that opens the ven-managed block in user rc files (Unix only).
#[cfg(unix)]
const VEN_ENV_BLOCK_START: &str = "# >>> ven env >>>";
/// Marker that closes it (Unix only).
#[cfg(unix)]
const VEN_ENV_BLOCK_END: &str = "# <<< ven env <<<";

/// Set `name=value` in the user's persistent environment. Returns `Ok(())`
/// on success, error with a human-readable message on failure. Caller should
/// downgrade to a warning if appropriate (most `ven path set` flows do).
pub fn set_user_env(name: &str, value: &str) -> Result<()> {
    validate_name(name)?;
    #[cfg(windows)]
    {
        set_windows_user_env(name, value)
    }
    #[cfg(unix)]
    {
        set_unix_user_env(name, value)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (name, value);
        Err(anyhow!(
            "Persistent user-env is not implemented for this platform"
        ))
    }
}

/// Remove `name` from the user's persistent environment.
pub fn unset_user_env(name: &str) -> Result<()> {
    validate_name(name)?;
    #[cfg(windows)]
    {
        unset_windows_user_env(name)
    }
    #[cfg(unix)]
    {
        unset_unix_user_env(name)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = name;
        Err(anyhow!(
            "Persistent user-env is not implemented for this platform"
        ))
    }
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Env var name cannot be empty"));
    }
    if name.chars().any(|c| c == '=' || c == '\0') {
        return Err(anyhow!(
            "Env var name {name:?} contains an illegal character ('=' or NUL)"
        ));
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// Windows
// ─────────────────────────────────────────────────────────────────────────

#[cfg(windows)]
fn set_windows_user_env(name: &str, value: &str) -> Result<()> {
    let name_ps = ps_single_quote(name);
    let value_ps = ps_single_quote(value);

    // The script mirrors `ensure_path_contains` in src/bin/setup/windows.rs:
    // SetEnvironmentVariable for the User scope, then broadcast
    // WM_SETTINGCHANGE so Explorer / running shells refresh their env. We
    // keep the two implementations independent (different bin crates) but
    // identical in behaviour; if you find yourself editing one, audit the
    // other.
    let script = format!(
        r#"$name = '{name_ps}'
$value = '{value_ps}'
[Environment]::SetEnvironmentVariable($name, $value, 'User')

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
namespace Win32VenEnv {{
  public static class Native {{
    [DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern IntPtr SendMessageTimeout(
      IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
      uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
  }}
}}
'@
$HWND_BROADCAST = [IntPtr]0xffff
$WM_SETTINGCHANGE = 0x001A
[UIntPtr]$result = [UIntPtr]::Zero
[Win32VenEnv.Native]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result) | Out-Null"#,
    );

    run_powershell(&script).with_context(|| format!("Failed to set {} in User env", name))
}

#[cfg(windows)]
fn unset_windows_user_env(name: &str) -> Result<()> {
    let name_ps = ps_single_quote(name);
    let script = format!(
        r#"$name = '{name_ps}'
[Environment]::SetEnvironmentVariable($name, $null, 'User')"#,
    );
    run_powershell(&script).with_context(|| format!("Failed to unset {} in User env", name))
}

#[cfg(windows)]
fn run_powershell(script: &str) -> Result<()> {
    use std::process::Command;
    let status = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .status()
        .context("Failed to spawn powershell.exe")?;
    if !status.success() {
        anyhow::bail!("PowerShell exited with status {}", status);
    }
    Ok(())
}

#[cfg(windows)]
fn ps_single_quote(s: &str) -> String {
    // PowerShell single-quoted strings: only `'` needs escaping (as `''`).
    s.replace('\'', "''")
}

// ─────────────────────────────────────────────────────────────────────────
// Unix
// ─────────────────────────────────────────────────────────────────────────

#[cfg(unix)]
fn set_unix_user_env(name: &str, value: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve $HOME"))?;
    let mut touched_any = false;

    for rc in candidate_rc_files(&home) {
        if !rc.is_file() {
            continue;
        }
        upsert_rc_block(&rc, name, value)?;
        touched_any = true;
    }

    // No rc file existed — write to ~/.profile so a future bash/sh sees it.
    if !touched_any {
        let profile = home.join(".profile");
        upsert_rc_block(&profile, name, value)?;
    }
    Ok(())
}

#[cfg(unix)]
fn unset_unix_user_env(name: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve $HOME"))?;
    for rc in candidate_rc_files(&home) {
        if !rc.is_file() {
            continue;
        }
        let existing =
            fs::read_to_string(&rc).with_context(|| format!("Failed to read {}", rc.display()))?;
        let updated = remove_block_for(&existing, name);
        if updated != existing {
            fs::write(&rc, updated).with_context(|| format!("Failed to write {}", rc.display()))?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn candidate_rc_files(home: &Path) -> Vec<std::path::PathBuf> {
    vec![
        home.join(".bashrc"),
        home.join(".zshrc"),
        home.join(".profile"),
        home.join(".config").join("fish").join("config.fish"),
    ]
}

#[cfg(unix)]
fn upsert_rc_block(rc: &Path, name: &str, value: &str) -> Result<()> {
    if let Some(parent) = rc.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
    }
    let existing = if rc.is_file() {
        fs::read_to_string(rc).with_context(|| format!("Failed to read {}", rc.display()))?
    } else {
        String::new()
    };

    let is_fish = rc.extension().is_some_and(|e| e == "fish");
    let block = render_block(name, value, is_fish);

    let updated = replace_block_for(&existing, name, &block);

    if updated == existing && !existing.contains(VEN_ENV_BLOCK_START) {
        // No existing block, just append.
        let mut out = existing;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&block);
        out.push('\n');
        fs::write(rc, out).with_context(|| format!("Failed to write {}", rc.display()))?;
    } else if updated != existing {
        fs::write(rc, updated).with_context(|| format!("Failed to write {}", rc.display()))?;
    }
    Ok(())
}

#[cfg(unix)]
fn render_block(name: &str, value: &str, is_fish: bool) -> String {
    // Double-quote the value and escape `"` and `\` so paths with spaces
    // round-trip cleanly. `$` is intentionally not escaped — a $value with
    // a literal dollar would already mean variable expansion in any shell.
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let export_line = if is_fish {
        format!("set -gx {name} \"{escaped}\"")
    } else {
        format!("export {name}=\"{escaped}\"")
    };
    format!(
        "{start}\n{export_line}\n{end}",
        start = VEN_ENV_BLOCK_START,
        export_line = export_line,
        end = VEN_ENV_BLOCK_END,
    )
}

/// Replace any existing `>>> ven env >>>` block for `name` with `block`.
/// Returns the input unchanged if no block exists (caller decides whether
/// to append).
#[cfg(unix)]
fn replace_block_for(content: &str, name: &str, block: &str) -> String {
    let Some(start) = content.find(VEN_ENV_BLOCK_START) else {
        return content.to_string();
    };
    let Some(end_rel) = content[start..].find(VEN_ENV_BLOCK_END) else {
        return content.to_string();
    };
    let end = start + end_rel + VEN_ENV_BLOCK_END.len();
    let block_content = &content[start..end];
    // Only replace the block when it actually mentions our var name. Future
    // versions may support multiple ven-managed vars per file, but for now
    // the block is owned 1:1 by VEN_HOME.
    if !block_content.contains(&format!(" {name}="))
        && !block_content.contains(&format!(" {name} "))
    {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len());
    out.push_str(&content[..start]);
    out.push_str(block);
    out.push_str(&content[end..]);
    out
}

#[cfg(unix)]
fn remove_block_for(content: &str, name: &str) -> String {
    let Some(start) = content.find(VEN_ENV_BLOCK_START) else {
        return content.to_string();
    };
    let Some(end_rel) = content[start..].find(VEN_ENV_BLOCK_END) else {
        return content.to_string();
    };
    let end = start + end_rel + VEN_ENV_BLOCK_END.len();
    let block_content = &content[start..end];
    if !block_content.contains(&format!(" {name}="))
        && !block_content.contains(&format!(" {name} "))
    {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len());
    out.push_str(&content[..start]);
    // Eat the trailing newline if present so we don't leave a stray blank line.
    let mut tail_start = end;
    if content[tail_start..].starts_with('\n') {
        tail_start += 1;
    }
    out.push_str(&content[tail_start..]);
    out
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn render_block_bash_quoting() {
        let b = render_block("VEN_HOME", "/tmp/has space", false);
        assert!(b.contains("export VEN_HOME=\"/tmp/has space\""));
        assert!(b.contains(VEN_ENV_BLOCK_START));
        assert!(b.contains(VEN_ENV_BLOCK_END));
    }

    #[test]
    fn render_block_fish_quoting() {
        let b = render_block("VEN_HOME", "/data/ven", true);
        assert!(b.contains("set -gx VEN_HOME \"/data/ven\""));
    }

    #[test]
    fn render_block_escapes_quotes_and_backslashes() {
        let b = render_block("VEN_HOME", r#"C:\path\with"quote"#, false);
        assert!(
            b.contains(r#"export VEN_HOME="C:\\path\\with\"quote""#),
            "unexpected block: {b}"
        );
    }

    #[test]
    fn upsert_appends_when_missing_and_replaces_when_present() {
        let temp = tempfile::tempdir().unwrap();
        let rc = temp.path().join(".bashrc");
        std::fs::write(&rc, "# original content\nalias ll='ls -la'\n").unwrap();

        upsert_rc_block(&rc, "VEN_HOME", "/tmp/a").unwrap();
        let after_one = std::fs::read_to_string(&rc).unwrap();
        assert!(after_one.contains(VEN_ENV_BLOCK_START));
        assert!(after_one.contains("VEN_HOME=\"/tmp/a\""));
        assert!(after_one.contains("alias ll='ls -la'"));

        upsert_rc_block(&rc, "VEN_HOME", "/tmp/b").unwrap();
        let after_two = std::fs::read_to_string(&rc).unwrap();
        assert!(after_two.contains("VEN_HOME=\"/tmp/b\""));
        assert!(!after_two.contains("VEN_HOME=\"/tmp/a\""));
        // Should still be exactly one block.
        assert_eq!(
            after_two.matches(VEN_ENV_BLOCK_START).count(),
            1,
            "block was duplicated; content: {after_two}"
        );
    }

    #[test]
    fn remove_block_leaves_other_content_intact() {
        let block = render_block("VEN_HOME", "/x", false);
        let content = format!("# user stuff\n{block}\n# more user stuff\n");
        let out = remove_block_for(&content, "VEN_HOME");
        assert!(out.contains("# user stuff"));
        assert!(out.contains("# more user stuff"));
        assert!(!out.contains(VEN_ENV_BLOCK_START));
    }

    #[test]
    fn remove_block_for_unrelated_name_is_noop() {
        let block = render_block("VEN_HOME", "/x", false);
        let content = format!("# user stuff\n{block}\n");
        let out = remove_block_for(&content, "SOME_OTHER_VAR");
        assert_eq!(out, content);
    }

    #[test]
    fn validate_name_rejects_bad_input() {
        assert!(validate_name("").is_err());
        assert!(validate_name("FOO=BAR").is_err());
        assert!(validate_name("FOO").is_ok());
        assert!(validate_name("VEN_HOME").is_ok());
    }
}
