@echo off
REM Live monitoring window for BTC Solver brute-force
REM Opens a visible console with real-time stats

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0monitor-live.ps1"
