#!/bin/bash

# LogWhisper 测试脚本

set -e

echo "🧪 开始运行 LogWhisper 测试..."

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust 未安装，请先安装 Rust"
    exit 1
fi

# 检查 Node.js 是否安装
if ! command -v node &> /dev/null; then
    echo "❌ Node.js 未安装，请先安装 Node.js"
    exit 1
fi

# 运行 Rust API 测试
echo "🔬 运行 Rust API 测试..."
cd src-rust
cargo test
cd ..

# 运行 Node.js 测试
echo "🔗 运行 Node.js 测试..."
npm test

echo "✅ 所有测试通过！"
