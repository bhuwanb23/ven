use anyhow::Result;
use std::path::Path;

use crate::core::{
    find_ven_toml, parse_ven_toml, project_venv, resolve_node_version, resolve_python_version,
};
use crate::plugins::{LanguagePlugin, NodePlugin, PythonPlugin};

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
__VEN_LAST_TOML_SIG=""
__VEN_BIN="{ven_path}"

__ven_toml_sig() {{
    local d="$1"
    while [ -n "$d" ]; do
        if [ -f "$d/ven.toml" ]; then
            local mt
            mt=$(stat -c %Y "$d/ven.toml" 2>/dev/null || stat -f %m "$d/ven.toml" 2>/dev/null || echo 0)
            printf '%s|%s\n' "$d/ven.toml" "$mt"
            return
        fi
        local parent
        parent=$(dirname "$d")
        if [ "$parent" = "$d" ]; then break; fi
        d="$parent"
    done
    echo ""
}}

ven-use() {{
    local d="${{1:-$PWD}}"
    local script
    script=$("$__VEN_BIN" shell activate "$d" 2>/dev/null) || true
    if [ -n "$script" ]; then eval "$script"; fi
}}

__ven_activate() {{
    local current_dir="$PWD"
    local sig
    sig=$(__ven_toml_sig "$current_dir")
    if [ "$__VEN_LAST_DIR" = "$current_dir" ] && [ "$__VEN_LAST_TOML_SIG" = "$sig" ]; then
        return
    fi
    __VEN_LAST_DIR="$current_dir"
    __VEN_LAST_TOML_SIG="$sig"

    local exports
    exports=$("$__VEN_BIN" shell activate "$current_dir" 2>/dev/null)

    if [ -n "$exports" ]; then
        eval "$exports"
    else
        export PATH="$__VEN_ORIGINAL_PATH"
        unset VEN_NODE_VERSION 2>/dev/null
        unset VEN_PYTHON_VERSION 2>/dev/null
        unset VEN_TOML 2>/dev/null
        unset VIRTUAL_ENV 2>/dev/null
    fi
}}

# Override cd command
cd() {{ builtin cd "$@" && __ven_activate; }}
__ven_activate  # activate for current directory on shell start
"#)
}

