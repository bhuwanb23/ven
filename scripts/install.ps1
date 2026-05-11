<#
.SYNOPSIS
  One-liner installer for ven on Windows.

.DESCRIPTION
  Downloads the latest (or pinned) ven release from GitHub and installs it either
  by delegating to the self-contained ven-setup.exe (preferred) or by replicating
  the install steps in PowerShell when only a raw-binary zip is available.

  Usage (piped, env-var config):
    irm https://get.ven.sh/install.ps1 | iex
    $env:VEN_INSTALL_MODE='system'; irm https://get.ven.sh/install.ps1 | iex

  Usage (param-style):
    & ([scriptblock]::Create((irm https://get.ven.sh/install.ps1))) -Mode system -Version v0.1.0

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

# Resolve config: -Param wins, then env var, then default. Lower-cased booleans
# are parsed with [bool]::Parse so callers can use 'true' or 'false' in env vars.
$mode            = if ($Mode)            { $Mode }            elseif ($env:VEN_INSTALL_MODE)    { $env:VEN_INSTALL_MODE }    else { '' }
$version         = if ($Version)         { $Version }         elseif ($env:VEN_VERSION)         { $env:VEN_VERSION }         else { 'latest' }
$repo            = if ($Repo)            { $Repo }            elseif ($env:VEN_REPO)            { $env:VEN_REPO }            else { 'yourorg/ven' }
$noVerify        = if ($NoVerify)        { $true }            elseif ($env:VEN_NO_VERIFY)       { [bool]::Parse($env:VEN_NO_VERIFY) } else { $false }
$dryRun          = if ($DryRun)          { $true }            elseif ($env:VEN_DRY_RUN)         { [bool]::Parse($env:VEN_DRY_RUN) }  else { $false }
$forceReplicate  = if ($ForceReplicate)  { $true }            elseif ($env:VEN_FORCE_REPLICATE) { [bool]::Parse($env:VEN_FORCE_REPLICATE) } else { $false }

# ---------------------------------------------------------------------------
# Banner + detection
# ---------------------------------------------------------------------------

Write-Host ''
Write-Host '  +-----------------------------------------+'
Write-Host '  |  ven one-liner installer (Windows)      |'
Write-Host '  +-----------------------------------------+'
Write-Host ("  repo:    {0}" -f $repo)
Write-Host ("  version: {0}" -f $version)
Write-Host ("  mode:    {0}" -f ($(if ($mode) { $mode } else { '(prompt)' })))
Write-Host ("  dry-run: {0}" -f $dryRun)
Write-Host ''

if ($PSVersionTable.PSVersion.Major -lt 5 -or `
    ($PSVersionTable.PSVersion.Major -eq 5 -and $PSVersionTable.PSVersion.Minor -lt 1)) {
    throw "PowerShell 5.1 or newer is required (found $($PSVersionTable.PSVersion))."
}

# Architecture: PROCESSOR_ARCHITECTURE is the cleanest, no CIM call required.
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

Write-Host ("  arch:    {0}" -f $arch)
Write-Host ("  admin:   {0}" -f $isAdmin)
Write-Host ("  tty:     {0}" -f $isTty)
Write-Host ''

# ---------------------------------------------------------------------------
# Mode selection
# ---------------------------------------------------------------------------

if (-not $mode) {
    if (-not $isTty) {
        $mode = 'user'
        Write-Host '[1/6] No mode supplied + non-interactive shell => defaulting to "user".'
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

# ---------------------------------------------------------------------------
# GitHub release fetch + asset selection
# ---------------------------------------------------------------------------

$headers = @{ 'User-Agent' = 'ven-install.ps1' }
if ($env:GITHUB_TOKEN) { $headers['Authorization'] = "Bearer $($env:GITHUB_TOKEN)" }

$apiUrl = if ($version -eq 'latest') {
    "https://api.github.com/repos/$repo/releases/latest"
} else {
    "https://api.github.com/repos/$repo/releases/tags/$version"
}

Write-Host "[2/6] Fetching release metadata: $apiUrl"
try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers $headers -Method Get
} catch {
    throw "Failed to fetch release JSON from $apiUrl : $($_.Exception.Message)"
}

$tagName = $release.tag_name
Write-Host "  resolved tag: $tagName"

$setupAssetName = "ven-setup-windows-$arch.exe"
$zipAssetName   = "ven-windows-$arch.zip"
$sumsAssetName  = 'SHA256SUMS'

function Find-Asset {
    param([string] $Name)
    $release.assets | Where-Object { $_.name -eq $Name } | Select-Object -First 1
}

$setupAsset = if (-not $forceReplicate) { Find-Asset $setupAssetName } else { $null }
$zipAsset   = Find-Asset $zipAssetName
$sumsAsset  = Find-Asset $sumsAssetName

if (-not $setupAsset -and -not $zipAsset) {
    $available = ($release.assets | ForEach-Object { $_.name }) -join ', '
    throw "Release $tagName has neither '$setupAssetName' nor '$zipAssetName'. Available: $available"
}

$useDelegate = [bool]$setupAsset
$asset       = if ($useDelegate) { $setupAsset } else { $zipAsset }
Write-Host ("  path:    {0}" -f $(if ($useDelegate) { "Delegate ($($asset.name))" } else { "Replicate ($($asset.name))" }))

# ---------------------------------------------------------------------------
# Temp scratch + download
# ---------------------------------------------------------------------------

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("ven-install-" + [guid]::NewGuid())
if (-not $dryRun) {
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null
}
$downloadPath = Join-Path $tempRoot $asset.name

Write-Host ''
Write-Host "[3/6] Downloading $($asset.name)"
if (-not $dryRun) {
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $downloadPath -UseBasicParsing -Headers $headers
    Write-Host ("  saved: {0} ({1:N0} bytes)" -f $downloadPath, (Get-Item $downloadPath).Length)
} else {
    Write-Host '  [dry-run] skipped download'
}

# ---------------------------------------------------------------------------
# SHA256 verify
# ---------------------------------------------------------------------------

if (-not $noVerify -and $sumsAsset -and -not $dryRun) {
    Write-Host "[4/6] Verifying SHA256 against $($sumsAsset.name)"
    $sumsPath = Join-Path $tempRoot $sumsAsset.name
    Invoke-WebRequest -Uri $sumsAsset.browser_download_url -OutFile $sumsPath -UseBasicParsing -Headers $headers
    $expected = $null
    Get-Content $sumsPath | ForEach-Object {
        $tokens = ($_ -split '\s+', 2)
        if ($tokens.Count -eq 2 -and ($tokens[1].TrimStart('*') -eq $asset.name)) {
            $expected = $tokens[0].ToLowerInvariant()
        }
    }
    if (-not $expected) {
        throw "SHA256SUMS did not contain an entry for $($asset.name)."
    }
    $actual = (Get-FileHash -Path $downloadPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        throw "SHA256 mismatch for $($asset.name): expected $expected, got $actual."
    }
    Write-Host "  ok  ($expected)"
} elseif ($noVerify) {
    Write-Host '[4/6] Skipping SHA256 verification (-NoVerify / VEN_NO_VERIFY)'
} elseif (-not $sumsAsset) {
    Write-Host "[4/6] Skipping SHA256 verification (SHA256SUMS not present in release)"
} else {
    Write-Host '[4/6] [dry-run] skipped SHA256 verification'
}

# ---------------------------------------------------------------------------
# Install: Delegate path
# ---------------------------------------------------------------------------

function Invoke-Delegate {
    param([string] $SetupExe, [string] $InstallMode, [bool] $IsDryRun)
    Write-Host ''
    Write-Host "[5/6] Delegating to ven-setup ($InstallMode)"
    if ($IsDryRun) {
        Write-Host "  [dry-run] would run: $SetupExe --mode $InstallMode --no-input"
        return
    }
    # ven-setup handles UAC and the elevated-child loop guard itself; do not
    # add -Verb RunAs here or we will double-elevate.
    $proc = Start-Process -FilePath $SetupExe `
        -ArgumentList @('--mode', $InstallMode, '--no-input') `
        -Wait -PassThru
    if ($proc.ExitCode -ne 0) {
        throw "ven-setup exited with code $($proc.ExitCode)"
    }
}

# ---------------------------------------------------------------------------
# Install: Replicate path (PowerShell port of src/bin/setup/windows.rs)
# ---------------------------------------------------------------------------

function Update-VenPath {
    param([string] $Entry, [ValidateSet('User','Machine')][string] $Scope)
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
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to update $Scope PATH"
    }
}

function Invoke-Replicate {
    param([string] $ZipPath, [string] $InstallMode, [bool] $IsDryRun)
    Write-Host ''
    Write-Host "[5/6] Replicate install ($InstallMode) -- PowerShell port of ven-setup windows logic"

    if ($InstallMode -eq 'system' -and -not $isAdmin -and -not $IsDryRun) {
        throw 'System install via Replicate path requires Administrator. Re-run from an elevated PowerShell.'
    }

    $installDir = if ($InstallMode -eq 'system') {
        Join-Path $env:ProgramFiles 'ven\bin'
    } else {
        Join-Path $env:USERPROFILE '.ven\bin'
    }
    $scope = if ($InstallMode -eq 'system') { 'Machine' } else { 'User' }

    Write-Host "  [a] Extract zip -> $tempRoot\extract"
    $extractDir = Join-Path $tempRoot 'extract'
    if (-not $IsDryRun) {
        Expand-Archive -Path $ZipPath -DestinationPath $extractDir -Force
    } else {
        Write-Host '      [dry-run] skipped'
    }

    Write-Host "  [b] Copy binaries -> $installDir"
    if (-not $IsDryRun) {
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null
        Copy-Item -Force (Join-Path $extractDir 'ven.exe')          (Join-Path $installDir 'ven.exe')
        Copy-Item -Force (Join-Path $extractDir 'ven-launcher.exe') (Join-Path $installDir 'ven-launcher.exe')
    } else {
        Write-Host '      [dry-run] skipped'
    }

    Write-Host "  [c] Update $scope PATH + WM_SETTINGCHANGE"
    if (-not $IsDryRun) {
        Update-VenPath -Entry $installDir -Scope $scope
    } else {
        Write-Host '      [dry-run] skipped'
    }

    Write-Host '  [d] Install shell hooks (ven setup)'
    $venExe = Join-Path $installDir 'ven.exe'
    if (-not $IsDryRun) {
        & $venExe setup
        if ($LASTEXITCODE -ne 0) { throw 'ven setup failed' }
    } else {
        Write-Host '      [dry-run] skipped'
    }
}

# ---------------------------------------------------------------------------
# Dispatch + verify + cleanup
# ---------------------------------------------------------------------------

try {
    if ($useDelegate) {
        Invoke-Delegate -SetupExe $downloadPath -InstallMode $mode -IsDryRun $dryRun
    } else {
        Invoke-Replicate -ZipPath $downloadPath -InstallMode $mode -IsDryRun $dryRun
    }

    Write-Host ''
    Write-Host '[6/6] Verifying ven --version in a new process'
    if (-not $dryRun) {
        $installDir = if ($mode -eq 'system') {
            Join-Path $env:ProgramFiles 'ven\bin'
        } else {
            Join-Path $env:USERPROFILE '.ven\bin'
        }
        $merged = "$installDir;$($env:PATH)"
        $out = & cmd.exe /C "set PATH=$merged && ven --version"
        if ($LASTEXITCODE -ne 0) {
            throw "Verification failed (exit $LASTEXITCODE): $out"
        }
        Write-Host ("  [OK] {0}" -f ($out -join ' ').Trim())
    } else {
        Write-Host '  [dry-run] skipped verification'
    }
} finally {
    if ((Test-Path $tempRoot) -and -not $dryRun) {
        Remove-Item -Path $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ''
Write-Host "Done. Open a new terminal and run: ven --version"
