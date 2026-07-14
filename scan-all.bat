@echo off
REM BTCSolver - Scanner les 128 phrases BIP39 completes
REM Usage: scan-all.bat

echo ============================================================
echo BTCSolver - Scan des 128 phrases BIP39
echo zoo zone zoo zone zoo zone zoo zone zoo zone zoo + 12eme mot
echo ============================================================
echo.
echo Temps estime: ~2h 15min
echo Blocs: 804 897 | Transactions: 884 714 086
echo Adresses: 512 (128 phrases x 4 types)
echo.
pause

call "%~dp0scan_blocks.exe"
