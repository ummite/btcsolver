@echo off
REM BTCSolver - Random Private Key Brute-Force (RAM mode)
REM
REM Usage: brute-force-random.bat [OPTIONS]
REM
REM Options:
REM   --threads N       Number of CPU threads (default: auto-detect all cores)
REM   --count N         Number of keys to test (default: unlimited, Ctrl+C to stop)
REM   --batch-size N    Batch size per thread (default: 256000)
REM   --addr-types T    Address types to check: legacy,segwit,wrapped,taproot (default: all)
REM   --db-path P       Path to UTXO index (default: utxo-index.redb)
REM
REM IMPORTANT: Requires utxo-index.redb to exist first.
REM   Build it with: build-index.bat
REM
REM Example:
REM   brute-force-random.bat          (unlimited, all cores, all address types)
REM   brute-force-random.bat --count 1000000000  (1 billion keys then stop)
REM   brute-force-random.bat --threads 8 --count 500000000  (8 threads, 500M keys)
REM

cd /d %~dp0

if not exist "utxo-index.redb" (
    echo ERROR: utxo-index.redb not found!
    echo Build the UTXO index first with: build-index.bat
    echo.
    pause
    exit /b 1
)

if not exist "brute_force.exe" (
    echo ERROR: brute_force.exe not found!
    echo Build with: cargo build --bin brute_force --release
    echo.
    pause
    exit /b 1
)

echo ========================================
echo  BTCSolver - Random Key Brute-Force
echo  (All operations in RAM, zero disk I/O)
echo ========================================
echo.

brute_force.exe --random %*

echo.
echo Done. Press any key to exit...
pause >nul
