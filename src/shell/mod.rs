use anyhow::Result;
use std::path::Path;

mod activation;

pub use activation::{
    activation_path_overlay, path_for_env_value, resolve_activation_environment, ActivationParts,
    ActivationResolve,
};

/// Canonical marker line that every `ven` shell hook prepends to its block.
///
/// Both `ven setup` (Windows) and `ven shell install` (PowerShell + bash) use
/// this exact string as the "already installed?" sentinel so the installer
/// doesn't append a *second* copy on re-run. Changing this string is a
/// breaking change for users with a prior install — the new install code
/// will then not detect the old block and may append a duplicate.
pub const HOOK_MARKER: &str = "# ven-managed-hook-v2";

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
        "bash" | "zsh" => bash_zsh_hook(),
        "fish" => fish_hook(),
        "powershell" | "pwsh" => powershell_hook(),
        other => format!("echo 'ven: unknown shell: {}' >&2\n", other),
    }
}

fn bash_zsh_hook() -> String {
    // Get the current executable path
    let ven_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "ven".to_string());

    format!(
        r#"
{HOOK_MARKER}
# ven shell hook (bash/zsh) - Auto-switches runtimes on cd
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
    unset VEN_SKIP_PROJECT_VENV 2>/dev/null || true
    local d="${{1:-$PWD}}"
    local script
    script=$("$__VEN_BIN" shell activate "$d" 2>/dev/null) || true
    if [ -n "$script" ]; then eval "$script"; fi
}}

__ven_activate() {{
    local current_dir="$PWD"
    local sig
    sig=$(__ven_toml_sig "$current_dir")
    if [ -n "$__VEN_LAST_DIR" ] && [ "$current_dir" != "$__VEN_LAST_DIR" ]; then
        unset VEN_SKIP_PROJECT_VENV 2>/dev/null || true
    fi
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
        # Only clear vars ven itself sets on activation. Non-ven vars
        # (JAVA_HOME, GOROOT, GOPATH, CARGO_HOME, RUSTUP_HOME, GEM_HOME,
        # GEM_PATH) are *only ever set* by ven, never clobbered by the
        # hook on transition out — the user may have set them by hand
        # before cd'ing into a ven project, and silently unsetting them
        # was a footgun. Use `ven shell deactivate` to fully clean.
        unset VEN_NODE_VERSION 2>/dev/null
        unset VEN_PYTHON_VERSION 2>/dev/null
        unset VEN_GO_VERSION 2>/dev/null
        unset VEN_RUST_VERSION 2>/dev/null
        unset VEN_JAVA_VERSION 2>/dev/null
        unset VEN_DENO_VERSION 2>/dev/null
        unset VEN_BUN_VERSION 2>/dev/null
        unset VEN_RUBY_VERSION 2>/dev/null
        unset VEN_TOML 2>/dev/null
        unset VIRTUAL_ENV 2>/dev/null
        unset NODE_PATH 2>/dev/null
        unset VEN_SKIP_PROJECT_VENV 2>/dev/null || true
    fi
}}

# Override cd command
cd() {{ builtin cd "$@" && __ven_activate; }}
__ven_activate  # activate for current directory on shell start
"#
    )
}

fn fish_hook() -> String {
    format!(
        r#"
{HOOK_MARKER}
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
    set -q VEN_SKIP_PROJECT_VENV; and set -e VEN_SKIP_PROJECT_VENV
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
    if set -q __ven_prev_pwd_ven_hook
        if test "$PWD" != "$__ven_prev_pwd_ven_hook"
            set -q VEN_SKIP_PROJECT_VENV; and set -e VEN_SKIP_PROJECT_VENV
        end
    end
    set -g __ven_prev_pwd_ven_hook $PWD
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
        # Only clear vars ven itself sets on activation. Non-ven vars
        # (JAVA_HOME, GOROOT, etc.) are never clobbered by the hook on
        # transition out — see the matching note in the bash hook.
        set -e VEN_NODE_VERSION 2>/dev/null
        set -e VEN_PYTHON_VERSION 2>/dev/null
        set -e VEN_GO_VERSION 2>/dev/null
        set -e VEN_RUST_VERSION 2>/dev/null
        set -e VEN_JAVA_VERSION 2>/dev/null
        set -e VEN_DENO_VERSION 2>/dev/null
        set -e VEN_BUN_VERSION 2>/dev/null
        set -e VEN_RUBY_VERSION 2>/dev/null
        set -e VEN_TOML 2>/dev/null
        set -e VIRTUAL_ENV 2>/dev/null
        set -q VEN_SKIP_PROJECT_VENV; and set -e VEN_SKIP_PROJECT_VENV
    end
end
"#
    )
}

// PowerShell hook — Invoke-Expression + $env:PATH; Set-Location + prompt so
// `cd`, `Set-Location`, and Cursor/pwsh hosts all pick up directory changes.
fn powershell_hook() -> String {
    // Get the current executable path
    let ven_path = std::env::current_exe()
        .map(|p| p.display().to_string().replace('/', "\\"))
        .unwrap_or_else(|_| "ven".to_string());

    format!(
        r#"
{HOOK_MARKER}
# ven shell hook (PowerShell) - Auto-switches runtimes on cd / Set-Location
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
        if (Test-Path Env:VEN_SKIP_PROJECT_VENV) {{ Remove-Item Env:VEN_SKIP_PROJECT_VENV }}
        $lines = & $global:VEN_BIN shell activate $Directory 2>$null
        $script = if ($null -eq $lines) {{ '' }} else {{ [string]::Join([Environment]::NewLine, @($lines)) }}
        if ($script) {{ Invoke-Expression $script }}
    }}
}}

