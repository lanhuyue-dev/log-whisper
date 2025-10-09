#!/bin/bash

echo ""
echo "=========================================="
echo "       LogWhisper Tauri Development Launcher"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

echo "[INFO] 检查开发环境..."

if [ ! -d "node_modules" ]; then
    echo "[INFO] 安装 Node.js 依赖..."
    npm install
    if [ $? -ne 0 ]; then
        echo "[ERROR] 依赖安装失败"
        exit 1
    fi
fi

if [ ! -d "src-tauri" ]; then
    echo "[ERROR] 找不到 src-tauri 目录，请先运行 Tauri 初始化"
    exit 1
fi

echo "[SUCCESS] 环境检查完成"
echo ""

echo "[INFO] 启动 LogWhisper Tauri 开发模式..."
echo ""
echo "[INFO] 架构: Tauri + Rust (集成后端)"
echo "[INFO] 前端: Web 技术栈"
echo "[INFO] 通信: Tauri invoke 系统"
echo ""

npm run dev:tauri

echo ""
if [ $? -ne 0 ]; then
    echo "[ERROR] Tauri 开发模式启动失败"
else
    echo "[SUCCESS] Tauri 开发模式正常退出"
fi