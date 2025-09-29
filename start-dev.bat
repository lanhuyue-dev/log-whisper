@echo off
echo LogWhisper Development Environment Startup Script
echo.

REM Check if we are in the correct directory
if not exist "src-tauri\Cargo.toml" (
    echo Please run this script from the project root directory
    echo Current directory: %CD%
    pause
    exit /b 1
)

echo Current directory: %CD%
echo.

REM Choose operation
echo Please select an operation:
echo 1. Development mode (cargo tauri dev)
echo 2. Run tests (cargo test)
echo 3. Build application (cargo tauri build)
echo 4. Clean build (cargo clean)
echo 5. Exit
echo.

set /p CHOICE=Enter your choice (1-5): 

if "%CHOICE%"=="1" (
    echo Starting development mode...
    call scripts\dev.bat
) else if "%CHOICE%"=="2" (
    echo Running tests...
    call scripts\test.bat
) else if "%CHOICE%"=="3" (
    echo Building application...
    call scripts\build.bat
) else if "%CHOICE%"=="4" (
    echo Cleaning build...
    cargo clean
    echo Clean completed
    pause
) else if "%CHOICE%"=="5" (
    echo Goodbye!
    exit /b 0
) else (
    echo Invalid choice
    pause
    exit /b 1
)
