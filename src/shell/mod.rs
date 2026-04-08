use anyhow::Result;
use std::path::Path;

use crate::core::{find_ven_toml, parse_ven_toml, resolve_node_version};
use crate::plugins::{LanguagePlugin, NodePlugin};

// ── Detect which shell is running ────────────────────────────────────
// On Windows: always PowerShell (we don't support cmd.exe)
// On Unix: read $SHELL env var

pub fn detect_shell() -> String {
    // On Windows, check if running in PowerShell
    #[cfg(target_os = "windows")]
    {
        // Check common PowerShell env vars
        if std::env::var("PSModulePath").is_ok() {
            return "powershell".to_string();
        }
        return "powershell".to_string(); // default to PowerShell on Windows
    }

    #[cfg(not(target_os = "windows"))]
    {
        let shell_path = std::env::var("SHELL").unwrap_or_default();
        std::path::Path::new(&shell_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bash")
            .to_string()
    }
}

// ── Generate hook script ─────────────────────────────────────────────
// Returns the shell code that activates ven on every cd

pub fn generate_hook(shell: &str) -> String {
    match shell {
        "bash" | "zsh"             => bash_zsh_hook(),
        "fish"                     => fish_hook(),
        "powershell" | "pwsh"      => powershell_hook(),
        other => format!("echo 'ven: unknown shell: {}' >&2\n", other),
    }
}

fn bash_zsh_hook() -> String {
    r#"
# ven shell hook
__ven_activate() {
    local exports
    exports=$(ven shell activate "$PWD" 2>/dev/null)
    if [ -n "$exports" ]; then
        eval "$exports"
    fi
}
cd() { builtin cd "$@" && __ven_activate; }
__ven_activate  # activate for current directory on shell start
"#
    .to_string()
}

fn fish_hook() -> String {
    r#"
# ven shell hook (fish)
function __ven_activate --on-variable PWD
    set exports (ven shell activate "$PWD" 2>/dev/null)
    if test -n "$exports"
        eval $exports
    end
end
__ven_activate  # activate on shell start
"#
    .to_string()
}

// FIXED: PowerShell hook — uses Invoke-Expression + $env:PATH syntax
fn powershell_hook() -> String {
    r#"
# ven shell hook (PowerShell)
function Set-VenLocation {
    param([string]$Path = "")
    if ($Path) {
        Set-Location $Path
    }
    $exports = ven shell activate "$PWD" 2>$null
    if ($exports) {
        Invoke-Expression $exports
    }
}
Set-Alias -Name cd -Value Set-VenLocation -Force -Option AllScope
# Activate for current directory on shell start
$_ven_exports = ven shell activate "$PWD" 2>$null
if ($_ven_exports) { Invoke-Expression $_ven_exports }
"#
    .to_string()
}

// ── Compute exports for a directory ─────────────────────────────────
// Called by: ven shell activate <dir>
// Reads ven.toml, resolves Node version, returns shell assignment text.
// The hook runs this text with eval (bash) or Invoke-Expression (PowerShell).

pub fn compute_exports(dir: &Path) -> Result<Option<String>> {
    // Find nearest ven.toml (walks up from dir)
    let toml_path = match find_ven_toml(dir) {
        Some(p) => p,
        None    => return Ok(None), // no ven.toml — print nothing
    };

    let config = parse_ven_toml(&toml_path)?;
    let node_spec = &config.runtime.node;

    // Resolve alias ("lts", "20") to installed concrete version ("20.11.0")
    let plugin = NodePlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = resolve_node_version(node_spec, &installed)?;

    // Get the bin/ path for this resolved version
    let bin_path = plugin.bin_path(&resolved)?;
    let bin_str = bin_path.display().to_string();

    // FIXED: output different syntax depending on platform
    // PowerShell uses $env:PATH = "...;" + $env:PATH
    // bash/zsh uses  export PATH="...:$PATH"
    let exports = if cfg!(target_os = "windows") {
        // PowerShell syntax — semicolon separator on Windows
        let mut out = format!(
            "$env:PATH = \"{bin};\" + $env:PATH\n$env:VEN_NODE_VERSION = \"{ver}\"\n$env:VEN_TOML = \"{toml}\"\n",
            bin  = bin_str,
            ver  = resolved,
            toml = toml_path.display(),
        );
        // Also export [env] section variables
        for (key, val) in &config.env {
            out.push_str(&format!("$env:{} = \"{}\"\n", key, val));
        }
        out
    } else {
        // bash/zsh/fish syntax — colon separator on Unix
        let mut out = format!(
            "export PATH=\"{bin}:$PATH\"\nexport VEN_NODE_VERSION=\"{ver}\"\nexport VEN_TOML=\"{toml}\"\n",
            bin  = bin_str,
            ver  = resolved,
            toml = toml_path.display(),
        );
        for (key, val) in &config.env {
            out.push_str(&format!("export {}=\"{}\"\n", key, val));
        }
        out
    };

    Ok(Some(exports))
}
