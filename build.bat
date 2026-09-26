@echo off
setlocal

rem Build and package cs2shock for Windows x64. Run from any directory or
rem double-click this file. Pass --no-pause when calling from automation.
if /i "%~1"=="--help" goto :help
set "REPO_DIR=%~dp0"
pushd "%REPO_DIR%" >nul
if errorlevel 1 goto :failed

where cargo >nul 2>nul
if errorlevel 1 (
    echo ERROR: Cargo was not found. Install Rust from https://rustup.rs/ and try again.
    goto :failed
)

echo Building cs2shock in release mode...
cargo build --release
if errorlevel 1 (
    echo ERROR: cargo build --release failed.
    goto :failed
)

set "EXE=%REPO_DIR%target\release\cs2shock.exe"
set "DIST=%REPO_DIR%dist\cs2shock-windows-x64"
set "ZIP=%REPO_DIR%dist\cs2shock-windows-x64.zip"
set "STAGE=%TEMP%\cs2shock-package-%RANDOM%-%RANDOM%"

if not exist "%EXE%" (
    echo ERROR: Build succeeded but the expected executable was not found:
    echo        %EXE%
    goto :failed
)

if not exist "%REPO_DIR%dist" mkdir "%REPO_DIR%dist"
if errorlevel 1 goto :package_failed
if not exist "%DIST%" mkdir "%DIST%"
if errorlevel 1 goto :package_failed
if exist "%STAGE%" (
    echo ERROR: Temporary package path already exists: %STAGE%
    goto :package_failed
)
mkdir "%STAGE%"
if errorlevel 1 goto :package_failed
set "STAGE_OWNED=1"

rem Populate the runnable folder. Preserve its config.json if it already exists.
copy /y "%EXE%" "%DIST%\cs2shock.exe" >nul
if errorlevel 1 goto :package_failed
copy /y "%REPO_DIR%README.md" "%DIST%\README.md" >nul
if errorlevel 1 goto :package_failed
copy /y "%REPO_DIR%gamestate_integration_cs2shock.cfg" "%DIST%\gamestate_integration_cs2shock.cfg" >nul
if errorlevel 1 goto :package_failed

if not exist "%DIST%\config.json" (
    (
        echo {
        echo   "shock_mode": "LastHitPercentage",
        echo   "min_duration": 1,
        echo   "max_duration": 3,
        echo   "min_intensity": 15,
        echo   "max_intensity": 83,
        echo   "beep_on_match_start": false,
        echo   "beep_on_round_start": true,
        echo   "shocker_ids": [],
        echo   "api_token": "your-openshock-api-token-here",
        echo   "api_server": "https://api.openshock.app"
        echo }
    ) > "%DIST%\config.json"
    if errorlevel 1 goto :package_failed
)

rem Build the archive from a fresh staging folder so a local token in DIST\config.json
rem can never be copied into the distributable ZIP.
copy /y "%EXE%" "%STAGE%\cs2shock.exe" >nul
if errorlevel 1 goto :package_failed
copy /y "%REPO_DIR%README.md" "%STAGE%\README.md" >nul
if errorlevel 1 goto :package_failed
copy /y "%REPO_DIR%gamestate_integration_cs2shock.cfg" "%STAGE%\gamestate_integration_cs2shock.cfg" >nul
if errorlevel 1 goto :package_failed
(
    echo {
    echo   "shock_mode": "LastHitPercentage",
    echo   "min_duration": 1,
    echo   "max_duration": 3,
    echo   "min_intensity": 15,
    echo   "max_intensity": 83,
    echo   "beep_on_match_start": false,
    echo   "beep_on_round_start": true,
    echo   "shocker_ids": [],
    echo   "api_token": "your-openshock-api-token-here",
    echo   "api_server": "https://api.openshock.app"
    echo }
) > "%STAGE%\config.json"
if errorlevel 1 goto :package_failed

echo Creating %ZIP% ...
set "CS2SHOCK_STAGE=%STAGE%"
set "CS2SHOCK_ZIP=%ZIP%"
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; Compress-Archive -Path (Join-Path $env:CS2SHOCK_STAGE '*') -DestinationPath $env:CS2SHOCK_ZIP -Force"
if errorlevel 1 goto :package_failed

call :cleanup_stage
popd
echo.
echo Build complete.
echo Folder: %DIST%
echo ZIP:    %ZIP%
if /i not "%~1"=="--no-pause" pause
exit /b 0

:package_failed
echo ERROR: Packaging failed. Check that PowerShell is available and the output files are writable.
call :cleanup_stage

:failed
if defined REPO_DIR popd 2>nul
echo.
echo Build did not complete. Review the error above.
if /i not "%~1"=="--no-pause" pause
exit /b 1

:cleanup_stage
if not defined STAGE_OWNED exit /b 0
del /q "%STAGE%\cs2shock.exe" "%STAGE%\README.md" "%STAGE%\gamestate_integration_cs2shock.cfg" "%STAGE%\config.json" >nul 2>nul
rmdir "%STAGE%" >nul 2>nul
set "STAGE_OWNED="
exit /b 0

:help
echo Usage: build.bat [--help^|--no-pause]
echo Builds the release executable and packages it under dist\cs2shock-windows-x64.
echo --no-pause  Do not wait for a keypress when the script finishes.
exit /b 0
