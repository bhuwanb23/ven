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

ven-use() {{
    local d="${{1:-$PWD}}"
    local script
    script=$("$__VEN_BIN" shell activate "$d" 2>/dev/null) || true
    if [ -n "$script" ]; then eval "$script"; fi
}}

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
function ven-use
    set -l d $argv[1]
    if test -z "$d"
        set d $PWD
    end
    set -l script (ven shell activate "$d" 2>/dev/null)
    if test -n "$script"
        eval $script
    end
end

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

// PowerShell hook — Invoke-Expression + $env:PATH; Set-Location + prompt so
// `cd`, `Set-Location`, and Cursor/pwsh hosts all pick up directory changes.
fn powershell_hook() -> String {
    // Get the current executable path
    let ven_path = std::env::current_exe()
        .map(|p| p.display().to_string().replace('/', "\\"))
        .unwrap_or_else(|_| "ven".to_string());
    
    format!(r#"
# ven shell hook (PowerShell) - Auto-switches Node.js on cd / Set-Location
if (-not $global:VEN_ORIGINAL_PATH) {{
    $global:VEN_ORIGINAL_PATH = $env:PATH
}}
$global:VEN_BIN = "{ven_path}"
$global:VEN_LAST_DIR = $null
$global:VEN_LAST_ACTIVATE_WARN = $null

# Manual apply (ven shell activate only prints; this runs those lines in-process)
if (-not $global:__ven_use_defined) {{
    $global:__ven_use_defined = $true
    function global:ven-use {{
        param([string]$Directory = $PWD.Path)
        $lines = & $global:VEN_BIN shell activate $Directory 2>$null
        $script = if ($null -eq $lines) {{ '' }} else {{ [string]::Join([Environment]::NewLine, @($lines)) }}
        if ($script) {{ Invoke-Expression $script }}
    }}
}}

function global:__ven_activate {{
    $current_dir = $PWD.Path

    if ($global:VEN_LAST_DIR -eq $current_dir) {{ return }}
    $global:VEN_LAST_DIR = $current_dir

    try {{
        $lines = & $global:VEN_BIN shell activate $current_dir 2>$null
        $exit = $LASTEXITCODE
        $script = if ($null -eq $lines) {{ '' }} else {{ [string]::Join([Environment]::NewLine, @($lines)) }}

        # Prefer stdout over $LASTEXITCODE (PS can leave a stale exit code between prompts)
        if ($script) {{
            Invoke-Expression $script
            $global:VEN_LAST_ACTIVATE_WARN = $null
        }} elseif ($exit -ne 0) {{
            $env:PATH = $global:VEN_ORIGINAL_PATH
            if (Test-Path Env:VEN_NODE_VERSION) {{ Remove-Item Env:VEN_NODE_VERSION }}
            if (Test-Path Env:VEN_TOML) {{ Remove-Item Env:VEN_TOML }}
            $key = "$current_dir|$exit"
            if ($global:VEN_LAST_ACTIVATE_WARN -ne $key) {{
                Write-Warning "ven: could not activate in `"$current_dir`" (exit $exit). Install the required Node version or fix ven.toml. Try: ven shell activate `"$current_dir`""
                $global:VEN_LAST_ACTIVATE_WARN = $key
            }}
        }} else {{
            $env:PATH = $global:VEN_ORIGINAL_PATH
            if (Test-Path Env:VEN_NODE_VERSION) {{ Remove-Item Env:VEN_NODE_VERSION }}
            if (Test-Path Env:VEN_TOML) {{ Remove-Item Env:VEN_TOML }}
        }}
    }} catch {{
        $env:PATH = $global:VEN_ORIGINAL_PATH
    }}
}}

# Wrap Set-Location so `cd` / Set-Location always re-run activation (prompt alone misses some hosts)
if (-not $global:__ven_set_location_wrapped) {{
    $global:__ven_set_location_wrapped = $true
    function global:Set-Location {{
        [CmdletBinding(DefaultParameterSetName='Path', SupportsShouldProcess=$true)]
        param(
            [Parameter(ParameterSetName='Path', Position=0, ValueFromPipeline=$true, ValueFromPipelineByPropertyName=$true)]
            [string] $Path,
            [Parameter(ParameterSetName='LiteralPath', Mandatory=$true, ValueFromPipelineByPropertyName=$true)]
            [string] $LiteralPath,
            [Parameter(ParameterSetName='StackName')]
            [string] $StackName,
            [switch] $PassThru,
            [Parameter(ValueFromPipeline=$true)]
            [psobject] $InputObject
        )
        Microsoft.PowerShell.Management\Set-Location @PSBoundParameters
        __ven_activate
    }}
}}

if (-not $global:__ven_prompt_hooked) {{
    $global:__ven_prompt_hooked = $true
    $global:__ven_old_prompt = ${{function:prompt}}

    function global:prompt {{
        __ven_activate
        if ($global:__ven_old_prompt) {{
            & $global:__ven_old_prompt
        }} else {{
            "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
        }}
    }}
}}

__ven_activate
"#)
}

/// Windows profile paths where we install the hook (pwsh, VS Code/Cursor host, Windows PowerShell 5.1).
pub fn windows_powershell_profile_paths(home: &std::path::Path) -> Vec<std::path::PathBuf> {
    vec![
        home.join("Documents")
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
        home.join("Documents")
            .join("PowerShell")
            .join("Microsoft.VSCode_profile.ps1"),
        home.join("Documents")
            .join("WindowsPowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    ]
}

// ── Compute exports for a directory ─────────────────────────────────
// Called by: ven shell activate <dir>
// Reads ven.toml, resolves Node version, returns shell assignment text.
// The hook runs this text with eval (bash) or Invoke-Expression (PowerShell).

#[derive(Debug)]
pub enum ComputeExportsOutcome {
    /// No ven.toml in this directory tree
    NoToml,
    /// PATH/env script ready for the shell
    Success(String),
    /// Resolved version / spec is not installed under ~/.ven yet
    MissingNode { install_with: String },
}

pub fn try_compute_exports(dir: &Path) -> Result<ComputeExportsOutcome> {
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
        None => return Ok(ComputeExportsOutcome::NoToml),
    };

    // Canonicalize the toml path to get clean absolute path (resolves . and ..)
    let toml_canonical = std::fs::canonicalize(&toml_path).unwrap_or_else(|_| {
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
        if toml_str.starts_with("\\\\?\\") {
            toml_str[4..].to_string()
        } else {
            toml_str
        }
    } else {
        toml_str
    };

    let config = parse_ven_toml(std::path::Path::new(&toml_absolute))?;
    let node_spec = config.runtime.node.trim();
    if node_spec.is_empty() {
        anyhow::bail!("ven.toml [runtime].node is empty; set a Node.js version.");
    }

    let plugin = NodePlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = match resolve_node_version(node_spec, &installed) {
        Ok(version) => version,
        Err(_) => {
            return Ok(ComputeExportsOutcome::MissingNode {
                install_with: node_spec.to_string(),
            });
        }
    };

    let bin_path = match plugin.bin_path(&resolved) {
        Ok(p) => p,
        Err(_) => {
            return Ok(ComputeExportsOutcome::MissingNode {
                install_with: resolved,
            });
        }
    };

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
            "$env:PATH = \"{bin};\" + $env:PATH\n$env:NODE_PATH = \"{bin}\"\n$env:VEN_NODE_VERSION = \"{ver}\"\n$env:VEN_TOML = \"{toml}\"\n",
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
            "export PATH=\"{bin}:$PATH\"\nexport NODE_PATH=\"{bin}\"\nexport VEN_NODE_VERSION=\"{ver}\"\nexport VEN_TOML=\"{toml}\"\n",
            bin  = bin_str,
            ver  = resolved,
            toml = toml_normalized,
        );
        for (key, val) in &config.env {
            out.push_str(&format!("export {}=\"{}\"\n", key, val));
        }
        out
    };

    Ok(ComputeExportsOutcome::Success(exports))
}
