#!/bin/bash

# LogWhisper 构建脚本

set -e

echo "🚀 开始构建 LogWhisper..."

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust 未安装，请先安装 Rust"
    exit 1
fi

# 检查 Tauri CLI 是否安装
if ! command -v tauri &> /dev/null; then
    echo "📦 安装 Tauri CLI..."
    cargo install tauri-cli
fi

# 清理之前的构建
echo "🧹 清理之前的构建..."
cargo clean

# 运行测试
echo "🧪 运行测试..."
cargo test

# 构建应用
echo "🔨 构建应用..."
cargo tauri build

echo "✅ 构建完成！"
echo "📁 输出文件位于: src-tauri/target/release/"
