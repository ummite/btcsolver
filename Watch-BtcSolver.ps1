# Watch-BtcSolver.ps1
# Ensures Dashboard (:3000), bitcoind and brute_force are running.
# Designed for Windows Scheduled Task (every 1-2 min) + logon.
# Log: Y:\btcsolver\watchdog.log

$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Datadir = "W:\Bitcoin"
$Bitcoind = "$Datadir\bin\daemon\bitcoind.exe"
$Port = 3000
$Log = "$Project\watchdog.log"

function Write-Log {
    param([string]$Message)
    $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $Message
    try {
        # Prefer local log if Y: NAS has stream locks
        $paths = @($Log, "C:\btcsolver-cache\watchdog.log", "$env:TEMP\btcsolver-watchdog.log")
        if (-not (Test-Path "C:\btcsolver-cache")) {
            New-Item -ItemType Directory -Path "C:\btcsolver-cache" -Force -ErrorAction SilentlyContinue | Out-Null
        }
        foreach ($p in $paths) {
            try {
                if ((Test-Path $p) -and ((Get-Item $p -ErrorAction SilentlyContinue).Length -gt 5MB)) {
                    Move-Item $p "$p.bak" -Force -ErrorAction SilentlyContinue
                }
                [System.IO.File]::AppendAllText($p, $line + [Environment]::NewLine)
                break
            } catch {
                continue
            }
        }
    } catch {}
}

# Single-instance mutex
$mutex = $null
try {
    $created = $false
    $mutex = New-Object System.Threading.Mutex($true, "Global\BTCSolverWatchdogMutex", [ref]$created)
    if (-not $created) {
        exit 0
    }
} catch {}

function Test-PortListen {
    param([int]$PortNum)
    try {
        $c = Get-NetTCPConnection -LocalPort $PortNum -State Listen -ErrorAction SilentlyContinue
        if ($c) { return $true }
    } catch {}
    $n = netstat -ano 2>$null | Select-String ":$PortNum\s+.*LISTENING"
    return [bool]$n
}

