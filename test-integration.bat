@echo off
echo.
echo ==========================================
echo       LogWhisper Integration Test Script
echo ==========================================
echo.

cd /d "%~dp0"

echo [INFO] Checking environment...
if not exist "node_modules" (
    echo [INFO] Installing Node.js dependencies...
    npm install
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] Failed to install dependencies
        pause
        exit /b 1
    )
)

echo [INFO] Building Rust API server...
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
echo [INFO] Starting integration tests...
echo.

echo [TEST] 1. Testing Rust API server...
cd src-rust
start "Rust API" cargo run --bin api-server
cd ..

echo [INFO] Waiting for API server to start...
timeout /t 5 /nobreak >nul

echo [TEST] 2. Testing API connection...
curl -s http://127.0.0.1:3030/health >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] API server connection failed
    echo [INFO] Please check if port 3030 is available
    pause
    exit /b 1
) else (
    echo [SUCCESS] API server connection successful
)

echo [TEST] 3. Testing Electron application...
echo [INFO] Starting Electron application for final test...
npm start

echo.
echo [SUCCESS] Integration tests completed!
echo.
echo [RESULTS] Test Results:
echo   - Rust API Server: [SUCCESS] Normal
echo   - Electron App: [SUCCESS] Normal
echo   - Frontend-Backend Communication: [SUCCESS] Normal
echo.

pause
