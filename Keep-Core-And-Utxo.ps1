# Keep-Core-And-Utxo.ps1
# Garantit : Bitcoin Core TOUJOURS en marche + UTXO auto au tip.
# Appele par la tache planifiee (toutes les 2-5 min) ou manuellement.
# Log: C:\btcsolver-cache\core-utxo.log  (et Y:\btcsolver\core-utxo.log si possible)
# Status JSON: Y:\btcsolver\data\CORE-UTXO-STATUS.json

$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Datadir = "W:\Bitcoin"
$Bitcoind = "$Datadir\bin\daemon\bitcoind.exe"
$Cli = "$Datadir\bin\daemon\bitcoin-cli.exe"
$StatusFile = "$Project\data\CORE-UTXO-STATUS.json"
$LogPaths = @("C:\btcsolver-cache\core-utxo.log", "$Project\core-utxo.log", "$env:TEMP\btcsolver-core-utxo.log")
$UtxoDump = "W:\Temp\utxo-tip.dat"
$SnapLive = "$Project\data\utxo-tip.snapshot"
$SnapMeta = "$Project\data\utxo-tip.snapshot.meta.json"
$SnapActive = "$Project\data\utxo-active.snapshot"  # symlink-like copy used by tools
$MarkerBuilding = "$Project\.UTXO_REBUILD_IN_PROGRESS"
$MaxUtxoAgeHours = 24

if (-not (Test-Path "C:\btcsolver-cache")) {
    New-Item -ItemType Directory -Path "C:\btcsolver-cache" -Force | Out-Null
}
if (-not (Test-Path "W:\Temp")) {
    New-Item -ItemType Directory -Path "W:\Temp" -Force | Out-Null
}
if (-not (Test-Path "$Project\data")) {
    New-Item -ItemType Directory -Path "$Project\data" -Force | Out-Null
}

function Write-Log([string]$m) {
    $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $m
    foreach ($p in $LogPaths) {
        try {
            if ((Test-Path $p) -and ((Get-Item $p).Length -gt 8MB)) {
                Move-Item $p "$p.bak" -Force -ErrorAction SilentlyContinue
            }
            [System.IO.File]::AppendAllText($p, $line + [Environment]::NewLine)
            break
        } catch { continue }
    }
}

function Write-Status($obj) {
    try {
        # UTF-8 sans BOM (serde_json / dashboard refuse le BOM EF BB BF)
        $json = $obj | ConvertTo-Json -Depth 6
        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($StatusFile, $json, $utf8NoBom)
    } catch {}
}

function Get-CoreInfo {
    if (-not (Test-Path $Cli)) { return $null }
    # -rpcclienttimeout avoids hung bitcoin-cli during heavy IBD
    try {
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = $Cli
        $psi.Arguments = "-datadir=$Datadir -rpcclienttimeout=15 getblockchaininfo"
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        $psi.UseShellExecute = $false
        $psi.CreateNoWindow = $true
        $p = [System.Diagnostics.Process]::Start($psi)
        if (-not $p.WaitForExit(20000)) {
            try { $p.Kill() } catch {}
            Write-Log "WARN: bitcoin-cli getblockchaininfo timeout 20s"
            return $null
        }
        $raw = $p.StandardOutput.ReadToEnd()
        if (-not $raw) { return $null }
        return ($raw | ConvertFrom-Json)
    } catch {
        Write-Log ("WARN: Get-CoreInfo: " + $_.Exception.Message)
        return $null
    }
}

