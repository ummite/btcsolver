<# Merge-Corpus.ps1 — Fusionne et déduplicer tous les corpus en un seul fichier trié #>
# Usage: .\Merge-Corpus.ps1

$ErrorActionPreference = "Stop"
Set-Location Y:\btcsolver

$files = @(
    "data\easy-keys-corpus.txt",
    "data\easy-keys-v2.txt",
    "data\human-2009-keys.txt",
    "data\math-constants-keys.txt",
    "data\pop-culture-2009-keys.txt",
    "data\forgotten-mega-corpus.txt"
)

$output = "data\merged-corpus.txt"

Write-Host "=== Corpus Merge & Dedup ===" -ForegroundColor Cyan
Write-Host "Files to merge:"
foreach ($f in $files) {
    if (Test-Path $f) {
        $size = [math]::Round((Get-Item $f).Length / 1MB, 1)
        Write-Host "  $f ($size MB)"
    } else {
        Write-Host "  $f (MISSING)" -ForegroundColor Yellow
    }
}

Write-Host "`nMerging and deduplicating..." -ForegroundColor Yellow
$sw = [System.Diagnostics.Stopwatch]::StartNew()

# Use cat + sort -u equivalent in PowerShell
# Sort-Object -Unique does a full in-memory sort, which is memory-intensive
# Alternative: use Rust sort utility or process in chunks

# Strategy: cat all files | Sort-Object -Unique | Out-File
# This loads everything into memory but deduplicates efficiently

$totalLines = 0
foreach ($f in $files) {
    if (Test-Path $f) {
        $c = (Get-Content $f | Measure-Object -Line).Lines
        $totalLines += $c
        Write-Host "  $f : $c lines"
    }
}
Write-Host "Total lines (before dedup): $totalLines"

# Merge with dedup
Get-Content $files | Sort-Object -Unique | Set-Content -NoNewline $output

$elapsed = $sw.Elapsed
$mergedLines = (Get-Content $output | Measure-Object -Line).Lines
$mergedSize = [math]::Round((Get-Item $output).Length / 1MB, 1)
$duplicates = $totalLines - $mergedLines

Write-Host ""
Write-Host "=== Results ===" -ForegroundColor Green
Write-Host "Output: $output"
Write-Host "Lines (after dedup): $mergedLines"
Write-Host "Duplicates removed: $duplicates"
Write-Host "File size: $mergedSize MB"
Write-Host "Time: $elapsed"
