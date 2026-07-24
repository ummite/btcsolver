# BTC Solver - start dashboard (+ Bitcoin Core if available)
# Portable: works from any clone path (not only Y:\btcsolver)
$ErrorActionPreference = "Continue"
$Project = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
$Port = 3000

function Join-Safe([string]$Root, [string]$Child) {
    if ([string]::IsNullOrWhiteSpace($Root)) { return $Child }
    $r = $Root.TrimEnd('\', '/')
    $c = $Child.TrimStart('\', '/')
    return "$r\$c"
}

function Test-DrivePath([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path)) { return $false }
    try { return [bool](Test-Path -LiteralPath $Path -ErrorAction SilentlyContinue) } catch { return $false }
}

# Prefer local full node on W:, else skip Core cleanly
$Datadir = $null
foreach ($cand in @("W:\Bitcoin", "C:\Bitcoin", (Join-Safe $env:APPDATA "Bitcoin"))) {
    if (Test-DrivePath (Join-Safe $cand "blocks")) {
        $Datadir = $cand
        break
    }
}
if (-not $Datadir) { $Datadir = "W:\Bitcoin" }

$Bitcoind = $null
foreach ($cand in @(
        (Join-Safe $Datadir "bin\daemon\bitcoind.exe"),
        "W:\Bitcoin\bin\daemon\bitcoind.exe",
        "C:\Program Files\Bitcoin\daemon\bitcoind.exe"
    )) {
    if (Test-DrivePath $cand) {
        $Bitcoind = $cand
        break
    }
}

Set-Location $Project
Write-Host ""
Write-Host "=== BTC Solver start ===" -ForegroundColor Cyan
Write-Host "[Path] project = $Project" -ForegroundColor DarkCyan

# 1) Core (optional)
$btcProc = Get-Process bitcoind -ErrorAction SilentlyContinue
if (-not $btcProc) {
    if ($Bitcoind -and (Test-DrivePath $Bitcoind) -and (Test-DrivePath $Datadir)) {
        Remove-Item -Force (Join-Safe $Datadir ".lock"), (Join-Safe $Datadir "blocks\.lock"), (Join-Safe $Datadir "bitcoind.pid") -ErrorAction SilentlyContinue
        Write-Host "[Core] starting bitcoind ($Datadir)..." -ForegroundColor Cyan
        Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir" -WindowStyle Hidden
        Start-Sleep -Seconds 4
    } else {
        Write-Host "[Core] skipped (bitcoind or datadir not found) - dashboard only" -ForegroundColor Yellow
    }
} else {
    Write-Host "[Core] already running PID=$($btcProc.Id)" -ForegroundColor Green
}

# 2) Dashboard exe
$candidates = @(
    (Join-Safe $Project "target\release\btcsolver_dashboard.exe"),
    (Join-Safe $Project "btcsolver_dashboard.exe")
)
$exe = $candidates | Where-Object { Test-DrivePath $_ } | Select-Object -First 1
if (-not $exe) {
    Write-Host "Building dashboard..." -ForegroundColor Cyan
    cargo build --release --bin btcsolver_dashboard
    $exe = Join-Safe $Project "target\release\btcsolver_dashboard.exe"
}
if (-not (Test-DrivePath $exe)) {
    Write-Error "Dashboard binary missing"
    exit 1
}

# Free port
Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue |
    Select-Object -ExpandProperty OwningProcess -Unique |
    ForEach-Object {
        if ($_ -and $_ -ne 0) {
            Write-Host "[Dash] free port $Port PID=$_" -ForegroundColor Yellow
            Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue
        }
    }
Start-Sleep -Seconds 1

# Snapshot: prefer local cache (fast), then project data/, then repo root
$Snap = $null
foreach ($cand in @(
        "C:\btcsolver-cache\utxo-index.snapshot",
        (Join-Safe $Project "data\utxo-day-935000.snapshot"),
        (Join-Safe $Project "data\utxo-active.snapshot"),
        (Join-Safe $Project "utxo-index.snapshot"),
        (Join-Safe $Project "data\utxo-index.snapshot")
    )) {
    if (Test-DrivePath $cand) {
        $Snap = $cand
        break
    }
}
if (-not $Snap) {
    $Snap = Join-Safe $Project "utxo-index.snapshot"
    Write-Host "[UTXO] no snapshot found yet - index empty until rebuild" -ForegroundColor Yellow
} else {
    $mb = [math]::Round((Get-Item -LiteralPath $Snap).Length / 1MB, 0)
    Write-Host "[UTXO] snapshot = $Snap ($mb MB)" -ForegroundColor DarkCyan
}

$blocksDir = Join-Safe $Datadir "blocks"

$dashArgs = New-Object System.Collections.Generic.List[string]
[void]$dashArgs.AddRange([string[]]@(
    "--port", "$Port",
    "--bitcoin-datadir", "$Datadir",
    "--blocks-dir", "$blocksDir",
    "--blocks-obf-key", "0000000000000000",
    "--rpc-user", "btcsolver",
    "--rpc-password", "btcsolver_rpc_2026",
    "--project-dir", "$Project",
    "--bin-dir", "$Project",
    "--cache-dir", "$Project",
    "--snapshot-path", "$Snap",
    "--static-dir", (Join-Safe $Project "static\dashboard"),
    "--max-snapshot-age", "0",
    "--auto-restart-check-secs", "20"
))
if ($Bitcoind) {
    [void]$dashArgs.Add("--bitcoind-path")
    [void]$dashArgs.Add("$Bitcoind")
}
if (-not $Bitcoind) {
    [void]$dashArgs.Add("--auto-restart-bitcoind")
    [void]$dashArgs.Add("false")
}

Write-Host "[Dash] starting $exe" -ForegroundColor Cyan
Start-Process -FilePath $exe -ArgumentList $dashArgs.ToArray() -WorkingDirectory $Project -WindowStyle Hidden

$ok = $false
for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Seconds 1
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/api/system/health" -UseBasicParsing -TimeoutSec 2
        if ($r.StatusCode -eq 200) { $ok = $true; break }
    } catch {}
}

if ($ok) {
    Write-Host "[Dash] OK http://127.0.0.1:$Port/" -ForegroundColor Green
    Start-Process "http://127.0.0.1:$Port/"
} else {
    Write-Host "[Dash] HTTP not ready yet - open http://127.0.0.1:$Port/" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Note: leave it running. Use Relancer only if Core is STOPPED." -ForegroundColor DarkCyan
Write-Host "Done."
