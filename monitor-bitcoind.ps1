# Monitor Bitcoin Core sync progress until fully synced
$cli = "C:\Program Files\Bitcoin\daemon\bitcoin-cli.exe"
$datadir = "C:\Bitcoin"
$log = "C:\Bitcoin\debug.log"

Write-Host "Monitoring Bitcoin Core until fully synced..."
Write-Host "Press Ctrl+C to stop monitoring (bitcoind keeps running)"
Write-Host ""

while ($true) {
    $proc = Get-Process bitcoind -ErrorAction SilentlyContinue
    if (-not $proc) {
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] bitcoind NOT RUNNING - check $log"
        Start-Sleep -Seconds 30
        continue
    }

    $ram = [math]::Round($proc.WorkingSet / 1GB, 2)
    $info = & $cli -datadir=$datadir getblockchaininfo 2>$null | ConvertFrom-Json -ErrorAction SilentlyContinue

    if ($info) {
        $height = $info.blocks
        $headers = $info.headers
        $progress = [math]::Round($info.verificationprogress * 100, 3)
        $ibd = $info.initialblockdownload
        Write-Host ("[{0}] height={1}/{2} progress={3}% IBD={4} RAM={5}GB" -f (Get-Date -Format 'HH:mm:ss'), $height, $headers, $progress, $ibd, $ram)

        if (-not $ibd -and $height -ge $headers -and $progress -ge 99.9) {
            Write-Host ""
            Write-Host "=== BITCOIN CORE FULLY SYNCED ==="
            Write-Host "Blocks: $height"
            Write-Host "Ready for UTXO index update."
            exit 0
        }
    } else {
        # During reindex, RPC may not be ready; fall back to log tail
        $last = Get-Content $log -Tail 1 -ErrorAction SilentlyContinue
        if ($last -match "Reindexing block file (blk\d+\.dat) \((\d+)%") {
            Write-Host ("[{0}] REINDEX {1} ({2}%) RAM={3}GB" -f (Get-Date -Format 'HH:mm:ss'), $Matches[1], $Matches[2], $ram)
        } elseif ($last -match "UpdateTip:.*height=(\d+).*progress=([\d.]+)") {
            Write-Host ("[{0}] VERIFY height={1} progress={2} RAM={3}GB" -f (Get-Date -Format 'HH:mm:ss'), $Matches[1], $Matches[2], $ram)
        } else {
            Write-Host ("[{0}] starting... RAM={1}GB | {2}" -f (Get-Date -Format 'HH:mm:ss'), $ram, $last)
        }
    }

    Start-Sleep -Seconds 30
}
