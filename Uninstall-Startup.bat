@echo off
setlocal
set "TARGET=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\BTC-Solver-Auto.bat"
if exist "%TARGET%" (
    del /f /q "%TARGET%"
    echo Desinstalle: %TARGET%
) else (
    echo Rien a desinstaller ^(fichier absent^).
)
if /I "%~1"=="/nopause" goto :eof
pause
