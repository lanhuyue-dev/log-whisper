#!/bin/bash

# LogWhisper 测试脚本

set -e

echo "🧪 开始运行 LogWhisper 测试..."

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust 未安装，请先安装 Rust"
    exit 1
fi

# 运行单元测试
echo "🔬 运行单元测试..."
cargo test --lib

# 运行集成测试
echo "🔗 运行集成测试..."
cargo test --test integration

# 运行性能测试
echo "⚡ 运行性能测试..."
cargo test --test performance

# 运行文档测试
echo "📚 运行文档测试..."
cargo test --doc

echo "✅ 所有测试通过！"
