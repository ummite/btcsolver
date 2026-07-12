$cookie = Get-Content "Y:\Bitcoin\.cookie" -Raw
$trimmed = $cookie.Trim()
$token = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($trimmed))
$header = @{Authorization = "Basic $token"}
$body = '{"jsonrpc":"1.0","id":"test","method":"getblockchaininfo","params":[]}'
try {
    $resp = Invoke-RestMethod -Uri "http://127.0.0.1:8332" -Method Post -ContentType "application/json" -Headers $header -Body $body
    Write-Output "Blocks: $($resp.blocks)"
    Write-Output "Verification progress: $($resp.verificationprogress)"
    Write-Output "Chain: $($resp.chain)"
    if ([double]$resp.verificationprogress -ge 0.999) {
        Write-Output "STATUS: SYNCHRONISE ✅"
    } else {
        Write-Output "STATUS: EN COURS DE SYNCHRO ❌ ($([math]::Round([double]$resp.verificationprogress * 100, 2))%)"
    }
} catch {
    Write-Output "ERREUR: bitcoind ne semble pas tourner ou RPC inaccessible"
    Write-Output $_.Exception.Message
}
