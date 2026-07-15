# BTC Solver - Live Monitoring Console
# Reads stats files and displays real-time information

$statsFile = "C:\btcsolver-bin\brute-force-stats.json"
$posFile = "C:\btcsolver-bin\brute-force-progress.position"

# Clear screen and set title
Clear-Host
$host.UI.RawUI.WindowTitle = "BTC Solver - Live Monitor"

# Set console colors
$host.UI.RawUI.BackgroundColor = "Black"
$host.UI.RawUI.ForegroundColor = "White"
Clear-Host

$lastUpdate = [DateTime]::Now
$frame = 0

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  BTC Solver - GPU Brute-Force Monitor" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Press Ctrl+C to stop monitoring" -ForegroundColor DarkGray
Write-Host ""

while ($true) {
    try {
        # Read stats JSON
        $stats = $null
        if (Test-Path $statsFile) {
            $stats = Get-Content $statsFile -Raw | ConvertFrom-Json
        }

        # Read position file
        $position = ""
        $posKey = ""
        $posKeysTested = 0
        if (Test-Path $posFile) {
            $position = Get-Content $posFile -Raw
            $parts = $position.Trim().Split(' ')
            if ($parts.Count -ge 2) {
                $posKey = $parts[1]
                $posKeysTested = [long]::Parse($parts[2])
            }
        }

        # Read GPU info
        $gpuInfo = ""
        try {
            $gpuInfo = nvidia-smi --query-gpu=utilization.gpu,utilization.memory,temperature.gpu,memory.used,memory.total --format=csv,noheader 2>$null
        } catch {}

        # Read process info
        $procInfo = ""
        try {
            $proc = Get-Process brute_force -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($proc) {
                $memMB = [math]::Round($proc.WorkingSet64 / 1MB, 1)
                $elapsed = $proc.TotalProcessorTime.ToString("hh\:mm\:ss")
                $procInfo = "PID: $($proc.Id) | RAM: ${memMB} MB | CPU Time: $elapsed"
            }
        } catch {}

        # Clear and redraw
        Clear-Host
        $now = [DateTime]::Now.ToString("HH:mm:ss")

        Write-Host "============================================================" -ForegroundColor Cyan
        Write-Host "  BTC Solver - GPU Brute-Force Monitor" -ForegroundColor Cyan
        Write-Host "  Updated: $now" -ForegroundColor DarkGray
        Write-Host "============================================================" -ForegroundColor Cyan
        Write-Host ""

        # Stats section
        Write-Host "  [STATS]" -ForegroundColor Yellow
        if ($stats) {
            $keysTested = [long]::Parse($stats.keys_tested)
            $keysPerSec = [long]::Parse($stats.keys_per_sec)
            $matches = $stats.matches_found
            $elapsedSec = [long]::Parse($stats.elapsed_seconds)

            # Format numbers with commas
            $keysStr = "{0:N0}" -f $keysTested
            $rateStr = "{0:N0}" -f $keysPerSec

            Write-Host "    Clés testées : " -NoNewline -ForegroundColor White
            Write-Host "$keysStr" -ForegroundColor Green
            Write-Host "    Vitesse      : " -NoNewline -ForegroundColor White
            Write-Host "$rateStr keys/sec" -ForegroundColor Green
            Write-Host "    Matches      : " -NoNewline -ForegroundColor White
            if ([int]$matches -gt 0) {
                Write-Host "$matches" -ForegroundColor Magenta
            } else {
                Write-Host "0" -ForegroundColor DarkGray
            }
            Write-Host "    Temps        : " -NoNewline -ForegroundColor White
            Write-Host "$($elapsedSec)s" -ForegroundColor Gray
        } else {
            Write-Host "    (aucune donnee)" -ForegroundColor Red
        }

        Write-Host ""

        # Position section
        Write-Host "  [POSITION]" -ForegroundColor Yellow
        if ($posKey) {
            Write-Host "    Derniere cle : " -NoNewline -ForegroundColor White
            Write-Host "$posKey" -ForegroundColor Cyan

            # Show last 16 chars highlighted
            $shortKey = $posKey.Substring([Math]::Max(0, $posKey.Length - 16))
            Write-Host "    Court        : 0...$shortKey" -ForegroundColor DarkGray

            $posStr = "{0:N0}" -f $posKeysTested
            Write-Host "    Clefs total  : " -NoNewline -ForegroundColor White
            Write-Host "$posStr" -ForegroundColor Green
        } else {
            Write-Host "    (pas de position)" -ForegroundColor Red
        }

        Write-Host ""

        # GPU section
        Write-Host "  [GPU]" -ForegroundColor Yellow
        if ($gpuInfo) {
            $gpuParts = $gpuInfo.Trim().Split(',')
            if ($gpuParts.Count -ge 5) {
                $gpuUtil = $gpuParts[0].Trim()
                $memUtil = $gpuParts[1].Trim()
                $temp = $gpuParts[2].Trim()
                $memUsed = $gpuParts[3].Trim()
                $memTotal = $gpuParts[4].Trim()

                Write-Host "    Utilisation  : " -NoNewline -ForegroundColor White
                Write-Host "$gpuUtil" -ForegroundColor Green
                Write-Host "    Temperature  : " -NoNewline -ForegroundColor White
                Write-Host "$temp" -ForegroundColor $(if ([int]$temp -gt 85) { "Red" } else { "Green" })
                Write-Host "    VRAM         : " -NoNewline -ForegroundColor White
                Write-Host "$memUsed / $memTotal" -ForegroundColor Green
            }
        } else {
            Write-Host "    (nvidia-smi non disponible)" -ForegroundColor Red
        }

        Write-Host ""

        # Process section
        Write-Host "  [PROCESSUS]" -ForegroundColor Yellow
        if ($procInfo) {
            Write-Host "    $procInfo" -ForegroundColor Gray
        } else {
            Write-Host "    (brute_force non trouve)" -ForegroundColor Red
        }

        Write-Host ""
        Write-Host "------------------------------------------------------------" -ForegroundColor DarkGray
        Write-Host "  Stats file : $statsFile" -ForegroundColor DarkGray
        Write-Host "  Position   : $posFile" -ForegroundColor DarkGray
        Write-Host "------------------------------------------------------------" -ForegroundColor DarkGray

        # Wait 2 seconds before next update
        Start-Sleep -Seconds 2

    } catch {
        Write-Host "Error: $_" -ForegroundColor Red
        Start-Sleep -Seconds 5
    }
}
