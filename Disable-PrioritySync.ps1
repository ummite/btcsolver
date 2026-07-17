# Disable-PrioritySync.ps1 — reautorise la chasse aux cles (apres Core tip + UTXO frais).
$ErrorActionPreference = "Continue"
$Project = "Y:\btcsolver"
$Flag = "$Project\data\PRIORITY-SYNC.flag"
$StatusFile = "$Project\data\CORE-UTXO-STATUS.json"

# Safety check
if (Test-Path $StatusFile) {
    try {
        $s = Get-Content $StatusFile -Raw | ConvertFrom-Json
        if (-not $s.at_tip) {
            Write-Host "ATTENTION: Core n'est PAS encore au tip (blocks=$($s.blocks) / headers=$($s.headers))." -ForegroundColor Yellow
            Write-Host "Continuer va re-activer la chasse aux cles avec UTXO potentiellement obsolete."
        }
        if (-not $s.utxo_valid_for_tests) {
            Write-Host "ATTENTION: UTXO pas encore valide pour tests (lag_h=$($s.utxo_lag_hours))." -ForegroundColor Yellow
        }
        if ($s.at_tip -and $s.utxo_valid_for_tests) {
            Write-Host "OK: Core tip + UTXO frais — safe pour chercher des cles." -ForegroundColor Green
        }
    } catch {}
}

if (Test-Path $Flag) {
    Remove-Item $Flag -Force
    Write-Host "PRIORITY-SYNC desactive (flag supprime)."
} else {
    Write-Host "Flag deja absent."
}

Write-Host "Watchdog pourra relancer brute_force au prochain tick."
Write-Host "Lance manuellement si besoin: .\Watch-BtcSolver.ps1"
