$genesis = Get-Date "2009-01-03T09:15:05Z"

function Get-BlockDate {
    param([int]$height)
    # Each block ~10 minutes on average
    $minutes = $height * 10
    return ($genesis.AddMinutes($minutes)).ToString("yyyy-MM-dd HH:mm")
}

$counter = 0

while ($true) {
    $counter++
    $time = Get-Date -Format "HH:mm:ss"
    $cookie_path = "Y:\Bitcoin\.cookie"
    $rpc_ok = $false
    $blocks = $null
    $headers = $null
    $progress = $null

    # Try RPC first
    if (Test-Path $cookie_path) {
        try {
            $cookie = Get-Content $cookie_path -Raw
            $trimmed = $cookie.Trim()
            $token = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($trimmed))
            $header = @{Authorization = "Basic $token"}
            $body = '{"jsonrpc":"1.0","id":"monitor","method":"getblockchaininfo","params":[]}'
            $resp = Invoke-RestMethod -Uri "http://127.0.0.1:8332" -Method Post -ContentType "application/json" -Headers $header -Body $body -TimeoutSec 5
            if ($resp.blocks -and [int]$resp.blocks -gt 0) {
                $blocks = $resp.blocks
                $headers = $resp.headers
                $progress = [math]::Round([double]$resp.verificationprogress * 100, 2)
                $rpc_ok = $true
            }
        } catch { }
    }

    # Fallback: parse debug log for latest block height
    if (-not $rpc_ok) {
        try {
            $debug_log = "Y:\Bitcoin\debug.log"
            $lines = Get-Content $debug_log -Tail 50
            $match = $lines | Select-String "UpdateTip: new best=.* height=(\d+) " | Select-Object -Last 1
            if ($match) {
                $height_match = [regex]::Match($match.Line, "height=(\d+)")
                if ($height_match.Success) {
                    $blocks = [int]$height_match.Groups[1].Value
                    $progress = [math]::Round(($blocks / 957649.0) * 100, 2)
                }
            }
        } catch { }
    }

    if ($blocks) {
        $pct = if ($progress) { "$progress%" } else { "N/A" }
        $hdr = if ($headers) { "/ $headers" } else { "" }
        $block_date = Get-BlockDate $blocks

        $disk = ""
        $blocks_dir = "Y:\Bitcoin\blocks"
        if (Test-Path $blocks_dir) {
            $size = (Get-ChildItem $blocks_dir -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
            if ($size -gt 1GB) {
                $disk = " | Disque: $([math]::Round($size / 1GB, 1)) Go"
            } elseif ($size -gt 1MB) {
                $disk = " | Disque: $([math]::Round($size / 1MB, 1)) Mo"
            }
        }

        # Show date every 5 minutes (every 5th check)
        if ($counter % 5 -eq 0) {
            Write-Output "[$time] Bloc $blocks$hdr | $pct | 📅 ~$block_date$disk"
        } else {
            Write-Output "[$time] Bloc $blocks$hdr | $pct$disk"
        }

        if ([double]$progress -ge 99.9) {
            Write-Output ">>> SYNCHRONISATION COMPLETEE !!! <<<"
            break
        }
    } else {
        Write-Output "[$time] En attente de donnees..."
    }

    Start-Sleep -Seconds 60
}
