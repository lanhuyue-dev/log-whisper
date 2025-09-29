# LogWhisper

> 轻量级桌面日志分析工具

LogWhisper 是一款轻量、本地优先、开发者友好的桌面日志分析工具，专注于解决日志分析中的核心痛点。

## ✨ 特性

- 🚀 **轻量高效** - 基于 Rust + Tauri 构建，启动快速，内存占用低
- 🔒 **本地优先** - 数据不上传，保护隐私安全
- 🔍 **智能解析** - 自动识别和解析 MyBatis SQL、JSON 等格式
- 🎨 **美观界面** - 双栏布局，左侧原始日志，右侧结构化结果
- 📋 **一键复制** - 解析结果可直接复制到剪贴板
- 🔧 **插件化** - 支持多种解析插件，易于扩展
- 🔍 **实时搜索** - 支持关键词过滤和实时搜索

## 🎯 核心功能

### MyBatis SQL 还原
自动将 MyBatis 日志中的 SQL 语句和参数合并为可执行的 SQL：

```
输入：
Preparing: SELECT * FROM user WHERE id = ? AND name = ?
Parameters: 123(Integer), "张三"(String)

输出：
SELECT * FROM user WHERE id = 123 AND name = '张三'
```

### JSON 修复与格式化
自动修复常见的 JSON 语法错误并格式化：

```
输入：{"name":"张三","age":25 "city":"北京"}

输出：
{
  "name": "张三",
  "age": 25,
  "city": "北京"
}
```

### 错误日志高亮
自动识别和高亮错误、警告日志，便于快速定位问题。

## 🚀 快速开始

### 系统要求

- Windows 10 或更高版本
- 无需安装 .NET、JRE 或 Python

### 安装

1. 下载最新版本的 LogWhisper
2. 运行安装程序
3. 启动应用

### 使用

1. **导入日志文件**
   - 拖拽日志文件到窗口
   - 或点击"选择文件"按钮

2. **选择解析插件**
   - Auto: 自动选择最佳插件
   - MyBatis: 仅解析 MyBatis SQL
   - JSON: 仅修复和格式化 JSON
   - Raw: 显示原始文本

3. **查看解析结果**
   - 左侧显示原始日志
   - 右侧显示结构化结果

4. **复制内容**
   - 点击解析结果旁的"复制"按钮
   - 内容将复制到剪贴板

## 🛠️ 开发

### 环境要求

- Rust 1.70+
- Node.js 16+
- Tauri CLI

### 构建

```bash
# 克隆仓库
git clone https://github.com/log-whisper/log-whisper.git
cd log-whisper

# 安装依赖
cargo install tauri-cli

# 开发模式运行
cargo tauri dev

# 构建发布版本
cargo tauri build
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration

# 运行性能测试
cargo test --test performance
```

## 📁 项目结构

```
log-whisper/
├── src/                    # 前端源码
│   ├── index.html         # 主页面
│   ├── style.css          # 样式文件
│   └── main.js            # 前端逻辑
├── src-tauri/             # Tauri 后端
│   ├── src/
│   │   ├── models/        # 数据模型
│   │   ├── plugins/       # 插件系统
│   │   ├── parser/        # 解析引擎
│   │   └── tauri/         # Tauri 集成
│   └── Cargo.toml         # Rust 依赖
├── tests/                 # 测试文件
├── scripts/               # 构建脚本
└── doc/                   # 文档
```

## 🔧 技术架构

### 后端 (Rust)
- **解析引擎**: 高性能日志解析
- **插件系统**: 可扩展的插件架构
- **缓存机制**: 智能缓存提升性能

### 前端 (HTML/CSS/JS)
- **双栏布局**: 原始日志 + 解析结果
- **实时搜索**: 关键词过滤
- **拖拽支持**: 文件拖拽导入

### 桌面框架 (Tauri)
- **轻量级**: 相比 Electron 更小更快
- **安全性**: Rust 后端，内存安全
- **跨平台**: 支持 Windows/macOS/Linux

## 📊 性能指标

- **启动时间**: ≤ 2 秒
- **解析速度**: ≤ 3 秒 (50MB 文件)
- **内存占用**: ≤ 200MB
- **文件大小**: ≤ 20MB

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

### 开发流程

1. Fork 仓库
2. 创建特性分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Tailwind CSS](https://tailwindcss.com/) - CSS 框架

## 📞 支持

- 📧 邮箱: support@logwhisper.com
- 🐛 问题: [GitHub Issues](https://github.com/log-whisper/log-whisper/issues)
- 💬 讨论: [GitHub Discussions](https://github.com/log-whisper/log-whisper/discussions)

---

**LogWhisper** - 让日志分析变得简单高效 🚀
