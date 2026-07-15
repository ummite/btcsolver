@echo off
REM Brute-force GPU avec cache local C: (lecture rapide)
REM Master reste sur Y: SAN
REM Mode SEQUENTIEL avec resume automatique depuis brute-force-progress.position

echo === BTC Solver - GPU Brute-Force ===
echo Index : C:\btcsolver-cache\utxo-index.snapshot (cache local)
echo Master: Y:\btcsolver\utxo-index.snapshot (SAN)
echo Mode  : SEQUENTIEL avec resume automatique
echo.

REM Sync cache avant de lancer
call sync-cache.bat

REM Lancer brute-force avec cache local, mode sequentiel, resume automatique
cd Y:\btcsolver\target\release
.\brute_force.exe --snapshot-path C:\btcsolver-cache\utxo-index.snapshot --use-gpu --batch-size 256000 --count 0 --threads 23 --stats-interval 10 --progress-interval 30