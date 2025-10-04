# LogWhisper 启动指南

> **注意**: 此文档已过时，项目已迁移到 Electron + Rust 架构。请参考 [README.md](./README.md) 获取最新的启动指南。

## 🚀 正确的启动方式

LogWhisper 是一个基于 Tauri 的桌面应用程序，**必须在 Tauri 环境中运行**。

### 方法一：使用启动脚本（推荐）
```bash
# Windows 用户
start-dev.bat

# 或者双击 start-dev.bat 文件
```

### 方法二：使用 Cargo 命令
```bash
# 在项目根目录下执行
cargo tauri dev

# 或者构建发布版本
cargo tauri build
```

### 方法三：运行编译好的应用
```bash
# 构建应用
cargo tauri build

# 运行生成的可执行文件
.\target\release\log-whisper.exe
```

## ❌ 错误的启动方式

**请勿直接在浏览器中打开 HTML 文件！**

以下方式是错误的：
- 直接双击 `src/index.html`
- 在浏览器中打开 `file://` 协议的文件
- 通过 HTTP 服务器访问静态文件

## 🔍 环境检测

LogWhisper 包含了智能环境检测功能：

1. **正确环境**：应用运行在 `http://127.0.0.1:1430` (或类似的 localhost 地址)
2. **Tauri API**：检测 `window.__TAURI__` 对象是否可用
3. **错误环境**：显示警告并阻止功能使用

## 🛠️ 开发环境要求

### 必要工具
- [Rust](https://rustup.rs/) (最新稳定版)
- [Node.js](https://nodejs.org/) (可选，用于前端开发)

### Tauri CLI 安装
```bash
cargo install tauri-cli
```

### 验证安装
```bash
cargo tauri --version
```

## 🏃‍♂️ 快速开始

1. **克隆项目**
   ```bash
   git clone <repository-url>
   cd log-whisper
   ```

2. **安装依赖**
   ```bash
   cargo check
   ```

3. **启动开发模式**
   ```bash
   cargo tauri dev
   ```

4. **访问应用**
   - Tauri 会自动打开桌面应用窗口
   - 应用运行在 `http://127.0.0.1:1430`

## 🐛 故障排除

### 问题：显示"请在Tauri环境中运行"
**原因**：在浏览器中直接打开了 HTML 文件

**解决方案**：
1. 关闭浏览器窗口
2. 使用 `cargo tauri dev` 命令启动
3. 等待 Tauri 桌面应用自动打开

### 问题：编译错误
**解决方案**：
1. 确保 Rust 已正确安装：`rustc --version`
2. 更新 Rust：`rustup update`
3. 清理并重新构建：`cargo clean && cargo tauri dev`

### 问题：端口占用
**解决方案**：
1. 检查端口：`netstat -ano | findstr :1430`
2. 结束占用进程或使用其他端口

## 📝 日志和调试

应用包含详细的日志系统：
- **开发模式**：控制台输出 DEBUG 级别日志
- **生产模式**：文件输出 INFO 级别日志
- **调试面板**：内置开发者工具

## 📁 项目结构

```
log-whisper/
├── src/                    # 前端源码
│   ├── index.html         # 主页面
│   ├── main.js           # 前端逻辑
│   └── style.css         # 样式文件
├── src-tauri/             # Rust 后端
│   ├── src/              # Rust 源码
│   ├── Cargo.toml        # Rust 依赖
│   └── tauri.conf.json   # Tauri 配置
├── scripts/               # 构建脚本
├── start-dev.bat         # Windows 启动脚本
└── README.md             # 本文件
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

开发前请确保：
1. 使用正确的启动方式测试
2. 遵循代码规范
3. 添加必要的测试

---

**注意**：LogWhisper 必须在 Tauri 桌面应用环境中运行，不支持纯浏览器访问。