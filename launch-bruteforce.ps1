# BTCSolver - Launch brute-force with auto-stop on balance
# Works with indexer v4 (lock-free between checkpoints)

$ErrorActionPreference = "Stop"
$DbPath = "Y:\btcsolver\utxo-index.redb"
$ExePath = "Y:\btcsolver\brute_force.exe"
$OutputFile = "Y:\btcsolver\found-keys.json"
$Threads = 16
$WaitInterval = 5
$MaxWait = 120  # Wait up to 2 min for index to exist

Write-Host "BTCSolver - Brute-Force Launcher" -ForegroundColor Cyan
Write-Host ""

# Wait for index to exist
$waited = 0
while (-not (Test-Path $DbPath)) {
    Write-Host "[$($waited)s] Waiting for index to be created..." -ForegroundColor Yellow
    Start-Sleep -Seconds $WaitInterval
    $waited += $WaitInterval
    if ($waited -ge $MaxWait) {
        Write-Host "ERROR: Index not created after ${MaxWait}s" -ForegroundColor Red
        exit 1
    }
}

$dbSize = ((Get-Item $DbPath) | Select-Object -First 1).Length
$dbSizeMB = [math]::Round(($dbSize / 1MB), 1)
Write-Host "Index found ($dbSizeMB MB)" -ForegroundColor Green
Write-Host ""

# Try to launch brute-force (may fail if indexer is writing checkpoint)
$maxRetries = 6
$retry = 0
$launched = $false

while ($retry -lt $maxRetries -and -not $launched) {
    $retry++
    try {
        Write-Host "Launch attempt $retry/$maxRetries..." -ForegroundColor Cyan
        
        $proc = Start-Process -FilePath $ExePath `
            -ArgumentList "--db-path $DbPath --threads $Threads --random --stop-on-match --output-file $OutputFile" `
            -NoNewWindow `
            -PassThru `
            -RedirectStandardError "Y:\btcsolver\brute-force-error.log"
        
        Write-Host "Brute-force launched! PID: $($proc.Id)" -ForegroundColor Green
        Write-Host "  Threads: $Threads" -ForegroundColor Green
        Write-Host "  Mode: Random keys (cryptographic RNG)" -ForegroundColor Green
        Write-Host "  Auto-stop: ON (saves to $OutputFile)" -ForegroundColor Green
        Write-Host ""
        Write-Host "Monitoring for results..." -ForegroundColor Yellow
        
        # Monitor for found-keys.json
        while ($true) {
            if (Test-Path $OutputFile) {
                Write-Host ""
                Write-Host "========================================" -ForegroundColor Green
                Write-Host "  FOUND KEY WITH BALANCE!" -ForegroundColor Green
                Write-Host "========================================" -ForegroundColor Green
                Write-Host ""
                Get-Content $OutputFile
                Write-Host ""
                Write-Host "Results saved to: $OutputFile" -ForegroundColor Green
                
                # Stop the brute-force process
                Get-Process brute_force -ErrorAction SilentlyContinue | Stop-Process -Force
                Write-Host "Brute-force process stopped." -ForegroundColor Yellow
                break
            }
            
            # Check if process is still running
            $bfProc = Get-Process brute_force -ErrorAction SilentlyContinue
            if (-not $bfProc) {
                Write-Host "Brute-force process ended." -ForegroundColor Yellow
                if (Test-Path "Y:\btcsolver\brute-force-error.log") {
                    Write-Host "Error log:" -ForegroundColor Red
                    Get-Content "Y:\btcsolver\brute-force-error.log" -Tail 5
                }
                break
            }
            
            Start-Sleep -Seconds 10
        }
        $launched = $true
    }
    catch {
        Write-Host "  Failed: $_" -ForegroundColor Red
        Write-Host "  Indexer likely writing checkpoint. Retrying in ${WaitInterval}s..." -ForegroundColor Yellow
        Start-Sleep -Seconds $WaitInterval
    }
}

if (-not $launched) {
    Write-Host "ERROR: Could not launch brute-force after $maxRetries attempts" -ForegroundColor Red
    Write-Host "The indexer may be holding an exclusive lock." -ForegroundColor Red
    exit 1
}
