@echo off
setlocal EnableDelayedExpansion

:: ============================================================================
::  BTCSolver — Bible Brainwallet Scanner (Quick Non-Interactive Mode)
:: ============================================================================
::  Version non-interactive — lance directement avec les paramètres par défaut.
::  Usage : run-bible-solver-quick.bat [light|normal|full]
:: ============================================================================

echo.
echo ================================================================
echo   BTCSolver — Bible Brainwallet Scanner (Quick Mode)
echo ================================================================
echo.

:: --- Configuration ---
set SNAPSHOT=utxo-index.snapshot
set ENGLISH_FILE=bible-english.txt
set FRENCH_FILE=bible-french.txt
set OUTPUT_PATTERNS=bible-patterns-all.txt
set SCAN_OUTPUT=brainwallet-bible-matches.json
set HASH_METHOD=both
set MIN_VALUE=0
set MAX_ACRONYM=8
set MAX_WORD_COMBO=3

:: Mode from command line arg (default: normal)
set MODE=%1
if "%MODE%"=="" set MODE=normal

if "%MODE%"=="light" (
    set SUFFIX_FLAGS=--no-suffixes --no-leet
) else (
    set SUFFIX_FLAGS=
)

echo [Config] Mode: %MODE%
echo [Config] Hash: %HASH_METHOD%
echo.

:: --- Etape 1: Générer les patterns ---
echo ================================================================
echo   ETAPE 1: Generation des patterns Bible
echo ================================================================

target\release\bible_pattern_generator.exe ^
    --english "%ENGLISH_FILE%" ^
    --french "%FRENCH_FILE%" ^
    --output "%OUTPUT_PATTERNS%" ^
    --max-acronym-len %MAX_ACRONYM% ^
    --max-word-combo %MAX_WORD_COMBO% ^
    %SUFFIX_FLAGS%

if errorlevel 1 (
    echo [ERROR] Pattern generation failed!
    pause
    exit /b 1
)

:: --- Compter les patterns ---
find /c /v "" "%OUTPUT_PATTERNS%" > pattern_count.tmp
set /p PATTERN_COUNT=<pattern_count.tmp
set /a PATTERN_COUNT=%PATTERN_COUNT% - 1
del pattern_count.tmp

echo.
echo [Info] Patterns generated: %PATTERN_COUNT%
echo.

:: --- Etape 2: Scanner ---
echo ================================================================
echo   ETAPE 2: Scan Brainwallet
echo ================================================================

target\release\brainwallet_extended.exe ^
    --texts "%OUTPUT_PATTERNS%" ^
    --snapshot "%SNAPSHOT%" ^
    --hash %HASH_METHOD% ^
    --min-value %MIN_VALUE% ^
    --output "%SCAN_OUTPUT%"

echo.
echo ================================================================
echo   RESULTATS
echo ================================================================

if exist "%SCAN_OUTPUT%" (
    for %%A in ("%SCAN_OUTPUT%") do set RESULT_SIZE=%%~zA
    if %RESULT_SIZE% GTR 100 (
        echo [!!!] MATCHES FOUND! See: %SCAN_OUTPUT%
        type "%SCAN_OUTPUT%"
    ) else (
        echo [Info] No matches found.
    )
)

echo.
echo [Done] Patterns: %PATTERN_COUNT% | Results: %SCAN_OUTPUT%
echo.