function global:__ven_activate {{
    $current_dir = $PWD.Path
    $sig = __ven_toml_sig $current_dir

    if ($null -ne $global:VEN_LAST_DIR -and $current_dir -ne $global:VEN_LAST_DIR) {{
        if (Test-Path Env:VEN_SKIP_PROJECT_VENV) {{ Remove-Item Env:VEN_SKIP_PROJECT_VENV }}
    }}

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
            __ven_clear_ven_state
            $key = "$current_dir|$exit"
            if ($global:VEN_LAST_ACTIVATE_WARN -ne $key) {{
                Write-Warning "ven: could not activate in `"$current_dir`" (exit $exit). Install required runtimes or fix ven.toml. Try: ven shell activate `"$current_dir`""
                $global:VEN_LAST_ACTIVATE_WARN = $key
            }}
        }} else {{
            __ven_clear_ven_state
        }}
    }} catch {{
        __ven_clear_ven_state
    }}
}}

# Restore the shell to its pre-ven state. Only clears vars ven itself owns
# (VEN_* and VIRTUAL_ENV). Non-ven vars (JAVA_HOME, GOROOT, GOPATH, etc.)
# are NEVER touched here — they may have been set by the user in their
# profile before the hook first fired, and clobbering them on cd-out was a
# footgun. Use `ven shell deactivate` for a full clear.
function global:__ven_clear_ven_state {{
    $env:PATH = $global:VEN_ORIGINAL_PATH
    if (Test-Path Env:VEN_NODE_VERSION)    {{ Remove-Item Env:VEN_NODE_VERSION }}
    if (Test-Path Env:VEN_PYTHON_VERSION) {{ Remove-Item Env:VEN_PYTHON_VERSION }}
    if (Test-Path Env:VEN_GO_VERSION)      {{ Remove-Item Env:VEN_GO_VERSION }}
    if (Test-Path Env:VEN_RUST_VERSION)   {{ Remove-Item Env:VEN_RUST_VERSION }}
    if (Test-Path Env:VEN_JAVA_VERSION)   {{ Remove-Item Env:VEN_JAVA_VERSION }}
    if (Test-Path Env:VEN_DENO_VERSION)   {{ Remove-Item Env:VEN_DENO_VERSION }}
    if (Test-Path Env:VEN_BUN_VERSION)     {{ Remove-Item Env:VEN_BUN_VERSION }}
    if (Test-Path Env:VEN_RUBY_VERSION)   {{ Remove-Item Env:VEN_RUBY_VERSION }}
    if (Test-Path Env:VEN_TOML)            {{ Remove-Item Env:VEN_TOML }}
    if (Test-Path Env:VIRTUAL_ENV)         {{ Remove-Item Env:VIRTUAL_ENV }}
    if (Test-Path Env:NODE_PATH)           {{ Remove-Item Env:NODE_PATH }}
    if (Test-Path Env:VEN_SKIP_PROJECT_VENV) {{ Remove-Item Env:VEN_SKIP_PROJECT_VENV }}
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
"#
    )
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
// Reads ven.toml, resolves runtime versions, returns shell assignment text.
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

pub fn try_compute_exports(dir: &Path) -> Result<ComputeExportsOutcome> {
    match activation::resolve_activation_environment(dir)? {
        ActivationResolve::NoToml => Ok(ComputeExportsOutcome::NoToml),
        ActivationResolve::MissingToolchain {
            language,
            install_with,
        } => Ok(ComputeExportsOutcome::MissingToolchain {
            language,
            install_with,
        }),
        ActivationResolve::Ready(parts) => Ok(ComputeExportsOutcome::Success(
            activation::format_activation_shell_script(&parts),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hook-marker must be a single source of truth. Changing it must break
    /// this test so we don't re-introduce duplicate installs.
    #[test]
    fn hook_marker_is_canonical() {
        assert_eq!(HOOK_MARKER, "# ven-managed-hook-v2");
        assert!(HOOK_MARKER.starts_with("# "));
        assert!(!HOOK_MARKER.contains('\n'));
    }

    /// The bash hook must embed the canonical marker so that running
    /// `ven setup` on Unix is idempotent.
    #[test]
    fn bash_hook_embeds_marker() {
        let hook = generate_hook("bash");
        assert!(
            hook.contains(HOOK_MARKER),
            "bash hook missing canonical marker"
        );
    }

    /// The PowerShell hook must embed the canonical marker.
    #[test]
    fn powershell_hook_embeds_marker() {
        let hook = generate_hook("powershell");
        assert!(
            hook.contains(HOOK_MARKER),
            "powershell hook missing canonical marker"
        );
    }

    /// The fish hook must embed the canonical marker.
    #[test]
    fn fish_hook_embeds_marker() {
        let hook = generate_hook("fish");
        assert!(
            hook.contains(HOOK_MARKER),
            "fish hook missing canonical marker"
        );
    }
}
