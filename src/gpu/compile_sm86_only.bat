@echo off
setlocal
set "MSVC_DIR=C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133"
set "VC_BIN=%MSVC_DIR%\bin\Hostx64\x64"
set PATH=%VC_BIN%;%PATH%
set "NVCC=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.3\bin\nvcc.exe"
set "SRC=Y:\btcsolver\src\gpu\secp256k1_kernel.cu"
set "OUT=Y:\btcsolver\libsecp_gpu_sm86.dll"

echo === Compile CUDA kernel (sm_86 only - RTX 3090) ===
echo This tests if nvcc can complete with a single architecture

"%NVCC%" -O3 -arch=sm_86 -Xcompiler /MD,/O2 -shared -o "%OUT%" "%SRC%" 2>&1
echo EXIT=%errorlevel%

if exist "%OUT%" (
  echo SUCCESS - DLL created
  dir "%OUT%"
) else (
  echo FAILED
)
