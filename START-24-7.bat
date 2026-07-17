@echo off
title BTC Solver 24/7 - scan fond + interface mots
cd /d Y:\btcsolver
echo.
echo  ============================================================
echo   MODE 24/7
echo.
echo   TOUJOURS:
echo     - Scan de cles en fond (brute GPU, ne s'arrete pas)
echo     - Interface web pour tester TES mots quand tu veux
echo     - Bitcoin Core (sauf pendant de-XOR des blocs)
echo.
echo   UI:  http://127.0.0.1:3000/
echo        Onglet "1. Tester une cle" = coller d'autres mots
echo        Onglet "2. Scan listes"    = corpus
echo.
echo   Hits: Y:\btcsolver\found-keys.json
echo   Log:  Y:\btcsolver\keep-alive-24-7.log
echo.
echo   Laisse cette fenetre ouverte (ou minimisee).
echo  ============================================================
echo.
start "BTC-Solver-KeepAlive" /MIN powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0keep-alive-24-7.ps1"
timeout /t 8 /nobreak >nul
start http://127.0.0.1:3000/
echo.
echo  Keep-alive lance. Navigateur ouvert.
echo  Tu peux fermer cette fenetre noire; le keep-alive reste en arriere-plan.
echo.
pause
