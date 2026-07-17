@echo off
schtasks /Delete /TN "BTCSolver-Core-Utxo" /F >nul 2>&1
schtasks /Delete /TN "BTCSolver-Watchdog" /F >nul 2>&1
del /f /q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\BTC-Solver-AlwaysOn.bat" >nul 2>&1
del /f /q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\BTC-Solver-Auto.bat" >nul 2>&1
echo Always-On desinstalle.
if /I "%~1"=="/nopause" goto :eof
pause
