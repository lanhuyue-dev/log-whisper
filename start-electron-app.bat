@echo off
chcp 65001 >nul
echo.
echo ==========================================
echo       LogWhisper Electron App Launcher
echo ==========================================
echo.

cd /d "%~dp0"

echo [INFO] Checking environment...

echo [INFO] Cleaning up existing Rust API processes...
taskkill /f /im api-server.exe >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [INFO] Found and terminated existing api-server.exe process
) else (
    echo [INFO] No existing api-server.exe process found
)

echo [INFO] Cleaning up any processes using port 3030...
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :3030') do (
    taskkill /f /pid %%a >nul 2>&1
    if !ERRORLEVEL! equ 0 (
        echo [INFO] Terminated process %%a using port 3030
    )
)

if not exist "node_modules" (
    echo [INFO] Installing Node.js dependencies...
    npm install
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] Failed to install dependencies
        pause
        exit /b 1
    )
)

if not exist "src-rust\target\release\api-server.exe" (
    echo [INFO] Building Rust API server...
    cd src-rust
    cargo build --release --bin api-server
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] Rust compilation failed
        pause
        exit /b 1
    )
    cd ..
)

echo [SUCCESS] Environment check completed
echo.

echo [INFO] Final cleanup before starting application...
taskkill /f /im api-server.exe >nul 2>&1
timeout /t 2 /nobreak >nul

echo [INFO] Starting LogWhisper Electron application...
echo.
echo [INFO] Architecture: Electron + Rust API
echo [INFO] API Server: http://127.0.0.1:3030
echo [INFO] Desktop App: Electron Window
echo.

npm start

echo.
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Application startup failed, error code: %ERRORLEVEL%
) else (
    echo [SUCCESS] Application exited normally
)

echo.
echo [INFO] Cleaning up processes after application exit...
taskkill /f /im api-server.exe >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [INFO] Terminated remaining api-server.exe process
) else (
    echo [INFO] No api-server.exe process to terminate
)

pause
