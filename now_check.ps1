$debug_log = "Y:\Bitcoin\debug.log"
$lines = Get-Content $debug_log -Tail 50
$match = $lines | Select-String "UpdateTip: new best=.* height=(\d+) " | Select-Object -Last 1
if ($match) {
    $height_match = [regex]::Match($match.Line, "height=(\d+)")
    if ($height_match.Success) {
        $blocks = [int]$height_match.Groups[1].Value
        $pct = [math]::Round(($blocks / 957649.0) * 100, 4)
        Write-Output "Bloc actuel: $blocks ($pct%)"
    }
}

$blocks_dir = "Y:\Bitcoin\blocks"
if (Test-Path $blocks_dir) {
    $size = (Get-ChildItem $blocks_dir -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
    Write-Output "Disque utilise: $([math]::Round($size / 1MB, 1)) Mo"
}
