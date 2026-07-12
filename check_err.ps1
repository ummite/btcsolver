$err_log = "Y:\Bitcoin\dbver"
if (Test-Path $err_log) {
    Write-Output "dbver: $(Get-Content $err_log)"
}

$debug_log = "Y:\Bitcoin\debug.log"
if (Test-Path $debug_log) {
    $size = (Get-Item $debug_log).Length
    Write-Output "debug.log: $([math]::Round($size / 1MB, 1)) Mo"
    Write-Output "--- 10 dernieres lignes ---"
    Get-Content $debug_log -Tail 10
}

$conf = "Y:\Bitcoin\bitcoin.conf"
if (Test-Path $conf) {
    Write-Output "--- bitcoin.conf ---"
    Get-Content $conf
}
