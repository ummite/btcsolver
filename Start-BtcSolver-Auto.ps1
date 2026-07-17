# Start-BtcSolver-Auto.ps1
# Demarre (ou maintient) Bitcoin Core + Dashboard + recherche brute-force.
# Idempotent : safe au login Windows et en relance manuelle.
# Log : Y:\btcsolver\startup-auto.log
#
# Usage:
#   .\Start-BtcSolver-Auto.ps1              # keep-alive (boucle)
#   .\Start-BtcSolver-Auto.ps1 -Once        # demarre une fois et sort
#   .\Start-BtcSolver-Auto.ps1 -OpenBrowser # + ouvre le navigateur

param(
    [switch]$Once,
    [switch]$OpenBrowser,
    [int]$LoopSeconds = 30
)

$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Datadir = "W:\Bitcoin"
$Bitcoind = "$Datadir\bin\daemon\bitcoind.exe"
$Port = 3000
$Log = "$Project\startup-auto.log"
$KeepAliveMarker = "$Project\.startup-keepalive.pid"

# Snapshot offline (prefer full day 935000)
$Snap = "$Project\data\utxo-day-935000.snapshot"
if (-not (Test-Path -LiteralPath $Snap)) {
    $Snap = "$Project\utxo-index.snapshot"
}

$Dash = "$Project\target\release\btcsolver_dashboard.exe"
if (-not (Test-Path -LiteralPath $Dash)) {
    $Dash = "$Project\btcsolver_dashboard.exe"
}

$Brute = "$Project\target\release\brute_force.exe"
if (-not (Test-Path -LiteralPath $Brute)) {
    $Brute = "$Project\brute_force.exe"
}

function Write-Log {
    param([string]$Message)
    $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $Message
    try {
        Add-Content -Path $Log -Value $line -Encoding UTF8
    } catch {
        # ignore log write errors
    }
    Write-Host $line
}

