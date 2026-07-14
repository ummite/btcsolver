@echo off
setlocal EnableDelayedExpansion

echo ============================================
echo   BTCSolver GPU Build Script
echo   Auto-detects GPU architecture + CPU cores
echo ============================================
echo.

:: Find nvcc
set "NVCC=nvcc"
where nvcc >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] nvcc not found in PATH.
    echo   Install CUDA Toolkit 12.8+ from https://developer.nvidia.com/cuda-toolkit
    exit /b 1
)

:: Find rustc/cargo
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] cargo not found in PATH.
    echo   Install Rust from https://rustup.rs
    exit /b 1
)

echo [1/5] Detecting hardware...

:: Detect GPU name via PowerShell (more reliable than wmic)
for /f "delims=" %%A in ('powershell -NoProfile -Command "(Get-CimInstance Win32_VideoController).Name"') do (
    set "GPU_NAME=%%A"
)
echo   GPU: !GPU_NAME!

:: Auto-detect compute capability from GPU name
set "ARCH=sm_89"
echo !GPU_NAME! | findstr /i "5090" >nul && set "ARCH=sm_120"
echo !GPU_NAME! | findstr /i "5080" >nul && set "ARCH=sm_120"
echo !GPU_NAME! | findstr /i "5070" >nul && set "ARCH=sm_120"
echo !GPU_NAME! | findstr /i "4090" >nul && set "ARCH=sm_89"
echo !GPU_NAME! | findstr /i "4080" >nul && set "ARCH=sm_89"
echo !GPU_NAME! | findstr /i "4070" >nul && set "ARCH=sm_89"
echo !GPU_NAME! | findstr /i "3090" >nul && set "ARCH=sm_86"
echo !GPU_NAME! | findstr /i "3080" >nul && set "ARCH=sm_86"
echo !GPU_NAME! | findstr /i "3070" >nul && set "ARCH=sm_86"
echo !GPU_NAME! | findstr /i "RTX 20" >nul && set "ARCH=sm_86"
echo !GPU_NAME! | findstr /i "1660" >nul && set "ARCH=sm_75"
echo !GPU_NAME! | findstr /i "1650" >nul && set "ARCH=sm_75"
echo !GPU_NAME! | findstr /i "1080" >nul && set "ARCH=sm_61"
echo !GPU_NAME! | findstr /i "1070" >nul && set "ARCH=sm_61"

echo   Architecture detected: !ARCH!

:: Detect RAM
for /f "tokens=2 delims==" %%A in ('wmic OS get TotalVisibleMemorySize /value ^| findstr "TotalVisibleMemorySize"') do (
    set "RAM_MB=%%A"
)
set /a "RAM_GB=!RAM_MB! / 1024"
echo   RAM: !RAM_GB! GB

:: CPU cores
set "NUM_CORES=!NUMBER_OF_PROCESSORS!"
echo   CPU logical cores: !NUM_CORES!
echo.

:: Allow arch override
set /p "ARCH_OVERRIDE=Override architecture? (leave empty for !ARCH!): "
if not "!ARCH_OVERRIDE!"=="" set "ARCH=!ARCH_OVERRIDE!"

echo [2/5] Compiling CUDA kernel (arch=!ARCH!)...
cd /d "%~dp0src\gpu"

:: Ensure target/release exists
if not exist "..\..\target\release" mkdir "..\..\target\release"

%NVCC% -O3 -arch=!ARCH! -Xcompiler /MD -Xcompiler /O2 -shared -o ..\..\target\release\libsecp_gpu.dll secp256k1_kernel.cu

if %errorlevel% neq 0 (
    echo [ERROR] CUDA compilation failed (exit code %errorlevel%).
    echo   - Verify CUDA Toolkit version matches your driver
    echo   - Try a different -arch= value (sm_86, sm_89, sm_120)
    echo   - Run nvcc --version to check toolkit
    exit /b 1
)

echo   DLL compiled: libsecp_gpu.dll
for %%A in ("..\..\target\release\libsecp_gpu.dll") do echo   Size: %%~zA bytes
echo.

echo [3/5] Building Rust binary (!NUM_CORES! jobs)...
cd /d "%~dp0"
set "CARGO_BUILD_JOBS=!NUM_CORES!"

cargo build --release --bin brute_force --jobs !NUM_CORES! 2>&1

if %errorlevel% neq 0 (
    echo [ERROR] Rust build failed.
    exit /b 1
)

echo.
echo [4/5] Verifying deployment...
set "OK=1"
if exist "target\release\brute_force.exe" (
    echo   [OK] brute_force.exe
) else (
    echo   [FAIL] brute_force.exe not found
    set "OK=0"
)
if exist "target\release\libsecp_gpu.dll" (
    echo   [OK] libsecp_gpu.dll
) else (
    echo   [FAIL] libsecp_gpu.dll not found
    set "OK=0"
)
echo.

if "!OK!"=="0" (
    echo [ERROR] Build incomplete.
    exit /b 1
)

echo [5/5] Recommended config...

:: Recommend batch size based on RAM
set "BATCH=256000"
if !RAM_GB! GTR 64 set "BATCH=512000"
if !RAM_GB! GTR 128 set "BATCH=1024000"

:: Recommend threads
set "THREADS=!NUM_CORES!"
if !THREADS! GTR 64 set "THREADS=64"

echo ============================================
echo   BUILD COMPLETE
echo ============================================
echo.
echo Run (max performance):
echo   cd target\release
echo   brute_force.exe --db-path ..\..\utxo-index.redb --threads !THREADS! --random --use-gpu --batch-size !BATCH! --stop-on-match --stats-interval 10
echo.
echo Or use the auto-launcher:
echo   ..\..\run-max-performance.bat
echo.

:: Copy to project root for convenience
if not exist "..\brute_force_gpu.exe" (
    copy "brute_force.exe" "..\brute_force_gpu.exe" >nul
    echo   Copied brute_force_gpu.exe to project root.
)
