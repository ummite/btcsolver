@echo off
REM BTCSolver - Brute-force de cl^es priv^ees (CPU multi-core, RAM uniquement)
REM Usage: brute-force.bat [options]
REM
REM Exemples:
REM   brute-force.bat                          # Toutes les cl^es, tous les cœurs CPU
REM   brute-force.bat /c 1000000               # 1 million de cl^es
REM   brute-force.bat /s a1b2c3d4...           # Démarrer depuis une cl^e spécifique
REM   brute-force.bat /t 4 /c 5000000          # 4 threads, 5M cl^es
REM
REM IMPORTANT: Lancez build-index.bat d'abord pour construire utxo-index.redb

echo ============================================================
echo BTCSolver - Brute-force de cl^es priv^ees
echo ============================================================
echo.
echo Mode: CPU multi-core (index UTXO en RAM, 0 I/O disque)
echo Base de donnees: utxo-index.redb
echo.
echo Appuyez sur Ctrl+C pour arr^eter
echo.

call "%~dp0brute_force.exe" %*
