#!/usr/bin/env node

/**
 * LogWhisper 稳定启动脚本
 * 解决端口不一致问题，确保Vite和Tauri使用相同端口
 */

const { spawn, exec } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🚀 Starting LogWhisper with stable port configuration...');

// 清理旧的配置文件
function cleanupOldConfig() {
  const configDir = path.join(process.cwd(), '.vite-config');
  const configFile = path.join(configDir, 'port.json');

  if (fs.existsSync(configFile)) {
    const config = JSON.parse(fs.readFileSync(configFile, 'utf8'));
    const configAge = Date.now() - config.timestamp;

    // 如果配置文件超过5分钟，删除它
    if (configAge > 300000) {
      fs.unlinkSync(configFile);
      console.log('🗑️ Cleaned up old port configuration');
    }
  }
}

// 启动Vite服务器
function startViteServer() {
  return new Promise((resolve, reject) => {
    console.log('📦 Starting Vite server...');

    const vite = spawn('npm', ['run', 'build:css:prod && npm run dev'], {
      shell: true,
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let output = '';
    vite.stdout.on('data', (data) => {
      const text = data.toString();
      output += text;
      console.log(text.trim());

      // 检查Vite是否已启动
      if (text.includes('Local:') && text.includes('http://localhost:')) {
        const match = text.match(/http:\/\/localhost:(\d+)/);
        if (match) {
          const port = match[1];
          console.log(`✅ Vite server started on port ${port}`);
          vite.vitePort = port;
          resolve(vite);
        }
      }
    });

    vite.stderr.on('data', (data) => {
      const text = data.toString();
      console.error(text.trim());
    });

    vite.on('error', (error) => {
      console.error('❌ Failed to start Vite server:', error.message);
      reject(error);
    });

    // 10秒超时
    setTimeout(() => {
      if (!vite.vitePort) {
        console.log('⚠️ Vite server startup timeout, using default port 1420');
        vite.vitePort = '1420';
        resolve(vite);
      }
    }, 10000);
  });
}

// 更新Tauri配置
function updateTauriConfig(port) {
  return new Promise((resolve, reject) => {
    console.log(`⚙️ Updating Tauri configuration for port ${port}...`);

    const updateScript = spawn('node', ['scripts/update-tauri-port.js'], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let output = '';
    updateScript.stdout.on('data', (data) => {
      const text = data.toString();
      output += text;
      console.log(text.trim());
    });

    updateScript.stderr.on('data', (data) => {
      console.error(data.toString().trim());
    });

    updateScript.on('close', (code) => {
      if (code === 0) {
        console.log('✅ Tauri configuration updated');
        resolve();
      } else {
        console.error('❌ Failed to update Tauri configuration');
        reject(new Error('Tauri config update failed'));
      }
    });
  });
}

// 启动Tauri应用
function startTauriApp() {
  return new Promise((resolve, reject) => {
    console.log('🖥️ Starting Tauri desktop application...');

    const tauri = spawn('npm', ['start'], {
      shell: true,
      stdio: ['pipe', 'pipe', 'pipe']
    });

    tauri.stdout.on('data', (data) => {
      const text = data.toString();
      console.log(text.trim());
    });

    tauri.stderr.on('data', (data) => {
      const text = data.toString();
      console.error(text.trim());
    });

    tauri.on('error', (error) => {
      console.error('❌ Failed to start Tauri application:', error.message);
      reject(error);
    });

    // 应用启动成功
    tauri.on('spawn', () => {
      console.log('✅ Tauri application started successfully');
      resolve(tauri);
    });
  });
}

// 主启动流程
async function main() {
  try {
    // 清理旧配置
    cleanupOldConfig();

    // 启动Vite服务器
    const viteProcess = await startViteServer();

    // 等待配置文件生成
    await new Promise(resolve => setTimeout(resolve, 2000));

    // 尝试读取端口配置
    let port = '1420'; // 默认端口
    const configFile = path.join(process.cwd(), '.vite-config', 'port.json');

    if (fs.existsSync(configFile)) {
      const config = JSON.parse(fs.readFileSync(configFile, 'utf8'));
      port = config.port.toString();
      console.log(`📋 Read port configuration: ${port}`);
    }

    // 更新Tauri配置
    await updateTauriConfig(port);

    // 等待配置生效
    await new Promise(resolve => setTimeout(resolve, 1000));

    // 启动Tauri应用
    const tauriProcess = await startTauriApp();

    console.log('🎉 LogWhisper started successfully!');
    console.log(`📱 Vite server: http://localhost:${port}`);
    console.log('🖥️ Desktop application: opening...');

    // 处理进程退出
    process.on('SIGINT', () => {
      console.log('\n🛑 Shutting down LogWhisper...');
      viteProcess.kill();
      tauriProcess.kill();
      process.exit(0);
    });

  } catch (error) {
    console.error('❌ Failed to start LogWhisper:', error.message);
    process.exit(1);
  }
}

// 启动应用
main();