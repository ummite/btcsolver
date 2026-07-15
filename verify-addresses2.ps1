# Verify addresses using multiple blockchain APIs

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
        $url = "https://blockchain.info/q/getbalance/$($a.Addr)"
        $headers = @{ "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" }
        $balance = Invoke-RestMethod -Uri $url -Headers $headers -TimeoutSec 15 -ErrorAction Stop
        Write-Host "  [blockchain.info] Balance (sats): $balance"
        Write-Host "  [blockchain.info] Balance (BTC):  $($balance / 100000000)"
    } catch {
        Write-Host "  [blockchain.info] Error: $($_.Exception.Message)"
    }

    # Try blockexplorer.com
    try {
        $url = "https://api.blockexplorer.com/api/addr/$($a.Addr)"
        $headers = @{ "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" }
        $response = Invoke-RestMethod -Uri $url -Headers $headers -TimeoutSec 15 -ErrorAction Stop
        if ($response.balance) {
            Write-Host "  [blockexplorer.com] Balance (sats): $($response.balance)"
            Write-Host "  [blockexplorer.com] Balance (BTC):  $($response.balance / 100000000)"
        }
    } catch {
        Write-Host "  [blockexplorer.com] Error: $($_.Exception.Message)"
    }

    # Try mempool.space
    try {
        $url = "https://mempool.space/api/address/$($a.Addr)"
        $headers = @{ "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" }
        $response = Invoke-RestMethod -Uri $url -Headers $headers -TimeoutSec 15 -ErrorAction Stop
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
