@echo off
setlocal
set "MSVC_DIR=C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133"
set "VC_BIN=%MSVC_DIR%\bin\Hostx64\x64"
set PATH=%VC_BIN%;%PATH%
set "NVCC=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.1\bin\nvcc.exe"
set "SRC=Y:\btcsolver\src\gpu\secp256k1_kernel.cu"
set "OUT=Y:\btcsolver\libsecp_gpu_131.dll"

echo === Compile with CUDA 13.1 (dual arch: sm_86 + sm_120) ===
echo NVCC: %NVCC%

"%NVCC%" --version
echo.
"%NVCC%" -O3 -gencode=arch=compute_86,code=sm_86 -gencode=arch=compute_120,code=sm_120 -Xcompiler /MD,/O2 -shared -o "%OUT%" "%SRC%"
echo EXIT=%errorlevel%

if exist "%OUT%" (
  echo SUCCESS
  dir "%OUT%"
) else (
  echo FAILED
)
