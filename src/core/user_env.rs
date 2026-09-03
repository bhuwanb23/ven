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

/// Marker that opens the ven-managed *global PATH* block in user rc files
/// (Unix only). One `export PATH="…:$PATH"` line per globally-enabled
/// runtime (`ven set global <lang>`), so a language binary is on PATH in
/// every new shell without any project `ven.toml`.
#[cfg(unix)]
const VEN_GLOBAL_BLOCK_START: &str = "# >>> ven global PATH >>>";
/// Marker that closes it (Unix only).
#[cfg(unix)]
const VEN_GLOBAL_BLOCK_END: &str = "# <<< ven global PATH <<<";

// ─────────────────────────────────────────────────────────────────────────
// Global PATH management (`ven set global`)
// ─────────────────────────────────────────────────────────────────────────
//
// Persists a runtime's bin dir on the *User* PATH (Windows) or in a
// fenced rc-file block (Unix) so the runtime is available in every new
// shell — no admin rights needed. All functions are idempotent: adding an
// entry that's already present is a no-op, removing a missing one is too.

/// Add `entry` to the user's persistent PATH if it isn't already there.
/// Returns `Ok(true)` when the entry was added, `Ok(false)` when it was
/// already present.
pub fn add_global_path(entry: &std::path::Path) -> Result<bool> {
    #[cfg(windows)]
    {
        add_global_path_windows(entry)
    }
    #[cfg(unix)]
    {
        add_global_path_unix(entry)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = entry;
        Err(anyhow!(
            "Global PATH management is not implemented for this platform"
        ))
    }
}

/// Remove `entry` from the user's persistent PATH. Returns `Ok(true)`
/// when it was removed, `Ok(false)` when it wasn't present.
pub fn remove_global_path(entry: &std::path::Path) -> Result<bool> {
    #[cfg(windows)]
    {
        remove_global_path_windows(entry)
    }
    #[cfg(unix)]
    {
        remove_global_path_unix(entry)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = entry;
        Err(anyhow!(
            "Global PATH management is not implemented for this platform"
        ))
    }
}

