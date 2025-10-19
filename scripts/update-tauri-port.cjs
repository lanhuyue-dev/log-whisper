#!/usr/bin/env node

/**
 * Tauri端口同步脚本
 * 读取Vite生成的端口配置，并更新Tauri配置文件
 */

const fs = require('fs');
const path = require('path');

// 配置文件路径
const VITE_PORT_CONFIG = path.join(process.cwd(), '.vite-config', 'port.json');
const TAURI_CONFIG = path.join(process.cwd(), 'src-tauri', 'tauri.conf.json');

// 默认端口
const DEFAULT_PORT = 1420;

function updateTauriConfig() {
  try {
    let port = DEFAULT_PORT;

    // 尝试读取Vite端口配置
    if (fs.existsSync(VITE_PORT_CONFIG)) {
      const viteConfig = JSON.parse(fs.readFileSync(VITE_PORT_CONFIG, 'utf8'));
      port = viteConfig.port || DEFAULT_PORT;

      // 检查配置是否过期（超过30秒）
      const configAge = Date.now() - viteConfig.timestamp;
      if (configAge > 30000) {
        console.warn(`⚠️ Vite port config is old (${configAge}ms), using default port ${DEFAULT_PORT}`);
        port = DEFAULT_PORT;
      }
    } else {
      console.warn(`⚠️ Vite port config not found, using default port ${DEFAULT_PORT}`);
    }

    // 读取Tauri配置
    const tauriConfig = JSON.parse(fs.readFileSync(TAURI_CONFIG, 'utf8'));
    const currentDevPath = tauriConfig.build.devPath;
    const expectedDevPath = `http://localhost:${port}`;

    // 检查是否需要更新
    if (currentDevPath !== expectedDevPath) {
      console.log(`🔄 Updating Tauri devPath: ${currentDevPath} -> ${expectedDevPath}`);

      // 更新配置
      tauriConfig.build.devPath = expectedDevPath;

      // 写回文件
      fs.writeFileSync(TAURI_CONFIG, JSON.stringify(tauriConfig, null, 2));

      console.log(`✅ Tauri configuration updated to use port ${port}`);
      return port;
    } else {
      console.log(`✅ Tauri configuration already using port ${port}`);
      return port;
    }

  } catch (error) {
    console.error('❌ Failed to update Tauri configuration:', error.message);
    return DEFAULT_PORT;
  }
}

// 如果直接运行此脚本
if (require.main === module) {
  const port = updateTauriConfig();
  console.log(`🚀 Tauri will connect to: http://localhost:${port}`);
}

module.exports = { updateTauriConfig };