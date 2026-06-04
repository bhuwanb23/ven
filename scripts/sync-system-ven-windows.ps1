<#
.SYNOPSIS
  Upgrade the system ven install from a newer per-user ~/.ven/bin copy.

.DESCRIPTION
  On Windows, Machine PATH is searched before User PATH. If an old ven lives in
  %ProgramFiles%\ven\bin while a newer ven is in %USERPROFILE%\.ven\bin, bare
  `ven` runs the old binary (no `ven update` on pre-0.1.7 builds).

  This script copies ven.exe and ven-launcher.exe from the user install into
  the system install directory. Requires Administrator (UAC prompt).

.EXAMPLE
  pwsh -File scripts/sync-system-ven-windows.ps1
#>
$ErrorActionPreference = 'Stop'
$userBin = Join-Path $env:USERPROFILE '.ven\bin'
$sysBin = Join-Path $env:ProgramFiles 'ven\bin'
if (-not (Test-Path (Join-Path $userBin 'ven.exe'))) {
    throw "No user install at $userBin\ven.exe"
}
if (-not (Test-Path $sysBin)) {
    New-Item -ItemType Directory -Force -Path $sysBin | Out-Null
}
$srcVen = Join-Path $userBin 'ven.exe'
$srcLauncher = Join-Path $userBin 'ven-launcher.exe'
$inner = @"
Copy-Item -Force '$srcVen' '$sysBin\ven.exe'
if (Test-Path '$srcLauncher') { Copy-Item -Force '$srcLauncher' '$sysBin\ven-launcher.exe' }
& '$sysBin\ven.exe' --version
"@
$proc = Start-Process powershell -Verb RunAs -Wait -PassThru -ArgumentList @(
    '-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', $inner
)
if ($proc.ExitCode -ne 0) { exit $proc.ExitCode }
Write-Host "[OK] System ven synced from $userBin"