/// List the entries currently on the user's persistent PATH.
pub fn list_global_paths() -> Result<Vec<std::path::PathBuf>> {
    #[cfg(windows)]
    {
        list_global_paths_windows()
    }
    #[cfg(unix)]
    {
        list_global_paths_unix()
    }
    #[cfg(not(any(windows, unix)))]
    {
        Err(anyhow!(
            "Global PATH management is not implemented for this platform"
        ))
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Windows: User-scope PATH in the registry
// ─────────────────────────────────────────────────────────────────────────

#[cfg(windows)]
fn add_global_path_windows(entry: &std::path::Path) -> Result<bool> {
    let target = entry.to_string_lossy();
    let target_ps = ps_single_quote(&target);
    let script = format!(
        r#"$target = '{target_ps}'
$current = [Environment]::GetEnvironmentVariable('Path', 'User')
if ([string]::IsNullOrWhiteSpace($current)) {{
  $new = $target
  $added = $true
}} elseif ($current -split ';' | Where-Object {{ $_.Trim().TrimEnd('\').ToLowerInvariant() -eq $target.TrimEnd('\').ToLowerInvariant() }}) {{
  $new = $current
  $added = $false
}} else {{
  $new = $current.TrimEnd(';') + ';' + $target
  $added = $true
}}
[Environment]::SetEnvironmentVariable('Path', $new, 'User')
Write-Output $added

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
namespace Win32VenGlobal {{
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
[Win32VenGlobal.Native]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result) | Out-Null"#,
    );
    let output = run_powershell_capture(&script)
        .with_context(|| format!("Failed to add {} to User PATH", target))?;
    Ok(output.contains("True"))
}

#[cfg(windows)]
fn remove_global_path_windows(entry: &std::path::Path) -> Result<bool> {
    let target = entry.to_string_lossy();
    let target_ps = ps_single_quote(&target);
    let script = format!(
        r#"$target = '{target_ps}'
$current = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $current) {{ Write-Output 'NOOP'; exit 0 }}
$parts = $current -split ';' | Where-Object {{ $_ -ne '' }}
$kept = $parts | Where-Object {{ $_.Trim().TrimEnd('\').ToLowerInvariant() -ne $target.TrimEnd('\').ToLowerInvariant() }}
if ($kept.Count -eq $parts.Count) {{
  Write-Output 'NOOP'
  exit 0
}}
$new = ($kept -join ';')
[Environment]::SetEnvironmentVariable('Path', $new, 'User')
Write-Output 'STRIPPED'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
namespace Win32VenGlobal {{
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
[Win32VenGlobal.Native]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result) | Out-Null"#,
    );
    let output = run_powershell_capture(&script)
        .with_context(|| format!("Failed to remove {} from User PATH", target))?;
    Ok(output.contains("STRIPPED"))
}

#[cfg(windows)]
fn list_global_paths_windows() -> Result<Vec<std::path::PathBuf>> {
    let script = r#"$current = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $current) {{ exit 0 }}
$current -split ';' | Where-Object {{ $_ -ne '' }}"#;
    let output = run_powershell_capture(script).context("Failed to read User PATH")?;
    Ok(output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .map(std::path::PathBuf::from)
        .collect())
}

#[cfg(windows)]
fn run_powershell_capture(script: &str) -> Result<String> {
    use std::process::Command;
    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .context("Failed to spawn powershell.exe")?;
    if !output.status.success() {
        anyhow::bail!(
            "PowerShell exited with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ─────────────────────────────────────────────────────────────────────────
// Unix: fenced `>>> ven global PATH >>>` block in rc files
// ─────────────────────────────────────────────────────────────────────────

#[cfg(unix)]
fn add_global_path_unix(entry: &std::path::Path) -> Result<bool> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve $HOME"))?;
    let candidates = candidate_rc_files(&home);
    let mut wrote_any = false;
    for rc in &candidates {
        if rc.is_file() && upsert_global_block(rc, entry)? {
            wrote_any = true;
        }
    }
    if wrote_any {
        return Ok(true);
    }
    // No rc file was modified. Either every existing rc already contained
    // the entry (already set) or no rc file exists at all (fall back to
    // ~/.profile so a future bash/sh sees it).
    if candidates.iter().any(|rc| rc.is_file()) {
        return Ok(false);
    }
    upsert_global_block(&home.join(".profile"), entry)?;
    Ok(true)
}

#[cfg(unix)]
fn remove_global_path_unix(entry: &std::path::Path) -> Result<bool> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve $HOME"))?;
    let mut removed = false;
    for rc in candidate_rc_files(&home) {
        if !rc.is_file() {
            continue;
        }
        if remove_global_block(&rc, entry)? {
            removed = true;
        }
    }
    Ok(removed)
}

#[cfg(unix)]
fn list_global_paths_unix() -> Result<Vec<std::path::PathBuf>> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot resolve $HOME"))?;
    let mut out = Vec::new();
    for rc in candidate_rc_files(&home) {
        if !rc.is_file() {
            continue;
        }
        let content =
            fs::read_to_string(&rc).with_context(|| format!("Failed to read {}", rc.display()))?;
        if let Some(start) = content.find(VEN_GLOBAL_BLOCK_START) {
            let Some(end_rel) = content[start..].find(VEN_GLOBAL_BLOCK_END) else {
                continue;
            };
            let end = start + end_rel;
            let block = &content[start..end];
            for path in parse_global_block(block) {
                if !out.contains(&path) {
                    out.push(path);
                }
            }
        }
    }
    Ok(out)
}

/// Insert/replace `entry` in the global-PATH block of `rc`. Returns
/// `Ok(true)` if the file changed.
#[cfg(unix)]
fn upsert_global_block(rc: &std::path::Path, entry: &std::path::Path) -> Result<bool> {
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
    let updated = upsert_global_content(&existing, entry, is_fish);
    if updated != existing {
        fs::write(rc, updated).with_context(|| format!("Failed to write {}", rc.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Remove `entry` from the global-PATH block of `rc`. Returns `Ok(true)`
/// if the file changed.
#[cfg(unix)]
fn remove_global_block(rc: &std::path::Path, entry: &std::path::Path) -> Result<bool> {
    if !rc.is_file() {
        return Ok(false);
    }
    let existing =
        fs::read_to_string(rc).with_context(|| format!("Failed to read {}", rc.display()))?;
    let updated = remove_global_content(&existing, entry);
    if updated != existing {
        fs::write(rc, updated).with_context(|| format!("Failed to write {}", rc.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Pure-string upsert: add or replace `entry`'s line inside the global
/// PATH block. Setting a new version of a language *replaces* the old
/// entry for that language (the last line sourced wins, so the newest
/// set version takes precedence). Appends a fresh block when none exists.
#[cfg(unix)]
fn upsert_global_content(content: &str, entry: &std::path::Path, is_fish: bool) -> String {
    let line = render_global_line(entry, is_fish);
    let entry_lang = path_language(entry);
    let Some(start) = content.find(VEN_GLOBAL_BLOCK_START) else {
        // No block yet — append one.
        let mut out = content.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(VEN_GLOBAL_BLOCK_START);
        out.push('\n');
        out.push_str(&line);
        out.push('\n');
        out.push_str(VEN_GLOBAL_BLOCK_END);
        out.push('\n');
        return out;
    };
    let Some(end_rel) = content[start..].find(VEN_GLOBAL_BLOCK_END) else {
        return content.to_string();
    };
    // Slice START..END (exclusive) so `inner` holds only the export lines.
    let end = start + end_rel + VEN_GLOBAL_BLOCK_END.len();
    let block = &content[start..start + end_rel];
    let inner = block[VEN_GLOBAL_BLOCK_START.len()..].trim_matches('\n');
    // Drop any existing line that is the same entry, or — when the new
    // entry has a recognizable `<root>/<lang>/<version>/bin` shape — the
    // same *language* (one global version per language).
    let kept: Vec<&str> = inner
        .lines()
        .filter(|l| {
            if global_line_matches(l, entry) {
                return false;
            }
            if let Some(lang) = &entry_lang {
                if let Some(existing) = parse_line_path(l) {
                    if path_language(&existing).as_deref() == Some(lang.as_str()) {
                        return false;
                    }
                }
            }
            true
        })
        .collect();
    let mut new_block = String::from(VEN_GLOBAL_BLOCK_START);
    for l in &kept {
        new_block.push('\n');
        new_block.push_str(l);
    }
    new_block.push('\n');
    new_block.push_str(&line);
    new_block.push('\n');
    new_block.push_str(VEN_GLOBAL_BLOCK_END);
    let mut out = String::with_capacity(content.len());
    out.push_str(&content[..start]);
    out.push_str(&new_block);
    out.push_str(&content[end..]);
    out
}

/// The `<lang>` component of a `<root>/<lang>/<version>/bin` path
/// (3rd-from-last), lowercased. `None` for paths without that shape.
#[cfg(unix)]
fn path_language(entry: &std::path::Path) -> Option<String> {
    let mut comps = entry.components().rev();
    let _bin = comps.next()?;
    let _version = comps.next()?;
    let lang = comps.next()?;
    Some(lang.as_os_str().to_string_lossy().to_lowercase())
}

/// Pure-string removal: drop `entry`'s line; remove the whole block when
/// it becomes empty.
#[cfg(unix)]
fn remove_global_content(content: &str, entry: &std::path::Path) -> String {
    let Some(start) = content.find(VEN_GLOBAL_BLOCK_START) else {
        return content.to_string();
    };
    let Some(end_rel) = content[start..].find(VEN_GLOBAL_BLOCK_END) else {
        return content.to_string();
    };
    // Slice START..END (exclusive) so `inner` holds only the export lines.
    let end = start + end_rel + VEN_GLOBAL_BLOCK_END.len();
    let block = &content[start..start + end_rel];
    let inner = block[VEN_GLOBAL_BLOCK_START.len()..].trim_matches('\n');
    let kept: Vec<&str> = inner
        .lines()
        .filter(|l| !global_line_matches(l, entry))
        .collect();

    if kept.is_empty() {
        // Block is now empty — remove it entirely (plus one trailing newline).
        let mut tail = end;
        if content[tail..].starts_with('\n') {
            tail += 1;
        }
        return format!("{}{}", &content[..start], &content[tail..]);
    }

    let mut new_block = String::from(VEN_GLOBAL_BLOCK_START);
    for l in &kept {
        new_block.push('\n');
        new_block.push_str(l);
    }
    new_block.push('\n');
    new_block.push_str(VEN_GLOBAL_BLOCK_END);
    let mut out = String::with_capacity(content.len());
    out.push_str(&content[..start]);
    out.push_str(&new_block);
    out.push_str(&content[end..]);
    out
}

/// Does this rc line reference `entry`'s path? Matches the quoted path
/// inside `export PATH="…:$PATH"` / `set -gx PATH "…" $PATH` lines.
#[cfg(unix)]
fn global_line_matches(line: &str, entry: &std::path::Path) -> bool {
    let want = entry.to_string_lossy().to_lowercase();
    line.to_lowercase().contains(&want)
}

/// Render one `export PATH="<entry>:$PATH"` line (or the fish equivalent).
#[cfg(unix)]
fn render_global_line(entry: &std::path::Path, is_fish: bool) -> String {
    let raw = entry.to_string_lossy();
    let escaped = raw.replace('\\', "\\\\").replace('"', "\\\"");
    if is_fish {
        format!("set -gx PATH \"{escaped}\" $PATH")
    } else {
        format!("export PATH=\"{escaped}:$PATH\"")
    }
}

/// Extract the quoted bin paths from a global-PATH block's inner content.
#[cfg(unix)]
fn parse_global_block(block: &str) -> Vec<std::path::PathBuf> {
    let inner = block
        .strip_prefix(VEN_GLOBAL_BLOCK_START)
        .unwrap_or(block)
        .trim_end_matches('\n');
    inner.lines().filter_map(parse_line_path).collect()
}

/// Extract the quoted bin path from a single block line, undoing the
/// escaping from [`render_global_line`]. `None` when the line has no
/// quoted path.
#[cfg(unix)]
fn parse_line_path(line: &str) -> Option<std::path::PathBuf> {
    let open = line.find('\"')?;
    let rest = &line[open + 1..];
    let close = rest.find('\"')?;
    let mut path = rest[..close].to_string();
    // Bash lines embed the interpolation tail inside the quotes
    // (`export PATH="<entry>:$PATH"`); fish lines keep it outside
    // (`set -gx PATH "<entry>" $PATH`). Strip it either way.
    for suffix in [":$PATH", ":$path", " $PATH", " $path"] {
        if path.ends_with(suffix) {
            path.truncate(path.len() - suffix.len());
            break;
        }
    }
    let path = path.replace("\\\\", "\\").replace("\\\"", "\"");
    if path.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(path))
    }
}

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

    // ── Global PATH block ────────────────────────────────────────────

    #[test]
    fn global_block_appends_fresh_block_when_absent() {
        let content = "# user stuff\nalias ll='ls -la'\n";
        let out = upsert_global_content(
            content,
            std::path::Path::new("/home/u/.ven/node/20.11.0/bin"),
            false,
        );
        assert!(out.contains(VEN_GLOBAL_BLOCK_START));
        assert!(out.contains(VEN_GLOBAL_BLOCK_END));
        assert!(out.contains("export PATH=\"/home/u/.ven/node/20.11.0/bin:$PATH\""));
        assert!(out.contains("alias ll='ls -la'"));
    }

    #[test]
    fn global_block_adds_second_entry_and_keeps_first() {
        let one = upsert_global_content(
            "",
            std::path::Path::new("/home/u/.ven/node/20.11.0/bin"),
            false,
        );
        let two = upsert_global_content(
            &one,
            std::path::Path::new("/home/u/.ven/rust/1.98.0/bin"),
            false,
        );
        assert_eq!(two.matches(VEN_GLOBAL_BLOCK_START).count(), 1);
        assert!(two.contains("node/20.11.0/bin"));
        assert!(two.contains("rust/1.98.0/bin"));
    }

    #[test]
    fn global_block_replaces_same_entry_without_duplicating() {
        let one = upsert_global_content(
            "",
            std::path::Path::new("/home/u/.ven/node/20.11.0/bin"),
            false,
        );
        let two = upsert_global_content(
            &one,
            std::path::Path::new("/home/u/.ven/node/22.0.0/bin"),
            false,
        );
        assert_eq!(
            two.matches("node/").count(),
            1,
            "old entry should be replaced: {two}"
        );
        assert!(two.contains("node/22.0.0/bin"));
        assert!(!two.contains("node/20.11.0/bin"));
    }

    #[test]
    fn global_block_removal_drops_line_and_keeps_others() {
        let one = upsert_global_content(
            "",
            std::path::Path::new("/home/u/.ven/node/20.11.0/bin"),
            false,
        );
        let two = upsert_global_content(
            &one,
            std::path::Path::new("/home/u/.ven/rust/1.98.0/bin"),
            false,
        );
        let out =
            remove_global_content(&two, std::path::Path::new("/home/u/.ven/node/20.11.0/bin"));
        assert!(!out.contains("node/20.11.0/bin"));
        assert!(out.contains("rust/1.98.0/bin"));
        assert!(out.contains(VEN_GLOBAL_BLOCK_START));
    }

    #[test]
    fn global_block_removal_of_last_entry_removes_block() {
        let one = upsert_global_content(
            "",
            std::path::Path::new("/home/u/.ven/node/20.11.0/bin"),
            false,
        );
        let out =
            remove_global_content(&one, std::path::Path::new("/home/u/.ven/node/20.11.0/bin"));
        assert!(!out.contains(VEN_GLOBAL_BLOCK_START));
        assert!(!out.contains("node/20.11.0/bin"));
    }

    #[test]
    fn global_block_parse_extracts_quoted_paths() {
        let one = upsert_global_content(
            "",
            std::path::Path::new("/home/u/.ven/node/20.11.0/bin"),
            false,
        );
        let two = upsert_global_content(
            &one,
            std::path::Path::new("/home/u/.ven/rust/1.98.0/bin"),
            false,
        );
        let start = two.find(VEN_GLOBAL_BLOCK_START).unwrap();
        let end_rel = two[start..].find(VEN_GLOBAL_BLOCK_END).unwrap();
        let block = &two[start..start + end_rel];
        let parsed = parse_global_block(block);
        assert_eq!(parsed.len(), 2);
        assert!(parsed.contains(&std::path::PathBuf::from("/home/u/.ven/node/20.11.0/bin")));
        assert!(parsed.contains(&std::path::PathBuf::from("/home/u/.ven/rust/1.98.0/bin")));
    }

    #[test]
    fn global_block_parse_handles_fish_syntax() {
        let one = upsert_global_content(
            "",
            std::path::Path::new("/home/u/.ven/python/3.12.7/bin"),
            true,
        );
        let start = one.find(VEN_GLOBAL_BLOCK_START).unwrap();
        let end_rel = one[start..].find(VEN_GLOBAL_BLOCK_END).unwrap();
        let block = &one[start..start + end_rel];
        let parsed = parse_global_block(block);
        assert_eq!(
            parsed,
            vec![std::path::PathBuf::from("/home/u/.ven/python/3.12.7/bin")]
        );
    }

    #[test]
    fn global_block_fish_rendering() {
        let line = render_global_line(std::path::Path::new("/home/u/.ven/python/3.12.7/bin"), true);
        assert_eq!(
            line,
            "set -gx PATH \"/home/u/.ven/python/3.12.7/bin\" $PATH"
        );
    }

    #[test]
    fn global_block_escapes_spaces() {
        let line = render_global_line(std::path::Path::new("/home/u/my ven/node/bin"), false);
        assert_eq!(line, "export PATH=\"/home/u/my ven/node/bin:$PATH\"");
    }
}
