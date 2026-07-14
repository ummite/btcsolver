@echo off
REM BTCSolver - Auto-launch brute-force when index is ready
REM Waits for first checkpoint, then launches with --stop-on-match

set DB_PATH=utxo-index.redb
set EXE=brute_force.exe
set OUTPUT=found-keys.json
set THREADS=16

echo BTCSolver - Auto Brute-Force Launcher
echo.

:wait_for_index
if not exist %DB_PATH% (
    echo Waiting for index file...
    timeout /t 5 /nobreak >nul
    goto wait_for_index
)

echo Index file found. Waiting for first checkpoint...

:wait_for_checkpoint
REM Try to run brute-force with a small count to test if index has data
%EXE% --db-path %DB_PATH% --threads 1 --random --count 1 2>nul | find "Scripts indexed: 0" >nul
if errorlevel 1 (
    echo Index has data! Launching brute-force...
    goto launch
) else (
    echo Index still empty. Waiting for checkpoint... (5s)
    timeout /t 5 /nobreak >nul
    goto wait_for_checkpoint
)

:launch
echo.
echo ========================================
echo Launching brute-force with auto-stop
echo ========================================
echo.
%EXE% --db-path %DB_PATH% --threads %THREADS% --random --stop-on-match --output-file %OUTPUT%

echo.
echo Done. Check %OUTPUT% for results.
pause
