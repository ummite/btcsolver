@echo off
setlocal
set "MSVC_DIR=C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133"
set "VC_BIN=%MSVC_DIR%\bin\Hostx64\x64"
set PATH=%VC_BIN%;%PATH%
set "NVCC=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.3\bin\nvcc.exe"
set "SRC=Y:\btcsolver\src\gpu\secp256k1_kernel.cu"
set "OUT=Y:\btcsolver\libsecp_gpu_new.dll"

echo === Compile CUDA kernel (PTX + sm_86 SASS) ===
echo PTX allows JIT on sm_120 (RTX 5090), SASS for sm_86 (RTX 3090)

"%NVCC%" -O3 -gencode=arch=compute_86,code=compute_86 -gencode=arch=compute_86,code=sm_86 -Xcompiler /MD,/O2 -shared -o "%OUT%" "%SRC%"
echo EXIT=%errorlevel%

if exist "%OUT%" (
  echo SUCCESS - DLL created
  dir "%OUT%"
  echo.
  echo Swapping to libsecp_gpu.dll...
  taskkill /IM brute_force.exe /F 2>nul
  timeout /t 3 /nobreak >nul
  copy /Y "%OUT%" "Y:\btcsolver\libsecp_gpu.dll"
  echo DLL deployed! Dashboard will restart brute_force.
) else (
  echo FAILED
)
