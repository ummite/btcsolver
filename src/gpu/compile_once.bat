@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set NVCC=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.3\bin\nvcc.exe
"%NVCC%" -O3 -gencode=arch=compute_86,code=sm_86 -gencode=arch=compute_89,code=sm_89 -gencode=arch=compute_120,code=sm_120 -Xcompiler /MD -Xcompiler /O2 -shared -o "Y:\btcsolver\target\release\libsecp_gpu.dll" "Y:\btcsolver\src\gpu\secp256k1_kernel.cu"
echo EXIT=%errorlevel%
dir "Y:\btcsolver\target\release\libsecp_gpu.dll"