fn fish_hook() -> String {
    r#"
# ven shell hook (fish) — re-check on each prompt so new ven.toml in same dir is picked up
set -g __VEN_ORIGINAL_PATH $PATH
set -g __VEN_LAST_SIG ""

function __ven_fish_toml_sig --argument-names start
    set -l d $start
    while test -n "$d"
        set -l p "$d/ven.toml"
        if test -f $p
            set mt (stat -c %Y $p 2>/dev/null; or stat -f %m $p 2>/dev/null; or echo 0)
            echo "$p|$mt"
            return
        end
        set -l parent (dirname $d)
        if test "$parent" = "$d"
            break
        end
        set d $parent
    end
    echo ""
end

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

function __ven_on_prompt --on-event fish_prompt
    set -l sig (__ven_fish_toml_sig $PWD)
    if test "$sig" = "$__VEN_LAST_SIG"
        return
    end
    set -g __VEN_LAST_SIG $sig
    set exports (ven shell activate "$PWD" 2>/dev/null)
    if test -n "$exports"
        eval $exports
    else
        set -gx PATH $__VEN_ORIGINAL_PATH
        set -e VEN_NODE_VERSION 2>/dev/null
        set -e VEN_PYTHON_VERSION 2>/dev/null
        set -e VEN_TOML 2>/dev/null
        set -e VIRTUAL_ENV 2>/dev/null
    end
end
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
$global:VEN_LAST_TOML_SIG = $null
$global:VEN_LAST_ACTIVATE_WARN = $null

function global:__ven_toml_sig {{
    param([string]$StartDir)
    $d = $StartDir
    while ($true) {{
        if ([string]::IsNullOrEmpty($d)) {{ return "" }}
        $p = Join-Path $d 'ven.toml'
        if (Test-Path -LiteralPath $p) {{
            $i = Get-Item -LiteralPath $p
            return "$($i.FullName)|$($i.LastWriteTimeUtc.Ticks)"
        }}
        $parent = [System.IO.Path]::GetDirectoryName($d)
        if ([string]::IsNullOrEmpty($parent) -or $parent -eq $d) {{ break }}
        $d = $parent
    }}
    return ""
}}

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
    $sig = __ven_toml_sig $current_dir

    if ($global:VEN_LAST_DIR -eq $current_dir -and $global:VEN_LAST_TOML_SIG -eq $sig) {{ return }}
    $global:VEN_LAST_DIR = $current_dir
    $global:VEN_LAST_TOML_SIG = $sig

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
            if (Test-Path Env:VEN_PYTHON_VERSION) {{ Remove-Item Env:VEN_PYTHON_VERSION }}
            if (Test-Path Env:VEN_TOML) {{ Remove-Item Env:VEN_TOML }}
            if (Test-Path Env:VIRTUAL_ENV) {{ Remove-Item Env:VIRTUAL_ENV }}
            $key = "$current_dir|$exit"
            if ($global:VEN_LAST_ACTIVATE_WARN -ne $key) {{
                Write-Warning "ven: could not activate in `"$current_dir`" (exit $exit). Install the required Node version or fix ven.toml. Try: ven shell activate `"$current_dir`""
                $global:VEN_LAST_ACTIVATE_WARN = $key
            }}
        }} else {{
            $env:PATH = $global:VEN_ORIGINAL_PATH
            if (Test-Path Env:VEN_NODE_VERSION) {{ Remove-Item Env:VEN_NODE_VERSION }}
            if (Test-Path Env:VEN_PYTHON_VERSION) {{ Remove-Item Env:VEN_PYTHON_VERSION }}
            if (Test-Path Env:VEN_TOML) {{ Remove-Item Env:VEN_TOML }}
            if (Test-Path Env:VIRTUAL_ENV) {{ Remove-Item Env:VIRTUAL_ENV }}
        }}
    }} catch {{
        $env:PATH = $global:VEN_ORIGINAL_PATH
        if (Test-Path Env:VEN_NODE_VERSION) {{ Remove-Item Env:VEN_NODE_VERSION }}
        if (Test-Path Env:VEN_PYTHON_VERSION) {{ Remove-Item Env:VEN_PYTHON_VERSION }}
        if (Test-Path Env:VEN_TOML) {{ Remove-Item Env:VEN_TOML }}
        if (Test-Path Env:VIRTUAL_ENV) {{ Remove-Item Env:VIRTUAL_ENV }}
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
    NoToml,
    Success(String),
    /// Runtime named in ven.toml is not installed under ~/.ven yet
    MissingToolchain {
        language: String,
        install_with: String,
    },
}

fn path_for_env_value(p: &Path) -> String {
    let s = p.display().to_string();
    if cfg!(target_os = "windows") {
        let s = if s.starts_with("\\\\?\\") {
            s[4..].to_string()
        } else {
            s
        };
        s.replace('/', "\\")
    } else {
        s.replace('\\', "/")
    }
}

pub fn try_compute_exports(dir: &Path) -> Result<ComputeExportsOutcome> {
    let absolute_dir = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(dir))
            .unwrap_or_else(|_| dir.to_path_buf())
    };

    let toml_path = match find_ven_toml(&absolute_dir) {
        Some(p) => p,
        None => return Ok(ComputeExportsOutcome::NoToml),
    };

    let toml_canonical = std::fs::canonicalize(&toml_path).unwrap_or_else(|_| {
        if toml_path.is_absolute() {
            toml_path
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&toml_path))
                .unwrap_or_else(|_| toml_path)
        }
    });

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
    let python_spec = config.runtime.python.trim();

    if node_spec.is_empty() && python_spec.is_empty() {
        anyhow::bail!("ven.toml [runtime]: set `node` and/or `python`");
    }

    let project_root = toml_canonical
        .parent()
        .ok_or_else(|| anyhow::anyhow!("ven.toml path has no parent directory"))?;

    let mut prepend_dirs: Vec<std::path::PathBuf> = Vec::new();
    let mut node_resolved: Option<String> = None;
    let mut node_bin_for_path: Option<std::path::PathBuf> = None;
    let mut python_resolved: Option<String> = None;
    let mut virtual_env_root: Option<std::path::PathBuf> = None;

    if !python_spec.is_empty() {
        if let Some(venv_bin) = project_venv::local_venv_bin_dir(project_root) {
            prepend_dirs.push(venv_bin);
            let venv_dir = project_root.join(".venv");
            virtual_env_root = Some(venv_dir.clone());
            python_resolved = Some(
                project_venv::local_venv_python_version(&venv_dir)
                    .or_else(|| {
                        let installed = PythonPlugin.list_installed().unwrap_or_default();
                        resolve_python_version(python_spec, &installed).ok()
                    })
                    .unwrap_or_else(|| python_spec.to_string()),
            );
        } else {
            #[cfg(target_os = "windows")]
            {
                let plugin = PythonPlugin;
                let installed = plugin.list_installed().unwrap_or_default();
                let resolved = match resolve_python_version(python_spec, &installed) {
                    Ok(v) => v,
                    Err(_) => {
                        return Ok(ComputeExportsOutcome::MissingToolchain {
                            language: "python".into(),
                            install_with: python_spec.to_string(),
                        });
                    }
                };
                let bin = match plugin.bin_path(&resolved) {
                    Ok(p) => p,
                    Err(_) => {
                        return Ok(ComputeExportsOutcome::MissingToolchain {
                            language: "python".into(),
                            install_with: resolved.clone(),
                        });
                    }
                };
                prepend_dirs.push(bin);
                python_resolved = Some(resolved);
            }
            #[cfg(not(target_os = "windows"))]
            {
                if node_spec.is_empty() {
                    anyhow::bail!(
                        "ven.toml sets `runtime.python` but there is no `.venv` under {}.\n\
                         Create it with:  python3 -m venv .venv\n\
                         On Windows, `ven init` for a Python project creates `.venv` when your ven Python is installed.",
                        project_root.display()
                    );
                }
                // Node still activates; Python takes effect once `.venv` exists.
            }
        }
    }

    if !node_spec.is_empty() {
        let plugin = NodePlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        let resolved = match resolve_node_version(node_spec, &installed) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ComputeExportsOutcome::MissingToolchain {
                    language: "node".into(),
                    install_with: node_spec.to_string(),
                });
            }
        };
        let bin = match plugin.bin_path(&resolved) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ComputeExportsOutcome::MissingToolchain {
                    language: "node".into(),
                    install_with: resolved.clone(),
                });
            }
        };
        node_bin_for_path = Some(bin.clone());
        prepend_dirs.push(bin);
        node_resolved = Some(resolved);
    }

    let toml_normalized = if cfg!(target_os = "windows") {
        toml_absolute.replace('/', "\\")
    } else {
        toml_absolute.replace('\\', "/")
    };

    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    let path_joined = prepend_dirs
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(sep);

    let exports = if cfg!(target_os = "windows") {
        let mut out = format!(
            "$env:PATH = \"{pj};\" + $env:PATH\n",
            pj = path_joined
        );
        if let Some(ref dir) = node_bin_for_path {
            out.push_str(&format!("$env:NODE_PATH = \"{}\"\n", dir.display()));
        }
        if let Some(ref v) = node_resolved {
            out.push_str(&format!("$env:VEN_NODE_VERSION = \"{}\"\n", v));
        }
        if let Some(ref v) = python_resolved {
            out.push_str(&format!("$env:VEN_PYTHON_VERSION = \"{}\"\n", v));
        }
        if let Some(ref vr) = virtual_env_root {
            out.push_str(&format!(
                "$env:VIRTUAL_ENV = \"{}\"\n",
                path_for_env_value(vr)
            ));
        }
        out.push_str(&format!("$env:VEN_TOML = \"{}\"\n", toml_normalized));
        for (key, val) in &config.env {
            out.push_str(&format!("$env:{} = \"{}\"\n", key, val));
        }
        out
    } else {
        let mut out = format!("export PATH=\"{pj}:$PATH\"\n", pj = path_joined);
        if let Some(ref dir) = node_bin_for_path {
            out.push_str(&format!("export NODE_PATH=\"{}\"\n", dir.display()));
        }
        if let Some(ref v) = node_resolved {
            out.push_str(&format!("export VEN_NODE_VERSION=\"{}\"\n", v));
        }
        if let Some(ref v) = python_resolved {
            out.push_str(&format!("export VEN_PYTHON_VERSION=\"{}\"\n", v));
        }
        if let Some(ref vr) = virtual_env_root {
            out.push_str(&format!(
                "export VIRTUAL_ENV=\"{}\"\n",
                path_for_env_value(vr)
            ));
        }
        out.push_str(&format!("export VEN_TOML=\"{}\"\n", toml_normalized));
        for (key, val) in &config.env {
            out.push_str(&format!("export {}=\"{}\"\n", key, val));
        }
        out
    };

    Ok(ComputeExportsOutcome::Success(exports))
}
