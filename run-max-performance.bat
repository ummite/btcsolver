@echo off
setlocal

echo ============================================
echo   BTCSolver - Max Performance Launcher
echo ============================================
echo.

:: Detect CPU cores
set "LOGICAL_CORES=!NUMBER_OF_PROCESSORS!"
echo   CPU logical cores: !LOGICAL_CORES!

:: Recommend threads (use all logical cores for hyperthreading)
set "THREADS=!LOGICAL_CORES!"
if !THREADS! GTR 64 set "THREADS=64"

:: Detect GPU
set "GPU_COUNT=0"
where nvcc >nul 2>&1
if !errorlevel! equ 0 (
    set "USE_GPU=--use-gpu"
    echo   GPU: CUDA available
) else (
    set "USE_GPU="
    echo   GPU: CUDA not found (CPU only)
)

:: Detect RAM
for /f "tokens=2 delims==" %%A in ('wmic OS get TotalVisibleMemorySize /value ^| findstr "TotalVisibleMemorySize"') do (
    set "RAM_MB=%%A"
)
set /a "RAM_GB=!RAM_MB! / 1024"
echo   RAM: !RAM_GB! GB

:: Adjust batch size based on RAM
set "BATCH=256000"
if !RAM_GB! GTR 64 set "BATCH=512000"
if !RAM_GB! GTR 128 set "BATCH=1024000"

echo   Threads: !THREADS!
echo   Batch size: !BATCH!
echo.

:: Check for binaries
if not exist "target\release\brute_force.exe" (
    echo [ERROR] brute_force.exe not found. Run build-gpu.bat first.
    pause
    exit /b 1
)

if defined USE_GPU (
    if not exist "target\release\libsecp_gpu.dll" (
        echo [ERROR] libsecp_gpu.dll not found. Run build-gpu.bat first.
        pause
        exit /b 1
    )
)

echo   Starting brute-force with GPU...
echo   Press Ctrl+C to stop.
echo.

cd target\release

brute_force.exe --db-path ..\..\utxo-index.redb --threads !THREADS! --random --stop-on-match --output-file found-keys.json --db-retries 0 --stats-interval 10 --use-gpu --batch-size !BATCH!
