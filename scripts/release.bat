@echo off
echo Starting LogWhisper release...

REM Check version number
for /f "tokens=2 delims==" %%i in ('findstr "version" src-tauri\Cargo.toml') do set VERSION=%%i
set VERSION=%VERSION:"=%
echo Current version: %VERSION%

REM Confirm release
set /p CONFIRM=Confirm release version %VERSION%? (y/N): 
if /i not "%CONFIRM%"=="y" (
    echo Release cancelled
    pause
    exit /b 1
)

REM Run tests
echo Running tests...
call scripts\test.bat
if %errorlevel% neq 0 (
    echo Tests failed, release aborted
    pause
    exit /b 1
)

REM Build release version
echo Building release version...
cargo tauri build --target x86_64-pc-windows-msvc
if %errorlevel% neq 0 (
    echo Build failed
    pause
    exit /b 1
)

REM Create release package
echo Creating release package...
set RELEASE_DIR=releases\v%VERSION%
if not exist "%RELEASE_DIR%" mkdir "%RELEASE_DIR%"

REM Copy build artifacts
copy "src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\LogWhisper_*.msi" "%RELEASE_DIR%\"
copy "src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\LogWhisper_*.exe" "%RELEASE_DIR%\"

REM Create release notes
echo # LogWhisper v%VERSION% > "%RELEASE_DIR%\CHANGELOG.md"
echo. >> "%RELEASE_DIR%\CHANGELOG.md"
echo ## New Features >> "%RELEASE_DIR%\CHANGELOG.md"
echo - Initial version release >> "%RELEASE_DIR%\CHANGELOG.md"
echo - MyBatis SQL parsing support >> "%RELEASE_DIR%\CHANGELOG.md"
echo - JSON repair and formatting >> "%RELEASE_DIR%\CHANGELOG.md"
echo - Error log highlighting >> "%RELEASE_DIR%\CHANGELOG.md"
echo - File drag and drop import >> "%RELEASE_DIR%\CHANGELOG.md"
echo - Plugin switching >> "%RELEASE_DIR%\CHANGELOG.md"
echo - Real-time search >> "%RELEASE_DIR%\CHANGELOG.md"

echo Release completed successfully!
echo Release files located at: %RELEASE_DIR%
echo Release notes: %RELEASE_DIR%\CHANGELOG.md
pause
