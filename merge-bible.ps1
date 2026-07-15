# Merge all Bible files and deduplicate
$all = @()
$all += Get-Content "Y:\btcsolver\bible-brainwallet-all.txt"
$all += Get-Content "Y:\btcsolver\bible-combined.txt"

# Remove empty lines and duplicates
$unique = $all | Where-Object { $_ -and $_.Trim() } | Sort-Object -Unique

Write-Host "Total unique patterns after merge: $($unique.Count)"
$unique | Out-File -FilePath "Y:\btcsolver\bible-full-corpus.txt" -Encoding UTF8
Write-Host "Written to bible-full-corpus.txt"
