@echo off
REM BTCSolver - Scanner le blockchain pour une cl^e prive^e
REM Usage: scan-key.bat "cle-privee"
REM Exemples:
REM   scan-key.bat "5HueCGU..."
REM   scan-key.bat "a1b2c3d4e5f6..."
REM   scan-key.bat "zoo zone zoo zone zoo zone zoo zone zoo zone zoo account"

if "%~1"=="" (
    echo.
    echo BTCSolver - Verifier le solde d'une cle privee
    echo.
    echo Usage: scan-key.bat "cle-privee"
    echo.
    echo Formats accepts:
    echo   WIF      - 5HueCGU...
    echo   Hex      - a1b2c3d4e5f6... (32 octets / 64 caracteres)
    echo   BIP39    - "mot1 mot2 ... mot12" (12+ mots)
    echo.
    echo Exemple: scan-key.bat "zoo zone zoo zone zoo zone zoo zone zoo zone zoo account"
    echo.
    echo Temps estime: ~2h 15min (3790 fichiers, 472 Go, 884M transactions)
    echo.
    goto :end
)

echo Scanning avec la cle: %*
echo.
call "%~dp0query_balance.exe" scan --key "%*"

:end
pause
