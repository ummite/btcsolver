# BTC Solver - start dashboard + Bitcoin Core on W:\
$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Port = 3000
$Datadir = "W:\Bitcoin"
$Bitcoind = "W:\Bitcoin\bin\daemon\bitcoind.exe"

Set-Location $Project
Write-Host ""
Write-Host "=== BTC Solver start ===" -ForegroundColor Cyan

# 1) Core
$btcProc = Get-Process bitcoind -ErrorAction SilentlyContinue
if (-not $btcProc) {
    Remove-Item -Force "$Datadir\.lock","$Datadir\blocks\.lock","$Datadir\bitcoind.pid" -ErrorAction SilentlyContinue
    Write-Host "[Core] not running - cleaned locks" -ForegroundColor Yellow
    if (Test-Path $Bitcoind) {
        Write-Host "[Core] starting bitcoind..." -ForegroundColor Cyan
        Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir" -WindowStyle Hidden
        Start-Sleep -Seconds 4
    } else {
        Write-Host "[Core] ERROR missing $Bitcoind" -ForegroundColor Red
    }
} else {
    Write-Host "[Core] already running PID=$($btcProc.Id)" -ForegroundColor Green
}

# 2) Dashboard exe
$candidates = @(
    "$Project\target\release\btcsolver_dashboard.exe",
    "$Project\btcsolver_dashboard.exe"
)
$exe = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $exe) {
    Write-Host "Building dashboard..." -ForegroundColor Cyan
    cargo build --release --bin btcsolver_dashboard
    $exe = "$Project\target\release\btcsolver_dashboard.exe"
}
if (-not (Test-Path $exe)) {
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

# Prefer full-day offline snapshot when present
$Snap = "$Project\data\utxo-day-935000.snapshot"
if (-not (Test-Path $Snap)) { $Snap = "$Project\utxo-index.snapshot" }

$dashArgs = @(
    "--port", "$Port",
    "--bitcoin-datadir", $Datadir,
    "--bitcoind-path", $Bitcoind,
    "--blocks-dir", "$Datadir\blocks",
    "--blocks-obf-key", "0000000000000000",
    "--rpc-user", "btcsolver",
    "--rpc-password", "btcsolver_rpc_2026",
    "--project-dir", $Project,
    "--bin-dir", $Project,
    "--cache-dir", $Project,
    "--snapshot-path", $Snap,
    "--static-dir", "$Project\static\dashboard",
    "--max-snapshot-age", "0",
    "--auto-restart-check-secs", "20"
)

Write-Host "[Dash] starting $exe" -ForegroundColor Cyan
Start-Process -FilePath $exe -ArgumentList $dashArgs -WorkingDirectory $Project -WindowStyle Hidden

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
Write-Host "Note: A jour NON = sync not finished (normal). Leave it running." -ForegroundColor DarkCyan
Write-Host "Use Relancer only if Core is STOPPED." -ForegroundColor DarkCyan
Write-Host "Done."
