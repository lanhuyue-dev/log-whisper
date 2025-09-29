#!/bin/bash

# LogWhisper 发布脚本

set -e

echo "🚀 开始发布 LogWhisper..."

# 检查版本号
VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
echo "📦 当前版本: $VERSION"

# 确认发布
read -p "确认发布版本 $VERSION? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ 发布已取消"
    exit 1
fi

# 运行测试
echo "🧪 运行测试..."
./scripts/test.sh

# 构建发布版本
echo "🔨 构建发布版本..."
cargo tauri build --target x86_64-pc-windows-msvc

# 创建发布包
echo "📦 创建发布包..."
RELEASE_DIR="releases/v$VERSION"
mkdir -p "$RELEASE_DIR"

# 复制构建产物
cp src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/LogWhisper_*.msi "$RELEASE_DIR/"
cp src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/LogWhisper_*.exe "$RELEASE_DIR/"

# 创建发布说明
cat > "$RELEASE_DIR/CHANGELOG.md" << EOF
# LogWhisper v$VERSION

## 新功能
- 初始版本发布
- 支持 MyBatis SQL 解析
- 支持 JSON 修复和格式化
- 支持错误日志高亮
- 支持文件拖拽导入
- 支持插件切换
- 支持实时搜索

## 技术特性
- 基于 Rust + Tauri 构建
- 轻量级，无外部依赖
- 支持大文件处理（最大 50MB）
- 高性能解析引擎
- 插件化架构

## 系统要求
- Windows 10 或更高版本
- 无需安装 .NET 或 JRE

## 安装说明
1. 下载 LogWhisper_*.msi 安装包
2. 双击运行安装程序
3. 按照提示完成安装

## 使用说明
1. 启动 LogWhisper
2. 拖拽日志文件到窗口或点击"选择文件"
3. 选择解析插件（Auto/MyBatis/JSON/Raw）
4. 查看解析结果并复制需要的内容

## 反馈
如有问题或建议，请提交 Issue 或 Pull Request。
EOF

echo "✅ 发布完成！"
echo "📁 发布文件位于: $RELEASE_DIR"
echo "📋 发布说明: $RELEASE_DIR/CHANGELOG.md"
