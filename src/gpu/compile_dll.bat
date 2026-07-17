@echo off
setlocal EnableDelayedExpansion
call "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if errorlevel 1 (
  echo FAIL vcvars
  exit /b 1
)
where cl
where nvcc
set "NVCC=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.3\bin\nvcc.exe"
set "SRC=Y:\btcsolver\src\gpu\secp256k1_kernel.cu"
set "OUT=Y:\btcsolver\target\release\libsecp_gpu.dll"
if not exist "Y:\btcsolver\target\release" mkdir "Y:\btcsolver\target\release"
echo Compiling...
"%NVCC%" -O3 -gencode=arch=compute_86,code=sm_86 -gencode=arch=compute_89,code=sm_89 -gencode=arch=compute_120,code=sm_120 -Xcompiler /MD -Xcompiler /O2 -shared -o "%OUT%" "%SRC%"
echo NVCC_EXIT=%errorlevel%
if exist "%OUT%" (
  copy /Y "%OUT%" "Y:\btcsolver\libsecp_gpu.dll"
  dir "%OUT%"
) else (
  echo DLL missing
  exit /b 1
)
exit /b 0
