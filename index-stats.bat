@echo off
REM BTCSolver - Statistiques de l'index UTXO
REM Usage: index-stats.bat

call "%~dp0full_utxo_indexer.exe" stats

pause
