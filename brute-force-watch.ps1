# BTCSolver - Brute-force with real-time monitoring and auto-stop
# Launches brute_force.exe and watches for balance matches
# Saves found keys to found-keys.json

$ErrorActionPreference = "Stop"

# Get DB path
$cacheDir = & "$PSScriptRoot\cache_manager.exe" path 2>$null
$localIndex = Join-Path $cacheDir "utxo-index.redb"

if (Test-Path $localIndex) {
    $dbPath = $localIndex
    Write-Host "Using LOCAL cache: $dbPath" -ForegroundColor Green
} elseif (Test-Path "Y:\btcsolver\utxo-index.redb") {
    $dbPath = "Y:\btcsolver\utxo-index.redb"
    Write-Host "Using SAN index: $dbPath" -ForegroundColor Yellow
} else {
    Write-Host "ERROR: No index found!" -ForegroundColor Red
    Write-Host "Run sync-cache.bat init or wait for build-index.bat" -ForegroundColor Red
    pause
    exit 1
}

Write-Host "========================================"
Write-Host " BTCSolver - Brute-Force Auto-Stop"
Write-Host " DB: $dbPath"
Write-Host " Results: found-keys.json"
Write-Host "========================================"
Write-Host ""

$resultsFile = "$PSScriptRoot\found-keys.json"
$startTime = Get-Date

# Launch brute_force.exe
$process = Start-Process -FilePath "$PSScriptRoot\brute_force.exe" `
    -ArgumentList "--random", "--db-path", "`"$dbPath`"" `
    -NoNewWindow -PassThru -RedirectStandardError "$PSScriptRoot\brute-force.log"

Write-Host "Started brute_force.exe (PID: $($process.Id))" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop manually" -ForegroundColor Gray
Write-Host ""

$found = $false
$elapsed = 0

try {
    while ($process.HasExited -eq $false) {
        Start-Sleep -Seconds 5

        $elapsed = (Get-Date) - $startTime

        # Check stderr log for match count
        $logContent = ""
        if (Test-Path "$PSScriptRoot\brute-force.log") {
            $logContent = Get-Content "$PSScriptRoot\brute-force.log" -Tail 5
        }

        # Parse the last progress line for match count
        $matchCount = 0
        foreach ($line in $logContent) {
            if ($line -match '\| (\d+) matches') {
                $matchCount = [int]$matches[1]
            }
        }

        if ($matchCount -gt 0) {
            $found = $true
            Write-Host ""
            Write-Host "!!! BALANCE FOUND !!!" -ForegroundColor Red -BackgroundColor White
            Write-Host "Matches: $matchCount" -ForegroundColor Red
            Write-Host "Time: $($elapsed.TotalMinutes.ToString('0.0')) minutes" -ForegroundColor Red
            Write-Host ""

            # Stop the process
            $process.Kill()
            break
        }

        # Show progress
        $minutes = $elapsed.TotalMinutes.ToString('0.0')
        $lastLine = $logContent | Select-Object -Last 1
        Write-Host "`r[$minutes min] $lastLine" -NoNewline -ForegroundColor DarkGray
    }
}
finally {
    # Ensure process is stopped
    if (-not $process.HasExited) {
        $process.Kill()
    }
}

$elapsed = (Get-Date) - $startTime

Write-Host ""
Write-Host "========================================"
Write-Host " Session ended after $($elapsed.TotalMinutes.ToString('0.0')) minutes"

if ($found) {
    Write-Host " STATUS: BALANCE FOUND!" -ForegroundColor Red -BackgroundColor White
    Write-Host ""
    Write-Host "Full log: brute-force.log" -ForegroundColor Yellow
    Write-Host ""

    # Extract results from log
    Write-Host "=== Log content (last 50 lines) ===" -ForegroundColor Cyan
    Get-Content "$PSScriptRoot\brute-force.log" -Tail 50
} else {
    Write-Host " STATUS: No balance found" -ForegroundColor Yellow
}

Write-Host "========================================"
pause
