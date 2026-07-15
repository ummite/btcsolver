foreach ($f in Get-ChildItem Y:\btcsolver\bible*.txt) {
    $lines = (Get-Content $f.FullName).Count
    $kb = [math]::Round($f.Length / 1024, 1)
    Write-Host "$($f.Name) : $($kb) KB, $lines lines"
}
