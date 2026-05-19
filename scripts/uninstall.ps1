# SPDX-License-Identifier: MIT
# scripts/uninstall.ps1 — canonical PowerShell fallback for `ven uninstall`.
#
# ## What this does
#
# Same teardown the native `ven uninstall` command performs, written in pure
# PowerShell so it stays useful in three escape-hatch scenarios:
#
#   1. The `ven` binary is broken / missing from PATH and can no longer
#      self-execute its own uninstall.
#   2. Sysadmins want to script the teardown without delegating to a
#      binary they need to first install.
#   3. CI matrices that don't have a recent ven on PATH but still need to
#      converge to a clean state between runs.
#
# This script is the SYNCED twin of `scripts/uninstall.sh` and of the Rust
# implementation in `src/core/uninstaller.rs`. If you edit one, audit the
# others; the website's UNINSTALL.advanced.windows snippet in
# `ven_website/src/content/site.js` is generated from the same shape.
#
# ## Idempotent
#
# Every step is wrapped in an existence check, so re-running this script
# after a partial uninstall just converges to the clean state. Use that
# property freely.
#
# ## Usage
#
#   irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/uninstall.ps1 | iex
#   # ── OR after a successful install ──
#   & "$env:USERPROFILE\.ven\bin\ven-uninstall.ps1"
#
# Flags (env vars; PowerShell `-` flags are awkward to pass through `iex`):
#   $env:VEN_UNINSTALL_USER_ONLY   = '1'   # skip the system layer
#   $env:VEN_UNINSTALL_SYSTEM_ONLY = '1'   # skip the user layer (admin scope)
#   $env:VEN_UNINSTALL_DRY_RUN     = '1'   # print plan, change nothing

$ErrorActionPreference = 'Continue'

# ── Helpers ─────────────────────────────────────────────────────────────────

