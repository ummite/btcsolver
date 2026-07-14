@echo off
REM BTCSolver - Verifier le solde depuis une cache UTXO existante
REM Usage: check-cache.bat "cle-privee" [chemin-cache]
REM Exemple: check-cache.bat "5HueCGU..." utxo-cache.bin

if "%~1"=="" (
    echo.
    echo BTCSolver - Verifier solde depuis cache
    echo.
    echo Usage: check-cache.bat "cle-privee" [chemin-cache]
    echo.
    echo Exemple: check-cache.bat "5HueCGU..." utxo-cache.bin
    echo          check-cache.bat "zoo zone zoo zone zoo zone zoo zone zoo zone zoo account"
    echo.
    echo Temps: ^< 1 seconde
    echo.
    goto :end
)

if "%~2"=="" (
    call "%~dp0query_balance.exe" cache --key "%~1"
) else (
    call "%~dp0query_balance.exe" cache --key "%~1" --cache "%~2"
)

:end
pause
