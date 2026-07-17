@echo off
title BTC Solver — demarrage simple
cd /d Y:\btcsolver
echo.
echo  ========================================
echo   BTC Solver — un seul clic
echo   Dashboard + Bitcoin Core (W:\Bitcoin)
echo  ========================================
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Launch-Dashboard.ps1"
echo.
echo  Interface: http://127.0.0.1:3000
echo  - Barre du haut = etat Core (a jour OUI/NON)
echo  - Bouton Relancer Core si ca ne marche pas
echo  - Onglet Cles pour tester une cle
echo.
pause
