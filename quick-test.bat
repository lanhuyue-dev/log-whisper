@echo off
echo.
echo ==========================================
echo       LogWhisper Quick Test Launcher
echo ==========================================
echo.

echo [INFO] Starting Rust API server...
cd /d "%~dp0\src-rust"
start "Rust API" cargo run --bin api-server

echo [INFO] Waiting for API server to start...
timeout /t 3 /nobreak >nul

echo [INFO] Starting frontend page...
cd /d "%~dp0\src"
start "Frontend" python -m http.server 8080

echo.
echo [SUCCESS] Application startup completed!
echo [INFO] API Server: http://127.0.0.1:3030
echo [INFO] Frontend Page: http://127.0.0.1:8080
echo.
echo [INFO] Please open in browser: http://127.0.0.1:8080
echo [INFO] Note: This is test mode, use 'npm start' for full Electron app
echo.
pause