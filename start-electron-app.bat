@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul
echo.
echo ==========================================
echo       LogWhisper Electron App Launcher
echo ==========================================
echo.

cd /d "%~dp0"

echo [INFO] Checking environment and cleaning up processes...

echo [INFO] Terminating existing processes...
taskkill /f /im api-server.exe >nul 2>&1
taskkill /f /im cargo.exe >nul 2>&1
taskkill /f /im rustc.exe >nul 2>&1
taskkill /f /im electron.exe >nul 2>&1

echo [INFO] Cleaning up processes using port 3030...
set /a port_cleanup_count=0
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :3030') do (
    echo [INFO] Terminating process %%a using port 3030
    taskkill /f /pid %%a >nul 2>&1
    if !ERRORLEVEL! equ 0 (
        echo [INFO] Successfully terminated process %%a
        set /a port_cleanup_count+=1
    ) else (
        echo [WARN] Failed to terminate process %%a
    )
)

if !port_cleanup_count! gtr 0 (
    echo [INFO] Terminated !port_cleanup_count! processes using port 3030
    echo [INFO] Waiting for processes to fully terminate...
    timeout /t 3 /nobreak >nul
)

echo [INFO] Verifying port 3030 is free...
set /a port_check_count=0
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :3030') do (
    set /a port_check_count+=1
)

if !port_check_count! gtr 0 (
    echo [ERROR] Port 3030 is still in use by !port_check_count! processes
    echo [ERROR] Please manually terminate these processes or restart your computer
    echo [ERROR] You can use: netstat -ano ^| findstr :3030
    pause
    exit /b 1
) else (
    echo [SUCCESS] Port 3030 is free and ready to use
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

echo [INFO] Choose startup mode:
echo [1] Development mode (fast startup, live compilation)
echo [2] Production mode (full build, optimized)
echo.
set /p choice="Enter your choice (1 or 2): "

if "%choice%"=="1" goto :dev_mode
if "%choice%"=="2" goto :prod_mode
echo [ERROR] Invalid choice. Please run the script again.
pause
exit /b 1

:dev_mode
echo [INFO] Starting in DEVELOPMENT mode...

echo [INFO] Final cleanup before starting development backend...
taskkill /f /im api-server.exe >nul 2>&1
taskkill /f /im cargo.exe >nul 2>&1
taskkill /f /im rustc.exe >nul 2>&1
timeout /t 2 /nobreak >nul

echo [INFO] Verifying port 3030 is free for development...
set /a dev_port_check=0
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :3030') do (
    set /a dev_port_check+=1
)

if !dev_port_check! gtr 0 (
    echo [ERROR] Port 3030 is still in use by !dev_port_check! processes
    echo [ERROR] Cannot start development backend
    pause
    exit /b 1
) else (
    echo [SUCCESS] Port 3030 is free for development backend
)

echo [INFO] Building latest Rust API server...
cd src-rust
cargo build --bin api-server
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust compilation failed
    pause
    exit /b 1
)
cd ..
echo [SUCCESS] Rust API server built successfully

echo [INFO] Starting Rust backend in background...
start "Rust Backend" cmd /c "cd src-rust && cargo run --bin api-server"

echo [INFO] Waiting for backend to start (up to 15 seconds)...
set /a wait_count=0
:wait_for_backend
timeout /t 1 /nobreak >nul
set /a wait_count+=1

for /f "tokens=5" %%a in ('netstat -ano ^| findstr :3030') do (
    echo [SUCCESS] Backend is running on port 3030 (PID: %%a)
    goto :backend_ready
)

if !wait_count! lss 15 (
    echo [INFO] Waiting for backend... (!wait_count!/15)
    goto :wait_for_backend
)

echo [ERROR] Backend failed to start within 15 seconds
echo [ERROR] Please check the Rust backend console for errors
pause
exit /b 1

:backend_ready
echo [INFO] Starting Electron frontend...
npm start
goto :cleanup

:prod_mode
echo [INFO] Starting in PRODUCTION mode...
echo [INFO] Building latest Rust API server...
cd src-rust
cargo build --release --bin api-server
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust compilation failed
    pause
    exit /b 1
)
cd ..
echo [SUCCESS] Rust API server built successfully

echo [INFO] Building latest frontend assets...
npm run build:css:prod
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Frontend build failed
    pause
    exit /b 1
)
echo [SUCCESS] Frontend assets built successfully

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

:cleanup

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
