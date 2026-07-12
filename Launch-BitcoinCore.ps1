# Launch-BitcoinCore.ps1
# Launch Bitcoin Core (GUI or bitcoind) with datadir on Y:\ (DS1821 24TB RAID0)

$datadir = "Y:\Bitcoin"
Write-Host "=== Launching Bitcoin Core with data on Y:\Bitcoin ===" -ForegroundColor Cyan
New-Item -ItemType Directory -Path $datadir -Force | Out-Null

$conf = Join-Path $datadir "bitcoin.conf"
if (-not (Test-Path $conf)) {
    @"
server=1
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
prune=0
dbcache=8192
"@ | Set-Content -Path $conf -Encoding ASCII
    Write-Host "Created default config at $conf"
}

# Search for exe (GUI preferred for "wallet", or daemon)
Write-Host "Searching for Bitcoin Core executable..."
$exe = $null
$roots = @("Y:\bitcoin-31.1\bin", "C:\Program Files", "C:\Program Files (x86)", $env:LOCALAPPDATA)
foreach ($r in $roots) {
    if (Test-Path $r) {
        $c = Get-ChildItem $r -Recurse -Include "bitcoin-qt.exe","bitcoind.exe" -ErrorAction SilentlyContinue -Depth 6 | Select-Object -First 1
        if ($c) { $exe = $c.FullName; break }
    }
}

if (-not $exe) {
    Write-Error "Bitcoin Core exe not found. Install from bitcoincore.org and re-run this script."
    exit
}

Write-Host "Using: $exe"
Write-Host "All files will go to $datadir (blocks, chainstate, etc.)"

$args = @("-datadir=$datadir")
if ($exe -like "*bitcoind*") {
    $args += "-server"
    Start-Process $exe -ArgumentList $args -WindowStyle Hidden
    Write-Host "bitcoind launched (headless). Check status with: bitcoin-cli -datadir=$datadir getblockchaininfo"
} else {
    Start-Process $exe -ArgumentList $args
    Write-Host "Bitcoin Core GUI launched."
}
Write-Host "Done. Data dir: $datadir"