function Ensure-Core {
    $proc = Get-Process -Name bitcoind, bitcoin-qt -ErrorAction SilentlyContinue
    if ($proc) {
        return @{ running = $true; started = $false; pid = $proc[0].Id }
    }

    if (-not (Test-Path $Bitcoind)) {
        Write-Log "FATAL: bitcoind missing $Bitcoind"
        return @{ running = $false; started = $false; pid = $null }
    }

    # Only clean locks if process truly dead
    Remove-Item -Force "$Datadir\.lock", "$Datadir\blocks\.lock", "$Datadir\bitcoind.pid" -ErrorAction SilentlyContinue

    # blocksxor=0: remove orphan xor.dat if present
    $xor = "$Datadir\blocks\xor.dat"
    $conf = "$Datadir\bitcoin.conf"
    if ((Test-Path $xor) -and (Test-Path $conf) -and
        (Select-String -Path $conf -Pattern "^\s*blocksxor\s*=\s*0" -Quiet)) {
        $bak = "$Datadir\blocks\xor.dat.bak-" + (Get-Date -Format "yyyyMMdd-HHmmss")
        Move-Item $xor $bak -Force -ErrorAction SilentlyContinue
        Write-Log "Removed orphan xor.dat -> $bak"
    }

    Write-Log "START bitcoind (always-on)"
    Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir" -WindowStyle Hidden
    Start-Sleep -Seconds 8

    $p2 = Get-Process -Name bitcoind -ErrorAction SilentlyContinue
    return @{
        running = [bool]$p2
        started = $true
        pid     = if ($p2) { $p2.Id } else { $null }
    }
}

function Find-DumpToFlat {
    $cands = @(
        "$Project\target\release\dump_to_flat.exe",
        "$Project\dump_to_flat.exe"
    )
    foreach ($c in $cands) {
        if (Test-Path $c) { return $c }
    }
    return $null
}

function Find-Btcsolver {
    $cands = @(
        "$Project\target\release\btcsolver.exe",
        "$Project\btcsolver.exe"
    )
    foreach ($c in $cands) {
        if (Test-Path $c) { return $c }
    }
    return $null
}

function Test-CoreAtTip($info) {
    if (-not $info) { return $false }
    if ($info.initialblockdownload -eq $true) { return $false }
    $blocks = [int64]$info.blocks
    $headers = [int64]$info.headers
    if ($headers -lt 100000) { return $false }
    # Within 3 blocks of tip headers
    return ($blocks -ge ($headers - 3))
}

function Get-ActiveSnapMeta {
    $metas = @(
        "$Project\data\utxo-tip.snapshot.meta.json",
        "$Project\data\utxo-active.snapshot.meta.json",
        "$Project\data\utxo-day-935000.snapshot.meta.json"
    )
    $best = $null
    foreach ($m in $metas) {
        if (Test-Path $m) {
            # Skip empty/corrupt metas (0 bytes or unparseable)
            if ((Get-Item $m).Length -lt 10) { continue }
            try {
                $parsed = Get-Content $m -Raw | ConvertFrom-Json
                # Must have block_height to be useful
                if ($parsed.block_height) {
                    if (-not $best -or $parsed.block_height -gt $best.block_height) {
                        $best = $parsed
                    }
                }
            } catch {}
        }
    }
    return $best
}

