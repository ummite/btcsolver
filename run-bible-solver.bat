@echo off
setlocal EnableDelayedExpansion

:: ============================================================================
::  BTCSolver — Bible Brainwallet Scanner (CPU Mode)
:: ============================================================================
::  Ce script génère TOUS les patterns dérivés de la Bible (anglais + français)
::  et les teste contre l'index UTXO Bitcoin pour trouver des soldes.
::
::  Patterns générés :
::    1. Chaque phrase : majuscule, minuscule, titlecase, original
::    2. Avec/sans ponctuation, avec/sans espaces, underscores, tirets
::    3. Première lettre de chaque phrase (EN+FR) -> acronymes/mots
::    4. Premier mot et dernier mot de chaque phrase
::    5. Premier mot + dernier mot combinés
::    6. Combinaisons multi-mots (1 à 3 mots consécutifs)
::    7. Combinaisons interlangues (anglais + français)
::    8. Patterns ASCII (hex, somme, XOR, ROT13, inversé)
::    9. Suffixes/prefixes brainwallet (private key, bitcoin, wallet...)
::   10. Leet-speak (a->4, e->3, i->1, o->0, t->7, s->5)
::   11. Patterns numériques extraits
::   12. Ordre des mots inversé
::
::  Modes :
::    light  = ~200K patterns (basique + mots, sans suffixes/leet)
::    normal = ~1M patterns   (suffixes + leet sur phrases originales)
::    full   = ~1M patterns   (identique à normal par défaut)
::
::  Hash : SHA-256 + MD5 (configurable)
::  Clés  : compressées + non-compressées
::  Adresses : P2PKH, P2WPKH, P2SH-P2WPKH, P2TR
:: ============================================================================

echo.
echo ================================================================
echo   BTCSolver — Bible Brainwallet Scanner
echo ================================================================
echo.

:: --- Configuration ---
set SNAPSHOT=utxo-index.snapshot
set ENGLISH_FILE=bible-english.txt
set FRENCH_FILE=bible-french.txt
set CUSTOM_FILE=
set OUTPUT_PATTERNS=bible-patterns-all.txt
set SCAN_OUTPUT=brainwallet-bible-matches.json
set HASH_METHOD=both
set MIN_VALUE=0
set THREADS=0
set MAX_ACRONYM=8
set MAX_WORD_COMBO=3
set MODE=normal

:: --- Mode selection ---
echo [Config] Select scan mode:
echo.
echo   [1] LIGHT  (~200K patterns, rapide, sans suffixes/leet)
echo   [2] NORMAL (~1M patterns, suffixes + leet sur phrases)
echo   [3] FULL   (~1M patterns, tout inclus)
echo.
set /p MODE_CHOICE="Choix (1/2/3, defaut=2) : "

if "%MODE_CHOICE%"=="1" (
    set MODE=light
    set NO_SUFFIXES=--no-suffixes
    set NO_LEET=--no-leet
) else if "%MODE_CHOICE%"=="3" (
    set MODE=full
    set NO_SUFFIXES=
    set NO_LEET=
) else (
    set MODE=normal
    set NO_SUFFIXES=
    set NO_LEET=
)

echo [Config] Mode: %MODE%
echo.

:: --- Verification des fichiers ---
echo [Verification] Checking required files...

if not exist "target\release\bible_pattern_generator.exe" (
    echo [ERROR] bible_pattern_generator.exe not found!
    echo         Run: cargo build --release --bin bible_pattern_generator
    pause
    exit /b 1
)

if not exist "target\release\brainwallet_extended.exe" (
    echo [ERROR] brainwallet_extended.exe not found!
    echo         Run: cargo build --release --bin brainwallet_extended
    pause
    exit /b 1
)

if not exist "%SNAPSHOT%" (
    echo [ERROR] UTXO snapshot not found at: %SNAPSHOT%
    echo         Run the indexer first: full_utxo_indexer
    pause
    exit /b 1
)

if not exist "%ENGLISH_FILE%" (
    echo [ERROR] English Bible file not found: %ENGLISH_FILE%
    pause
    exit /b 1
)

if not exist "%FRENCH_FILE%" (
    echo [ERROR] French Bible file not found: %FRENCH_FILE%
    pause
    exit /b 1
)

echo [OK] All files verified.
echo.

