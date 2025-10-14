# LogWhisper - 强大的日志分析桌面工具

> 🚀 基于 **Tauri + Rust** 架构的高性能日志分析应用
>
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build Status](https://github.com/lanhuyue-dev/log-whisper/workflows/CI/badge.svg)](https://github.com/lanhuyue-dev/log-whisper/actions)
[![Release](https://img.shields.io/github/release/lanhuyue-dev/log-whisper.svg)](https://github.com/lanhuyue-dev/log-whisper/releases)

## 📖 项目简介

LogWhisper 是一个专业的桌面日志分析工具，旨在帮助开发者和运维人员快速解析、分析和理解各种格式的日志文件。项目采用现代化的 **Tauri + Rust** 架构，提供了卓越的性能和安全性，同时保持了轻量级的资源占用。

### ✨ 核心特性

- 🔍 **智能日志解析**: 支持多种日志格式自动识别和解析
- 🎨 **现代化UI**: 基于Tauri的轻量级响应式桌面界面
- ⚡ **高性能处理**: Rust后端引擎，处理速度极快，内存占用低
- 🔧 **智能缩略**: 自动识别并缩略冗长的日志前缀信息
- 📊 **实时分析**: 解析结果统计和可视化展示
- 🧩 **插件化架构**: 支持自定义解析插件扩展
- 🖥️ **跨平台支持**: Windows、macOS、Linux全平台兼容
- 🔒 **安全可靠**: Tauri沙箱架构确保文件处理安全

## 🏗️ 架构设计

```
┌─────────────────┐    Tauri IPC    ┌─────────────────┐
│   Frontend UI   │ ◄────────────► │   Rust Backend  │
│                 │                │                 │
│ • Web Interface │                │ • Log Processing │
│ • Drag & Drop   │                │ • Plugin System  │
│ • Real-time UI  │                │ • Performance   │
└─────────────────┘                └─────────────────┘
```

### 技术栈

- **前端**: HTML5 + Tailwind CSS + JavaScript
- **后端**: Rust + Tauri + Tokio
- **通信**: Tauri IPC (进程间通信)
- **构建**: Cargo (Rust) + npm (Node.js)
- **UI框架**: Tauri + WebView

## 🚀 快速开始

### 环境要求

- **Node.js**: v16.0+
- **Rust**: v1.70+
- **系统**: Windows 10+, macOS 10.15+, Ubuntu 18.04+

### 安装步骤

1. **克隆项目**
   ```bash
   git clone https://github.com/lanhuyue-dev/log-whisper.git
   cd log-whisper
   ```

2. **安装依赖**
   ```bash
   npm install
   cd src-tauri && cargo build
   cd ..
   ```

3. **启动开发模式**
   ```bash
   npm run dev
   # 或者使用
   npm start
   ```

### 构建应用

```bash
# 开发构建
npm run build

# 生产构建
npm run dist

# 运行测试
npm run test
```

### 快速启动脚本

**Windows:**
```bash
# 开发模式
start-tauri-dev.bat

# 构建应用
build-tauri.bat
```

**Linux/macOS:**
```bash
chmod +x start-tauri.sh
./start-tauri.sh
```

## 📋 支持的日志格式

| 格式类型 | 描述 | 支持状态 | 特性 |
|----------|------|----------|------|
| **Auto Detection** | 智能格式自动识别 | ✅ 完全支持 | 自动选择最佳解析器 |
| **SpringBoot** | Java应用日志，支持堆栈跟踪 | ✅ 完全支持 | 智能前缀缩略、ISO时间戳 |
| **Docker JSON** | 容器JSON格式日志 | ✅ 完全支持 | Stream标签、时间戳解析 |
| **MyBatis SQL** | SQL执行日志与参数绑定 | ✅ 完全支持 | SQL格式化、参数显示 |
| **Raw Text** | 通用文本日志文件 | ✅ 完全支持 | 按行解析、级别识别 |
| **Nginx访问日志** | Web服务器访问日志 | 🚧 开发中 | IP解析、状态码统计 |
| **Apache访问日志** | Apache服务器日志 | 📋 计划中 | 时间戳解析、请求分析 |

## 🎯 使用方法

### 基本操作

1. **选择日志文件**
   - 点击"选择日志文件"按钮
   - 或直接拖拽文件到应用窗口

2. **选择解析插件**
   - 自动检测（推荐）
   - 手动选择特定格式

3. **开始解析**
   - 点击"开始解析"按钮
   - 查看实时解析进度

4. **查看结果**
   - 右侧面板显示解析结果
   - 支持按日志级别筛选
   - 显示详细统计信息

### 高级功能

- **批量处理**: 支持同时处理多个日志文件
- **导出结果**: 将解析结果导出为JSON/CSV格式
- **自定义筛选**: 按时间范围、关键词筛选
- **性能分析**: 查看解析性能指标

## 🔧 开发指南

### 项目结构

```
log-whisper/
├── src/                    # 前端界面
│   ├── index.html          # 主界面HTML
│   ├── main.js             # 前端JavaScript逻辑
│   └── style.css           # Tailwind CSS样式
├── src-tauri/              # Tauri Rust后端
│   ├── src/
│   │   ├── main.rs         # Tauri应用入口和命令处理
│   │   └── plugins/        # 日志解析插件系统
│   │       ├── mod.rs      # 插件管理器
│   │       ├── springboot.rs # SpringBoot日志解析器
│   │       ├── docker_json.rs # Docker JSON解析器
│   │       ├── mybatis.rs  # MyBatis SQL解析器
│   │       └── raw.rs      # 原始文本解析器
│   ├── Cargo.toml          # Rust项目配置
│   └── tauri.conf.json     # Tauri应用配置
├── doc/                    # 项目文档
├── tests/                  # 测试文件
├── dist/                   # 构建输出目录
├── build-styles.sh         # CSS构建脚本
├── build-tauri.bat         # Windows构建脚本
├── package.json            # Node.js项目配置
├── README.md               # 项目说明文档
├── CONTRIBUTING.md         # 贡献指南
├── CHANGELOG.md            # 更新日志
└── LICENSE                 # Apache 2.0许可证
```

### 开发模式

```bash
# 启动开发模式（自动重载）
npm run dev
# 或者
npm start

# 单独运行Rust测试
cd src-tauri && cargo test

# 构建CSS样式
npm run build:css
```

### 构建部署

```bash
# 开发构建
npm run build

# 打包桌面应用
npm run package

# 构建各平台安装包
npm run dist:win    # Windows
npm run dist:mac    # macOS
npm run dist:linux  # Linux

# 清理构建文件
npm run clean
```

## 🧩 插件开发

LogWhisper支持自定义解析插件扩展。插件采用Rust编写，提供标准化的接口。

### 插件示例

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomLogParser;

impl LogParser for CustomLogParser {
    fn name(&self) -> &str {
        "custom-parser"
    }
    
    fn description(&self) -> &str {
        "自定义日志解析器"
    }
    
    fn parse(&self, content: &str) -> Result<Vec<LogEntry>, ParseError> {
        // 解析逻辑实现
        todo!()
    }
}
```

详细的插件开发指南请参考 [Plugin Development Guide](./docs/plugin-development.md)。

## 📊 性能指标

在标准测试环境下的性能表现：

| 测试项目 | 数据量 | 处理时间 | 内存占用 | 特性优化 |
|----------|--------|----------|----------|----------|
| **SpringBoot日志** | 888行 | ~16ms | ~5MB | 智能前缀缩略 |
| **Docker JSON日志** | 50MB | ~1秒 | ~30MB | 流式解析 |
| **MyBatis SQL日志** | 200MB | ~5秒 | ~80MB | 参数格式化 |
| **原始文本日志** | 100MB | ~2秒 | ~50MB | 级别识别 |

*测试环境: Intel i7-8750H, 16GB RAM, SSD*

### 性能优化特性

- **预编译正则表达式**: 提升解析速度40-50%
- **智能内存管理**: 预分配内存，减少重新分配
- **流式处理**: 大文件分块处理，避免内存溢出
- **并发处理**: 利用多核CPU并行解析

## 🤝 贡献指南

我们欢迎社区贡献！请遵循以下步骤：

1. **Fork 项目**
2. **创建特性分支** (`git checkout -b feature/AmazingFeature`)
3. **提交更改** (`git commit -m 'Add some AmazingFeature'`)
4. **推送分支** (`git push origin feature/AmazingFeature`)
5. **创建 Pull Request**

### 代码规范

- **Rust代码**: 使用 `cargo fmt` 格式化
- **JavaScript代码**: 使用 ESLint 检查
- **提交信息**: 遵循 [Conventional Commits](https://conventionalcommits.org/)

## 🐛 问题报告

遇到问题？请通过以下方式报告：

- [GitHub Issues](https://github.com/your-org/log-whisper/issues)
- [问题模板](./docs/issue-template.md)

## 📄 许可证

本项目采用 [Apache License 2.0](./LICENSE) 开源协议。

## 📜 更新日志

### v1.0.0 (2025-10-14)
- ✨ **初始发布**：基于Tauri + Rust架构的桌面日志分析工具
- ✨ **智能日志解析**：支持SpringBoot、Docker JSON、MyBatis等格式
- ✨ **智能前缀缩略**：自动识别并缩略冗长的日志前缀信息
- ✨ **高性能处理**：优化的解析引擎，支持大文件处理
- ✨ **插件化架构**：可扩展的日志解析器系统
- ✨ **现代化界面**：直观的拖拽操作和实时反馈
- 🐛 完善的错误处理和稳定性优化
- 📚 完整的文档和贡献指南

详细更新日志请查看 [CHANGELOG.md](./CHANGELOG.md)。

## 🙏 致谢

感谢所有贡献者和以下开源项目：

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Rust](https://rust-lang.org/) - 系统编程语言
- [Tokio](https://tokio.rs/) - 异步运行时
- [Tailwind CSS](https://tailwindcss.com/) - CSS框架
- [Regex](https://docs.rs/regex/) - 正则表达式库
- [Serde](https://serde.rs/) - 序列化框架

---

## 📞 联系我们

- **项目主页**: https://github.com/lanhuyue-dev/log-whisper
- **问题反馈**: [GitHub Issues](https://github.com/lanhuyue-dev/log-whisper/issues)
- **功能建议**: [GitHub Discussions](https://github.com/lanhuyue-dev/log-whisper/discussions)

---

<div align="center">

**⭐ 如果这个项目对您有帮助，请给我们一个Star！⭐**

Made with ❤️ by LogWhisper Team

</div>