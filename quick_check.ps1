$cookie = Get-Content "Y:\Bitcoin\.cookie" -Raw
$trimmed = $cookie.Trim()
$token = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($trimmed))
$header = @{Authorization = "Basic $token"}
$body = '{"jsonrpc":"1.0","id":"quick","method":"getblockchaininfo","params":[]}'
try {
    $resp = Invoke-RestMethod -Uri "http://127.0.0.1:8332" -Method Post -ContentType "application/json" -Headers $header -Body $body
    Write-Output "Blocs: $($resp.blocks)"
    Write-Output "Headers: $($resp.headers)"
    Write-Output "Progress: $([math]::Round([double]$resp.verificationprogress * 100, 2))%"
} catch {
    Write-Output "Pas pret: $_.Exception.Message"
}
