@echo off
REM BTCSolver - Construire une cache UTXO pour des cles personnalisees
REM Usage: build-cache.bat "cle1" "cle2" "cle3"

if "%~1"=="" (
    echo.
    echo BTCSolver - Construire une cache UTXO
    echo.
    echo Usage: build-cache.bat "cle1" "cle2" ...
    echo.
    echo Exemple: build-cache.bat "5HueCGU..." "a1b2c3d4..."
    echo.
    echo Temps estime: ~2h 15min
    echo.
    goto :end
)

call "%~dp0query_balance.exe" build --keys %*

:end
pause
