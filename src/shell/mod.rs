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
    // Get the current executable path
    let ven_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "ven".to_string());
    
    format!(r#"
# ven shell hook (bash/zsh) - Auto-switches Node.js on cd
__VEN_ORIGINAL_PATH="$PATH"
__VEN_LAST_DIR=""
__VEN_BIN="{ven_path}"

__ven_activate() {{
    local current_dir="$PWD"
    
    # Only re-activate if directory changed
    if [ "$__VEN_LAST_DIR" != "$current_dir" ]; then
        __VEN_LAST_DIR="$current_dir"
        
        # Try to find and activate ven.toml
        local exports
        exports=$("$__VEN_BIN" shell activate "$current_dir" 2>/dev/null)
        
        if [ -n "$exports" ]; then
            # ven.toml found - activate it
            eval "$exports"
        else
            # No ven.toml - restore original PATH
            export PATH="$__VEN_ORIGINAL_PATH"
            unset VEN_NODE_VERSION 2>/dev/null
            unset VEN_TOML 2>/dev/null
        fi
    fi
}}

# Override cd command
cd() {{ builtin cd "$@" && __ven_activate; }}
__ven_activate  # activate for current directory on shell start
"#)
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
// Monitors directory changes and automatically switches Node versions
fn powershell_hook() -> String {
    // Get the current executable path
    let ven_path = std::env::current_exe()
        .map(|p| p.display().to_string().replace('/', "\\"))
        .unwrap_or_else(|_| "ven".to_string());
    
    format!(r#"
# ven shell hook (PowerShell) - Auto-switches Node.js on cd
$script:VEN_LAST_PATH = $null
$script:VEN_ORIGINAL_PATH = $env:PATH
$script:VEN_BIN = "{ven_path}"

function Set-VenLocation {{
    param([string]$Path = "")
    
    # Change directory
    if ($Path) {{
        Set-Location $Path
    }}
    
    $current_dir = $PWD.Path
    
    # Only re-activate if directory changed
    if ($script:VEN_LAST_PATH -ne $current_dir) {{
        $script:VEN_LAST_PATH = $current_dir
        
        # Try to find and activate ven.toml
        $exports = & $script:VEN_BIN shell activate "$current_dir" 2>$null
        
        if ($exports) {{
            # ven.toml found - activate it
            Invoke-Expression $exports
        }} else {{
            # No ven.toml - restore original PATH (remove ven paths)
            $env:PATH = $script:VEN_ORIGINAL_PATH
            Remove-Item Env:VEN_NODE_VERSION -ErrorAction SilentlyContinue
            Remove-Item Env:VEN_TOML -ErrorAction SilentlyContinue
        }}
    }}
}}

# Override cd command
Set-Alias -Name cd -Value Set-VenLocation -Force -Option AllScope

# Activate for current directory on shell start
Set-VenLocation
"#)
}

// ── Compute exports for a directory ─────────────────────────────────
// Called by: ven shell activate <dir>
// Reads ven.toml, resolves Node version, returns shell assignment text.
// The hook runs this text with eval (bash) or Invoke-Expression (PowerShell).

pub fn compute_exports(dir: &Path) -> Result<Option<String>> {
    // Make directory absolute first
    let absolute_dir = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(dir))
            .unwrap_or_else(|_| dir.to_path_buf())
    };
    
    // Try to find ven.toml starting from this directory
    let toml_path = match find_ven_toml(&absolute_dir) {
        Some(p) => p,
        None    => return Ok(None), // no ven.toml — print nothing
    };
    
    // Canonicalize the toml path to get clean absolute path (resolves . and ..)
    let toml_canonical = std::fs::canonicalize(&toml_path)
        .unwrap_or_else(|_| {
            // If canonicalize fails, use the path as-is
            if toml_path.is_absolute() {
                toml_path
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(&toml_path))
                    .unwrap_or_else(|_| toml_path)
            }
        });
    
    // On Windows, canonicalize adds \\?\ prefix which we need to strip
    let toml_str = toml_canonical.display().to_string();
    let toml_absolute = if cfg!(target_os = "windows") {
        // Strip \\?\ prefix if present
        if toml_str.starts_with("\\\\?\\") {
            toml_str[4..].to_string()
        } else {
            toml_str
        }
    } else {
        toml_str
    };

    let config = parse_ven_toml(std::path::Path::new(&toml_absolute))?;
    let node_spec = &config.runtime.node;

    // Resolve alias ("lts", "20") to installed concrete version ("20.11.0")
    let plugin = NodePlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = match resolve_node_version(node_spec, &installed) {
        Ok(version) => version,
        Err(_) => {
            // Check if any versions are installed at all
            if installed.is_empty() {
                anyhow::bail!(
                    "No Node.js versions installed.\n\nInstall: ven install node {}", 
                    node_spec
                );
            } else {
                anyhow::bail!(
                    "Node.js {} required but not installed.\n\nInstall: ven install node {}", 
                    node_spec,
                    node_spec
                );
            }
        }
    };

    // Get the bin/ path for this resolved version
    let bin_path = plugin.bin_path(&resolved)?;
    let bin_str = bin_path.display().to_string();
    
    // Normalize slashes for the platform
    let toml_normalized = if cfg!(target_os = "windows") {
        // Windows: ensure backslashes (already correct from canonicalize)
        toml_absolute.replace('/', "\\")
    } else {
        // Unix: ensure forward slashes
        toml_absolute.replace('\\', "/")
    };

    // FIXED: output different syntax depending on platform
    // PowerShell uses $env:PATH = "...;" + $env:PATH
    // bash/zsh uses  export PATH="...:$PATH"
    let exports = if cfg!(target_os = "windows") {
        // PowerShell syntax — semicolon separator on Windows
        let mut out = format!(
            "$env:PATH = \"{bin};\" + $env:PATH\n$env:VEN_NODE_VERSION = \"{ver}\"\n$env:VEN_TOML = \"{toml}\"\n",
            bin  = bin_str,
            ver  = resolved,
            toml = toml_normalized,
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
            toml = toml_normalized,
        );
        for (key, val) in &config.env {
            out.push_str(&format!("export {}=\"{}\"\n", key, val));
        }
        out
    };

    Ok(Some(exports))
}
