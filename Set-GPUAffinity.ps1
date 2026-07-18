<#
.SYNOPSIS
    Set GPU affinity for BTCSolver processes to minimize contention with llama-server.

.DESCRIPTION
    Uses CUDA_VISIBLE_DEVICES environment variable to isolate GPU workloads.
    brute_force gets GPUs 0,1 (RTX 5090s) and llama-server gets GPU 2 (RTX 3090).
    Or vice versa depending on preference.

    Alternatively, uses nvidia-smi to compute-priority scheduling.

.PARAMETER Mode
    - Isolate: brute_force on 0,1 llama on 2 (default)
    - AllGpu: brute_force on all 3 GPUs (current)
    - Balance: Set compute priorities

.EXAMPLE
    .\Set-GPUAffinity.ps1 -Mode Isolate
    .\Set-GPUAffinity.ps1 -Mode AllGpu
#>

param(
    [ValidateSet("Isolate", "AllGpu", "Balance", "Status")]
    [string]$Mode = "Status"
)

function Show-GPUStatus {
    Write-Host "=== GPU Status ===" -ForegroundColor Cyan
    nvidia-smi --query-gpu=index,name,utilization.gpu,memory.used,memory.total,temperature.gpu --format=csv
    Write-Host ""

    Write-Host "=== Compute Apps ===" -ForegroundColor Cyan
    nvidia-smi --query-compute-apps=pid,name,gpu_instance_id,compute_instance_id,used_memory --format=csv
    Write-Host ""

    Write-Host "=== Process Info ===" -ForegroundColor Cyan
    Get-Process -Name brute_force,llama-server -ErrorAction SilentlyContinue |
        Select-Object Name, Id, @{N='RAM_GB';E={[math]::Round($_.WorkingSet64/1GB,1)}},
                     @{N='CPU_s';E={[math]::Round($_.CPU,1)}} |
        Format-Table
}

function Set-IsolateMode {
    Write-Host "=== Isolating GPUs ===" -ForegroundColor Yellow
    Write-Host "brute_force -> GPUs 0,1 (RTX 5090s)"
    Write-Host "llama-server -> GPU 2 (RTX 3090)"
    Write-Host ""

    # Set compute priority: brute_force = high, llama = low
    # GPU 0,1: restrict to brute_force PID
    # GPU 2: restrict to llama PID

    $bf = Get-Process -Name brute_force -ErrorAction SilentlyContinue
    $ll = Get-Process -Name llama-server -ErrorAction SilentlyContinue

    if ($bf) {
        Write-Host "brute_force PID: $($bf.Id)"
        # Set environment for next restart
        "[GPU]" | Out-File "Y:\btcsolver\brute_force.ini" -Encoding ASCII
        "CUDA_VISIBLE_DEVICES=0,1" | Out-File "Y:\btcsolver\brute_force.ini" -Append -Encoding ASCII
    }

    if ($ll) {
        Write-Host "llama-server PID: $($ll.Id)"
        # Note: llama-server needs restart with CUDA_VISIBLE_DEVICES=2
        Write-Host "NOTE: llama-server needs restart with CUDA_VISIBLE_DEVICES=2"
        Write-Host "Command: `$env:CUDA_VISIBLE_DEVICES='2'; & C:\Llama\llama-server.exe [args]"
    }

    Write-Host ""
    Write-Host "IMPORTANT: GPU affinity via CUDA_VISIBLE_DEVICES requires process restart."
    Write-Host "The dashboard will restart brute_force on next scan cycle."
}

function Set-AllGpuMode {
    Write-Host "=== All GPUs for brute_force ===" -ForegroundColor Yellow
    Write-Host "Removing GPU restrictions"

    if (Test-Path "Y:\btcsolver\brute_force.ini") {
        Remove-Item "Y:\btcsolver\brute_force.ini" -Force
    }

    Write-Host "brute_force will use all 3 GPUs on next restart"
}

function Set-BalanceMode {
    Write-Host "=== Balanced Mode ===" -ForegroundColor Yellow
    Write-Host "Setting GPU scheduling to time-share"

    # Use nvidia-smi to set compute mode to all processes with time slicing
    # This is the default, but let's make sure
    for ($i = 0; $i -lt 3; $i++) {
        nvidia-smi -i $i -c DEFAULT 2>$null
    }

    Write-Host "All GPUs set to DEFAULT compute mode (time-sharing)"
    Write-Host "GPU utilization will be shared between brute_force and llama-server"
}

switch ($Mode) {
    "Status" { Show-GPUStatus }
    "Isolate" { Set-IsolateMode }
    "AllGpu" { Set-AllGpuMode }
    "Balance" { Set-BalanceMode }
}
