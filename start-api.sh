#!/bin/bash

echo "🚀 启动 LogWhisper API 服务器..."

# 设置环境变量
export LOGWHISPER_PORT=3030
export LOGWHISPER_LOG_FILE=logs/log-whisper.log
export LOGWHISPER_LOG_LEVEL=info

# 创建日志目录
mkdir -p logs

# 启动 API 服务器
echo "📋 配置信息:"
echo "  - 端口: $LOGWHISPER_PORT"
echo "  - 日志文件: $LOGWHISPER_LOG_FILE"
echo "  - 日志级别: $LOGWHISPER_LOG_LEVEL"
echo ""

# 检查是否在开发环境
if [ -f "src-rust/target/debug/api-server" ]; then
    echo "🔧 开发模式启动..."
    ./src-rust/target/debug/api-server
elif [ -f "resources/api-server" ]; then
    echo "📦 生产模式启动..."
    ./resources/api-server
else
    echo "❌ 找不到 API 服务器可执行文件"
    echo "请先运行: npm run build:rust"
    exit 1
fi
