# Verify the 7 matches on mempool.space

$phrases = @("", "1", "test", "market", "password", "to be or not to be", "satoshinakamoto")

# First compute the SHA256 + uncompressed P2PKH addresses
foreach ($phrase in $phrases) {
    $displayPhrase = if ($phrase -eq "") { '(empty string)' } else { $phrase }

    # SHA256
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($phrase)
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    $hash = $sha256.ComputeHash($bytes)
    $hexKey = -join ($hash | ForEach-Object { $_.ToString("x2") })

    Write-Host "Phrase: `"$displayPhrase`""
    Write-Host "  SHA256: $hexKey"
    Write-Host ""
}

Write-Host "============================================"
Write-Host "To verify: use check_keys or an explorer"
Write-Host "These are UNCOMPRESSED P2PKH addresses"
Write-Host "The FlatIndex bug may have inflated the values"
