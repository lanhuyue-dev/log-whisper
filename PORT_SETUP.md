# LogWhisper 端口配置解决方案

## 问题描述

在Tauri桌面应用开发中，经常遇到Vite开发服务器和Tauri应用端口不一致的问题：
- Vite启动在一个端口（如5177）
- Tauri配置指向另一个端口（如5179）
- 导致应用无法正常加载

## 解决方案

我们设计了一个三层的端口管理系统：

### 1. 智能端口检测 (vite.config.ts)
- 使用固定端口范围：1420-1424
- 自动检测可用端口
- 将端口配置写入 `.vite-config/port.json`

### 2. 配置同步 (scripts/update-tauri-port.js)
- 读取Vite生成的端口配置
- 自动更新 `src-tauri/tauri.conf.json`
- 确保 Tauri 连接到正确的端口

### 3. 稳定启动 (scripts/start-stable.cjs)
- 按顺序启动各个组件
- 等待端口配置生成
- 自动同步配置
- 统一启动流程

## 使用方法

### 方法1：稳定启动（推荐）
```bash
npm run start:stable
```

### 方法2：手动同步
```bash
# 1. 启动 Vite
npm run dev

# 2. 同步端口配置
npm run sync:port

# 3. 启动 Tauri
npm start
```

### 方法3：传统方式（可能遇到端口问题）
```bash
npm start
```

## 工作原理

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   vite.config   │    │update-tauri-port│    │ start-stable    │
│                 │    │                  │    │                 │
│ 1. 检测可用端口  │ -> │ 1. 读取端口配置   │ -> │ 1. 启动Vite     │
│ 2. 生成配置文件   │    │ 2. 更新Tauri配置  │    │ 2. 等待配置     │
│ 3. 使用固定端口   │    │ 3. 确保端口一致    │    │ 3. 同步配置     │
└─────────────────┘    └──────────────────┘    │ 4. 启动Tauri   │
                                                 │ 5. 清理旧配置   │
                                                 └─────────────────┘
```

## 配置文件

### .vite-config/port.json
```json
{
  "port": 1420,
  "timestamp": 1640995200000,
  "url": "http://localhost:1420"
}
```

### src-tauri/tauri.conf.json
```json
{
  "build": {
    "devPath": "http://localhost:1420"
  }
}
```

## 故障排除

### 如果仍然遇到端口问题：

1. 清理配置文件：
   ```bash
   rm -rf .vite-config
   ```

2. 使用稳定启动：
   ```bash
   npm run start:stable
   ```

3. 检查端口占用：
   ```bash
   netstat -ano | findstr :1420
   ```

4. 手动指定端口：
   - 修改 `vite.config.ts` 中的 `PORT_RANGE`
   - 重新运行 `npm run start:stable`

## 开发建议

1. **日常开发**：使用 `npm run start:stable`
2. **快速测试**：直接修改代码，Vite会自动重载
3. **生产构建**：使用 `npm run dist:win`
4. **清理环境**：使用 `npm run clean`

## 技术细节

- Vite端口范围：1420-1424（Tauri推荐）
- 配置文件过期时间：30秒
- Vite启动超时：10秒
- 配置同步延迟：2秒

这个系统确保了端口的一致性，避免了开发中常见的端口冲突问题。