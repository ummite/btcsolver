@echo off
REM BTCSolver - Construire/Mettre a jour l'index UTXO complet
REM Usage: build-index.bat

echo ============================================================
echo BTCSolver - Construction de l'index UTXO complet
echo ============================================================
echo.
echo Cette operation scanne TOUS les blocs et construit un index
echo qui permet de verifier le solde de N'IMPORTE QUELLE cle
echo en moins d'une seconde.
echo.
echo Temps: ~3-5 heures (premiere construction)
echo       ~1-2 minutes (update incrementale)
echo.
echo Base de donnees: utxo-index.redb
echo.
pause

call "%~dp0full_utxo_indexer.exe" build
