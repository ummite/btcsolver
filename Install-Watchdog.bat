@echo off
setlocal
title Install BTC Solver Watchdog (toujours actif)
cd /d Y:\btcsolver

echo.
echo  ============================================================
echo   Installation watchdog BTC Solver
echo   - Au login Windows
echo   - Toutes les 2 minutes (relance si mort)
echo   - Demarrage immediat maintenant
echo  ============================================================
echo.

REM 1) Startup folder (login)
set "STARTUP=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup"
set "TARGET=%STARTUP%\BTC-Solver-Auto.bat"
(
echo @echo off
echo REM Auto: delai disques puis watchdog
echo timeout /t 20 /nobreak ^>nul
echo if not exist Y:\btcsolver\Watch-BtcSolver.ps1 exit /b 1
echo powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "Y:\btcsolver\Watch-BtcSolver.ps1"
echo start "" /MIN powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "Y:\btcsolver\Start-BtcSolver-Auto.ps1"
) > "%TARGET%"
echo  [OK] Startup: %TARGET%

REM 2) Scheduled task every 2 minutes (survives closed terminals)
schtasks /Delete /TN "BTCSolver-Watchdog" /F >nul 2>&1
schtasks /Create /TN "BTCSolver-Watchdog" /F /SC MINUTE /MO 2 /RL LIMITED ^
  /TR "powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File Y:\btcsolver\Watch-BtcSolver.ps1" ^
  /RU "%USERNAME%"
if errorlevel 1 (
  echo  [WARN] schtasks create failed — essaie en admin ou relance ce bat
) else (
  echo  [OK] Tache planifiee: BTCSolver-Watchdog (toutes les 2 min)
)

REM 3) Also at user logon via schtasks (more reliable than Startup only)
schtasks /Delete /TN "BTCSolver-OnLogon" /F >nul 2>&1
schtasks /Create /TN "BTCSolver-OnLogon" /F /SC ONLOGON /DELAY 0001:00 /RL LIMITED ^
  /TR "powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File Y:\btcsolver\Watch-BtcSolver.ps1" ^
  /RU "%USERNAME%"
if errorlevel 1 (
  echo  [WARN] schtasks OnLogon failed
) else (
  echo  [OK] Tache planifiee: BTCSolver-OnLogon
)

REM 4) Run now
echo.
echo  Demarrage immediat...
powershell -NoProfile -ExecutionPolicy Bypass -File "Y:\btcsolver\Watch-BtcSolver.ps1"
echo.
echo  Ouvre: http://127.0.0.1:3000/
echo  Log:   Y:\btcsolver\watchdog.log
echo.
if /I "%~1"=="/nopause" goto :eof
pause