function Refresh-UtxoFromTip {
    # --- stale marker recovery ---
    if (Test-Path $MarkerBuilding) {
        $age = (Get-Date) - (Get-Item $MarkerBuilding).LastWriteTime
        if ($age.TotalHours -lt 6) {
            # Check if another instance is actually working (pid file or recent dump activity)
            $realPid = $null
            if (Test-Path "$MarkerBuilding.pid") {
                try { $realPid = Get-Process -Id ([int](Get-Content "$MarkerBuilding.pid")) -ErrorAction SilentlyContinue } catch {}
            }
            if ($realPid) {
                Write-Log "UTXO rebuild already in progress (PID $($realPid.Id), marker age $($age.TotalMinutes) min)"
                return @{ ok = $false; reason = "in_progress" }
            }
            Write-Log "STALE marker detected (age $($age.TotalMinutes) min, no owner PID) - removing and continuing"
            Remove-Item $MarkerBuilding -Force -ErrorAction SilentlyContinue
            Remove-Item "$MarkerBuilding.pid" -Force -ErrorAction SilentlyContinue
        } else {
            Write-Log "Expired marker (age $($age.TotalHours)h) - removing"
            Remove-Item $MarkerBuilding -Force -ErrorAction SilentlyContinue
            Remove-Item "$MarkerBuilding.pid" -Force -ErrorAction SilentlyContinue
        }
    }

    $dumpTool = Find-DumpToFlat
    if (-not $dumpTool) {
        Write-Log "WARN: dump_to_flat.exe missing - build with: cargo build --release --bin dump_to_flat"
        return @{ ok = $false; reason = "no_dump_to_flat" }
    }

    # Safety check: if a valid snapshot already exists (fresh meta + recent height), skip rebuild
    $existingMeta = Get-ActiveSnapMeta
    $refreshInfo = Get-CoreInfo
    if ($existingMeta -and $existingMeta.block_height) {
        $existingH = [int64]$existingMeta.block_height
        $tipH = if ($refreshInfo) { [int64]$refreshInfo.blocks } else { 0 }
        $existingLagBlocks = if ($tipH -gt 0) { $tipH - $existingH } else { 0 }
        if ($existingLagBlocks -le 10) {
            Write-Log "UTXO snapshot already fresh (height=$existingH, tip=$tipH, lag=$existingLagBlocks blocks) - skipping rebuild"
            # Activate the existing snapshot
            if (Test-Path $SnapLive) {
                Copy-Item $SnapLive $SnapActive -Force -ErrorAction SilentlyContinue
                Copy-Item $SnapLive "$Project\utxo-index.snapshot" -Force -ErrorAction SilentlyContinue
            }
            return @{ ok = $true; height = $existingH; path = $SnapLive }
        }
    }

    # Create marker + write our PID so other ticks can verify we're alive
    New-Item -ItemType File -Path $MarkerBuilding -Force | Out-Null
    $PID | Set-Content "$MarkerBuilding.pid" -Encoding UTF8
    try {
        # --- Step 1: dumptxoutset (skip if valid dump already exists) ---
        $needDump = $false
        if (Test-Path $UtxoDump) {
            $existingSize = (Get-Item $UtxoDump).Length
            if ($existingSize -gt 1GB) {
                Write-Log "UTXO refresh: existing dump OK ($([math]::Round($existingSize/1GB,2)) GB) - skipping dumptxoutset"
            } else {
                Write-Log "UTXO refresh: existing dump too small ($([math]::Round($existingSize/1MB,1)) MB) - regenerating"
                $needDump = $true
            }
        } else {
            $needDump = $true
        }

        if ($needDump) {
            Write-Log "UTXO refresh: dumptxoutset -> $UtxoDump"
            Remove-Item $UtxoDump -Force -ErrorAction SilentlyContinue

            # Bitcoin Core 26+: dumptxoutset path type
            # IMPORTANT: passer -datadir en argument string explicite (évite "$Datadir" littéral
            # quand le script est invoqué depuis certaines tâches planifiées / wrappers).
            $dumpOk = $false
            $datadirArg = "-datadir=$Datadir"
            # dumptxoutset est long (souvent 10–40+ min) : pas de timeout RPC client
            $timeoutArg = "-rpcclienttimeout=0"
            Write-Log "UTXO refresh: cli=$Cli $datadirArg $timeoutArg dumptxoutset $UtxoDump latest"
            & $Cli $datadirArg $timeoutArg dumptxoutset $UtxoDump latest 2>&1 | ForEach-Object { Write-Log "cli: $_" }
            if ((Test-Path $UtxoDump) -and ((Get-Item $UtxoDump).Length -gt 1GB)) {
                $dumpOk = $true
            } else {
                Write-Log "UTXO refresh: retry without 'latest' type"
                & $Cli $datadirArg $timeoutArg dumptxoutset $UtxoDump 2>&1 | ForEach-Object { Write-Log "cli2: $_" }
                if ((Test-Path $UtxoDump) -and ((Get-Item $UtxoDump).Length -gt 1GB)) {
                    $dumpOk = $true
                }
            }

            if (-not $dumpOk) {
                Write-Log "ERROR: dumptxoutset failed or file too small"
                return @{ ok = $false; reason = "dumptxoutset_failed" }
            }
        }

        # --- Step 2: dump_to_flat conversion ---
        $info = Get-CoreInfo
        $height = if ($info) { [int64]$info.blocks } else { 0 }
        $hash = if ($info) { $info.bestblockhash } else { "" }
        $mediantime = if ($info) { [int64]$info.mediantime } else { 0 }

        $dumpGb = [math]::Round(((Get-Item $UtxoDump).Length / 1GB), 2)
        Write-Log ("UTXO refresh: dump_to_flat size_gb=" + $dumpGb)
        $tmpSnap = "$Project\data\utxo-tip.building.snapshot"

        # Remove stale building snapshot if any
        if (Test-Path $tmpSnap) { Remove-Item $tmpSnap -Force -ErrorAction SilentlyContinue }

        Write-Log "UTXO refresh: launching dump_to_flat --snapshot $UtxoDump --output $tmpSnap"
        $flatProc = Start-Process -FilePath $dumpTool `
            -ArgumentList "--snapshot `"$UtxoDump`" --output `"$tmpSnap`"" `
            -RedirectStandardOutput "$Project\data\dump_to_flat.out" `
            -RedirectStandardError "$Project\data\dump_to_flat.err" `
            -Wait -PassThru -NoNewWindow
        Write-Log "UTXO refresh: dump_to_flat exited with code $($flatProc.ExitCode)"

        # Log the output files
        if (Test-Path "$Project\data\dump_to_flat.out") {
            Get-Content "$Project\data\dump_to_flat.out" | ForEach-Object { Write-Log "flat-out: $_" }
        }
        if (Test-Path "$Project\data\dump_to_flat.err") {
            Get-Content "$Project\data\dump_to_flat.err" | ForEach-Object { Write-Log "flat-err: $_" }
        }

        if (-not (Test-Path $tmpSnap)) {
            Write-Log "ERROR: dump_to_flat produced no snapshot (exit code $($flatProc.ExitCode))"
            return @{ ok = $false; reason = "flat_failed" }
        }

        # --- Step 3: activate snapshot ---
        Move-Item $tmpSnap $SnapLive -Force
        Copy-Item $SnapLive $SnapActive -Force -ErrorAction SilentlyContinue
        # Also update default path used by many tools
        Copy-Item $SnapLive "$Project\utxo-index.snapshot" -Force -ErrorAction SilentlyContinue

        $snapGb = [math]::Round(((Get-Item $SnapLive).Length / 1GB), 2)
        Write-Log "UTXO refresh: snapshot activated ($snapGb GB)"

        $meta = @{
            base_block_hash   = $hash
            block_height      = $height
            block_time_unix   = $mediantime
            block_time_utc    = if ($mediantime -gt 0) {
                [DateTimeOffset]::FromUnixTimeSeconds($mediantime).UtcDateTime.ToString("yyyy-MM-dd HH:mm:ss") + " UTC"
            } else { "" }
            built_at          = (Get-Date).ToUniversalTime().ToString("o")
            source            = $UtxoDump
            num_scripts       = $null
            dumptxoutset      = $true
        }
        $meta | ConvertTo-Json -Depth 4 | Set-Content $SnapMeta -Encoding UTF8
        Copy-Item $SnapMeta "$Project\data\utxo-active.snapshot.meta.json" -Force -ErrorAction SilentlyContinue

        Write-Log "UTXO refresh OK height=$height hash=$hash size=$snapGb GB"
        return @{ ok = $true; height = $height; path = $SnapLive }
    } finally {
        Remove-Item $MarkerBuilding -Force -ErrorAction SilentlyContinue
        Remove-Item "$MarkerBuilding.pid" -Force -ErrorAction SilentlyContinue
    }
}

# --- mutex ---
$mutex = $null
try {
    $created = $false
    $mutex = New-Object System.Threading.Mutex($true, "Global\BTCSolverCoreUtxoMutex", [ref]$created)
    if (-not $created) { exit 0 }
} catch {}

Write-Log "=== Keep-Core-And-Utxo tick ==="

if (-not ((Test-Path "W:\") -and (Test-Path $Datadir))) {
    Write-Log "ABORT: W:\Bitcoin not available"
    if ($mutex) { try { $mutex.ReleaseMutex() | Out-Null } catch {} }
    exit 1
}

$core = Ensure-Core
$info = $null
# Wait briefly for RPC after start (max ~30s, each call has 20s hard timeout)
for ($i = 0; $i -lt 3; $i++) {
    $info = Get-CoreInfo
    if ($info) { break }
    if ($i -lt 2) { Start-Sleep -Seconds 5 }
}

$atTip = Test-CoreAtTip $info
$blocks = if ($info) { [int64]$info.blocks } else { 0 }
$headers = if ($info) { [int64]$info.headers } else { 0 }
$ibd = if ($info) { [bool]$info.initialblockdownload } else { $true }
$progress = if ($info) { [double]$info.verificationprogress } else { 0 }

$snapMeta = Get-ActiveSnapMeta
$snapHeight = if ($snapMeta -and $snapMeta.block_height) { [int64]$snapMeta.block_height } else { $null }
$snapTime = if ($snapMeta -and $snapMeta.block_time_unix) { [int64]$snapMeta.block_time_unix } else { $null }
$nowUnix = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
# NEVER use Core mediantime during IBD as "tip time" (it is years in the past while headers are at network tip).
# Lag vs real network tip: wall-clock vs snapshot block time, else height gap vs headers.
$lagHours = $null
if ($snapTime -and $snapTime -gt 1400000000) {
    $lagHours = [math]::Max(0, ($nowUnix - $snapTime) / 3600.0)
} elseif ($snapHeight -and $headers -gt 0) {
    $lagHours = [math]::Max(0, (($headers - $snapHeight) * 10.0) / 60.0)
}

# UTXO valid for balance tests only if < MaxUtxoAgeHours behind real tip.
# During Core IBD we cannot rebuild tip UTXO yet, so stale snap stays invalid.
$utxoValid = ($lagHours -ne $null) -and ($lagHours -le $MaxUtxoAgeHours)
$refreshResult = $null

# Auto-refresh UTXO when Core is at tip and UTXO is stale / missing
if ($atTip -and (-not $utxoValid)) {
    Write-Log "Core at tip (blocks=$blocks headers=$headers) but UTXO stale (lagHours=$lagHours) - refreshing"
    $refreshResult = Refresh-UtxoFromTip
    if ($refreshResult.ok) {
        $snapHeight = $refreshResult.height
        $lagHours = 0
        $utxoValid = $true
    }
} elseif (-not $atTip) {
    Write-Log "Core IBD/sync: blocks=$blocks headers=$headers progress=$([math]::Round($progress*100,4))% ibd=$ibd - UTXO wait for tip"
} else {
    Write-Log "Core tip OK + UTXO valid (lagHours=$lagHours)"
}

$status = [ordered]@{
    updated_at_utc     = (Get-Date).ToUniversalTime().ToString("o")
    core_running       = [bool]$core.running
    core_pid           = $core.pid
    core_started_now   = [bool]$core.started
    blocks             = $blocks
    headers            = $headers
    verification_pct   = [math]::Round($progress * 100, 4)
    initialblockdownload = $ibd
    at_tip             = $atTip
    utxo_valid_for_tests = $utxoValid
    utxo_lag_hours     = $lagHours
    utxo_height        = $snapHeight
    utxo_max_age_hours = $MaxUtxoAgeHours
    active_snapshot    = if (Test-Path $SnapLive) { $SnapLive } elseif (Test-Path "$Project\data\utxo-day-935000.snapshot") { "$Project\data\utxo-day-935000.snapshot" } else { $null }
    last_refresh       = $refreshResult
    priority_sync      = (Test-Path "$Project\data\PRIORITY-SYNC.flag")
    message            = if (-not $core.running) {
        "CRITIQUE: bitcoind arrete - relance en cours ou echec"
    } elseif ($atTip -and $utxoValid) {
        "OK: Core tip + UTXO frais (<${MaxUtxoAgeHours}h) - pret pour chercher des cles"
    } elseif ($atTip -and -not $utxoValid) {
        "Core tip - refresh UTXO en cours ou a lancer (dumptxoutset)"
    } else {
        "PRIORITE SYNC: Core IBD $blocks / $headers ($([math]::Round($progress*100,2))%) - cles OFF tant que pas tip+UTXO"
    }
}
Write-Status $status
Write-Log $status.message

if ($mutex) {
    try { $mutex.ReleaseMutex() | Out-Null } catch {}
    try { $mutex.Dispose() } catch {}
}
exit 0
