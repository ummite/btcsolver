@echo off
setlocal
schtasks /Delete /TN "BTCSolver-Watchdog" /F >nul 2>&1
schtasks /Delete /TN "BTCSolver-OnLogon" /F >nul 2>&1
del /f /q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\BTC-Solver-Auto.bat" >nul 2>&1
echo Watchdog desinstalle (taches + Startup).
if /I "%~1"=="/nopause" goto :eof
pause
