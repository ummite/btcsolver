@echo off
setlocal
title Launch Bitcoin Core (W:\Bitcoin)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Launch-BitcoinCore.ps1"
if errorlevel 1 pause
