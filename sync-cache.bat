@echo off
REM BTCSolver - Sync local cache with SAN index
REM
REM Usage:
REM   sync-cache.bat init      - First-time: copy index from SAN to local disk
REM   sync-cache.bat sync      - Check for updates and sync if needed
REM   sync-cache.bat status    - Show cache status
REM   sync-cache.bat path      - Print local cache path
REM
REM Auto-detects fastest local disk with 10+ GB free space.
REM Override with: sync-cache.bat init --cache-dir D:\btcsolver-cache

cd /d %~dp0

if not exist "cache_manager.exe" (
    echo ERROR: cache_manager.exe not found!
    pause
    exit /b 1
)

cache_manager.exe --san-path "Y:\btcsolver\utxo-index.redb" %*
