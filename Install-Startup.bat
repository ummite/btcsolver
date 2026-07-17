@echo off
REM Installe BTC Solver au demarrage de la session Windows (Startup folder)
setlocal
set "STARTUP=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup"
set "TARGET=%STARTUP%\BTC-Solver-Auto.bat"
set "SRC=Y:\btcsolver\START-BTC-SOLVER.bat"

echo.
echo  Installation demarrage automatique...
echo  Cible: %TARGET%
echo.

(
echo @echo off
echo REM Auto-genere — ne pas editer; source: Y:\btcsolver\START-BTC-SOLVER.bat
echo REM Delai pour monter W: / Y: apres login
echo timeout /t 25 /nobreak ^>nul
echo if not exist Y:\btcsolver\Start-BtcSolver-Auto.ps1 exit /b 1
echo start "BTC-Solver-KeepAlive" /MIN powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "Y:\btcsolver\Start-BtcSolver-Auto.ps1"
) > "%TARGET%"

if exist "%TARGET%" (
    echo  OK — au prochain login Windows, Core + Dashboard + Recherche demarreront.
    echo  Fichier: %TARGET%
) else (
    echo  ERREUR: impossible d'ecrire dans le dossier Startup
    exit /b 1
)
echo.
echo  Pour desinstaller: supprime "%TARGET%"
echo  ou lance: Y:\btcsolver\Uninstall-Startup.bat
echo.
if /I "%~1"=="/nopause" goto :eof
pause
