$body = Get-Content Y:\btcsolver\data\scan-restart.json -Raw
$response = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/scan/start' -Method POST -ContentType 'application/json' -Body $body
$response | ConvertTo-Json
