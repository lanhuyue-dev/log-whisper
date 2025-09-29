@echo off
echo Starting LogWhisper build...

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Rust is not installed. Please install Rust first.
    echo Visit https://rustup.rs/ to install Rust
    pause
    exit /b 1
)

REM Check if Tauri CLI is installed
where tauri >nul 2>nul
if %errorlevel% neq 0 (
    echo Installing Tauri CLI...
    cargo install tauri-cli
    if %errorlevel% neq 0 (
        echo Failed to install Tauri CLI
        pause
        exit /b 1
    )
)

REM Clean previous build
echo Cleaning previous build...
cargo clean
if %errorlevel% neq 0 (
    echo Failed to clean build
    pause
    exit /b 1
)

REM Run tests
echo Running tests...
cargo test
if %errorlevel% neq 0 (
    echo Tests failed
    pause
    exit /b 1
)

REM Build application
echo Building application...
cargo tauri build
if %errorlevel% neq 0 (
    echo Build failed
    pause
    exit /b 1
)

echo Build completed successfully!
echo Output files are located at: src-tauri\target\release\
pause
