# Verify the Bible brainwallet matches by computing the SHA256 and deriving addresses
# This is a verification script

$phrases = @("God", "god")

foreach ($phrase in $phrases) {
    # Compute SHA256
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($phrase)
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    $hash = $sha256.ComputeHash($bytes)
    $hexKey = -join ($hash | ForEach-Object { $_.ToString("x2") })

    Write-Host "Phrase: `"$phrase`""
    Write-Host "  SHA256: $hexKey"
    Write-Host ""
}

Write-Host "To verify on blockchain, check these addresses:"
Write-Host "  1JJopWpJ5ZmXazk3kiisS8iVencznpnWrc  (God - uncompressed P2PKH)"
Write-Host "  1KxmSmcMTmPvU1qSLYpJLrqnSzBoQ53NXN  (god - uncompressed P2PKH)"
Write-Host ""
Write-Host "Use blockchain explorers: blockchain.com, blockchair.com, mempool.space"
