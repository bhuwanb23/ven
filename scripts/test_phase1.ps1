# Phase 1 Automated Test Runner for PowerShell
# Run this after installing Rust and fnm

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "ven Phase 1 Automated Test Suite" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

$passCount = 0
$failCount = 0
$testDir = Join-Path $env:TEMP "ven_test_$(Get-Random)"

# Helper function to run a test
function Run-Test {
    param(
        [string]$TestName,
        [scriptblock]$Command
    )
    
    Write-Host -NoNewline "Testing: $TestName ... "
    
    try {
        & $Command > $testOutputFile 2>&1
        Write-Host "PASS" -ForegroundColor Green
        $script:passCount++
    } catch {
        Write-Host "FAIL" -ForegroundColor Red
        Write-Host "  Error: $_"
        $script:failCount++
    }
}

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow
Write-Host ""

try {
    $cargoVersion = cargo --version 2>&1
    Write-Host "✓ Rust/Cargo found: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust not found. Install from: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

try {
    $fnmVersion = fnm --version 2>&1
    Write-Host "✓ fnm found: $fnmVersion" -ForegroundColor Green
} catch {
    Write-Host "⚠ fnm not found. Some tests will fail." -ForegroundColor Yellow
    Write-Host "  Install from: https://github.com/Schniz/fnm"
}

Write-Host ""
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Running Tests" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

# Build first
Write-Host "Building ven..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful" -ForegroundColor Green
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Create temporary test directory
New-Item -ItemType Directory -Path $testDir -Force | Out-Null
Write-Host "Test directory: $testDir" -ForegroundColor Gray
Write-Host ""

# Set up test output file
$testOutputFile = Join-Path $testDir "test_output.txt"
$venBin = Join-Path (Get-Location).Path "target\release\ven.exe"

# Test suite
Write-Host "--- Unit Tests ---" -ForegroundColor Cyan
Run-Test "Unit tests (all)" { cargo test --all }

Write-Host ""
Write-Host "--- CLI Commands ---" -ForegroundColor Cyan

Set-Location $testDir

Run-Test "ven init" { & $venBin init }
Run-Test "ven.toml created" { 
    if (!(Test-Path "ven.toml")) { throw "ven.toml not created" }
}
Run-Test "ven status" { & $venBin status }
Run-Test "ven list" { & $venBin list }

Write-Host ""
Write-Host "--- Configuration System ---" -ForegroundColor Cyan

$tomlContent = @"
[runtime]
node = "20.11.1"

[packages]
express = "^4.18.2"

[env]
NODE_ENV = "production"
"@

Run-Test "Create test config" { 
    Set-Content -Path "test_config.toml" -Value $tomlContent
}

Write-Host ""
Write-Host "--- Shell Hook ---" -ForegroundColor Cyan

Run-Test "Generate bash hook" { & $venBin shell hook bash }
Run-Test "Generate zsh hook" { & $venBin shell hook zsh }
Run-Test "Generate fish hook" { & $venBin shell hook fish }

Write-Host ""
Write-Host "--- Cleanup ---" -ForegroundColor Yellow

Set-Location (Join-Path (Get-Location).Parent "scripts")
Remove-Item -Path $testDir -Recurse -Force
Write-Host "Test directory cleaned up" -ForegroundColor Gray

Write-Host ""
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

Write-Host "Passed: $passCount" -ForegroundColor Green
Write-Host "Failed: $failCount" -ForegroundColor Red
Write-Host ""

if ($failCount -eq 0) {
    Write-Host "All tests passed! ✓" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some tests failed. Check output above." -ForegroundColor Red
    exit 1
}
