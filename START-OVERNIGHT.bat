@echo off
title BTC Solver OVERNIGHT - Core sync + UTXO + key scan
cd /d Y:\btcsolver
echo.
echo  ============================================================
echo   OVERNIGHT PIPELINE
echo   1. De-XOR remaining block files
echo   2. Bitcoin Core -reindex then sync to tip
echo   3. Rebuild UTXO when synced
echo   4. GPU brute-force non-stop (starts immediately)
echo  ============================================================
echo.
echo  Log: Y:\btcsolver\overnight-pipeline.log
echo  Hits: Y:\btcsolver\found-keys.json
echo  UI:   start Launch-Dashboard.ps1 in another window if needed
echo.
echo  Do NOT close this window overnight.
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0overnight-pipeline.ps1"
pause
