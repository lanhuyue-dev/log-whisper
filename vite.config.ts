import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // 指定项目根目录
  root: '.',

  // 构建配置
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'src/index.html')
      }
    }
  },

  // 开发服务器配置
  server: {
    port: 3005,
    strictPort: true
  },

  // 路径解析
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },

  // 清除控制台警告
  clearScreen: false,

  // 环境变量
  envPrefix: 'VITE_'
})