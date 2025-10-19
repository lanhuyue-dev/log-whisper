import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import fs from 'fs'
import path from 'path'

// 固定端口配置解决方案
// 1. 使用固定端口范围避免冲突
// 2. 将端口信息写入文件供Tauri读取
// 3. 提供多端口备选方案

const DEV_PORT = 1420  // Tauri推荐的开发端口
const PORT_RANGE = [1420, 1421, 1422, 1423, 1424] // 备选端口范围

// 检查端口是否可用的函数
function isPortAvailable(port) {
  try {
    const net = require('net')
    const server = net.createServer()

    return new Promise((resolve) => {
      server.listen(port, () => {
        server.once('close', () => {
          resolve(true)
        })
        server.close()
      })

      server.on('error', () => {
        resolve(false)
      })
    })
  } catch (error) {
    console.warn(`Port check failed for ${port}:`, error.message)
    return Promise.resolve(false)
  }
}

// 获取可用端口
async function getAvailablePort() {
  for (const port of PORT_RANGE) {
    if (await isPortAvailable(port)) {
      return port
    }
  }
  // 如果所有预设端口都不可用，返回0让系统自动选择
  return 0
}

// 将端口信息写入配置文件
function writePortConfig(port) {
  const configDir = path.join(process.cwd(), '.vite-config')
  if (!fs.existsSync(configDir)) {
    fs.mkdirSync(configDir, { recursive: true })
  }

  const configFile = path.join(configDir, 'port.json')
  fs.writeFileSync(configFile, JSON.stringify({
    port,
    timestamp: Date.now(),
    url: `http://localhost:${port}`
  }))
}

// Tauri 桌面应用 Vite 配置
export default defineConfig(async () => {
  const availablePort = await getAvailablePort()
  writePortConfig(availablePort)

  console.log(`🚀 Vite will run on port: ${availablePort}`)
  console.log(`📝 Port configuration saved to .vite-config/port.json`)

  return {
    plugins: [react()],

    // 指定项目根目录
    root: '.',

    // 构建配置 - 针对 Tauri 桌面应用优化
    build: {
      outDir: 'dist',
      emptyOutDir: true,
      assetsDir: 'assets',
      // 优化桌面应用的构建
      minify: 'terser',
      sourcemap: false
    },

    // 开发服务器配置 - 稳定端口配置
    server: {
      port: availablePort || 1420, // 使用找到的可用端口，或默认端口
      host: 'localhost', // 明确指定主机
      strictPort: availablePort > 0, // 如果指定了端口，必须使用它
      open: false, // 不自动打开浏览器
      cors: true // 允许跨域
    },

    // 清除控制台警告
    clearScreen: false,

    // 环境变量
    envPrefix: 'VITE_',

    // 预览服务器配置（生产构建时使用）
    preview: {
      port: 1420,
      host: 'localhost'
    }
  }
})