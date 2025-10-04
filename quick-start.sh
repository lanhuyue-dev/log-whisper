#!/bin/bash

echo "=========================================="
echo "       LogWhisper Electron 快速启动"
echo "=========================================="

# 首先启动 Rust API 服务器
echo "🦀 启动 Rust API 服务器..."
cd src-rust
cargo run --bin api-server &
RUST_PID=$!

# 等待 API 服务器启动
sleep 3

# 启动简单的 HTML 服务器来模拟 Electron
echo "🌐 启动前端页面..."
cd ../src
python -m http.server 8080 &
SERVER_PID=$!

echo "✅ 应用启动完成！"
echo "📡 API 服务器: http://127.0.0.1:3030"
echo "🖥️  前端页面: http://127.0.0.1:8080"
echo ""
echo "按 Ctrl+C 停止所有服务"

# 等待用户停止
trap "kill $RUST_PID $SERVER_PID; exit" INT
wait