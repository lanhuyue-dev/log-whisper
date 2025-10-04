@echo off
echo 🚀 启动 LogWhisper API 服务器...

REM 设置环境变量
set LOGWHISPER_PORT=3030
set LOGWHISPER_LOG_FILE=logs\log-whisper.log
set LOGWHISPER_LOG_LEVEL=info

REM 创建日志目录
if not exist logs mkdir logs

REM 启动 API 服务器
echo 📋 配置信息:
echo   - 端口: %LOGWHISPER_PORT%
echo   - 日志文件: %LOGWHISPER_LOG_FILE%
echo   - 日志级别: %LOGWHISPER_LOG_LEVEL%
echo.

REM 检查是否在开发环境
if exist "src-rust\target\debug\api-server.exe" (
    echo 🔧 开发模式启动...
    src-rust\target\debug\api-server.exe
) else if exist "resources\api-server.exe" (
    echo 📦 生产模式启动...
    resources\api-server.exe
) else (
    echo ❌ 找不到 API 服务器可执行文件
    echo 请先运行: npm run build:rust
    pause
    exit /b 1
)

pause
