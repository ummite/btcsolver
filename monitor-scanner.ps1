# BTCSolver - Monitor de scanner en temps reel
# Affiche les stats dans une fenetre visible, mise a jour toutes les 5 secondes

$PC_NAME = $env:COMPUTERNAME
$StatsFile = "brute-force-stats-$PC_NAME.json"
$OutputFile = "found-keys-$PC_NAME.json"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  BTCSolver - Monitor de Scanner" -ForegroundColor Cyan
Write-Host "  PC: $PC_NAME" -ForegroundColor Cyan
Write-Host "  Fichier stats: $StatsFile" -ForegroundColor Gray
Write-Host "  Appuyez sur Ctrl+C pour fermer ce monitor" -ForegroundColor Gray
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$lastKeysTested = 0
$startTime = $null

while ($true) {
    # Nettoyer l'ecran (garder l'en-tete)
    Clear-Host
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "  BTCSolver - Scanner de Cleees Privees - Monitor Live" -ForegroundColor Cyan
    Write-Host "  PC: $PC_NAME" -ForegroundColor Cyan
    Write-Host "  Heure: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Cyan
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host ""

    if (Test-Path $StatsFile) {
        try {
            $stats = Get-Content $StatsFile -Raw | ConvertFrom-Json

            $keysTested = [ulong]$stats.keys_tested
            $keysPerSec = [ulong]$stats.keys_per_sec
            $matches = $stats.matches_found
            $elapsed = $stats.elapsed_human

            # Format les nombres pour la lecture
            $keysDisplay = if ($keysTested -gt 1万亿) {
                "$([math]::Round($keysTested / 1万亿, 2))万亿"
            } elseif ($keysTested -gt 1亿) {
                "$([math]::Round($keysTested / 1亿, 2))亿"
            } elseif ($keysTested -gt 1000000) {
                "$([math]::Round($keysTested / 1000000, 2))M"
            } elseif ($keysTested -gt 1000) {
                "$([math]::Round($keysTested / 1000, 2))K"
            } else {
                "$keysTested"
            }

            $rateDisplay = if ($keysPerSec -gt 1000000) {
                "$([math]::Round($keysPerSec / 1000000, 2))M clees/sec"
            } elseif ($keysPerSec -gt 1000) {
                "$([math]::Round($keysPerSec / 1000, 2))K clees/sec"
            } else {
                "$keysPerSec clees/sec"
            }

            Write-Host "  Cleees testeess:     $keysDisplay ($keysTested)" -ForegroundColor White
            Write-Host "  Vitesse:             $rateDisplay" -ForegroundColor Green
            Write-Host "  Temps ecoule:        $elapsed" -ForegroundColor White
            Write-Host "  Matches trouves:     " -NoNewline -ForegroundColor White

            if ($matches -gt 0) {
                Write-Host "$matches" -ForegroundColor Yellow
            } else {
                Write-Host "0" -ForegroundColor Gray
            }

            # Afficher les clees trouvees si existantes
            if (Test-Path $OutputFile) {
                try {
                    $foundKeys = Get-Content $OutputFile -Raw | ConvertFrom-Json
                    if ($foundKeys -and $foundKeys.Count -gt 0) {
                        Write-Host ""
                        Write-Host "  >>> CLEES TROUVEES ($($foundKeys.Count)):" -ForegroundColor Yellow
                        foreach ($key in $foundKeys) {
                            Write-Host "    Cle:  $($key.key_hex)" -ForegroundColor Yellow
                            Write-Host "    Solde: $($key.btc) BTC ($($key.sats) sats)" -ForegroundColor Yellow
                            foreach ($addr in $key.addresses) {
                                Write-Host "    Adr:  $addr" -ForegroundColor Yellow
                            }
                        }
                    }
                } catch {
                    # Ignorer les erreurs de parsing
                }
            }

            # Barre de progression visuelle (vitesse relative)
            $barWidth = 40
            if ($keysPerSec -gt 0) {
                $fillCount = [math]::Min($barWidth, [math]::Max(1, [int]($keysPerSec / 100000)))
                $bar = "[" + ("#" * $fillCount) + ("." * ($barWidth - $fillCount)) + "]"
                Write-Host ""
                Write-Host "  [$bar]" -ForegroundColor Green
            }

        } catch {
            Write-Host "  En attente des donnees du scanner..." -ForegroundColor Yellow
        }
    } else {
        Write-Host "  En attente du demarrage du scanner..." -ForegroundColor Yellow
        Write-Host "  Le fichier $StatsFile n'existe pas encore." -ForegroundColor Gray
    }

    Write-Host ""
    Write-Host "------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host "  Prochaine mise a jour dans 5 secondes..." -ForegroundColor DarkGray
    Write-Host "------------------------------------------------------------" -ForegroundColor DarkGray

    Start-Sleep -Seconds 5
}