function Wait-Drives {
    param([int]$Seconds = 90)
    $deadline = (Get-Date).AddSeconds($Seconds)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path "W:\") -and (Test-Path "Y:\btcsolver")) {
            return $true
        }
        Start-Sleep -Seconds 3
    }
    return ((Test-Path "W:\") -and (Test-Path "Y:\btcsolver"))
}

if (-not (Wait-Drives 90)) {
    Write-Log "ABORT: disks W:/Y: not ready"
    if ($mutex) { try { $mutex.ReleaseMutex() | Out-Null } catch {}; try { $mutex.Dispose() } catch {} }
    exit 1
}

$Snap = "$Project\data\utxo-day-935000.snapshot"
if (-not (Test-Path -LiteralPath $Snap)) {
    $Snap = "$Project\utxo-index.snapshot"
}
# (overridden after Core ensure if tip snapshot exists)

# Prefer release build, then stable bin dir, then project root copy
$Dash = $null
foreach ($cand in @(
    "$Project\target\release\btcsolver_dashboard.exe",
    "C:\btcsolver-bin\btcsolver_dashboard.exe",
    "$Project\btcsolver_dashboard.exe"
)) {
    if (Test-Path -LiteralPath $cand) {
        $Dash = $cand
        break
    }
}
if (-not $Dash) {
    $Dash = "$Project\target\release\btcsolver_dashboard.exe"  # for error message
}

$Brute = "$Project\target\release\brute_force.exe"
$BruteWorkDir = "$Project\target\release"
if (-not (Test-Path -LiteralPath $Brute)) {
    $Brute = "$Project\brute_force.exe"
    $BruteWorkDir = $Project
}

# --- Core ---
$dexorActive = $false
try {
    $dexorActive = [bool](Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object {
        $_.CommandLine -and ($_.CommandLine -match "dexor_blocks|tools\\dexor")
    })
} catch {}

# Core: NEVER kill a running bitcoind. Only start if dead.
# Prefer Keep-Core-And-Utxo.ps1 for full Core+UTXO policy.
if (-not $dexorActive) {
    if (-not (Get-Process -Name bitcoind, bitcoin-qt -ErrorAction SilentlyContinue)) {
        if (Test-Path -LiteralPath $Bitcoind) {
            Remove-Item -Force "$Datadir\.lock", "$Datadir\blocks\.lock", "$Datadir\bitcoind.pid" -ErrorAction SilentlyContinue
            $xor = "$Datadir\blocks\xor.dat"
            $conf = "$Datadir\bitcoin.conf"
            if ((Test-Path $xor) -and (Test-Path $conf)) {
                if (Select-String -Path $conf -Pattern "^\s*blocksxor\s*=\s*0" -Quiet) {
                    $bak = "$Datadir\blocks\xor.dat.bak-" + (Get-Date -Format "yyyyMMdd-HHmmss")
                    Move-Item $xor $bak -Force -ErrorAction SilentlyContinue
                    Write-Log "Removed orphan xor.dat"
                }
            }
            Write-Log "START bitcoind (was down)"
            Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir" -WindowStyle Hidden
            Start-Sleep -Seconds 6
        } else {
            Write-Log "MISSING bitcoind: $Bitcoind"
        }
    }
} else {
    Write-Log "SKIP core (dexor active)"
}

# Prefer freshest snapshot: tip > active > day-935000
foreach ($cand in @(
    "$Project\data\utxo-tip.snapshot",
    "$Project\data\utxo-active.snapshot",
    "$Project\data\utxo-day-935000.snapshot",
    "$Project\utxo-index.snapshot"
)) {
    if (Test-Path -LiteralPath $cand) {
        $Snap = $cand
        break
    }
}

# --- Dashboard ---
if (-not (Test-PortListen -PortNum $Port)) {
    Get-Process -Name btcsolver_dashboard -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 600

    if (-not (Test-Path -LiteralPath $Dash)) {
        Write-Log "MISSING dashboard: $Dash"
    } else {
        foreach ($lf in @("$Project\dashboard.out.log", "$Project\dashboard.err.log")) {
            if ((Test-Path $lf) -and ((Get-Item $lf).Length -gt 20MB)) {
                Move-Item $lf "$lf.old" -Force -ErrorAction SilentlyContinue
            }
        }
        Write-Log "START dashboard port=$Port snap=$Snap"
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
            "--auto-restart-check-secs", "30",
            "--rpc-user", "btcsolver",
            "--rpc-password", "btcsolver_rpc_2026"
        )
        $started = $false
        try {
            Start-Process -FilePath $Dash -ArgumentList $dashArgs -WorkingDirectory $Project -WindowStyle Hidden `
                -RedirectStandardOutput "$Project\dashboard.out.log" `
                -RedirectStandardError "$Project\dashboard.err.log" `
                -ErrorAction Stop
            $started = $true
        } catch {
            Write-Log "dashboard redirect failed: $($_.Exception.Message)"
        }
        if (-not $started) {
            Start-Process -FilePath $Dash -ArgumentList $dashArgs -WorkingDirectory $Project -WindowStyle Hidden
        }

        $up = $false
        for ($i = 0; $i -lt 50; $i++) {
            Start-Sleep -Seconds 1
            if (Test-PortListen -PortNum $Port) {
                $up = $true
                break
            }
        }
        if ($up) {
            Write-Log "OK dashboard listening on $Port"
        } else {
            Write-Log "FAIL dashboard not listening after 50s"
            if (Test-Path "$Project\dashboard.err.log") {
                $tail = @(Get-Content "$Project\dashboard.err.log" -Tail 5 -ErrorAction SilentlyContinue)
                Write-Log ("dash.err: " + ($tail -join " ; "))
            }
        }
    }
}

# --- Brute (disabled during PRIORITY-SYNC: Core tip + UTXO first) ---
$prioritySync = Test-Path -LiteralPath "$Project\data\PRIORITY-SYNC.flag"
if ($prioritySync) {
    # Kill any leftover key hunt so CPU/RAM/GPU stay on Core IBD
    Get-Process -Name brute_force -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Log "PRIORITY-SYNC: stop brute_force pid=$($_.Id)"
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    }
    Write-Log "PRIORITY-SYNC: brute disabled (Core+UTXO first)"
} elseif (-not (Get-Process -Name brute_force -ErrorAction SilentlyContinue)) {
    # Si le dashboard tourne (:3000), c'est LUI qui gère les fenêtres De→À (2^30) + journal de plages.
    # Ne pas lancer un brute « nu » qui court-circuite le range log.
    $dashUp = Test-PortListen -PortNum $Port
    if ($dashUp) {
        Write-Log "SKIP brute start: dashboard :$Port gère auto-scan + fenêtres (scan-ranges-log)"
    } elseif ((Test-Path -LiteralPath $Brute) -and (Test-Path -LiteralPath $Snap)) {
        # Fallback sans dashboard : une fenêtre 2^30 depuis le journal si présent
        $step = [uint64]1073741824
        $startHex = $null
        $endHex = $null
        $rangeLog = Join-Path $Project "data\scan-ranges-log.json"
        if (Test-Path $rangeLog) {
            try {
                $rj = Get-Content $rangeLog -Raw | ConvertFrom-Json
                if ($rj.range_step -gt 0) { $step = [uint64]$rj.range_step }
                if ($rj.manual_start) { $startHex = [string]$rj.manual_start }
                elseif ($rj.current -and $rj.current.start) { $startHex = [string]$rj.current.start }
            } catch {}
        }
        Write-Log "START brute_force fallback (no dash) step=$step start=$startHex"
        $bruteArgs = @(
            "--snapshot-path", $Snap,
            "--threads", "0",
            "--cpu-pct", "50",
            "--use-gpu",
            "--gpus", "0,1,2",
            "--batch-size", "4194304",
            "--count", "$step",
            "--addr-types", "legacy,segwit,wrapped,taproot",
            "--transforms", "identity",
            "--max-snapshot-age", "0",
            "--output-file", "$Project\found-keys.json",
            "--progress-file", "$Project\brute-force-progress.json",
            "--stats-file", "$Project\brute-force-stats.json",
            "--stats-interval", "5",
            "--progress-interval", "15"
        )
        if ($startHex) {
            $bruteArgs += @("--start", $startHex, "--no-resume")
        }
        $bStarted = $false
        try {
            Start-Process -FilePath $Brute -ArgumentList $bruteArgs -WorkingDirectory $BruteWorkDir -WindowStyle Hidden `
                -RedirectStandardOutput "$Project\brute_force_24h.out.log" `
                -RedirectStandardError "$Project\brute_force_24h.err.log" `
                -ErrorAction Stop
            $bStarted = $true
        } catch {
            Write-Log "brute redirect failed: $($_.Exception.Message)"
        }
        if (-not $bStarted) {
            Start-Process -FilePath $Brute -ArgumentList $bruteArgs -WorkingDirectory $BruteWorkDir -WindowStyle Hidden
        }
    } else {
        Write-Log "SKIP brute (missing exe or snap)"
    }
}

$dashOk = Test-PortListen -PortNum $Port
$bruteOk = [bool](Get-Process -Name brute_force -ErrorAction SilentlyContinue)
$coreOk = [bool](Get-Process -Name bitcoind -ErrorAction SilentlyContinue)
Write-Log "status dash=$dashOk brute=$bruteOk core=$coreOk"

if ($dashOk) {
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/" -UseBasicParsing -TimeoutSec 5
        Write-Log ("HTTP / status=" + $r.StatusCode)
    } catch {
        Write-Log ("HTTP / FAIL: " + $_.Exception.Message)
    }
}

if ($mutex) {
    try { $mutex.ReleaseMutex() | Out-Null } catch {}
    try { $mutex.Dispose() } catch {}
}
exit 0
