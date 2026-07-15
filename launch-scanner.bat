@echo off
REM BTCSolver - Scanner de clefs privees avec solde
REM Lance une fenetre visible avec les resultats en temps reel
REM Fichiers de sortie specifiques a ce PC pour eviter les conflits

set PC_NAME=%COMPUTERNAME%
set OUTPUT_FILE=found-keys-%PC_NAME%.json
set PROGRESS_FILE=brute-force-progress-%PC_NAME%.json
set STATS_FILE=brute-force-stats-%PC_NAME%.json
set THREADS=22
set SNAPSHOT=utxo-index.snapshot

cd /d %~dp0

echo ============================================================
echo  BTCSolver - Scanner de clees privees Bitcoin
echo  PC: %PC_NAME%
echo  Threads CPU: %THREADS% (sur 24 cœurs, 2 reservés systeme)
echo  Mode: CLEES ALEATOIRES (aucun conflit inter-PC)
echo  Snapshot: %SNAPSHOT%
echo  Sortie: %OUTPUT_FILE%
echo  Stats: %STATS_FILE% (misse a jour toutes les 15s)
echo ============================================================
echo.
echo Appuyez sur Ctrl+C pour arreter
echo.

REM Lance en mode aleatoire avec arret auto sur premier match
REM --random = pas de conflit avec un autre PC en mode sequentiel
REM --stop-on-match = s'arrete des qu'un solde est trouve
REM --threads 22 = 24 cœurs - 2 pour le systeme
REM --stats-interval 15 = stats mises a jour frequemment
REM --progress-interval 60 = sauvegarde progression toutes les 60s

start "BTCSolver Scanner - %PC_NAME%" cmd /k "brute_force.exe --random --stop-on-match --threads %THREADS% --batch-size 256000 --addr-types legacy,segwit,wrapped,taproot --output-file %OUTPUT_FILE% --stats-interval 15 --stats-file %STATS_FILE% --progress-file %PROGRESS_FILE% --progress-interval 60 --snapshot-path %SNAPSHOT%"

echo.
echo Scanner lance dans une fenetre separee!
echo.
echo Pour voir les stats en temps reel, ouvrez: %STATS_FILE%
echo Les clees trouvees seront sauvees dans: %OUTPUT_FILE%
echo.
pause
