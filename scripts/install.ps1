<#
.SYNOPSIS
  One-liner installer for ven on Windows.

.DESCRIPTION
  Downloads the latest (or pinned) ven release from GitHub and installs it either
  by delegating to the self-contained ven-setup.exe (preferred) or by replicating
  the install steps in PowerShell when only a raw-binary zip is available.

  Usage (piped, env-var config):
    irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.ps1 | iex
    $env:VEN_INSTALL_MODE='system'; irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.ps1 | iex

  Usage (param-style):
    & ([scriptblock]::Create((irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.ps1))) -Mode system -Version v0.1.0

  Local invocation:
    pwsh -NoProfile -File scripts\install.ps1 -Mode user -DryRun

.NOTES
  Mirrors src/bin/setup/windows.rs. Keep the PATH + WM_SETTINGCHANGE logic in
  this file in sync with the Rust installer's ensure_path_contains helper.
#>

[CmdletBinding()]
param(
    [ValidateSet('user', 'system')]
    [string] $Mode,
    [string] $Version,
    [string] $Repo,
    [switch] $NoVerify,
    [switch] $DryRun,
    [switch] $ForceReplicate
)

# ---------------------------------------------------------------------------
# Preamble
# ---------------------------------------------------------------------------

$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'

# Force TLS 1.2 on Windows PowerShell 5.1 (default is 1.0 which GitHub rejects).
try {
    [Net.ServicePointManager]::SecurityProtocol = `
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
} catch { }

# Ensure box-drawing characters render correctly on PS 5.1 (default OEM).
try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch { }

# Resolve config: -Param wins, then env var, then default.
$mode            = if ($Mode)            { $Mode }            elseif ($env:VEN_INSTALL_MODE)    { $env:VEN_INSTALL_MODE }    else { '' }
$version         = if ($Version)         { $Version }         elseif ($env:VEN_VERSION)         { $env:VEN_VERSION }         else { 'latest' }
$repo            = if ($Repo)            { $Repo }            elseif ($env:VEN_REPO)            { $env:VEN_REPO }            else { 'bhuwanb23/ven' }
$noVerify        = if ($NoVerify)        { $true }            elseif ($env:VEN_NO_VERIFY)       { [bool]::Parse($env:VEN_NO_VERIFY) }       else { $false }
$dryRun          = if ($DryRun)          { $true }            elseif ($env:VEN_DRY_RUN)         { [bool]::Parse($env:VEN_DRY_RUN) }         else { $false }
$forceReplicate  = if ($ForceReplicate)  { $true }            elseif ($env:VEN_FORCE_REPLICATE) { [bool]::Parse($env:VEN_FORCE_REPLICATE) } else { $false }
$docsUrl         = if ($env:VEN_DOCS_URL) { $env:VEN_DOCS_URL } else { 'https://bhuwanb23.github.io/ven/docs' }

$Line = ('{0}' -f ([string]::new([char]0x2501, 56)))

# ---------------------------------------------------------------------------
# Step helpers (right-aligned [ok] / [skip] / [dry-run] / [FAIL])
# ---------------------------------------------------------------------------

function Step-Begin {
    param([string] $Label)
    Write-Host ('  {0,-50}' -f ("$Label...")) -NoNewline
}
function Step-Done { param([string] $Marker = '[ok]') Write-Host (' ' + $Marker) }
function Step-Skip { Step-Done '[skip]' }
function Step-Dry  { Step-Done '[dry-run]' }
function Step-Fail { Step-Done '[FAIL]' }

# Run a scriptblock silently; print [ok] on success, [FAIL] + captured output
# on failure.
function Run-Step {
    param(
        [string] $Label,
        [scriptblock] $Action
    )
    Step-Begin $Label
    if ($dryRun) { Step-Dry; return }
    $logPath = Join-Path $tempRoot 'step.log'
    try {
        & $Action *>&1 | Out-File -FilePath $logPath -Encoding utf8 -Force
        Step-Done
    } catch {
        Step-Fail
        Write-Host ''
        Write-Host '----- step output -----' -ForegroundColor Yellow
        if (Test-Path $logPath) { Get-Content $logPath | Write-Host }
        throw
    }
}

# ---------------------------------------------------------------------------
# Banner
# ---------------------------------------------------------------------------

Write-Host ''
Write-Host 'ven Installer'
Write-Host $Line
Write-Host ''

# ---------------------------------------------------------------------------
# Detection
# ---------------------------------------------------------------------------

if ($PSVersionTable.PSVersion.Major -lt 5 -or `
    ($PSVersionTable.PSVersion.Major -eq 5 -and $PSVersionTable.PSVersion.Minor -lt 1)) {
    throw "PowerShell 5.1 or newer is required (found $($PSVersionTable.PSVersion))."
}

