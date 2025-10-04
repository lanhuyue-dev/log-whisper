# LogWhisper 部署指南

## 🚀 快速开始

### 1. 开发环境

```bash
# 安装依赖
npm install

# 开发模式启动
npm run dev
```

### 2. 生产环境打包

```bash
# 构建所有组件
npm run build

# 打包应用
npm run dist

# 特定平台打包
npm run dist:win    # Windows
npm run dist:mac    # macOS
npm run dist:linux  # Linux
```

## 📦 部署流程

### 1. 启动 API 服务器

**Windows:**
```cmd
start-api.bat
```

**Linux/macOS:**
```bash
./start-api.sh
```

### 2. 启动 Electron 应用

```bash
npm start
```

## ⚙️ 配置说明

### 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `LOGWHISPER_PORT` | 3030 | API 服务器端口 |
| `LOGWHISPER_LOG_FILE` | logs/log-whisper.log | 日志文件路径 |
| `LOGWHISPER_LOG_LEVEL` | info | 日志级别 |

### 配置文件

`config.json` 包含应用配置：

```json
{
  "api": {
    "port": 3030,
    "host": "127.0.0.1"
  },
  "logging": {
    "level": "info",
    "file": "logs/log-whisper.log"
  }
}
```

## 📁 目录结构

```
log-whisper/
├── dist/                    # 打包输出目录
├── logs/                    # 日志文件目录
├── resources/               # 打包后的资源文件
│   ├── api-server.exe      # Rust API 服务器
│   ├── start-api.bat       # Windows 启动脚本
│   ├── start-api.sh        # Unix 启动脚本
│   └── config.json         # 配置文件
├── src/                    # 前端源码
├── src-rust/               # Rust API 源码
└── electron/               # Electron 主进程
```

## 🔧 故障排除

### 1. API 服务器启动失败

- 检查端口是否被占用
- 检查日志文件权限
- 查看 `logs/log-whisper.log` 文件

### 2. 应用无法连接 API

- 确认 API 服务器已启动
- 检查端口配置
- 查看网络连接

### 3. 日志文件问题

- 检查 `logs/` 目录权限
- 确认磁盘空间充足
- 查看日志级别设置

## 📋 部署检查清单

- [ ] Rust API 服务器编译成功
- [ ] Electron 应用打包成功
- [ ] 启动脚本可执行
- [ ] 配置文件正确
- [ ] 日志目录可写
- [ ] 端口未被占用
- [ ] 防火墙设置正确

## 🚀 生产环境部署

### 1. 服务器部署

```bash
# 1. 上传打包文件到服务器
scp -r dist/ user@server:/opt/log-whisper/

# 2. 设置权限
chmod +x /opt/log-whisper/resources/start-api.sh

# 3. 启动服务
cd /opt/log-whisper/resources/
./start-api.sh
```

### 2. 系统服务配置

**systemd 服务文件** (`/etc/systemd/system/log-whisper-api.service`):

```ini
[Unit]
Description=LogWhisper API Server
After=network.target

[Service]
Type=simple
User=logwhisper
WorkingDirectory=/opt/log-whisper/resources
ExecStart=/opt/log-whisper/resources/api-server
Restart=always
RestartSec=5
Environment=LOGWHISPER_PORT=3030
Environment=LOGWHISPER_LOG_FILE=/var/log/log-whisper/api.log
Environment=LOGWHISPER_LOG_LEVEL=info

[Install]
WantedBy=multi-user.target
```

**启动服务:**
```bash
sudo systemctl enable log-whisper-api
sudo systemctl start log-whisper-api
sudo systemctl status log-whisper-api
```

## 📊 监控和维护

### 日志轮转

```bash
# 设置日志轮转
sudo logrotate -f /etc/logrotate.d/log-whisper
```

### 性能监控

```bash
# 查看 API 服务器状态
curl http://localhost:3030/health

# 查看日志
tail -f logs/log-whisper.log
```

## 🔒 安全考虑

1. **防火墙配置**: 只开放必要端口
2. **用户权限**: 使用专用用户运行服务
3. **日志安全**: 定期清理敏感日志
4. **网络安全**: 使用 HTTPS 和认证
