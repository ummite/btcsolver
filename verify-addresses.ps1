# Verify addresses on blockchain explorers
$addresses = @(
    @{Name = "God uncompressed P2PKH"; Addr = "1JJopWpJ5ZmXazk3kiisS8iVencznpnWrc"},
    @{Name = "god uncompressed P2PKH"; Addr = "1KxmSmcMTmPvU1qSLYpJLrqnSzBoQ53NXN"},
    @{Name = "God compressed P2PKH"; Addr = "13nBobQswNJQjVyyBhaPeVsBDu3ZyLYVqR"}
)

foreach ($a in $addresses) {
    Write-Host "============================================"
    Write-Host "Checking: $($a.Name)"
    Write-Host "Address: $($a.Addr)"
    Write-Host ""

    # Try blockchain.info
    try {
        $url = "https://blockchain.info/q/addressbalance/$($a.Addr)"
        $headers = @{ "User-Agent" = "Mozilla/5.0" }
        $balance = Invoke-RestMethod -Uri $url -Headers $headers -TimeoutSec 10 -ErrorAction Stop
        Write-Host "  [blockchain.info] Balance (sats): $balance"
        Write-Host "  [blockchain.info] Balance (BTC):  $($balance / 100000000)"
    } catch {
        Write-Host "  [blockchain.info] Error: $($_.Exception.Message)"
    }

    # Try mempool.space
    try {
        $url = "https://mempool.space/api/address/$($a.Addr)"
        $headers = @{ "User-Agent" = "Mozilla/5.0" }
        $response = Invoke-RestMethod -Uri $url -Headers $headers -TimeoutSec 10 -ErrorAction Stop
        if ($response.chain_stats) {
            $received = $response.chain_stats.funded_txo_sum
            $spent = $response.chain_stats.spent_txo_sum
            $bal = $received - $spent
            Write-Host "  [mempool.space] Received (BTC): $($received / 100000000)"
            Write-Host "  [mempool.space] Spent (BTC):     $($spent / 100000000)"
            Write-Host "  [mempool.space] Balance (BTC):   $($bal / 100000000)"
        }
    } catch {
        Write-Host "  [mempool.space] Error: $($_.Exception.Message)"
    }

    Write-Host ""
}
