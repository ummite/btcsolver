# Launch-BitcoinCore.ps1
# Launch Bitcoin Core with datadir on W:\Bitcoin (local SwordFish — preferred)
# Legacy NAS copy: Y:\Bitcoin (do not use by default)

$ErrorActionPreference = "Stop"
$datadir = "W:\Bitcoin"
$preferredBitcoind = @(
    "W:\Bitcoin\bin\daemon\bitcoind.exe",
    "W:\Bitcoin\bin\bitcoind.exe",
    "W:\Bitcoin\bin\bitcoin-qt.exe"
)

Write-Host "=== Launching Bitcoin Core (datadir = $datadir) ===" -ForegroundColor Cyan

if (-not (Test-Path $datadir)) {
    Write-Error "Datadir not found: $datadir"
    exit 1
}

$existing = Get-Process -Name bitcoind, bitcoin-qt -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Bitcoin Core already running:" -ForegroundColor Yellow
    $existing | Format-Table Id, ProcessName, @{N = 'MB'; E = { [math]::Round($_.WorkingSet64 / 1MB) } } -AutoSize
    Write-Host "Status check:"
    $cli = "W:\Bitcoin\bin\daemon\bitcoin-cli.exe"
    if (Test-Path $cli) {
        & $cli "-datadir=$datadir" getblockchaininfo 2>$null
    }
    exit 0
}

# Stale lock only if no process
$lock = Join-Path $datadir ".lock"
if (Test-Path $lock) {
    Write-Host "Removing stale .lock (no bitcoind process)" -ForegroundColor Yellow
    Remove-Item -Force $lock, (Join-Path $datadir "blocks\.lock"), (Join-Path $datadir "bitcoind.pid") -ErrorAction SilentlyContinue
}

# blocksxor=0 requires NO xor.dat (blocks on W: are plaintext)
$xor = Join-Path $datadir "blocks\xor.dat"
$conf = Join-Path $datadir "bitcoin.conf"
if ((Test-Path $xor) -and (Test-Path $conf) -and (Select-String -Path $conf -Pattern "^\s*blocksxor\s*=\s*0" -Quiet)) {
    $bak = Join-Path $datadir ("blocks\xor.dat.bak-" + (Get-Date -Format "yyyyMMdd-HHmmss"))
    Write-Host "blocksxor=0 + xor.dat present (plaintext blocks) -> backup & remove xor.dat" -ForegroundColor Yellow
    Move-Item $xor $bak -Force
}

$exe = $preferredBitcoind | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $exe) {
    Write-Error "Bitcoin Core exe not found under W:\Bitcoin\bin. See W:\Bitcoin\README-PORTABLE.md"
    exit 1
}

Write-Host "Using: $exe"
Write-Host "Datadir: $datadir"

if ($exe -like "*bitcoind*") {
    Start-Process -FilePath $exe -ArgumentList "-datadir=$datadir" -WindowStyle Hidden
    Write-Host "bitcoind started (headless)."
    Start-Sleep -Seconds 8
    $cli = "W:\Bitcoin\bin\daemon\bitcoin-cli.exe"
    if (Test-Path $cli) {
        Write-Host "--- getblockchaininfo (may be empty while loading) ---"
        & $cli "-datadir=$datadir" getblockchaininfo 2>&1
    }
    Write-Host "Log: $datadir\debug.log"
    Write-Host "Stop: W:\Bitcoin\Stop-BitcoinCore.bat"
} else {
    Start-Process -FilePath $exe -ArgumentList "-datadir=$datadir"
    Write-Host "Bitcoin Core GUI launched."
}

Write-Host "Done."
