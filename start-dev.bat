@echo off
echo.
echo ==========================================
echo       LogWhisper Development Mode Launcher
echo ==========================================
echo.

cd /d "%~dp0"

echo [INFO] Checking development environment...
if not exist "node_modules" (
    echo [INFO] Installing Node.js dependencies...
    npm install
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] Failed to install dependencies
        pause
        exit /b 1
    )
)

echo [INFO] Building Rust API server (debug version)...
cd src-rust
cargo build --bin api-server
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust compilation failed
    pause
    exit /b 1
)
cd ..

echo [SUCCESS] Compilation completed
echo.
echo [INFO] Starting development mode...
echo.
echo [INFO] Architecture: Electron + Rust API (Development Mode)
echo [INFO] API Server: http://127.0.0.1:3030
echo [INFO] Desktop App: Electron Window (DevTools Enabled)
echo.

npm run dev

echo.
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Development mode startup failed, error code: %ERRORLEVEL%
) else (
    echo [SUCCESS] Development mode exited normally
)

pause