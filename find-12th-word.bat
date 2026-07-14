@echo off
REM BTCSolver - Trouver le 12eme mot valide pour 11 mots BIP39
REM Usage: find-12th-word.bat mot1 mot2 mot3 mot4 mot5 mot6 mot7 mot8 mot9 mot10 mot11
REM Exemple: find-12th-word.bat zoo zone zoo zone zoo zone zoo zone zoo zone zoo

if "%~1"=="" (
    echo.
    echo BTCSolver - Trouver le 12eme mot BIP39 valide
    echo.
    echo Usage: find-12th-word.bat mot1 mot2 ... mot11
    echo.
    echo Exemple: find-12th-word.bat zoo zone zoo zone zoo zone zoo zone zoo zone zoo
    echo.
    goto :end
)

call "%~dp0fix_mnemonic.exe" %*

:end
pause
