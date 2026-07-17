# 24/7 keep-alive: dashboard + brute always; Core only when not de-XORing
# Safe to run forever. Does NOT kill de-XOR or overnight.

$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Datadir = "W:\Bitcoin"
$Bitcoind = "$Datadir\bin\daemon\bitcoind.exe"
$Cli = "$Datadir\bin\daemon\bitcoin-cli.exe"
$Port = 3000
$Log = "$Project\keep-alive-24-7.log"
$Snap = "$Project\data\utxo-day-935000.snapshot"
if (-not (Test-Path $Snap)) { $Snap = "$Project\utxo-index.snapshot" }
$Brute = "$Project\target\release\brute_force.exe"
if (-not (Test-Path $Brute)) { $Brute = "$Project\brute_force.exe" }
$Dash = "$Project\target\release\btcsolver_dashboard.exe"

function Log($m) {
    $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $m
    Add-Content $Log $line
    Write-Host $line
}

function Ensure-Dashboard {
    $listening = Get-NetTCPConnection -LocalPort $Port -State Listen -EA SilentlyContinue
    if ($listening) { return }
    if (-not (Test-Path $Dash)) {
        Log "dashboard exe missing"
        return
    }
    Log "Starting dashboard on :$Port"
    Start-Process -FilePath $Dash -ArgumentList @(
        "--port", "$Port",
        "--bitcoin-datadir", $Datadir,
        "--bitcoind-path", $Bitcoind,
        "--snapshot-path", $Snap,
        "--project-dir", $Project,
        "--bin-dir", $Project,
        "--cache-dir", $Project,
        "--static-dir", "$Project\static\dashboard",
        "--max-snapshot-age", "0",
        "--auto-restart-check-secs", "45",
        "--rpc-user", "btcsolver",
        "--rpc-password", "btcsolver_rpc_2026"
    ) -WorkingDirectory $Project -WindowStyle Hidden
}

function Ensure-Brute {
    if (Get-Process brute_force -EA SilentlyContinue) { return }
    if (-not (Test-Path $Brute)) { Log "brute missing"; return }
    if (-not (Test-Path $Snap)) { Log "snapshot missing $Snap"; return }
    $threads = [Math]::Max(12, (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors - 6)
    Log "Starting brute_force non-stop (threads=$threads)"
    Start-Process -FilePath $Brute -ArgumentList @(
        "--snapshot-path", $Snap,
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
    ) -WorkingDirectory $Project -WindowStyle Hidden `
      -RedirectStandardOutput "$Project\brute_force_24h.out.log" `
      -RedirectStandardError "$Project\brute_force_24h.err.log"
}

function Ensure-Core {
    # Never start Core while de-XOR rewrites blocks
    if (Test-Path "$Project\.DEXOR_IN_PROGRESS") {
        return
    }
    if (Get-Process bitcoind -EA SilentlyContinue) { return }
    if (-not (Test-Path $Bitcoind)) { return }
    Remove-Item -Force "$Datadir\.lock","$Datadir\blocks\.lock","$Datadir\bitcoind.pid" -EA SilentlyContinue
    Log "Starting bitcoind"
    Start-Process -FilePath $Bitcoind -ArgumentList "-datadir=$Datadir" -WindowStyle Hidden
}

Set-Location $Project
Log "========== KEEP-ALIVE 24/7 START =========="
Log "UI: http://127.0.0.1:$Port/  |  brute: always  |  Core: when not de-XORing"
Log "Manual word tests in UI do NOT stop background scan."

while ($true) {
    try {
        Ensure-Dashboard
        Ensure-Brute
        Ensure-Core
        $b = [bool](Get-Process brute_force -EA SilentlyContinue)
        $d = [bool](Get-NetTCPConnection -LocalPort $Port -State Listen -EA SilentlyContinue)
        $c = [bool](Get-Process bitcoind -EA SilentlyContinue)
        $x = Test-Path "$Project\.DEXOR_IN_PROGRESS"
        $stats = ""
        if (Test-Path "$Project\brute-force-stats.json") {
            try {
                $j = Get-Content "$Project\brute-force-stats.json" -Raw | ConvertFrom-Json
                $stats = " keys=$($j.keys_tested) rate=$($j.keys_per_sec) hits=$($j.matches_found)"
            } catch {}
        }
        Log "alive dash=$d brute=$b core=$c dexor=$x$stats"
    } catch {
        Log "loop error: $_"
    }
    Start-Sleep -Seconds 30
}
