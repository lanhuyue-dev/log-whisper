@echo off
echo.
echo ==========================================
       LogWhisper Tauri Build Script
echo ==========================================
echo.

cd /d "%~dp0"

echo [INFO] 检查构建环境...

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
    echo [ERROR] 找不到 src-tauri 目录
    pause
    exit /b 1
)

echo [SUCCESS] 环境检查完成
echo.

echo [INFO] 构建前端样式...
npm run build:css:prod
if %ERRORLEVEL% neq 0 (
    echo [ERROR] 样式构建失败
    pause
    exit /b 1
)

echo [SUCCESS] 前端样式构建完成
echo.

echo [INFO] 构建 LogWhisper Tauri 应用...
echo.

npm run build:tauri

echo.
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Tauri 应用构建失败，错误代码: %ERRORLEVEL%
) else (
    echo [SUCCESS] LogWhisper Tauri 应用构建完成
    echo [INFO] 可执行文件位置: src-tauri/target/release/
)

pause