$blocks = Get-ChildItem "Y:\Bitcoin\blocks" -Directory | Sort-Object Name -Descending | Select-Object -First 5
foreach ($d in $blocks) {
    Write-Output $d.Name
}
Write-Output "---"
$size = (Get-ChildItem "Y:\Bitcoin\blocks" -Recurse -File | Measure-Object -Property Length -Sum).Sum
Write-Output "Taille totale blocks: $([math]::Round($size / 1GB, 1)) Go"

$chainstate = Get-ChildItem "Y:\Bitcoin\chainstate" -Recurse -File | Measure-Object -Property Length -Sum
Write-Output "Taille chainstate: $([math]::Round($chainstate.Sum / 1GB, 1)) Go"
