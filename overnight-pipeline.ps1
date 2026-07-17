# Overnight pipeline:
#  1) Stop Core if mid-useless reindex
#  2) De-XOR remaining blk/rev files
#  3) bitcoind -reindex with optimized conf
#  4) Start continuous GPU brute-force on best UTXO snapshot NOW
#  5) When Core IBD finishes -> rebuild UTXO + restart scan on fresh index
#
# Run:  powershell -ExecutionPolicy Bypass -File Y:\btcsolver\overnight-pipeline.ps1

$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Datadir = "W:\Bitcoin"
$Blocks = "$Datadir\blocks"
$Bitcoind = "$Datadir\bin\daemon\bitcoind.exe"
$Cli = "$Datadir\bin\daemon\bitcoin-cli.exe"
$Log = "$Project\overnight-pipeline.log"
$SnapBest = "$Project\data\utxo-day-935000.snapshot"
if (-not (Test-Path $SnapBest)) { $SnapBest = "$Project\utxo-index.snapshot" }
$Brute = "$Project\target\release\brute_force.exe"
if (-not (Test-Path $Brute)) { $Brute = "$Project\brute_force.exe" }
$Python = "C:\Python311\python.exe"
if (-not (Test-Path $Python)) { $Python = "python" }

function Log($msg) {
    $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $msg
    Add-Content -Path $Log -Value $line
    Write-Host $line
}

function Stop-Bitcoind {
    if (Get-Process bitcoind -EA SilentlyContinue) {
        Log "Stopping bitcoind..."
        & $Cli -datadir=$Datadir stop 2>$null
        $deadline = (Get-Date).AddMinutes(3)
        while ((Get-Date) -lt $deadline -and (Get-Process bitcoind -EA SilentlyContinue)) {
            Start-Sleep -Seconds 2
        }
        Get-Process bitcoind -EA SilentlyContinue | Stop-Process -Force -EA SilentlyContinue
        Start-Sleep -Seconds 2
    }
    Remove-Item -Force "$Datadir\.lock","$Blocks\.lock","$Datadir\bitcoind.pid" -EA SilentlyContinue
}

function Start-BitcoindReindex {
    Log "Starting bitcoind -reindex (datadir=$Datadir)"
    Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir","-reindex" -WindowStyle Hidden
    Start-Sleep -Seconds 8
}

function Start-BitcoindNormal {
    if (Test-Path "$Project\.DEXOR_IN_PROGRESS") {
        Log "Skip Core start - DEXOR_IN_PROGRESS"
        return
    }
    Log "Starting bitcoind normal"
    Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir" -WindowStyle Hidden
    Start-Sleep -Seconds 8
}

function Get-ChainInfo {
    try {
        $j = & $Cli -datadir=$Datadir -rpcclienttimeout=30 getblockchaininfo 2>$null | ConvertFrom-Json
        return $j
    } catch { return $null }
}

function Start-BruteForce([string]$snapshot) {
    Get-Process brute_force -EA SilentlyContinue | Stop-Process -Force -EA SilentlyContinue
    Start-Sleep -Seconds 1
    if (-not (Test-Path $Brute)) {
        Log "ERROR: brute_force.exe missing at $Brute"
        return
    }
    if (-not (Test-Path $snapshot)) {
        Log "ERROR: snapshot missing $snapshot"
        return
    }
    $threads = [Math]::Max(16, (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors - 4)
    $args = @(
        "--snapshot-path", $snapshot,
        "--threads", "$threads",
        "--use-gpu",
        "--random",
        "--batch-size", "512000",
        "--count", "0",
        "--addr-types", "legacy,segwit,wrapped,taproot",
        "--max-snapshot-age", "0",
        "--output-file", "$Project\found-keys.json",
        "--progress-file", "$Project\brute-force-progress.json",
        "--stats-interval", "15",
        "--progress-interval", "30"
    )
    Log "Starting brute_force GPU random threads=$threads snap=$snapshot"
    # Work from project so libsecp_gpu.dll resolves
    $p = Start-Process -FilePath $Brute -ArgumentList $args -WorkingDirectory $Project -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput "$Project\brute_force_overnight.out.log" `
        -RedirectStandardError "$Project\brute_force_overnight.err.log"
    Log "brute_force PID=$($p.Id)"
}

function Wait-IbdComplete {
    Log "Waiting for IBD complete (blocks==headers, initialblockdownload=false)..."
    $last = -1
    while ($true) {
        $info = Get-ChainInfo
        if ($null -eq $info) {
            Log "RPC not ready - waiting..."
            # If process dead, restart without reindex
            if (-not (Get-Process bitcoind -EA SilentlyContinue)) {
                Log "bitcoind died - restarting (no reindex)"
                Start-BitcoindNormal
            }
            Start-Sleep -Seconds 30
            continue
        }
        $blocks = [int64]$info.blocks
        $headers = [int64]$info.headers
        $ibd = [bool]$info.initialblockdownload
        $behind = $headers - $blocks
        if ($blocks -ne $last) {
            Log ("progress blocks={0} headers={1} behind={2} ibd={3} verif={4:P4}" -f `
                $blocks, $headers, $behind, $ibd, [double]$info.verificationprogress)
            $last = $blocks
        }
        if ((-not $ibd) -and ($behind -le 2) -and ($blocks -gt 100000)) {
            Log "IBD COMPLETE tip~=$blocks"
            return $info
        }
        Start-Sleep -Seconds 60
    }
}

