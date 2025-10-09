@echo off
echo.
echo ==========================================
       LogWhisper Tauri Development Launcher
echo ==========================================
echo.

cd /d "%~dp0"

echo [INFO] 检查开发环境...

if not exist "node_modules" (
    echo [INFO] 安装 Node.js 依赖...
    npm install
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] 依赖安装失败
        pause
        exit /b 1
    )
)

if not exist "src-tauri" (
    echo [ERROR] 找不到 src-tauri 目录，请先运行 Tauri 初始化
    pause
        exit /b 1
)

echo [SUCCESS] 环境检查完成
echo.

echo [INFO] 启动 LogWhisper Tauri 开发模式...
echo.
echo [INFO] 架构: Tauri + Rust (集成后端)
echo [INFO] 前端: Web 技术栈
echo [INFO] 通信: Tauri invoke 系统
echo.

npm run dev:tauri

echo.
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Tauri 开发模式启动失败，错误代码: %ERRORLEVEL%
) else (
    echo [SUCCESS] Tauri 开发模式正常退出
)

pause