:: --- Custom phrases file ---
set /p CUSTOM_FILE="Custom phrases file (or ENTER to skip) : "
if not "%CUSTOM_FILE%"=="" (
    if not exist "%CUSTOM_FILE%" (
        echo [WARNING] Custom file not found: %CUSTOM_FILE%
        set CUSTOM_FILE=
    )
)

:: --- Etape 1: Générer les patterns ---
echo.
echo ================================================================
echo   ETAPE 1/2: Generation des patterns Bible (mode %MODE%)
echo ================================================================
echo.

set GEN_CMD=target\release\bible_pattern_generator.exe --english "%ENGLISH_FILE%" --french "%FRENCH_FILE%" --output "%OUTPUT_PATTERNS%" --max-acronym-len %MAX_ACRONYM% --max-word-combo %MAX_WORD_COMBO%

if defined CUSTOM_FILE (
    set GEN_CMD=!GEN_CMD! --custom "%CUSTOM_FILE%"
    echo [Info] Custom phrases: %CUSTOM_FILE%
)

if defined NO_SUFFIXES (
    set GEN_CMD=!GEN_CMD! %NO_SUFFIXES%
)

if defined NO_LEET (
    set GEN_CMD=!GEN_CMD! %NO_LEET%
)

echo [Info] Starting pattern generation...
echo.

%GEN_CMD%

if errorlevel 1 (
    echo.
    echo [ERROR] Pattern generation failed!
    pause
    exit /b 1
)

echo.

:: --- Compter les patterns ---
for %%A in ("%OUTPUT_PATTERNS%") do set FILESIZE=%%~zA
echo [Info] Pattern file size: %FILESIZE% bytes

find /c /v "" "%OUTPUT_PATTERNS%" > pattern_count.tmp
set /p PATTERN_COUNT=<pattern_count.tmp
set /a PATTERN_COUNT=%PATTERN_COUNT% - 1
del pattern_count.tmp

echo [Info] Total unique patterns: %PATTERN_COUNT%
echo.

:: --- Confirmation avant scan ---
echo [Info] Estimated scan time:
if %PATTERN_COUNT% LSS 500000 (
    echo   ~Few minutes (fast)
) else if %PATTERN_COUNT% LSS 5000000 (
    echo   ~Several minutes to tens of minutes
) else (
    echo   ~Could take a while
)
echo.

set /p CONFIRM="Continue with scan? (y/n, default=y) : "
if /i not "%CONFIRM%"=="n" (
    goto :SCAN
) else (
    echo [Info] Scan skipped. Patterns saved to: %OUTPUT_PATTERNS%
    goto :END
)

:: --- Etape 2: Scanner les patterns ---
:SCAN
echo.
echo ================================================================
echo   ETAPE 2/2: Scan Brainwallet (SHA256 + MD5, CPU multi-thread)
echo ================================================================
echo.

echo [Info] Hash method: %HASH_METHOD%
echo [Info] Threads: %THREADS% (0 = auto)
echo [Info] Min value: %MIN_VALUE% sats
echo [Info] Output: %SCAN_OUTPUT%
echo.

set SCAN_CMD=target\release\brainwallet_extended.exe --texts "%OUTPUT_PATTERNS%" --snapshot "%SNAPSHOT%" --hash %HASH_METHOD% --min-value %MIN_VALUE% --output "%SCAN_OUTPUT%"

if "%THREADS%" NEQ "0" (
    set SCAN_CMD=!SCAN_CMD! --threads %THREADS%
)

echo [Info] Starting scan...
echo.

%SCAN_CMD%

echo.
echo ================================================================
echo   RESULTATS
echo ================================================================
echo.

if exist "%SCAN_OUTPUT%" (
    for %%A in ("%SCAN_OUTPUT%") do set RESULT_SIZE=%%~zA
    if %RESULT_SIZE% GTR 100 (
        echo [!!!] MATCHES FOUND! See: %SCAN_OUTPUT%
        echo.
        type "%SCAN_OUTPUT%"
    ) else (
        echo [Info] No matches found.
    )
) else (
    echo [Info] Scan completed. No results file generated.
)

echo.

:END
echo ================================================================
echo   Terminé!
echo ================================================================
echo.
echo [Summary]
echo   Mode: %MODE%
echo   Patterns generated: %PATTERN_COUNT%
echo   Hash methods: %HASH_METHOD%
echo   Address types: P2PKH, P2WPKH, P2SH-P2WPKH, P2TR
echo   Key types: Compressed + Uncompressed
echo   Results: %SCAN_OUTPUT%
echo.

pause