function Rebuild-Utxo {
    Log "Rebuilding UTXO index from blocks..."
    $indexer = "$Project\full_utxo_indexer.exe"
    if (-not (Test-Path $indexer)) { $indexer = "$Project\target\release\full_utxo_indexer.exe" }
    if (-not (Test-Path $indexer)) {
        Log "ERROR: full_utxo_indexer missing"
        return $null
    }
    # After full de-XOR, key is identity zeros
    $out = & $indexer build `
        --blocks-dir $Blocks `
        --db-path "$Project\utxo-index.redb" `
        --obf-key "0000000000000000" `
        --start-file 0 `
        --checkpoint-interval 200 2>&1
    $out | Add-Content $Log
    $snap = "$Project\utxo-index.snapshot"
    if (Test-Path $snap) {
        Log "UTXO snapshot ready: $snap size=$((Get-Item $snap).Length)"
        return $snap
    }
    Log "WARNING: snapshot not found after build"
    return $null
}

# ═══════════════════════════ MAIN ═══════════════════════════
Set-Location $Project
Log "========== OVERNIGHT PIPELINE START =========="
Log "Project=$Project Datadir=$Datadir SnapshotNow=$SnapBest"

# 0) Flag: Core must stay down while rewriting blocks (dashboard keep-alive respects this)
New-Item -ItemType File -Path "$Project\.DEXOR_IN_PROGRESS" -Force | Out-Null
Log "DEXOR_IN_PROGRESS set (Core will not auto-start; UI can still run)"

# 1) Stop Core BEFORE touching block files (do NOT kill dashboard or brute)
Stop-Bitcoind

# 2) Continuous key hunt ASAP (independent of Core / UI)
if (Test-Path $SnapBest) {
    if (-not (Get-Process brute_force -EA SilentlyContinue)) {
        Start-BruteForce $SnapBest
    } else {
        Log "brute_force already running - leave it"
    }
} else {
    Log "No snapshot yet - will start brute after UTXO rebuild"
}

# 3) De-XOR remaining obfuscated files (Core MUST stay down; UI OK)
Log "De-XOR phase (Core offline, UI+brute OK)..."
& $Python "$Project\tools\dexor_blocks.py" --blocks-dir $Blocks --key b3a2cd522df3a049 --workers 6 2>&1 | Tee-Object -FilePath "$Project\dexor-overnight.log" -Append
$dexorExit = $LASTEXITCODE
Log "De-XOR exit=$dexorExit"

# Remove xor.dat so blocksxor=0 is consistent
if (Test-Path "$Blocks\xor.dat") {
    Copy-Item "$Blocks\xor.dat" "$Blocks\xor.dat.bak-pre-overnight" -Force
    Remove-Item "$Blocks\xor.dat" -Force
    Log "Removed blocks\xor.dat"
}
Remove-Item -Force "$Project\.DEXOR_IN_PROGRESS" -EA SilentlyContinue
Log "DEXOR_IN_PROGRESS cleared - Core may start again"

# Ensure conf optimized
if (-not (Test-Path "$Datadir\bitcoin.conf")) {
    Log "WARNING: missing bitcoin.conf"
}

# 3) Reindex with local plaintext blocks
Start-BitcoindReindex

# 4) Wait until fully synced
$tip = Wait-IbdComplete

# 5) Rebuild UTXO
$newSnap = Rebuild-Utxo
if ($newSnap) {
    Start-BruteForce $newSnap
    Log "Switched brute_force to fresh UTXO snapshot"
}

# 6) Keep Core running; monitor brute alive every 10 min forever until machine stop
Log "Entering keep-alive loop (Core + brute)"
while ($true) {
    if (-not (Get-Process bitcoind -EA SilentlyContinue)) {
        Log "bitcoind missing - restart"
        Start-BitcoindNormal
    }
    if (-not (Get-Process brute_force -EA SilentlyContinue)) {
        $snap = if (Test-Path "$Project\utxo-index.snapshot") { "$Project\utxo-index.snapshot" } else { $SnapBest }
        Log "brute_force missing - restart on $snap"
        Start-BruteForce $snap
    }
    $info = Get-ChainInfo
    if ($info) {
        Log ("keepalive blocks={0} headers={1} ibd={2}" -f $info.blocks, $info.headers, $info.initialblockdownload)
    }
    if (Test-Path "$Project\found-keys.json") {
        $sz = (Get-Item "$Project\found-keys.json").Length
        if ($sz -gt 2) { Log "FOUND KEYS FILE size=$sz bytes" }
    }
    Start-Sleep -Seconds 600
}
