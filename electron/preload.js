const { contextBridge, ipcRenderer } = require('electron');

// 向渲染进程暴露安全的 API
contextBridge.exposeInMainWorld('electronAPI', {
  // 获取 API 配置
  getApiConfig: () => ipcRenderer.invoke('get-api-config'),
  
  // 窗口控制
  window: {
    minimize: () => ipcRenderer.invoke('window-minimize'),
    maximize: () => ipcRenderer.invoke('window-maximize'),
    close: () => ipcRenderer.invoke('window-close'),
    isMaximized: () => ipcRenderer.invoke('window-is-maximized')
  },
  
  // 平台信息
  platform: process.platform,
  
  // 环境标识
  isElectron: true
});

console.log('✅ Electron preload script loaded');