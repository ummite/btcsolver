@echo off
REM BTCSolver - Brute-force with auto-stop on first balance found
REM Launches the PowerShell monitoring script
cd /d %~dp0
powershell -ExecutionPolicy Bypass -File "%~dp0brute-force-watch.ps1"
