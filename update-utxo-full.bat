@echo off
echo ================================================================
echo   BTCSolver - Full UTXO Index Rebuild
echo ================================================================
echo.
echo   Blocks dir : Y:\Bitcoin\blocks
echo   XOR key    : b3a2cd522df3a049
echo   Output     : utxo-index.redb + utxo-index.snapshot
echo.
echo   This rebuilds the entire UTXO index from block files.
echo   Expect several hours depending on disk/network speed.
echo.

cd /d Y:\btcsolver

if exist "target\release\full_utxo_indexer.exe" (
    set INDEXER=target\release\full_utxo_indexer.exe
) else if exist "full_utxo_indexer.exe" (
    set INDEXER=full_utxo_indexer.exe
) else (
    echo [ERROR] full_utxo_indexer.exe not found. Build first:
    echo   cargo build --release --bin full_utxo_indexer
    pause
    exit /b 1
)

echo Using: %INDEXER%
echo Starting at %DATE% %TIME%
echo.

%INDEXER% build --blocks-dir "Y:\Bitcoin\blocks" --db-path "utxo-index.redb" --obf-key "b3a2cd522df3a049" --start-file 0 --checkpoint-interval 100

echo.
echo Finished at %DATE% %TIME%
echo.
if exist "utxo-index.snapshot" (
    for %%A in ("utxo-index.snapshot") do echo Snapshot: %%~zA bytes
)
if exist "utxo-index.redb" (
    for %%A in ("utxo-index.redb") do echo Redb: %%~zA bytes
)
echo.
pause
