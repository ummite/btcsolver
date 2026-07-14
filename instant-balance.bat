@echo off
REM BTCSolver - Verifier le solde instantane d'une cle privee
REM Usage: instant-balance.bat "cle-privee"
REM Temps: < 1 seconde (utilise l'index UTXO pre-construit)

if "%~1"=="" (
    echo.
    echo BTCSolver - Solde instantane d'une cle privee
    echo.
    echo Usage: instant-balance.bat "cle-privee"
    echo.
    echo Formats accepts:
    echo   WIF   - 5HueCGU...
    echo   Hex   - a1b2c3d4e5f6... (32 octets / 64 caracteres)
    echo   BIP39 - "mot1 mot2 ... mot12" (12+ mots)
    echo.
    echo Exemple: instant-balance.bat "zoo zone zoo zone zoo zone zoo zone zoo zone zoo account"
    echo.
    echo IMPORTANT: Lancez build-index.bat d'abord pour construire l'index
    echo.
    goto :end
)

call "%~dp0full_utxo_indexer.exe" query --key "%*"

:end
pause
