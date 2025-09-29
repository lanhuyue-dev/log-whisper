@echo off
echo Starting LogWhisper development mode...

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

REM Start development mode
echo Starting development mode...
cargo tauri dev
