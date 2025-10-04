@echo off
echo.
echo ==========================================
echo       LogWhisper Application Build Script
echo ==========================================
echo.

cd /d "%~dp0"

echo [INFO] Checking build environment...
if not exist "node_modules" (
    echo [INFO] Installing Node.js dependencies...
    npm install
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] Failed to install dependencies
        pause
        exit /b 1
    )
)

echo [INFO] Building Rust API server (release version)...
cd src-rust
cargo build --release --bin api-server
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust compilation failed
    pause
    exit /b 1
)
cd ..

echo [INFO] Building Electron application...
npm run build

echo.
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Build failed, error code: %ERRORLEVEL%
) else (
    echo [SUCCESS] Build completed!
    echo [INFO] Output directory: dist/
    echo.
    echo [INFO] Executable files generated in dist/ directory
)

pause
