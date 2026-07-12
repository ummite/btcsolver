$debug_log = "Y:\Bitcoin\debug.log"
if (Test-Path $debug_log) {
    $size = (Get-Item $debug_log).Length
    Write-Output "debug.log existe: $([math]::Round($size / 1MB, 1)) Mo"
    $last_lines = Get-Content $debug_log -Tail 20
    Write-Output "--- Dernières lignes ---"
    $last_lines
} else {
    Write-Output "debug.log introuvable"
}