function _Norm([string]$s) {
    if ([string]::IsNullOrWhiteSpace($s)) { return '' }
    return ([Environment]::ExpandEnvironmentVariables($s.Trim())).TrimEnd('\','/').ToLowerInvariant()
}

# Strip every PATH entry whose normalised form matches one of $targets in the
# given scope (User | Machine). Returns the array of stripped entries.
function _StripPath([string]$scope, [string[]]$targets) {
    $current = [Environment]::GetEnvironmentVariable('Path', $scope)
    if (-not $current) { return @() }
    $targetSet = @{}
    foreach ($t in $targets) { $targetSet[(_Norm $t)] = $true }
    $kept = @()
    $removed = @()
    foreach ($e in ($current -split ';')) {
        if ([string]::IsNullOrWhiteSpace($e)) { continue }
        if ($targetSet.ContainsKey((_Norm $e))) { $removed += $e } else { $kept += $e }
    }
    if ($removed.Count -gt 0 -and -not $script:DryRun) {
        try { [Environment]::SetEnvironmentVariable('Path', ($kept -join ';'), $scope) }
        catch {
            Write-Warning ("Could not update $scope PATH: " + $_.Exception.Message)
            return @()
        }
    }
    return $removed
}

# Strip a single fenced `# >>> name >>> ... # <<< name <<<` block from a file.
# Returns $true if the file changed.
function _StripBlock([string]$path, [string]$name) {
    if (-not (Test-Path $path)) { return $false }
    $content = Get-Content -Raw -Path $path -ErrorAction SilentlyContinue
    if (-not $content) { return $false }
    $startMarker = "# >>> $name >>>"
    $endMarker   = "# <<< $name <<<"
    if (-not $content.Contains($startMarker)) { return $false }
    $pattern = [regex]::Escape($startMarker) + '[\s\S]*?' + [regex]::Escape($endMarker) + '\r?\n?'
    $stripped = [regex]::Replace($content, $pattern, '')
    if ($stripped -ne $content) {
        if (-not $script:DryRun) { Set-Content -Path $path -Value $stripped -NoNewline }
        return $true
    }
    return $false
}

function _IsAdmin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    return ([Security.Principal.WindowsPrincipal]$id).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# ── Scope from env (so this can be piped through `iex` with knobs) ───────────

$script:DryRun     = ($env:VEN_UNINSTALL_DRY_RUN -eq '1')
$script:UserOnly   = ($env:VEN_UNINSTALL_USER_ONLY -eq '1')
$script:SystemOnly = ($env:VEN_UNINSTALL_SYSTEM_ONLY -eq '1')

if ($script:UserOnly -and $script:SystemOnly) {
    throw 'VEN_UNINSTALL_USER_ONLY and VEN_UNINSTALL_SYSTEM_ONLY are mutually exclusive.'
}

$tag = if ($script:DryRun) { '[DRY-RUN] ' } else { '' }
Write-Host ("${tag}ven uninstall (PowerShell fallback script)")
if ($script:DryRun) { Write-Host '  [i] Nothing will be removed; this is a plan-only run.' }

# ── 1. User install ─────────────────────────────────────────────────────────
if (-not $script:SystemOnly) {
    $userRoot = Join-Path $env:USERPROFILE '.ven'
    $userBin  = Join-Path $userRoot 'bin'
    if (Test-Path $userRoot) {
        if (-not $script:DryRun) {
            Remove-Item -Recurse -Force $userRoot -ErrorAction SilentlyContinue
        }
        Write-Host "${tag}Removed user install: $userRoot"
    }
    $ur = _StripPath 'User' @($userBin)
    if ($ur) { Write-Host ("${tag}Stripped from User PATH: " + ($ur -join '; ')) }

    # 1b. Persisted user env vars (written by `ven path set`).
    foreach ($var in @('VEN_HOME')) {
        $cur = [Environment]::GetEnvironmentVariable($var, 'User')
        if ($null -ne $cur) {
            if (-not $script:DryRun) {
                [Environment]::SetEnvironmentVariable($var, $null, 'User')
            }
            Write-Host "${tag}Cleared User env var: `$$var"
        }
    }

    # 1c. Pointer file (~/AppData/Roaming/ven/config.toml).
    $pointerDir  = Join-Path $env:APPDATA 'ven'
    $pointerFile = Join-Path $pointerDir 'config.toml'
    if (Test-Path $pointerFile) {
        if (-not $script:DryRun) { Remove-Item -Force $pointerFile -ErrorAction SilentlyContinue }
        Write-Host "${tag}Removed pointer file: $pointerFile"
    }
    # Drop the dir too if it's now empty so we don't leave litter behind.
    if ((Test-Path $pointerDir) -and -not (Get-ChildItem -Force $pointerDir -ErrorAction SilentlyContinue)) {
        if (-not $script:DryRun) { Remove-Item -Force $pointerDir -ErrorAction SilentlyContinue }
    }

    # 1d. PowerShell user profile — strip ven-managed blocks.
    foreach ($prof in @(
        (Join-Path (Split-Path $PROFILE -Parent) 'Microsoft.PowerShell_profile.ps1'),
        (Join-Path $env:USERPROFILE 'Documents\PowerShell\Microsoft.PowerShell_profile.ps1'),
        (Join-Path $env:USERPROFILE 'Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1')
    )) {
        $changedEnv  = _StripBlock $prof 'ven env'
        $changedPath = _StripBlock $prof 'ven-setup PATH'
        $changedHook = _StripBlock $prof 'ven shell hook'
        if ($changedEnv -or $changedPath -or $changedHook) {
            Write-Host "${tag}Cleaned ven blocks from: $prof"
        }
    }
}

# ── 2. System install ───────────────────────────────────────────────────────
if (-not $script:UserOnly) {
    $sysRoot = Join-Path $env:ProgramFiles 'ven'
    $sysBin  = Join-Path $sysRoot 'bin'
    $isAdmin = _IsAdmin
    $sysDirExists = Test-Path $sysRoot

    # Check Machine PATH separately so a half-finished prior run still
    # converges to clean state.
    $machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $sysPathHasIt = $false
    $sysBinNorm = _Norm $sysBin
    if ($machinePath) {
        foreach ($e in ($machinePath -split ';')) {
            if ((_Norm $e) -eq $sysBinNorm) { $sysPathHasIt = $true; break }
        }
    }

    if ($sysDirExists -or $sysPathHasIt) {
        if ($isAdmin) {
            if ($sysDirExists) {
                if (-not $script:DryRun) {
                    Remove-Item -Recurse -Force $sysRoot -ErrorAction SilentlyContinue
                }
                if ($script:DryRun -or -not (Test-Path $sysRoot)) {
                    Write-Host "${tag}Removed system install: $sysRoot"
                } else {
                    Write-Warning "Could not remove $sysRoot (file in use? close all ven shells)"
                }
            }
            $sr = _StripPath 'Machine' @($sysBin)
            if ($sr) { Write-Host ("${tag}Stripped from Machine PATH: " + ($sr -join '; ')) }
        } else {
            Write-Warning 'System install detected. Re-run in an elevated PowerShell to remove it cleanly:'
            if ($sysDirExists)  { Write-Warning "  - Directory still present: $sysRoot" }
            if ($sysPathHasIt)  { Write-Warning "  - Machine PATH still contains: $sysBin" }
        }
    }
}

Write-Host ''
if ($script:DryRun) {
    Write-Host 'Dry-run finished — nothing was touched. Unset $env:VEN_UNINSTALL_DRY_RUN to execute.'
} else {
    Write-Host 'Done. Open a NEW terminal so the cleaned PATH takes effect.'
}
