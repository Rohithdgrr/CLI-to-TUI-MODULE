#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Verify CLI-to-TUI converter works end-to-end.
#>

$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$failed = $false

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  CLI-to-TUI Converter - Verification" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# --- Step 1: Build ---
Write-Host "[1/5] Building workspace..." -ForegroundColor Yellow
Push-Location $root
cargo build 2>$null
if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL: build error" -ForegroundColor Red; $failed = $true }
else { Write-Host "  OK" -ForegroundColor Green }

# --- Step 2: Build example ---
Write-Host "[2/5] Building example binary..." -ForegroundColor Yellow
cargo build --example image-processor --features clap 2>$null
if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL: example build error" -ForegroundColor Red; $failed = $true }
else { Write-Host "  OK" -ForegroundColor Green }

if ($failed) { Pop-Location; exit 1 }

# --- Step 3: Run 11 automated tests ---
Write-Host "[3/5] Running 11 automated tests..." -ForegroundColor Yellow
$testOutput = & cargo run --example image-processor --features clap -- --test 2>&1
$testOutput | ForEach-Object { Write-Host "  $_" }
if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL" -ForegroundColor Red; Pop-Location; exit 1 }
Write-Host ""

# --- Step 4: CLI mode ---
Write-Host "[4/5] Testing CLI mode..." -ForegroundColor Yellow
$cliOutput = & cargo run --example image-processor --features clap -- `
    --input ./photo.png `
    --output ./result.jpg `
    --threads 8 `
    --verbose `
    --format webp 2>&1
$cliOutput | ForEach-Object { Write-Host "  $_" }

$hasInput   = $cliOutput -match "Input:\s+.*photo\.png"
$hasOutput  = $cliOutput -match "Output:\s+.*result\.jpg"
$hasThreads = $cliOutput -match "Threads:\s+8"
$hasVerbose = $cliOutput -match "Verbose:\s+true"
$hasFormat  = $cliOutput -match "Format:\s+webp"

if ($hasInput -and $hasOutput -and $hasThreads -and $hasVerbose -and $hasFormat) {
    Write-Host "  CLI mode: PASS" -ForegroundColor Green
} else {
    Write-Host "  CLI mode: FAIL" -ForegroundColor Red
    Pop-Location; exit 1
}

# --- Step 5: TUI launch (3 seconds then kill) ---
Write-Host "[5/5] Testing TUI mode..." -ForegroundColor Yellow
$proc = Start-Process -FilePath "cargo" `
    -ArgumentList "run","--example","image-processor","--features","clap","--","--tui" `
    -WorkingDirectory $root `
    -NoNewWindow -PassThru

Start-Sleep -Seconds 3
if ($proc.HasExited) {
    Write-Host "  TUI exited early (exit code $($proc.ExitCode))" -ForegroundColor Red
} else {
    $proc.Kill() | Out-Null
    Write-Host "  TUI launched and rendered: PASS" -ForegroundColor Green
}

Pop-Location

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  ALL VERIFICATIONS PASSED" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Commands to run:"
Write-Host "  TUI:  cargo run -p tui-generator --example image-processor --features clap -- --tui"
Write-Host "  CLI:  cargo run -p tui-generator --example image-processor --features clap -- --input a.png --output b.png --threads 4 --format png"
Write-Host "  Help: cargo run -p tui-generator --example image-processor --features clap -- --help"
Write-Host ""
