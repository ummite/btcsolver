# Stop current scan
Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/scan/stop' -Method POST | Out-Null
Start-Sleep -Seconds 3

# Kill brute_force if still running
Get-Process brute_force -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# Deploy new binary
Copy-Item -Force Y:\btcsolver\target\release\brute_force.exe Y:\btcsolver\brute_force.exe
Write-Output "Deployed brute_force.exe"

# Restart scan with larger batch
$body = Get-Content Y:\btcsolver\data\scan-restart.json -Raw
$response = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/scan/start' -Method POST -ContentType 'application/json' -Body $body
Write-Output ("Restarted scan with PID: " + $response.pid)
