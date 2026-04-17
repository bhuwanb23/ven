# Phase 0 Test Script - Shell Integration & Auto-Switching
# Run this to verify the complete cd hook workflow

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  PHASE 0: Foundation Fixes Test" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$testDir = Join-Path $PWD "test-phase0"
$projectA = Join-Path $testDir "project-a"
$projectB = Join-Path $testDir "project-b"

# Cleanup previous tests
if (Test-Path $testDir) {
    Write-Host "[CLEANUP] Removing old test directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $testDir
}

# Create test structure
Write-Host "[SETUP] Creating test projects..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path $projectA -Force | Out-Null
New-Item -ItemType Directory -Path $projectB -Force | Out-Null

# Project A: Node 20
@'
[runtime]
node = "20"

[packages]
express = "^4.18.2"

[env]
NODE_ENV = "development"
PORT = "3000"
'@ | Set-Content (Join-Path $projectA "ven.toml")

# Project B: Node 18
@'
[runtime]
node = "18"

[env]
NODE_ENV = "production"
'@ | Set-Content (Join-Path $projectB "ven.toml")

Write-Host "  ✓ Created project-a (Node 20)" -ForegroundColor Green
Write-Host "  ✓ Created project-b (Node 18)" -ForegroundColor Green

# Test 1: ven.toml parsing
Write-Host "`n[TEST 1] ven.toml parsing..." -ForegroundColor Cyan
Write-Host "  Running: ven status --verbose" -ForegroundColor Dim
ven status --verbose
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✓ ven status works" -ForegroundColor Green
} else {
    Write-Host "  ✗ ven status failed" -ForegroundColor Red
}

# Test 2: Shell hook generation
Write-Host "`n[TEST 2] Shell hook generation..." -ForegroundColor Cyan
Write-Host "  Running: ven shell hook powershell" -ForegroundColor Dim
$hook = ven shell hook powershell
if ($hook -and $hook.Contains("Set-VenLocation")) {
    Write-Host "  ✓ PowerShell hook generated" -ForegroundColor Green
} else {
    Write-Host "  ✗ PowerShell hook invalid" -ForegroundColor Red
}

# Test 3: Shell activate (PATH output)
Write-Host "`n[TEST 3] ven shell activate PATH output..." -ForegroundColor Cyan
Set-Location $projectA
Write-Host "  Running: ven shell activate '$projectA'" -ForegroundColor Dim
$exports = ven shell activate $projectA
if ($exports) {
    Write-Host "  Exports returned:" -ForegroundColor Dim
    $exports -split "`n" | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    
    if ($exports.Contains("VEN_NODE_VERSION") -and $exports.Contains("PATH")) {
        Write-Host "  ✓ PATH exports generated correctly" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Missing required exports" -ForegroundColor Red
    }
} else {
    Write-Host "  ✗ No exports returned" -ForegroundColor Red
}

# Test 4: Test with Project B
Write-Host "`n[TEST 4] Switch to project-b..." -ForegroundColor Cyan
Set-Location $projectB
$exportsB = ven shell activate $projectB
if ($exportsB -and $exportsB.Contains("18")) {
    Write-Host "  ✓ Project B activates Node 18" -ForegroundColor Green
} else {
    Write-Host "  ✗ Project B activation failed" -ForegroundColor Red
}

# Test 5: No ven.toml scenario
Write-Host "`n[TEST 5] Directory without ven.toml..." -ForegroundColor Cyan
Set-Location $testDir
$noExports = ven shell activate $testDir
if (-not $noExports -or $noExports -eq "") {
    Write-Host "  ✓ No exports when ven.toml missing (correct)" -ForegroundColor Green
} else {
    Write-Host "  ✗ Should return empty when no ven.toml" -ForegroundColor Red
}

# Cleanup
Write-Host "`n[CLEANUP] Removing test directory..." -ForegroundColor Yellow
Set-Location $PSScriptRoot
Remove-Item -Recurse -Force $testDir

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Phase 0 Test Complete!" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "  1. Run 'ven setup' to install shell hooks" -ForegroundColor White
Write-Host "  2. Restart PowerShell" -ForegroundColor White
Write-Host "  3. Manually test: cd into project with ven.toml" -ForegroundColor White
Write-Host "  4. Verify: node --version matches ven.toml" -ForegroundColor White
Write-Host ""
