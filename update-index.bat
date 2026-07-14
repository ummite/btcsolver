@echo off
REM BTCSolver - Mettre a jour l'index UTXO avec les nouveaux blocs
REM Usage: update-index.bat
REM Re-scanne a partir du dernier checkpoint sauvegarde

echo ============================================================
echo BTCSolver - Update de l'index UTXO
echo ============================================================
echo.
echo Cette operation ne scanne que les nouveaux blocs
echo depuis la derniere mise a jour.
echo.
echo Temps: ~1-2 minutes (si peu de nouveaux blocs)
echo.
pause

call "%~dp0full_utxo_indexer.exe" build