$archRaw = $env:PROCESSOR_ARCHITECTURE
if (-not $archRaw) { $archRaw = 'AMD64' }
$arch = switch ($archRaw.ToUpperInvariant()) {
    'AMD64' { 'x64' }
    'X86'   { throw 'x86 (32-bit) Windows is not supported.' }
    'ARM64' { 'arm64' }
    default { throw "Unsupported architecture: $archRaw" }
}

$isAdmin = ([Security.Principal.WindowsPrincipal]`
    [Security.Principal.WindowsIdentity]::GetCurrent()`
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

$isTty = -not [Console]::IsInputRedirected

$osHuman   = "Windows $([Environment]::OSVersion.Version.Major)"
$psHuman   = "$($PSVersionTable.PSVersion)"
$adminHuman = if ($isAdmin) { 'Yes' } else { 'No' }

Write-Host 'Detecting system...'
Write-Host ('  OS:           {0}' -f $osHuman)
Write-Host ('  Architecture: {0}' -f $arch)
Write-Host ('  PowerShell:   {0}' -f $psHuman)
Write-Host ('  Admin rights: {0}' -f $adminHuman)
Write-Host ''

# ---------------------------------------------------------------------------
# Mode selection
# ---------------------------------------------------------------------------

if (-not $mode) {
    if (-not $isTty) {
        $mode = 'user'
    } else {
        Write-Host 'Select install mode:'
        Write-Host '  [1] User Install (recommended) -- no admin required, only for you'
        Write-Host '  [2] System Install            -- requires admin (UAC), all users on this machine'
        $choice = Read-Host 'Choose [1/2]'
        switch ($choice.Trim()) {
            '1' { $mode = 'user' }
            '2' { $mode = 'system' }
            default { throw "Invalid selection: '$choice'. Set -Mode or `$env:VEN_INSTALL_MODE explicitly." }
        }
    }
}

if ($mode -ne 'user' -and $mode -ne 'system') {
    throw "Invalid mode '$mode'. Expected 'user' or 'system'."
}

if ($mode -eq 'system') {
    $installDir = Join-Path $env:ProgramFiles 'ven\bin'
    $modeHuman  = "System ($installDir)"
} else {
    $installDir = Join-Path $env:USERPROFILE '.ven\bin'
    $modeHuman  = "User (no admin)"
}

Write-Host ('Install mode: {0}' -f $modeHuman)
Write-Host ('Install path: {0}' -f $installDir)
Write-Host ''

# ---------------------------------------------------------------------------
# Temp scratch (try/finally cleanup)
# ---------------------------------------------------------------------------

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("ven-install-" + [guid]::NewGuid())
if (-not $dryRun) {
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null
} else {
    # Step-Done writes the log file in non-dry mode; no log path needed otherwise.
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("ven-install-dry-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null
}

# ---------------------------------------------------------------------------
# Resolve release + select asset
# ---------------------------------------------------------------------------

$headers = @{ 'User-Agent' = 'ven-install.ps1' }
if ($env:GITHUB_TOKEN) { $headers['Authorization'] = "Bearer $($env:GITHUB_TOKEN)" }

$apiUrl = if ($version -eq 'latest') {
    "https://api.github.com/repos/$repo/releases/latest"
} else {
    "https://api.github.com/repos/$repo/releases/tags/$version"
}

Step-Begin "Resolving release ($repo $version)"
try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers $headers -Method Get
    Step-Done
} catch {
    Step-Fail
    throw "Failed to fetch release JSON from $apiUrl : $($_.Exception.Message)"
}
$tagName = $release.tag_name

function Find-Asset {
    param([string] $Name)
    $release.assets | Where-Object { $_.name -eq $Name } | Select-Object -First 1
}

$setupAssetName = "ven-setup-windows-$arch.exe"
$zipAssetName   = "ven-windows-$arch.zip"

Step-Begin 'Selecting asset'
$setupAsset = if (-not $forceReplicate) { Find-Asset $setupAssetName } else { $null }
$zipAsset   = Find-Asset $zipAssetName
if ($setupAsset) {
    $useDelegate = $true
    $asset       = $setupAsset
    Step-Done '[ok: Delegate]'
} elseif ($zipAsset) {
    $useDelegate = $false
    $asset       = $zipAsset
    Step-Done '[ok: Replicate]'
} else {
    Step-Fail
    $available = ($release.assets | ForEach-Object { $_.name }) -join ', '
    throw "Release $tagName has neither '$setupAssetName' nor '$zipAssetName'. Available: $available"
}

# Per-asset .sha256 sidecar (preferred) or SHA256SUMS manifest (fallback).
$shaSidecarAsset = Find-Asset "$($asset.name).sha256"
$sumsAsset       = Find-Asset 'SHA256SUMS'

# ---------------------------------------------------------------------------
# Download
# ---------------------------------------------------------------------------

$downloadPath = Join-Path $tempRoot $asset.name

function Format-Bytes {
    param([long] $Bytes)
    if ($Bytes -ge 1MB) { return ('{0:N1} MB' -f ($Bytes / 1MB)) }
    if ($Bytes -ge 1KB) { return ('{0:N1} KB' -f ($Bytes / 1KB)) }
    return ('{0} B' -f $Bytes)
}

Run-Step "Downloading $($asset.name)" {
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $downloadPath -UseBasicParsing -Headers $headers | Out-Null
}
if (-not $dryRun) {
    Write-Host ('    {0} downloaded' -f (Format-Bytes (Get-Item $downloadPath).Length))
}

# ---------------------------------------------------------------------------
# Verify SHA256
# ---------------------------------------------------------------------------

Step-Begin 'Verifying SHA256'
if ($noVerify) {
    Step-Skip
} elseif ($dryRun) {
    Step-Dry
} elseif ($shaSidecarAsset) {
    try {
        $shaPath = Join-Path $tempRoot ($asset.name + '.sha256')
        Invoke-WebRequest -Uri $shaSidecarAsset.browser_download_url -OutFile $shaPath -UseBasicParsing -Headers $headers | Out-Null
        $expected = ((Get-Content $shaPath -TotalCount 1) -split '\s+')[0].ToLowerInvariant()
        $actual   = (Get-FileHash -Path $downloadPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -eq $expected) {
            Step-Done
        } else {
            Step-Fail
            throw "SHA256 mismatch for $($asset.name) (sidecar): expected $expected, got $actual"
        }
    } catch {
        Step-Fail; throw
    }
} elseif ($sumsAsset) {
    try {
        $sumsPath = Join-Path $tempRoot $sumsAsset.name
        Invoke-WebRequest -Uri $sumsAsset.browser_download_url -OutFile $sumsPath -UseBasicParsing -Headers $headers | Out-Null
        $expected = $null
        Get-Content $sumsPath | ForEach-Object {
            $tokens = ($_ -split '\s+', 2)
            if ($tokens.Count -eq 2 -and ($tokens[1].TrimStart('*') -eq $asset.name)) {
                $expected = $tokens[0].ToLowerInvariant()
            }
        }
        if (-not $expected) {
            Step-Fail
            throw "SHA256SUMS did not contain an entry for $($asset.name)."
        }
        $actual = (Get-FileHash -Path $downloadPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -eq $expected) {
            Step-Done
        } else {
            Step-Fail
            throw "SHA256 mismatch for $($asset.name) (manifest): expected $expected, got $actual"
        }
    } catch {
        Step-Fail; throw
    }
} else {
    Step-Skip
    Write-Host ('    note: neither {0}.sha256 nor SHA256SUMS published in this release' -f $asset.name)
}

# ---------------------------------------------------------------------------
# Install: Delegate vs Replicate
# ---------------------------------------------------------------------------

function Update-VenPath {
    param(
        [string] $Entry,
        [ValidateSet('User','Machine')][string] $Scope
    )
    $entryPs = $Entry.Replace("'", "''")
    $script = @"
`$target = '$entryPs'
`$scope = '$Scope'
`$current = [Environment]::GetEnvironmentVariable('Path', `$scope)
if ([string]::IsNullOrWhiteSpace(`$current)) {
  `$new = `$target
} elseif (`$current -split ';' | Where-Object { `$_.Trim().ToLowerInvariant() -eq `$target.ToLowerInvariant() }) {
  `$new = `$current
} else {
  `$new = `$current.TrimEnd(';') + ';' + `$target
}
[Environment]::SetEnvironmentVariable('Path', `$new, `$scope)

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
namespace Win32 {
  public static class Native {
    [DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern IntPtr SendMessageTimeout(
      IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
      uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
  }
}
'@
`$HWND_BROADCAST = [IntPtr]0xffff
`$WM_SETTINGCHANGE = 0x001A
[UIntPtr]`$result = [UIntPtr]::Zero
[Win32.Native]::SendMessageTimeout(`$HWND_BROADCAST, `$WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]`$result) | Out-Null
"@
    & powershell.exe -NoLogo -NoProfile -NonInteractive -Command $script
    if ($LASTEXITCODE -ne 0) { throw "Failed to update $Scope PATH" }
}

try {
    if ($useDelegate) {
        Run-Step "Delegating to ven-setup ($mode)" {
            $proc = Start-Process -FilePath $downloadPath `
                -ArgumentList @('--mode', $mode, '--no-input') `
                -Wait -PassThru
            if ($proc.ExitCode -ne 0) {
                throw "ven-setup exited with code $($proc.ExitCode)"
            }
        }
    } else {
        if ($mode -eq 'system' -and -not $isAdmin) {
            throw 'System install via Replicate path requires Administrator. Re-run from an elevated PowerShell.'
        }
        $extractDir = Join-Path $tempRoot 'extract'
        Run-Step 'Extracting' {
            Expand-Archive -Path $downloadPath -DestinationPath $extractDir -Force
        }
        Run-Step ("Installing to $installDir") {
            New-Item -ItemType Directory -Force -Path $installDir | Out-Null
            Copy-Item -Force (Join-Path $extractDir 'ven.exe')          (Join-Path $installDir 'ven.exe')
            Copy-Item -Force (Join-Path $extractDir 'ven-launcher.exe') (Join-Path $installDir 'ven-launcher.exe')
        }
        $scope = if ($mode -eq 'system') { 'Machine' } else { 'User' }
        Run-Step ("Updating $scope PATH + WM_SETTINGCHANGE") {
            Update-VenPath -Entry $installDir -Scope $scope
        }
        Run-Step 'Installing shell hook (ven setup)' {
            $venExe = Join-Path $installDir 'ven.exe'
            & $venExe setup
            if ($LASTEXITCODE -ne 0) { throw 'ven setup failed' }
        }
    }

    # ---------------------------------------------------------------------------
    # Verify
    # ---------------------------------------------------------------------------

    $verifyOut = ''
    Run-Step 'Verifying installation' {
        $merged = "$installDir;$($env:PATH)"
        $script:verifyOut = & cmd.exe /C "set PATH=$merged && ven --version"
        if ($LASTEXITCODE -ne 0) {
            throw "Verification failed (exit $LASTEXITCODE)"
        }
    }

    # ---------------------------------------------------------------------------
    # Done banner
    # ---------------------------------------------------------------------------

    Write-Host ''
    Write-Host $Line
    if ($dryRun) {
        Write-Host ('[OK] dry-run complete (release {0})' -f $tagName)
    } elseif ($verifyOut) {
        Write-Host ('[OK] {0} installed successfully!' -f (($verifyOut -join ' ').Trim()))
    } else {
        Write-Host ('[OK] ven {0} installed successfully!' -f $tagName)
    }
    Write-Host ''
    Write-Host 'Open a NEW terminal and run:'
    Write-Host '  ven --version'
    Write-Host '  ven init'
    Write-Host ''
    Write-Host ('Documentation: {0}' -f $docsUrl)
    Write-Host $Line

} finally {
    if ((Test-Path $tempRoot) -and -not $dryRun) {
        Remove-Item -Path $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
    } elseif ((Test-Path $tempRoot) -and $dryRun) {
        Remove-Item -Path $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
