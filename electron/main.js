const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { spawn } = require('child_process');

// 优化Electron启动性能
app.commandLine.appendSwitch('--disable-gpu-sandbox');
app.commandLine.appendSwitch('--disable-software-rasterizer');
app.commandLine.appendSwitch('--disable-gpu');
app.commandLine.appendSwitch('--disable-gpu-compositing');
app.commandLine.appendSwitch('--disable-gpu-rasterization');
app.commandLine.appendSwitch('--disable-gpu-sandbox');
app.commandLine.appendSwitch('--disable-accelerated-2d-canvas');
app.commandLine.appendSwitch('--disable-accelerated-jpeg-decoding');
app.commandLine.appendSwitch('--disable-accelerated-mjpeg-decode');
app.commandLine.appendSwitch('--disable-accelerated-video-decode');
app.commandLine.appendSwitch('--disable-accelerated-video-encode');
app.commandLine.appendSwitch('--disable-background-timer-throttling');
app.commandLine.appendSwitch('--disable-backgrounding-occluded-windows');
app.commandLine.appendSwitch('--disable-renderer-backgrounding');
app.commandLine.appendSwitch('--disable-features', 'TranslateUI');
app.commandLine.appendSwitch('--disable-ipc-flooding-protection');

// 保持对窗口对象的全局引用
let mainWindow;
let rustApiProcess;

// API 配置
const API_PORT = 3030;
const API_BASE_URL = `http://127.0.0.1:${API_PORT}`;

function createWindow() {
  // 创建浏览器窗口
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 1200,
    minHeight: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      enableRemoteModule: false,
      preload: path.join(__dirname, 'preload.js'),
      // 优化渲染性能
      offscreen: false,
      backgroundThrottling: false,
      webSecurity: false
    },
    show: false, // 先不显示，等加载完成
    center: true,
    title: 'LogWhisper - 日志分析工具',
    backgroundColor: '#f9fafb', // 设置背景色防止闪烁
    frame: true, // 使用系统标题栏
    titleBarStyle: 'default', // 使用默认标题栏样式
    // 优化窗口显示
    transparent: false,
    alwaysOnTop: false,
    skipTaskbar: false,
    // 禁用窗口动画
    resizable: true,
    movable: true,
    minimizable: true,
    maximizable: true,
    closable: true
  });

  // 加载应用的 index.html
  mainWindow.loadFile(path.join(__dirname, '../src/index.html'));

  // 优化窗口显示逻辑
  mainWindow.once('ready-to-show', () => {
    console.log('🪟 窗口准备显示');
    // 立即显示窗口，不等待其他条件
    mainWindow.show();
    console.log('✅ 窗口已显示');
    
    // 开发模式下打开开发者工具
    if (process.env.NODE_ENV === 'development') {
      mainWindow.webContents.openDevTools();
    }
  });
  
  // 防止页面重载时的闪烁
  mainWindow.webContents.on('did-finish-load', () => {
    // 页面加载完成，可以安全显示
    console.log('📄 页面加载完成');
  });
  
  // 等待所有资源加载完成
  mainWindow.webContents.on('did-frame-finish-load', () => {
    console.log('🎨 框架渲染完成');
  });
  
  // 禁用导航，防止意外跳转
  mainWindow.webContents.on('will-navigate', (event) => {
    event.preventDefault();
  });
  
  // 禁用新窗口创建
  mainWindow.webContents.setWindowOpenHandler(() => {
    return { action: 'deny' };
  });

  // 当窗口被关闭时
  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

// 启动 Rust API 服务器
function startRustApi() {
  console.log('🚀 启动 Rust API 服务器...');
  
  const rustBinary = process.env.NODE_ENV === 'development' 
    ? path.join(__dirname, '../src-rust/target/debug/api-server.exe')
    : path.join(__dirname, '../src-rust/target/release/api-server.exe');
    
  rustApiProcess = spawn(rustBinary, [], {
    stdio: ['ignore', 'pipe', 'pipe']
  });
  
  rustApiProcess.stdout.on('data', (data) => {
    console.log(`[Rust API] ${data.toString().trim()}`);
  });
  
  rustApiProcess.stderr.on('data', (data) => {
    console.error(`[Rust API Error] ${data.toString().trim()}`);
  });
  
  rustApiProcess.on('close', (code) => {
    console.log(`Rust API 进程退出，代码: ${code}`);
  });
  
  // 等待 API 服务器启动
  setTimeout(() => {
    console.log('✅ Rust API 服务器应该已经启动');
  }, 2000);
}

// 应用准备完成
app.whenReady().then(() => {
  console.log('🚀 Electron 应用准备完成');
  
  // 立即创建窗口
  createWindow();
  
  // 同时启动 Rust API（不阻塞窗口创建）
  startRustApi();
  
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

// 所有窗口关闭时退出应用 (macOS 除外)
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    // 关闭 Rust API 进程
    if (rustApiProcess) {
      rustApiProcess.kill();
    }
    app.quit();
  }
});

// 应用即将退出时清理资源
app.on('before-quit', (event) => {
  if (rustApiProcess) {
    console.log('🛑 关闭 Rust API 服务器...');
    rustApiProcess.kill('SIGTERM');
    
    // 等待进程关闭
    setTimeout(() => {
      if (!rustApiProcess.killed) {
        console.log('⚠️ 强制关闭 Rust API 服务器...');
        rustApiProcess.kill('SIGKILL');
      }
    }, 2000);
  }
});

// 应用退出时确保清理
app.on('will-quit', (event) => {
  if (rustApiProcess && !rustApiProcess.killed) {
    console.log('🛑 应用退出，关闭 Rust API 服务器...');
    rustApiProcess.kill('SIGTERM');
  }
});

// IPC 处理器
ipcMain.handle('get-api-config', () => {
  return {
    baseUrl: API_BASE_URL,
    port: API_PORT
  };
});

// 窗口控制
ipcMain.handle('window-minimize', () => {
  if (mainWindow) {
    mainWindow.minimize();
  }
});

ipcMain.handle('window-maximize', () => {
  if (mainWindow) {
    if (mainWindow.isMaximized()) {
      mainWindow.unmaximize();
    } else {
      mainWindow.maximize();
    }
  }
});

ipcMain.handle('window-close', () => {
  if (mainWindow) {
    mainWindow.close();
  }
});

ipcMain.handle('window-is-maximized', () => {
  return mainWindow ? mainWindow.isMaximized() : false;
});