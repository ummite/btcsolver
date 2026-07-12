@echo off
echo Launching Bitcoin Core with data on A:\Bitcoin ...
set DATADIR=A:\Bitcoin
if not exist "%DATADIR%" mkdir "%DATADIR%"

REM Try common locations for the GUI wallet
if exist "C:\Program Files\Bitcoin\bitcoin-qt.exe" (
    start "" "C:\Program Files\Bitcoin\bitcoin-qt.exe" -datadir=%DATADIR%
    goto :eof
)
if exist "C:\Program Files (x86)\Bitcoin\bitcoin-qt.exe" (
    start "" "C:\Program Files (x86)\Bitcoin\bitcoin-qt.exe" -datadir=%DATADIR%
    goto :eof
)

echo Bitcoin Core GUI not found in standard paths.
echo Please run manually with:
echo "C:\Program Files\Bitcoin\bitcoin-qt.exe" -datadir=Y:\Bitcoin
echo or install Bitcoin Core first from bitcoincore.org
pause
