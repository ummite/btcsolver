@echo off
setlocal
title BTCSolver Always-On (Core + UTXO + Dashboard)
cd /d Y:\btcsolver

echo.
echo  ============================================================
echo   Installation ALWAYS-ON
echo   - bitcoind toujours vivant (W:\Bitcoin)
echo   - UTXO auto des que tip atteint (dumptxoutset)
echo   - Dashboard + brute surveilles
echo   - Taches planifiees Windows
echo  ============================================================
echo.

if not exist "C:\btcsolver-cache" mkdir "C:\btcsolver-cache"
if not exist "W:\Temp" mkdir "W:\Temp"
if not exist "Y:\btcsolver\data" mkdir "Y:\btcsolver\data"

REM --- Startup folder ---
set "STARTUP=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup"
set "TARGET=%STARTUP%\BTC-Solver-AlwaysOn.bat"
(
echo @echo off
echo timeout /t 30 /nobreak ^>nul
echo if not exist Y:\btcsolver\Keep-Core-And-Utxo.ps1 exit /b 1
echo start "" /MIN powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "Y:\btcsolver\Keep-Core-And-Utxo.ps1"
echo start "" /MIN powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "Y:\btcsolver\Watch-BtcSolver.ps1"
) > "%TARGET%"
echo  [OK] Startup: %TARGET%

REM --- Core + UTXO every 3 minutes ---
schtasks /Delete /TN "BTCSolver-Core-Utxo" /F >nul 2>&1
schtasks /Create /TN "BTCSolver-Core-Utxo" /F /SC MINUTE /MO 3 /RL LIMITED ^
  /TR "powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File Y:\btcsolver\Keep-Core-And-Utxo.ps1"
if errorlevel 1 (echo  [WARN] tache Core-Utxo) else (echo  [OK] Tache BTCSolver-Core-Utxo / 3 min)

REM --- Dashboard + brute every 2 minutes ---
schtasks /Delete /TN "BTCSolver-Watchdog" /F >nul 2>&1
schtasks /Create /TN "BTCSolver-Watchdog" /F /SC MINUTE /MO 2 /RL LIMITED ^
  /TR "powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File Y:\btcsolver\Watch-BtcSolver.ps1"
if errorlevel 1 (echo  [WARN] tache Watchdog) else (echo  [OK] Tache BTCSolver-Watchdog / 2 min)

REM --- Enable if disabled ---
schtasks /Change /TN "BTCSolver-Core-Utxo" /ENABLE >nul 2>&1
schtasks /Change /TN "BTCSolver-Watchdog" /ENABLE >nul 2>&1

echo.
echo  Demarrage immediat...
powershell -NoProfile -ExecutionPolicy Bypass -File "Y:\btcsolver\Keep-Core-And-Utxo.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File "Y:\btcsolver\Watch-BtcSolver.ps1"

echo.
echo  Status: Y:\btcsolver\data\CORE-UTXO-STATUS.json
echo  Logs:   C:\btcsolver-cache\core-utxo.log
echo          C:\btcsolver-cache\watchdog.log
echo  UI:     http://127.0.0.1:3000/
echo.
echo  IMPORTANT: tant que Core n'est pas au tip (blocks~headers),
echo  l'UTXO tip ne peut PAS etre genere localement.
echo  Core doit rester allume 24/7 pour finir le sync.
echo.
if /I "%~1"=="/nopause" goto :eof
pause
