@echo off
setlocal
set "MSVC_DIR=C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133"
set "VC_BIN=%MSVC_DIR%\bin\Hostx64\x64"
set PATH=%VC_BIN%;%PATH%
set "NVCC=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.3\bin\nvcc.exe"
set "SRC=Y:\btcsolver\src\gpu\secp256k1_kernel.cu"
set "OUT=Y:\btcsolver\libsecp_gpu_new.dll"

echo === Compile secp256k1 kernel ===
echo NVCC: %NVCC%
echo CL: %VC_BIN%\cl.exe
echo SRC: %SRC%
echo OUT: %OUT%
echo.

echo Compiling (this takes 5-15 minutes)...
"%NVCC%" -O3 -gencode=arch=compute_86,code=sm_86 -gencode=arch=compute_120,code=sm_120 -Xcompiler /MD,/O2 -shared -o "%OUT%" "%SRC%"
echo NVCC_EXIT=%errorlevel%

if exist "%OUT%" (
  echo.
  echo DLL compiled successfully
  dir "%OUT%"
  echo.
  echo Stopping brute_force for DLL swap...
  taskkill /IM brute_force.exe /F 2>nul
  timeout /t 3 /nobreak >nul
  copy /Y "%OUT%" "Y:\btcsolver\libsecp_gpu.dll"
  echo DLL swapped to libsecp_gpu.dll
  echo.
  echo Next: dashboard will auto-restart brute_force
) else (
  echo.
  echo FAILED - DLL not created
)