function Wait-Drives {
    # Au login, W: / Y: peuvent arriver quelques secondes apres
    $deadline = (Get-Date).AddMinutes(5)
    while ((Get-Date) -lt $deadline) {
        $wOk = Test-Path -LiteralPath "W:\"
        $yOk = Test-Path -LiteralPath "Y:\"
        if ($wOk -and $yOk) {
            return $true
        }
        Write-Log "Attente disques W:/Y: (W=$wOk Y=$yOk)..."
        Start-Sleep -Seconds 5
    }
    return ((Test-Path -LiteralPath "W:\") -and (Test-Path -LiteralPath "Y:\"))
}

function Test-DexorActive {
    # Skip Core only if a real de-XOR process is running (stale marker alone is ignored)
    $procs = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object {
        $_.CommandLine -and (
            $_.CommandLine -match "dexor_blocks" -or
            $_.CommandLine -match "de-xor" -or
            $_.CommandLine -match "tools\\dexor"
        )
    }
    if ($procs) { return $true }
    $marker = "$Project\.DEXOR_IN_PROGRESS"
    if (Test-Path -LiteralPath $marker) {
        $ageH = ((Get-Date) - (Get-Item -LiteralPath $marker).LastWriteTime).TotalHours
        if ($ageH -gt 0.5) {
            Write-Log "Marqueur .DEXOR_IN_PROGRESS obsolete (${ageH}h) - suppression"
            Remove-Item -LiteralPath $marker -Force -ErrorAction SilentlyContinue
        } else {
            # Marker recent but no process: still allow Core (safer for "make it work")
            Write-Log "Marqueur DEXOR present sans process - ignore et demarre Core"
            Remove-Item -LiteralPath $marker -Force -ErrorAction SilentlyContinue
        }
    }
    return $false
}

function Ensure-Core {
    if (Test-DexorActive) {
        Write-Log "de-XOR process actif - skip bitcoind"
        return
    }
    $existing = Get-Process -Name bitcoind, bitcoin-qt -ErrorAction SilentlyContinue
    if ($existing) {
        return
    }
    if (-not (Test-Path -LiteralPath $Bitcoind)) {
        Write-Log "ERREUR: bitcoind introuvable: $Bitcoind"
        return
    }

    Remove-Item -Force "$Datadir\.lock", "$Datadir\blocks\.lock", "$Datadir\bitcoind.pid" -ErrorAction SilentlyContinue

    $xor = "$Datadir\blocks\xor.dat"
    $conf = "$Datadir\bitcoin.conf"
    if ((Test-Path -LiteralPath $xor) -and (Test-Path -LiteralPath $conf)) {
        $hasBlocksxor0 = Select-String -Path $conf -Pattern "^\s*blocksxor\s*=\s*0" -Quiet
        if ($hasBlocksxor0) {
            $bak = "$Datadir\blocks\xor.dat.bak-" + (Get-Date -Format "yyyyMMdd-HHmmss")
            Write-Log "Backup/suppression xor.dat (blocksxor=0) -> $bak"
            Move-Item -LiteralPath $xor -Destination $bak -Force -ErrorAction SilentlyContinue
        }
    }

    Write-Log "Demarrage bitcoind (datadir=$Datadir)"
    Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir" -WindowStyle Hidden
    Start-Sleep -Seconds 6
}

function Ensure-Dashboard {
    $listening = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
    if ($listening) {
        return
    }
    if (-not (Test-Path -LiteralPath $Dash)) {
        Write-Log "ERREUR: dashboard manquant: $Dash"
        return
    }
    if (-not (Test-Path -LiteralPath $Snap)) {
        Write-Log "WARN: snapshot manquant $Snap - demarrage dashboard quand meme"
    }

    Write-Log "Demarrage dashboard :$Port snapshot=$Snap"
    $dashArgs = @(
        "--port", "$Port",
        "--bitcoin-datadir", $Datadir,
        "--bitcoind-path", $Bitcoind,
        "--blocks-dir", "$Datadir\blocks",
        "--blocks-obf-key", "0000000000000000",
        "--snapshot-path", $Snap,
        "--project-dir", $Project,
        "--bin-dir", $Project,
        "--cache-dir", $Project,
        "--static-dir", "$Project\static\dashboard",
        "--max-snapshot-age", "0",
        "--auto-restart-check-secs", "45",
        "--rpc-user", "btcsolver",
        "--rpc-password", "btcsolver_rpc_2026"
    )
    Start-Process -FilePath $Dash -ArgumentList $dashArgs -WorkingDirectory $Project -WindowStyle Hidden `
        -RedirectStandardOutput "$Project\dashboard.out.log" `
        -RedirectStandardError "$Project\dashboard.err.log"
}

function Ensure-Brute {
    $existing = Get-Process -Name brute_force -ErrorAction SilentlyContinue
    if ($existing) {
        return
    }
    if (-not (Test-Path -LiteralPath $Brute)) {
        Write-Log "ERREUR: brute_force manquant: $Brute"
        return
    }
    if (-not (Test-Path -LiteralPath $Snap)) {
        Write-Log "ERREUR: snapshot manquant pour la recherche: $Snap"
        return
    }

    $logical = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
    $threads = [Math]::Max(8, $logical - 6)

    Write-Log "Demarrage recherche brute_force (GPU, threads=$threads, snap=$Snap)"
    $bruteArgs = @(
        "--snapshot-path", $Snap,
        "--threads", "$threads",
        "--use-gpu",
        "--random",
        "--batch-size", "512000",
        "--count", "0",
        "--addr-types", "legacy,segwit,wrapped,taproot",
        "--transforms", "identity",
        "--max-snapshot-age", "0",
        "--output-file", "$Project\found-keys.json",
        "--progress-file", "$Project\brute-force-progress.json",
        "--stats-file", "$Project\brute-force-stats.json",
        "--stats-interval", "15",
        "--progress-interval", "30"
    )
    Start-Process -FilePath $Brute -ArgumentList $bruteArgs -WorkingDirectory $Project -WindowStyle Hidden `
        -RedirectStandardOutput "$Project\brute_force_24h.out.log" `
        -RedirectStandardError "$Project\brute_force_24h.err.log"
}

function Write-Status {
    $b = [bool](Get-Process -Name brute_force -ErrorAction SilentlyContinue)
    $d = [bool](Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue)
    $c = [bool](Get-Process -Name bitcoind -ErrorAction SilentlyContinue)
    $stats = ""
    $statsPath = "$Project\brute-force-stats.json"
    if (Test-Path -LiteralPath $statsPath) {
        try {
            $j = Get-Content -LiteralPath $statsPath -Raw | ConvertFrom-Json
            $stats = " keys=$($j.keys_tested) rate=$($j.keys_per_sec)/s hits=$($j.matches_found)"
        } catch {
            # ignore parse errors
        }
    }
    Write-Log "status dash=$d brute=$b core=$c$stats"
}

# --- main ---
if (-not (Test-Path -LiteralPath $Project)) {
    Write-Error "Projet introuvable: $Project"
    exit 1
}

Set-Location -LiteralPath $Project
Write-Log "========== BTC SOLVER AUTO START =========="
Write-Log "Once=$Once OpenBrowser=$OpenBrowser"

if (-not (Wait-Drives)) {
    Write-Log "ERREUR: disques W: ou Y: indisponibles apres attente"
    if ($Once) {
        exit 1
    }
}

$myPid = $PID
try {
    Set-Content -Path $KeepAliveMarker -Value "$myPid" -Encoding ASCII
} catch {
    # ignore
}

Ensure-Core
Ensure-Dashboard
Start-Sleep -Seconds 3
Ensure-Brute

if ($OpenBrowser -or $Once) {
    $ok = $false
    for ($i = 0; $i -lt 45; $i++) {
        try {
            $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/api/system/health" -UseBasicParsing -TimeoutSec 2
            if ($r.StatusCode -eq 200) {
                $ok = $true
                break
            }
        } catch {
            # not ready yet
        }
        Start-Sleep -Seconds 1
    }
    if ($ok) {
        Write-Log "Dashboard OK http://127.0.0.1:$Port/"
        if ($OpenBrowser) {
            Start-Process "http://127.0.0.1:$Port/"
        }
    } else {
        Write-Log "Dashboard pas encore pret - ouvrir plus tard http://127.0.0.1:$Port/"
    }
}

Write-Status

if ($Once) {
    Write-Log "Mode -Once: fin (services laisses en arriere-plan)"
    exit 0
}

Write-Log "Mode keep-alive: surveillance toutes les ${LoopSeconds}s"
while ($true) {
    try {
        if (Test-Path -LiteralPath $KeepAliveMarker) {
            $other = Get-Content -LiteralPath $KeepAliveMarker -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($other -and ($other.ToString().Trim() -ne "$myPid")) {
                $otherId = 0
                if ([int]::TryParse($other.ToString().Trim(), [ref]$otherId)) {
                    $alive = Get-Process -Id $otherId -ErrorAction SilentlyContinue
                    if ($alive) {
                        Write-Log "Autre keep-alive PID=$otherId actif - sortie"
                        break
                    }
                }
            }
        }
        try {
            Set-Content -Path $KeepAliveMarker -Value "$myPid" -Encoding ASCII
        } catch {
            # ignore
        }

        Ensure-Core
        Ensure-Dashboard
        Ensure-Brute
        Write-Status
    } catch {
        Write-Log "loop error: $_"
    }
    Start-Sleep -Seconds $LoopSeconds
}
