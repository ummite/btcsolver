# Enable-PrioritySync.ps1 — priorite absolue: Core tip + UTXO tip, puis cles.
$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Flag = "$Project\data\PRIORITY-SYNC.flag"
$Datadir = "W:\Bitcoin"
$Cli = "$Datadir\bin\daemon\bitcoin-cli.exe"

if (-not (Test-Path "$Project\data")) { New-Item -ItemType Directory -Path "$Project\data" -Force | Out-Null }

@"
PRIORITY-SYNC active
Goal: Bitcoin Core tip + UTXO tip first. Key hunting DISABLED until flag removed.
Created: $((Get-Date).ToString('o'))
Remove this file (or run Disable-PrioritySync.ps1) only when Core is at tip AND UTXO lag < 24h.
"@ | Set-Content -Path $Flag -Encoding ASCII

# Stop key hunting immediately
Get-Process -Name brute_force -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
try {
    Invoke-WebRequest "http://127.0.0.1:3000/api/scan/stop" -Method POST -UseBasicParsing -TimeoutSec 8 | Out-Null
} catch {}
try {
    Invoke-WebRequest "http://127.0.0.1:3000/api/dict/stop" -Method POST -UseBasicParsing -TimeoutSec 8 | Out-Null
} catch {}

# Ensure Core + status
& powershell -NoProfile -ExecutionPolicy Bypass -File "$Project\Keep-Core-And-Utxo.ps1"
& powershell -NoProfile -ExecutionPolicy Bypass -File "$Project\Watch-BtcSolver.ps1"

Write-Host ""
Write-Host "=== PRIORITY-SYNC ACTIVE ===" -ForegroundColor Cyan
Write-Host "  - brute_force / scans cles: OFF"
Write-Host "  - bitcoind: always-on (ne pas tuer)"
Write-Host "  - UTXO tip: auto des que Core at_tip"
Write-Host "  - flag: $Flag"
Write-Host ""
if (Test-Path "$Project\data\CORE-UTXO-STATUS.json") {
    Get-Content "$Project\data\CORE-UTXO-STATUS.json" -Raw
}
if (Test-Path $Cli) {
    try {
        $info = & $Cli -datadir=$Datadir -rpcclienttimeout=15 getblockchaininfo 2>$null | ConvertFrom-Json
        Write-Host ("Core: blocks={0} headers={1} ibd={2} progress={3:P2}" -f $info.blocks, $info.headers, $info.initialblockdownload, $info.verificationprogress)
    } catch {}
}
Write-Host ""
Write-Host "Pour desactiver (apres tip + UTXO frais): .\Disable-PrioritySync.ps1"
