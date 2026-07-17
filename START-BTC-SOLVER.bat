@echo off
REM ============================================================
REM  BTC Solver — demarrage complet + verification HTTP
REM  Double-clic = tout relance (Core + Dashboard + Recherche)
REM ============================================================
setlocal
title BTC Solver - Start
cd /d Y:\btcsolver

echo.
echo  ============================================================
echo   BTC Solver — demarrage
echo   Dashboard: http://127.0.0.1:3000/
echo  ============================================================
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "Y:\btcsolver\Watch-BtcSolver.ps1"

REM Keep-alive loop en fond (en plus de la tache planifiee)
start "BTC-Solver-KeepAlive" /MIN powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "Y:\btcsolver\Start-BtcSolver-Auto.ps1"

echo  Attente du dashboard...
set /a n=0
:wait
set /a n+=1
powershell -NoProfile -Command "try { $r=Invoke-WebRequest -Uri 'http://127.0.0.1:3000/' -UseBasicParsing -TimeoutSec 2; exit 0 } catch { exit 1 }"
if %errorlevel%==0 goto ready
if %n% geq 60 goto fail
timeout /t 2 /nobreak >nul
goto wait

:ready
echo.
echo  [OK] Dashboard repond: http://127.0.0.1:3000/
start http://127.0.0.1:3000/
echo  Log watchdog: Y:\btcsolver\watchdog.log
echo.
if /I "%~1"=="/nopause" goto :eof
if /I "%~1"=="--startup" goto :eof
pause
goto :eof

:fail
echo.
echo  [ERREUR] Dashboard ne repond pas apres ~2 min.
echo  Voir: Y:\btcsolver\watchdog.log
echo        Y:\btcsolver\dashboard.err.log
echo.
if /I "%~1"=="/nopause" exit /b 1
pause
exit /b 1
