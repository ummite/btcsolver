@echo off
REM Sync UTXO index from SAN (Y:) master to local cache (C:)
REM Y: is always the source of truth; C: is a read-only cache

set SAN_PATH=Y:\btcsolver\utxo-index.snapshot
set CACHE_PATH=C:\btcsolver-cache\utxo-index.snapshot

echo === Sync UTXO Index: SAN -> Local Cache ===
echo Master : %SAN_PATH%
echo Cache  : %CACHE_PATH%

REM Check if master exists
if not exist "%SAN_PATH%" (
    echo ERREUR: Master non trouve sur Y:
    exit /b 1
)

REM Check if cache needs update (compare timestamps)
for %%F in ("%SAN_PATH%") do set MASTER_TIME=%%~tF
for %%F in ("%CACHE_PATH%") do set CACHE_TIME=%%~tF 2>nul

if "%CACHE_TIME%"=="" (
    echo Cache inexistant - copie initiale...
    mkdir "C:\btcsolver-cache" 2>nul
    copy /Y "%SAN_PATH%" "%CACHE_PATH%" >nul
    echo Cache cree sur C:
) else (
    echo Timestamps - Master: %MASTER_TIME% | Cache: %CACHE_TIME%
    REM If master is newer, sync
    if /i "%MASTER_TIME%" gtr "%CACHE_TIME%" (
        echo Master plus recente - sync en cours...
        copy /Y "%SAN_PATH%" "%CACHE_PATH%" >nul
        echo Cache mis a jour!
    ) else (
        echo Cache deja a jour - nothing to do.
    )
)

echo.
echo === Sync termine ===