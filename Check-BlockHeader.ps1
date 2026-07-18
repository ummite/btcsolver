$bytes = [System.IO.File]::ReadAllBytes('W:\Bitcoin\blocks\blk00000.dat')
Write-Output "File size: $($bytes.Length) bytes"

# Magic at offset 0
$magic = [BitConverter]::ToUInt32($bytes[0..3], $false)
Write-Output "Magic: 0x{0:x8}" -f $magic

# Block size at offset 84 (after magic 4 + header 80)
$blockSize = [BitConverter]::ToUInt32($bytes[84..87], $false)
Write-Output "Block size at offset 84: $blockSize"

# Also check offset 80 (maybe no size field)
Write-Output "Bytes at offset 80-91:"
for ($i = 80; $i -lt 92; $i++) {
    if ($i -lt $bytes.Length) {
        Write-Host ("{0:x2}" -f $bytes[$i]) -NoNewline
        Write-Host " " -NoNewline
    }
}
Write-Host ""

# Try to decode as block size at offset 80
$blockSize2 = [BitConverter]::ToUInt32($bytes[80..83], $false)
Write-Output "Bytes 80-83 as uint32: $blockSize2"
