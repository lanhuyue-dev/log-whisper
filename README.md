# LogWhisper - 强大的日志分析桌面工具

> 🚀 基于 **Electron + Rust** 架构的高性能日志分析应用
> 
> 📋 **架构迁移完成**: 已从 Tauri 成功迁移到 Electron + Rust 架构，提供更稳定的桌面应用体验

## 📖 项目简介

LogWhisper 是一个专业的桌面日志分析工具，旨在帮助开发者和运维人员快速解析、分析和理解各种格式的日志文件。项目采用现代化的 **Electron + Rust** 混合架构，结合了前端的用户友好性和后端的高性能处理能力。

### ✨ 核心特性

- 🔍 **智能日志解析**: 支持多种日志格式自动识别和解析
- 🎨 **现代化UI**: 基于Electron的响应式桌面界面
- ⚡ **高性能后端**: Rust编写的API服务，处理速度极快
- 🔧 **插件化架构**: 支持自定义解析插件扩展
- 📊 **实时统计**: 解析结果统计和可视化展示
- 🖥️ **跨平台支持**: Windows、macOS、Linux全平台兼容

## 🏗️ 架构设计

```
┌─────────────────┐    HTTP API    ┌─────────────────┐
│   Electron UI   │ ◄────────────► │   Rust API      │
│                 │                │   Server        │
│ • 用户界面      │                │ • 日志解析      │
│ • 窗口管理      │                │ • 插件系统      │
│ • 文件操作      │                │ • 性能优化      │
└─────────────────┘                └─────────────────┘
```

### 技术栈

- **前端**: Electron + HTML5 + CSS3 + JavaScript
- **后端**: Rust + Axum + Tokio
- **通信**: HTTP API (RESTful)
- **构建**: Cargo (Rust) + npm (Node.js)

## 🚀 快速开始

### 环境要求

- **Node.js**: v16.0+ 
- **Rust**: v1.70+
- **系统**: Windows 10+, macOS 10.15+, Ubuntu 18.04+

### 安装步骤

1. **克隆项目**
   ```bash
   git clone https://github.com/your-org/log-whisper.git
   cd log-whisper
   ```

2. **安装前端依赖**
   ```bash
   npm install
   ```

3. **编译Rust后端**
   ```bash
   cd src-rust
   cargo build --release
   cd ..
   ```

4. **启动应用**
   ```bash
   npm start
   ```

### 快速测试

如果遇到依赖安装问题，可以使用快速测试模式：

```bash
# Windows - 完整 Electron 应用
start-electron-app.bat

# Windows - 开发模式
start-dev.bat

# Windows - 快速测试（浏览器模式）
quick-test.bat

# Linux/macOS  
chmod +x quick-start.sh
./quick-start.sh
```

## 📋 支持的日志格式

| 格式类型 | 描述 | 支持状态 |
|----------|------|----------|
| **通用文本** | 普通的文本日志文件 | ✅ 完全支持 |
| **MyBatis SQL** | MyBatis框架的SQL执行日志 | ✅ 完全支持 |
| **Docker JSON** | Docker容器的JSON格式日志 | ✅ 完全支持 |
| **Apache访问日志** | Web服务器访问日志 | 🚧 开发中 |
| **Nginx访问日志** | Nginx服务器日志 | 🚧 开发中 |
| **系统日志** | Linux/Windows系统日志 | 📋 计划中 |

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
├── electron/                # Electron主进程代码
│   ├── main.js             # 应用入口和窗口管理
│   └── preload.js          # 安全的API桥接
├── src/                    # 前端页面
│   ├── index.html          # 主界面
│   └── main.js             # 前端JavaScript（已重构）
├── src-rust/               # 独立Rust API服务器
│   ├── src/
│   │   └── main.rs         # HTTP API 服务器
│   └── Cargo.toml          # Rust项目配置
├── docs/                   # 项目文档
├── scripts/                # 构建和部署脚本
├── start-electron-app.bat # Electron启动脚本
├── start-dev.bat          # 开发模式启动脚本
├── build-app.bat          # 构建脚本
├── test-integration.bat   # 集成测试脚本
└── package.json            # Node.js项目配置
```

### 开发模式

```bash
# 启动开发模式（自动重载）
npm run dev

# 单独启动Rust API服务器
npm run dev:rust

# 单独启动Electron应用
npm run dev:electron
```

### 构建部署

```bash
# 构建生产版本
npm run build

# 打包桌面应用
npm run package

# 构建安装包
npm run dist
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

| 测试项目 | 数据量 | 处理时间 | 内存占用 |
|----------|--------|----------|----------|
| **文本日志解析** | 100MB | ~2秒 | ~50MB |
| **JSON日志解析** | 50MB | ~1秒 | ~30MB |
| **复杂SQL日志** | 200MB | ~5秒 | ~80MB |

*测试环境: Intel i7-8750H, 16GB RAM, SSD*

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

## 📜 更新日志

### v1.0.0 (2025-10-02)
- ✨ **架构重构**：从Tauri成功迁移到Electron+Rust
- ✨ **独立API服务器**：Rust HTTP API服务器，更稳定的通信
- ✨ **简化环境检测**：Electron API检测更可靠
- ✨ **优化开发体验**：前后端独立开发，热重载更稳定
- ✨ **新增启动脚本**：多种启动方式，便于开发和测试
- 🐛 修复多项稳定性问题
- 📚 完善迁移文档和使用指南

详细更新日志请查看 [CHANGELOG.md](./CHANGELOG.md)。

## 📄 许可证

本项目采用 [MIT 许可证](./LICENSE)。

## 🙏 致谢

感谢所有贡献者和以下开源项目：

- [Electron](https://electronjs.org/) - 跨平台桌面应用框架
- [Rust](https://rust-lang.org/) - 系统编程语言
- [Axum](https://github.com/tokio-rs/axum) - Rust Web框架
- [Tokio](https://tokio.rs/) - 异步运行时

---

## 📞 联系我们

- **项目主页**: https://github.com/your-org/log-whisper
- **文档站点**: https://log-whisper.docs.com
- **技术支持**: support@log-whisper.com

---

<div align="center">

**⭐ 如果这个项目对您有帮助，请给我们一个Star！⭐**

Made with ❤️ by LogWhisper Team

</div>