@echo off
REM BTCSolver - Random Key Brute-Force (LOCAL cache mode)
REM
REM Uses local disk cache for fast index loading.
REM Run sync-cache.bat first to ensure local cache exists.
REM
REM Usage:
REM   brute-force-local.bat                        (unlimited, all cores)
REM   brute-force-local.bat --count 1000000000     (1 billion keys)
REM   brute-force-local.bat --threads 8 --count 500M
REM

cd /d %~dp0

REM Get local cache path
for /f "delims=" %%p in ('cache_manager.exe path') do set CACHE_DIR=%%p

set LOCAL_INDEX=%CACHE_DIR%\utxo-index.redb

if not exist "%LOCAL_INDEX%" (
    echo ERROR: Local cache not found at: %LOCAL_INDEX%
    echo Run sync-cache.bat init first to create the local cache.
    echo.
    pause
    exit /b 1
)

if not exist "brute_force.exe" (
    echo ERROR: brute_force.exe not found!
    pause
    exit /b 1
)

echo ========================================
echo  BTCSolver - Random Key Brute-Force
echo  Local cache: %LOCAL_INDEX%
echo ========================================
echo.

brute_force.exe --random --db-path "%LOCAL_INDEX%" %*

echo.
echo Done. Press any key to exit...
pause >nul
