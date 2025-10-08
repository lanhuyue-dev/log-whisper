// LogWhisper 前端应用 - Electron 版本 (基于 Tailwind CSS)
class LogWhisperApp {
    constructor() {
        this.currentFile = null;
        this.currentEntries = [];
        this.searchTerm = '';
        this.isLoading = false;
        this.currentTheme = 'light';
        this.debugMode = false;
        
        // API 配置
        this.API_BASE_URL = 'http://127.0.0.1:3030';
        this.isApiAvailable = false;
        this.isElectronEnv = false;
        
           // 插件管理
           this.installedPlugins = [];
           this.availablePlugins = [];
           
           // 配置管理
           this.configs = {
               theme: null,
               parse: null,
               plugin: null,
               window: null
           };

           // EmEditor 风格编辑器状态
           this.logLines = [];
           this.filteredLines = [];
           this.currentFilter = 'all';
           this.searchTerm = '';
           this.searchResults = [];
           this.currentLine = 0;
           this.totalLines = 0;
           this.pluginCategories = {};
           this.sidebarCollapsed = false;
        this.pluginSettings = {
            autoUpdate: false,
            notifications: true
        };
        
        // 解析时间
        this.parseTime = null;
        
        // 日志格式标志（用于控制异常处理）
        this.isDockerJsonFormat = false;
        
        this.init();
    }
    
           async init() {
               console.log('🚀 LogWhisper 前端应用初始化...');
               console.time('⏱️ 初始化总耗时');
               
               // 更新加载状态
               this.updateLoadingStatus('初始化组件...');
               console.log('📋 1. 组件初始化开始');
               
               // 设置事件监听器
               this.setupEventListeners();
               console.log('📋 2. 事件监听器设置完成');
               
               // 设置拖拽功能
               this.setupDragAndDrop();
               console.log('📋 3. 拖拽功能设置完成');
               
               // 初始化主题
               this.initTheme();
               
               // 强制应用暗色主题进行测试
               // this.forceDarkTheme(); // 注释掉强制暗色主题
               
               // 加载配置（异步等待）
               await this.loadConfigs();
               console.log('📋 4. 主题初始化完成');
               
               // 更新加载状态
               this.updateLoadingStatus('检测环境...');
               console.log('📋 5. 开始环境检测');
               
               // 初始化 Electron 环境
               this.initElectron();
               console.log('📋 6. Electron 环境初始化完成');
               
               // 更新加载状态
               this.updateLoadingStatus('连接 API 服务器...');
               console.log('📋 7. 开始连接 API 服务器');
               
               // 检查 API 状态（异步）
               await this.checkApiStatus();
               console.log('📋 8. API 连接完成，开始最终初始化');
               
               // 初始化插件管理
               this.initPluginManager();
               console.log('📋 9. 插件管理初始化完成');
               
               // 解析按钮已移除，无需初始化
               console.log('📋 10. 解析按钮状态初始化完成');
               
               // 所有初始化完成，显示主应用
               console.log('📋 11. 开始显示主应用');
               this.showMainApp();
               
               console.timeEnd('⏱️ 初始化总耗时');
               console.log('✅ LogWhisper 前端应用初始化完成');
           }
    
    updateLoadingStatus(message) {
        const statusElement = document.getElementById('loadingStatus');
        if (statusElement) {
            statusElement.textContent = message;
        }
        console.log('📋 加载状态:', message);
    }
    
           showMainApp() {
               console.log('🎯 开始显示主应用...');
               
               // 更新加载状态
               this.updateLoadingStatus('加载完成！');
               console.log('📋 加载状态更新完成');
               
               // 立即显示主应用，不等待
               const loadingPage = document.getElementById('loadingPage');
               const mainApp = document.getElementById('mainApp');
               
               console.log('🔍 查找元素:', { loadingPage: !!loadingPage, mainApp: !!mainApp });
               
               if (loadingPage && mainApp) {
                   console.log('✅ 元素找到，开始切换...');
                   
                   // 先显示主应用，再隐藏加载页面
                   mainApp.classList.remove('hidden');
                   mainApp.style.opacity = '1';
                   mainApp.style.transition = 'none'; // 立即显示，无过渡
                   console.log('📥 主应用立即显示');
                   
                   // 延迟隐藏加载页面，确保主应用已经显示
                   setTimeout(() => {
                       loadingPage.style.opacity = '0';
                       loadingPage.style.transition = 'opacity 0.3s ease-out';
                       console.log('📤 加载页面开始淡出');
                       
                       // 完全隐藏加载页面
                       setTimeout(() => {
                           loadingPage.style.display = 'none';
                           console.log('✅ 加载页面完全隐藏');
                       }, 300);
                   }, 100);
                   
                   console.log('✨ 主应用显示完成');
               } else {
                   console.error('❌ 找不到必要的元素:', { loadingPage, mainApp });
               }
           }
           
           // 加载配置
           async loadConfigs() {
               try {
                   console.log('📋 开始加载配置...');
                   
                   // 加载主题配置
                   const themeResponse = await fetch(`${this.API_BASE_URL}/api/config/theme`);
                   if (themeResponse.ok) {
                       const themeData = await themeResponse.json();
                       if (themeData.success && themeData.data) {
                           this.configs.theme = themeData.data;
                           this.applyTheme(themeData.data);
                           console.log('✅ 主题配置加载成功');
                       }
                   }
                   
                   // 解析配置已移除，不再需要自动解析功能
                   
                   // 加载插件配置
                   try {
                       const pluginResponse = await fetch(`${this.API_BASE_URL}/api/config/plugin`);
                       if (pluginResponse.ok) {
                           const pluginData = await pluginResponse.json();
                           if (pluginData.success && pluginData.data) {
                               this.configs.plugin = pluginData.data;
                               console.log('✅ 插件配置加载成功');
                           }
                       } else {
                           console.warn('⚠️ 插件配置API不可用:', pluginResponse.status);
                       }
                   } catch (error) {
                       console.warn('⚠️ 插件配置加载失败:', error.message);
                   }
                   
                   // 加载窗口配置
                   try {
                       const windowResponse = await fetch(`${this.API_BASE_URL}/api/config/window`);
                       if (windowResponse.ok) {
                           const windowData = await windowResponse.json();
                           if (windowData.success && windowData.data) {
                               this.configs.window = windowData.data;
                               console.log('✅ 窗口配置加载成功');
                           }
                       } else {
                           console.warn('⚠️ 窗口配置API不可用:', windowResponse.status);
                       }
                   } catch (error) {
                       console.warn('⚠️ 窗口配置加载失败:', error.message);
                   }
                   
                   console.log('✅ 所有配置加载完成');
               } catch (error) {
                   console.warn('⚠️ 配置加载失败:', error.message);
               }
           }
           
           // 应用主题
           applyTheme(themeConfig) {
               const { mode, primary_color, accent_color, font_size, font_family } = themeConfig;
               
               console.log('🎨 开始应用主题:', { mode, primary_color, accent_color, font_size, font_family });
               
               // 应用主题模式
               if (mode === 'dark') {
                   document.documentElement.classList.add('dark');
                   document.body.classList.add('dark');
                   this.currentTheme = 'dark';
                   console.log('🌙 暗色主题已应用');
               } else if (mode === 'light') {
                   document.documentElement.classList.remove('dark');
                   document.body.classList.remove('dark');
                   this.currentTheme = 'light';
                   console.log('☀️ 亮色主题已应用');
               } else if (mode === 'auto') {
                   // 自动模式：根据系统偏好设置
                   const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
                   if (prefersDark) {
                       document.documentElement.classList.add('dark');
                       document.body.classList.add('dark');
                       this.currentTheme = 'dark';
                       console.log('🌙 自动模式：暗色主题已应用');
                   } else {
                       document.documentElement.classList.remove('dark');
                       document.body.classList.remove('dark');
                       this.currentTheme = 'light';
                       console.log('☀️ 自动模式：亮色主题已应用');
                   }
               }
               
               // 应用颜色
               if (primary_color) {
                   document.documentElement.style.setProperty('--primary-color', primary_color);
               }
               if (accent_color) {
                   document.documentElement.style.setProperty('--accent-color', accent_color);
               }
               
               // 应用字体
               if (font_size) {
                   document.documentElement.style.setProperty('--font-size', `${font_size}px`);
               }
               if (font_family) {
                   document.documentElement.style.setProperty('--font-family', font_family);
               }
               
           // 更新主题切换按钮图标
           this.updateThemeToggleIcon();
           
           console.log('✅ 主题应用完成:', { 
               currentTheme: this.currentTheme, 
               hasDarkClass: document.documentElement.classList.contains('dark'),
               bodyHasDarkClass: document.body.classList.contains('dark')
           });
       }
       
           // 强制应用暗色主题进行测试
           forceDarkTheme() {
               console.log('🌙 强制应用暗色主题进行测试...');
               document.documentElement.classList.add('dark');
               document.body.classList.add('dark');
               this.currentTheme = 'dark';
               this.updateThemeToggleIcon();
               console.log('✅ 强制暗色主题已应用');
           }

           // EmEditor 风格日志渲染
           renderLogEditor(entries) {
               console.log('📝 开始渲染 EmEditor 风格日志编辑器...');
               this.logLines = entries;
               this.totalLines = entries.length;
               this.filteredLines = [...entries];
               
               // 隐藏欢迎界面，显示编辑器
               const welcomeScreen = document.getElementById('welcomeScreen');
               const logEditor = document.getElementById('logEditor');
               const editorToolbar = document.getElementById('editorToolbar');
               
               if (welcomeScreen) welcomeScreen.classList.add('hidden');
               if (logEditor) {
                   logEditor.classList.remove('hidden');
                   // 移除任何强制设置的高度样式，让flex布局正常工作
                   logEditor.style.removeProperty('height');
                   logEditor.style.removeProperty('max-height');
               }
               if (editorToolbar) editorToolbar.classList.remove('hidden');
               
               // 确保logContent容器正确使用flex布局
               const logContent = document.getElementById('logContent');
               if (logContent) {
                   logContent.style.removeProperty('height');
                   logContent.style.removeProperty('max-height');
               }
               
               // 渲染日志行
               this.renderLogLines();
               
               // 更新侧边栏导航
               this.updateSidebarNavigation();
               
               // 更新状态栏
               this.updateStatusBar();
               
               console.log('✅ EmEditor 风格日志编辑器渲染完成');
           }

           // 分块渲染日志编辑器（用于处理大文件）
           async renderLogEditorChunked(entries) {
               console.log('📝 开始分块渲染 EmEditor 风格日志编辑器...');
               console.log('📊 总条目数:', entries.length);
               
               this.logLines = entries;
               this.totalLines = entries.length;
               this.filteredLines = [...entries];
               
               // 隐藏欢迎界面，显示编辑器
               const welcomeScreen = document.getElementById('welcomeScreen');
               const logEditor = document.getElementById('logEditor');
               const editorToolbar = document.getElementById('editorToolbar');
               
               if (welcomeScreen) {
                   welcomeScreen.classList.add('hidden');
                   welcomeScreen.style.setProperty('display', 'none', 'important');
                   welcomeScreen.style.setProperty('visibility', 'hidden', 'important');
                   console.log('🔧 隐藏欢迎界面');
                   console.log('🔧 欢迎界面类名:', welcomeScreen.className);
                   console.log('🔧 欢迎界面计算样式:', window.getComputedStyle(welcomeScreen).display);
               }
               if (logEditor) logEditor.classList.remove('hidden');
               if (editorToolbar) editorToolbar.classList.remove('hidden');
               
               // 根据文件大小动态调整分块参数
               const isLargeFile = entries.length > 10000;
               const CHUNK_SIZE = isLargeFile ? 50 : 100; // 大文件使用更小的块
               const RENDER_DELAY = isLargeFile ? 5 : 10; // 大文件使用更短的延迟
               
               const logLinesContainer = document.getElementById('logLines');
               if (!logLinesContainer) {
                   console.error('❌ 找不到logLines容器');
                   return;
               }
               
               console.log('📝 清空并设置logLines容器');
               logLinesContainer.innerHTML = '';
               logLinesContainer.className = 'log-editor';
               
               // 重新设计布局系统 - 移除强制高度设置，让flex布局正常工作
               const logContentForHeight = document.getElementById('logContent');
               const logEditorForHeight = document.getElementById('logEditor');
               
               // 移除所有强制高度设置，让容器使用flex布局
               if (logContentForHeight) {
                   logContentForHeight.style.removeProperty('height');
                   logContentForHeight.style.removeProperty('max-height');
                   console.log('🔧 移除logContent的强制高度设置');
               }
               if (logEditorForHeight) {
                   logEditorForHeight.style.removeProperty('height');
                   logEditorForHeight.style.removeProperty('max-height');
                   console.log('🔧 移除logEditor的强制高度设置');
               }
               
               // 确保logLines容器使用自然高度
               logLinesContainer.style.removeProperty('height');
               logLinesContainer.style.removeProperty('max-height');
               console.log('🔧 设置logLines容器样式');
               
               // 移除测试行，避免影响布局
               // const testDiv = document.createElement('div');
               // testDiv.textContent = '测试行 - 如果看到这行说明渲染工作正常';
               // testDiv.style.cssText = 'padding: 10px; background: yellow; border: 1px solid red;';
               // logLinesContainer.appendChild(testDiv);
               
               // 开始分块渲染
               
               let renderedCount = 0;
               const totalCount = entries.length;
               
               // 分块渲染
               for (let i = 0; i < entries.length; i += CHUNK_SIZE) {
                   const chunk = entries.slice(i, i + CHUNK_SIZE);
                   
                   // 渲染当前块
                   chunk.forEach((entry, chunkIndex) => {
                       const globalIndex = i + chunkIndex;
                       const lineElement = this.createLogLineElement(entry, globalIndex);
                       logLinesContainer.appendChild(lineElement);
                       
                       // 调试：每100行输出一次
                       if (globalIndex % 100 === 0) {
                           console.log(`📝 已渲染 ${globalIndex + 1} 行`);
                       }
                   });
                   
                   renderedCount += chunk.length;
                   
                   // 让出控制权，避免阻塞UI
                   if (i + CHUNK_SIZE < entries.length) {
                       await new Promise(resolve => setTimeout(resolve, RENDER_DELAY));
                   }
               }
               
               // 渲染完成
               
               // 更新侧边栏导航
               this.updateSidebarNavigation();
               
               // 更新状态栏
               this.updateStatusBar();
               
               console.log('✅ 分块渲染完成，共渲染', renderedCount, '行');
               console.log('📊 最终渲染的行数:', logLinesContainer.children.length);
               console.log('📊 容器内容:', logLinesContainer.innerHTML.substring(0, 200) + '...');
               
               // 渲染完成后不再强制设置高度，使用flex布局
               
               // 确保欢迎界面被隐藏
               const welcomeScreenAfter = document.getElementById('welcomeScreen');
               if (welcomeScreenAfter) {
                   welcomeScreenAfter.classList.add('hidden');
                   console.log('🔧 渲染后确认隐藏欢迎界面');
                   console.log('🔧 欢迎界面类名:', welcomeScreenAfter.className);
                   console.log('🔧 欢迎界面计算样式:', window.getComputedStyle(welcomeScreenAfter).display);
               }
               
               // 调试容器可见性
               console.log('🔍 logLines容器信息:');
               console.log('  - 可见性:', window.getComputedStyle(logLinesContainer).visibility);
               console.log('  - 显示:', window.getComputedStyle(logLinesContainer).display);
               console.log('  - 高度:', window.getComputedStyle(logLinesContainer).height);
               console.log('  - 宽度:', window.getComputedStyle(logLinesContainer).width);
               console.log('  - 位置:', logLinesContainer.getBoundingClientRect());
               
               // 调试父容器
               const logContent = document.getElementById('logContent');
               if (logContent) {
                   console.log('🔍 logContent容器信息:');
                   console.log('  - 可见性:', window.getComputedStyle(logContent).visibility);
                   console.log('  - 显示:', window.getComputedStyle(logContent).display);
                   console.log('  - 高度:', window.getComputedStyle(logContent).height);
                   console.log('  - 最大高度:', window.getComputedStyle(logContent).maxHeight);
                   console.log('  - Flex:', window.getComputedStyle(logContent).flex);
                   console.log('  - 位置:', logContent.getBoundingClientRect());
               }
               
               // 调试logEditor容器
               const logEditorElement = document.getElementById('logEditor');
               if (logEditorElement) {
                   console.log('🔍 logEditor容器信息:');
                   console.log('  - 可见性:', window.getComputedStyle(logEditorElement).visibility);
                   console.log('  - 显示:', window.getComputedStyle(logEditorElement).display);
                   console.log('  - 高度:', window.getComputedStyle(logEditorElement).height);
                   console.log('  - 最大高度:', window.getComputedStyle(logEditorElement).maxHeight);
                   console.log('  - Flex:', window.getComputedStyle(logEditorElement).flex);
                   console.log('  - 位置:', logEditorElement.getBoundingClientRect());
                   console.log('  - 是否有hidden类:', logEditorElement.classList.contains('hidden'));
               }
               
               // 调试主编辑区容器
               const mainEditor = document.querySelector('.flex-1.flex.flex-col.min-h-0');
               if (mainEditor) {
                   console.log('🔍 主编辑区容器信息:');
                   console.log('  - 可见性:', window.getComputedStyle(mainEditor).visibility);
                   console.log('  - 显示:', window.getComputedStyle(mainEditor).display);
                   console.log('  - 高度:', window.getComputedStyle(mainEditor).height);
                   console.log('  - 位置:', mainEditor.getBoundingClientRect());
               }
               
               // 对于超大文件，启用虚拟滚动
               if (entries.length > 50000) {
                   this.enableVirtualScrolling();
               }
           }

           // 启用虚拟滚动（用于超大文件）
           enableVirtualScrolling() {
               console.log('🔄 启用虚拟滚动以优化超大文件性能');
               
               const logLinesContainer = document.getElementById('logLines');
               if (!logLinesContainer) return;
               
               // 不强制设置高度，让容器使用flex布局
               // logLinesContainer.style.height = '600px';
               // logLinesContainer.style.overflowY = 'auto';
               
               // 添加虚拟滚动监听器
               logLinesContainer.addEventListener('scroll', this.throttle(() => {
                   this.handleVirtualScroll();
               }, 16)); // 60fps
           }

           // 处理虚拟滚动
           handleVirtualScroll() {
               const logLinesContainer = document.getElementById('logLines');
               if (!logLinesContainer) return;
               
               const scrollTop = logLinesContainer.scrollTop;
               const containerHeight = logLinesContainer.clientHeight;
               const lineHeight = 20; // 假设每行高度为20px
               
               const startIndex = Math.floor(scrollTop / lineHeight);
               const endIndex = Math.min(startIndex + Math.ceil(containerHeight / lineHeight) + 10, this.totalLines);
               
               // 只渲染可见区域的行
               this.renderVisibleLines(startIndex, endIndex);
           }

           // 渲染可见行
           renderVisibleLines(startIndex, endIndex) {
               const logLinesContainer = document.getElementById('logLines');
               if (!logLinesContainer) return;
               
               // 清空容器
               logLinesContainer.innerHTML = '';
               
               // 渲染可见行
               for (let i = startIndex; i < endIndex; i++) {
                   if (this.logLines[i]) {
                       const lineElement = this.createLogLineElement(this.logLines[i], i);
                       logLinesContainer.appendChild(lineElement);
                   }
               }
           }

           // 节流函数
           throttle(func, limit) {
               let inThrottle;
               return function() {
                   const args = arguments;
                   const context = this;
                   if (!inThrottle) {
                       func.apply(context, args);
                       inThrottle = true;
                       setTimeout(() => inThrottle = false, limit);
                   }
               }
           }


           // 渲染日志行
           renderLogLines() {
               const logLinesContainer = document.getElementById('logLines');
               if (!logLinesContainer) return;

               logLinesContainer.innerHTML = '';
               logLinesContainer.className = 'log-editor';

               this.filteredLines.forEach((entry, index) => {
                   const lineElement = this.createLogLineElement(entry, index);
                   logLinesContainer.appendChild(lineElement);
               });
           }

           // 创建日志行元素（统一背景版）
           createLogLineElement(entry, index) {
               const lineDiv = document.createElement('div');
               
               // 使用统一的样式，不区分级别背景
               lineDiv.className = 'log-line';
               
               lineDiv.dataset.lineNumber = index + 1;
               lineDiv.dataset.originalIndex = this.logLines.indexOf(entry);

               // 行号
               const lineNumber = document.createElement('div');
               lineNumber.className = 'log-line-number';
               lineNumber.textContent = (index + 1).toString().padStart(4, ' ');
               lineDiv.appendChild(lineNumber);

               // 左侧边距区域（移除图标）
               const marginDiv = document.createElement('div');
               marginDiv.className = 'log-line-margin';
               // 不再添加图标
               lineDiv.appendChild(marginDiv);

               // 时间戳（增强时间显示）
               if (entry.timestamp) {
                   const timestamp = document.createElement('span');
                   timestamp.className = 'log-line-timestamp clickable';
                   timestamp.textContent = this.formatTimestamp(entry.timestamp);
                   timestamp.title = '点击查看完整时间: ' + entry.timestamp;
                   timestamp.addEventListener('click', (e) => {
                       e.stopPropagation();
                       this.showTimeDetails(entry.timestamp);
                   });
                   lineDiv.appendChild(timestamp);
               }

               // 线程信息（简化显示）
               if (entry.thread) {
                   const threadInfo = document.createElement('span');
                   threadInfo.className = 'log-line-thread';
                   threadInfo.textContent = this.formatThreadInfo(entry.thread);
                   threadInfo.title = '线程: ' + entry.thread;
                   lineDiv.appendChild(threadInfo);
               }

               // 类名（简化显示）
               if (entry.logger) {
                   const loggerInfo = document.createElement('span');
                   loggerInfo.className = 'log-line-logger';
                   loggerInfo.textContent = this.formatLoggerName(entry.logger);
                   loggerInfo.title = '完整类名: ' + entry.logger;
                   lineDiv.appendChild(loggerInfo);
               }

               // 日志级别
               if (entry.level) {
                   const level = document.createElement('span');
                   level.className = `log-line-level level-${entry.level.toLowerCase()}`;
                   level.textContent = entry.level;
                   lineDiv.appendChild(level);
               }

               // 主要内容区域（给内容更多空间）
               const contentDiv = document.createElement('div');
               contentDiv.className = 'log-line-content-wrapper';
               
               // 应用插件处理后的内容
               const processedContent = this.processLogContent(entry);
               contentDiv.innerHTML = processedContent;
               
               lineDiv.appendChild(contentDiv);

               // 添加点击事件
               lineDiv.addEventListener('click', () => {
                   this.selectLogLine(lineDiv, index);
               });

               return lineDiv;
           }

           // 获取插件图标
           getPluginIcon(pluginType) {
               const icons = {
                   'mybatis': '🗄️',
                   'json': '📄',
                   'error': '🔥',
                   'security': '🔒',
                   'default': '📝'
               };
               return icons[pluginType] || icons.default;
           }
           
           // ========== Spring Boot 日志处理辅助函数 ==========
           
           // 格式化时间戳（支持时间漫游）
           formatTimestamp(timestamp) {
               try {
                   const date = new Date(timestamp);
                   const now = new Date();
                   const diffMs = now - date;
                   
                   // 如果是今天，显示时间
                   if (diffMs < 24 * 60 * 60 * 1000) {
                       return date.toLocaleTimeString('zh-CN', { 
                           hour12: false,
                           hour: '2-digit',
                           minute: '2-digit',
                           second: '2-digit'
                       });
                   }
                   
                   // 否则显示日期和时间
                   return date.toLocaleString('zh-CN', {
                       month: '2-digit',
                       day: '2-digit',
                       hour: '2-digit',
                       minute: '2-digit',
                       second: '2-digit',
                       hour12: false
                   });
               } catch (error) {
                   return timestamp; // 如果解析失败，返回原文
               }
           }
           
           // 格式化线程信息（简化显示）
           formatThreadInfo(thread) {
               if (!thread) return '';
               
               // 提取线程名称，忽略备注信息
               const threadName = thread.replace(/\[([^\]]+)\]/, '$1');
               
               // 简化常见的线程名
               const simplified = threadName
                   .replace(/^http-nio-\d+-exec-/, 'http-')
                   .replace(/^scheduling-/, 'sched-')
                   .replace(/^quartzScheduler_Worker-/, 'qz-')
                   .replace(/^pool-\d+-thread-/, 'pool-');
                   
               return simplified.length > 15 ? simplified.substring(0, 12) + '...' : simplified;
           }
           
           // 格式化类名（只显示最后一部分）
           formatLoggerName(logger) {
               if (!logger) return '';
               
               const parts = logger.split('.');
               if (parts.length <= 2) return logger;
               
               // 只显示最后两部分
               return parts.slice(-2).join('.');
           }
           
           // 获取日志级别对应的样式类（统一背景版）
           getLogLevelClass(level) {
               // 所有级别都使用统一的基础样式，不区分背景色
               return 'log-line-unified';
           }
           
           // 显示时间详情（用于时间漫游）
           showTimeDetails(timestamp) {
               const date = new Date(timestamp);
               const formatOptions = {
                   year: 'numeric',
                   month: '2-digit',
                   day: '2-digit',
                   hour: '2-digit',
                   minute: '2-digit',
                   second: '2-digit',
                   millisecond: '3-digit',
                   hour12: false
               };
               
               const formattedTime = date.toLocaleString('zh-CN', formatOptions);
               const isoTime = date.toISOString();
               const timestamp_ms = date.getTime();
               
               // 显示时间详情对话框
               const details = `
                   时间详情:
                   本地时间: ${formattedTime}
                   ISO 时间: ${isoTime}
                   时间戳: ${timestamp_ms}
               `;
               
               alert(details.trim());
           }
           
           // 处理日志内容（集成多个插件处理结果）
           processLogContent(entry) {
               let content = entry.content || '';
               
               // 调试：输出Docker JSON格式标志状态
               if (entry.line_number <= 3) {
                   console.log(`🔍 处理第 ${entry.line_number} 行，isDockerJsonFormat=${this.isDockerJsonFormat}, content=${content.substring(0, 50)}...`);
               }
               
               // 1. 先进行 HTML 转义
               content = this.escapeHtml(content);
               
               // 2. 检测并处理 JSON 内容
               content = this.processJsonContent(content);
               
               // 3. 检测并处理 MyBatis SQL
               content = this.processMybatisContent(content);
               
               // 4. 检测并处理异常信息（仅对非Docker JSON格式应用）
               // 如果当前使用的是Docker JSON格式，跳过异常处理，避免误识别
               if (!this.isDockerJsonFormat) {
                   // 检查是否为聚合后的异常内容（包含多行）
                   if (this.isAggregatedException(content)) {
                       console.log('🔍 检测到聚合后的异常内容，进行特殊处理');
                       content = this.processAggregatedException(content);
                   } else {
                       content = this.processErrorContent(content);
                   }
               } else {
                   if (entry.line_number <= 3) {
                       console.log(`🔍 跳过异常处理（Docker JSON格式）`);
                   }
               }
               
               // 5. 通用关键词高亮
               content = this.highlightKeywords(content);
               
               return content;
           }
           
           // 处理 JSON 内容
           processJsonContent(content) {
               // 检测是否包含 JSON
               const jsonRegex = /\{[^{}]*\}/g;
               return content.replace(jsonRegex, (match) => {
                   try {
                       const parsed = JSON.parse(match);
                       const formatted = JSON.stringify(parsed, null, 2);
                       return `<span class="json-content clickable" data-json="${this.escapeHtml(formatted)}" title="点击查看 JSON 详情">${match}</span>`;
                   } catch (e) {
                       return `<span class="invalid-json" title="无效的 JSON 格式">${match}</span>`;
                   }
               });
           }
           
           // 处理 MyBatis SQL
           processMybatisContent(content) {
               console.log('🔧 [MyBatis处理] 输入内容:', content);
               
               // 检测是否是合并后的MyBatis SQL（包含实际参数值，没有?占位符）
               if (content.includes('Preparing:') && content.includes('SELECT') && content.includes('FROM') && !content.includes('?')) {
                   console.log('🔧 [MyBatis处理] 检测到合并后的SQL，进行语法高亮');
                   // 这是合并后的完整SQL，进行SQL语法高亮
                   content = this.highlightSqlSyntax(content);
               } else if (content.includes('Preparing:') || content.includes('Parameters:') || content.includes('Total:')) {
                   console.log('🔧 [MyBatis处理] 检测到原始MyBatis格式，进行传统处理');
                   // 这是原始的MyBatis日志格式，进行传统处理
                   // SQL 语句高亮
                   content = content.replace(
                       /(Preparing:)\s*(.+)/g,
                       '$1 <span class="sql-statement" title="SQL 语句">$2</span>'
                   );
                   
                   // 参数高亮
                   content = content.replace(
                       /(Parameters:)\s*(.+)/g,
                       '$1 <span class="sql-parameters" title="SQL 参数">$2</span>'
                   );
                   
                   // 执行时间高亮
                   content = content.replace(
                       /(Total:)\s*(\d+)/g,
                       '$1 <span class="sql-time" title="执行时间">$2</span>'
                   );
               }
               
               console.log('🔧 [MyBatis处理] 输出内容:', content);
               return content;
           }
           
           // SQL语法高亮
           highlightSqlSyntax(content) {
               console.log('🔧 [SQL高亮] 输入内容:', content);
               
               // 先清理可能存在的错误标签和HTML片段
               let result = content
                   .replace(/"sql-keyword">/g, '')
                   .replace(/'sql-string'>/g, '')
                   .replace(/<span[^>]*>/g, '')
                   .replace(/<\/span>/g, '');
               
               console.log('🔧 [SQL高亮] 清理后内容:', result);
               
               // 按优先级处理，先处理长关键字，再处理短关键字
               const keywordMap = [
                   { pattern: /\bORDER BY\b/gi, replacement: '<span class="sql-keyword">ORDER BY</span>' },
                   { pattern: /\bGROUP BY\b/gi, replacement: '<span class="sql-keyword">GROUP BY</span>' },
                   { pattern: /\bINNER JOIN\b/gi, replacement: '<span class="sql-keyword">INNER JOIN</span>' },
                   { pattern: /\bLEFT JOIN\b/gi, replacement: '<span class="sql-keyword">LEFT JOIN</span>' },
                   { pattern: /\bRIGHT JOIN\b/gi, replacement: '<span class="sql-keyword">RIGHT JOIN</span>' },
                   { pattern: /\bOUTER JOIN\b/gi, replacement: '<span class="sql-keyword">OUTER JOIN</span>' },
                   { pattern: /\bSELECT\b/gi, replacement: '<span class="sql-keyword">SELECT</span>' },
                   { pattern: /\bINSERT\b/gi, replacement: '<span class="sql-keyword">INSERT</span>' },
                   { pattern: /\bUPDATE\b/gi, replacement: '<span class="sql-keyword">UPDATE</span>' },
                   { pattern: /\bDELETE\b/gi, replacement: '<span class="sql-keyword">DELETE</span>' },
                   { pattern: /\bFROM\b/gi, replacement: '<span class="sql-keyword">FROM</span>' },
                   { pattern: /\bWHERE\b/gi, replacement: '<span class="sql-keyword">WHERE</span>' },
                   { pattern: /\bHAVING\b/gi, replacement: '<span class="sql-keyword">HAVING</span>' },
                   { pattern: /\bAND\b/gi, replacement: '<span class="sql-keyword">AND</span>' },
                   { pattern: /\bOR\b/gi, replacement: '<span class="sql-keyword">OR</span>' },
                   { pattern: /\bNOT\b/gi, replacement: '<span class="sql-keyword">NOT</span>' },
                   { pattern: /\bIN\b/gi, replacement: '<span class="sql-keyword">IN</span>' },
                   { pattern: /\bIS\b/gi, replacement: '<span class="sql-keyword">IS</span>' },
                   { pattern: /\bNULL\b/gi, replacement: '<span class="sql-keyword">NULL</span>' },
                   { pattern: /\bJOIN\b/gi, replacement: '<span class="sql-keyword">JOIN</span>' },
                   { pattern: /\bON\b/gi, replacement: '<span class="sql-keyword">ON</span>' },
                   { pattern: /\bAS\b/gi, replacement: '<span class="sql-keyword">AS</span>' }
               ];
               
               // 应用关键字高亮
               keywordMap.forEach(({ pattern, replacement }) => {
                   result = result.replace(pattern, replacement);
               });
               
               // 高亮字符串（单引号和双引号）
               result = result.replace(/'([^']*)'/g, '<span class="sql-string">\'$1\'</span>');
               result = result.replace(/"([^"]*)"/g, '<span class="sql-string">"$1"</span>');
               
               // 高亮数字（避免在字符串中高亮）
               result = result.replace(/\b(\d+)\b/g, '<span class="sql-number">$1</span>');
               
               console.log('🔧 [SQL高亮] 最终结果:', result);
               
               return result;
           }
           
           // 检测是否为聚合后的异常内容
           isAggregatedException(content) {
               const lines = content.split('\n');
               if (lines.length <= 1) return false;
               
               // 检查是否包含异常特征
               const hasException = lines.some(line => 
                   line.includes('Exception:') || 
                   line.includes('Error:') ||
                   line.includes('Caused by:') ||
                   line.includes('Suppressed:')
               );
               
               // 检查是否包含堆栈跟踪
               const hasStackTrace = lines.some(line => 
                   line.trim().startsWith('at ') ||
                   line.trim().startsWith('\tat ')
               );
               
               return hasException && hasStackTrace;
           }
           
           // 处理聚合后的异常内容
           processAggregatedException(content) {
               console.log('🔍 处理聚合后的异常内容，行数:', content.split('\n').length);
               
               const lines = content.split('\n');
               const processedLines = lines.map((line, index) => {
                   let processedLine = line;
                   
                   // 异常类名高亮
                   const exceptionRegex = /(\w+Exception|\w+Error)(:.*)?/g;
                   processedLine = processedLine.replace(exceptionRegex, '<span class="exception-name" title="异常类型">$1</span>$2');
                   
                   // 堆栈跟踪高亮
                   if (processedLine.trim().startsWith('at ')) {
                       processedLine = processedLine.replace(
                           /at\s+([\w.$]+)\(([^)]+)\)/g,
                           'at <span class="stack-trace" title="堆栈跟踪">$1</span>(<span class="stack-location">$2</span>)'
                       );
                   }
                   
                   // Caused by 高亮
                   if (processedLine.trim().startsWith('Caused by:')) {
                       processedLine = processedLine.replace(
                           /(Caused by:)\s*(.*)/g,
                           '<span class="exception-caused-by" title="异常原因">$1</span> $2'
                       );
                   }
                   
                   // Suppressed 高亮
                   if (processedLine.trim().startsWith('Suppressed:')) {
                       processedLine = processedLine.replace(
                           /(Suppressed:)\s*(.*)/g,
                           '<span class="exception-suppressed" title="被抑制的异常">$1</span> $2'
                       );
                   }
                   
                   // 添加缩进（除了第一行）
                   if (index > 0) {
                       processedLine = '&nbsp;&nbsp;&nbsp;&nbsp;' + processedLine;
                   }
                   
                   return processedLine;
               });
               
               // 创建可折叠的异常显示
               const firstLine = processedLines[0];
               const stackTrace = processedLines.slice(1);
               
               if (stackTrace.length > 0) {
                   const stackTraceHtml = stackTrace.map(line => 
                       `<div class="stack-trace-line">${line}</div>`
                   ).join('');
                   
                   return `
                       <div class="exception-block">
                           <div class="exception-header" onclick="toggleException(this)">
                               <span class="exception-toggle">▼</span>
                               <span class="exception-summary">${firstLine}</span>
                           </div>
                           <div class="exception-details" style="display: none;">
                               ${stackTraceHtml}
                           </div>
                       </div>
                   `;
               }
               
               return processedLines.join('<br>');
           }
           
           // 处理异常内容（增强版，支持聚合的异常信息）
           processErrorContent(content) {
               // Docker JSON检测：检查是否包含Docker JSON解析后的特征
               const trimmedContent = content.trim();
               
               // 检测1：原始Docker JSON格式（主要检测）
               if (trimmedContent.startsWith('{"log":') && trimmedContent.includes('"stream":') && trimmedContent.includes('"time":')) {
                   console.log('🔍 检测到原始Docker JSON内容，跳过异常处理:', content.substring(0, 100));
                   return content;
               }
               
               // 检测2：更宽松的Docker JSON检测
               if (trimmedContent.startsWith('{') && trimmedContent.includes('"log":')) {
                   console.log('🔍 检测到Docker JSON格式，跳过异常处理:', content.substring(0, 100));
                   return content;
               }
               
               // 检测是否包含多行异常信息（聚合后的异常）
               const lines = content.split('\n');
               
               // 更严格的异常检测：必须包含异常特征
               const hasExceptionFeatures = lines.some(line => 
                   line.includes('Exception:') || 
                   line.includes('Error:') ||
                   line.includes('Caused by:') ||
                   line.includes('Suppressed:') ||
                   line.includes('at ') ||
                   line.includes('\tat ')
               );
               
               if (lines.length > 1 && hasExceptionFeatures) {
                   console.log('🔍 检测到多行异常内容，行数:', lines.length);
                   
                   // 多行异常信息，需要特殊处理
                   const processedLines = lines.map((line, index) => {
                       let processedLine = line;
                       
                       // 异常类名高亮
                       const exceptionRegex = /(\w+Exception|\w+Error)(:.*)?/g;
                       processedLine = processedLine.replace(exceptionRegex, '<span class="exception-name" title="异常类型">$1</span>$2');
                       
                       // 堆栈跟踪高亮
                       if (processedLine.trim().startsWith('at ')) {
                           processedLine = processedLine.replace(
                               /at\s+([\w.$]+)\(([^)]+)\)/g,
                               'at <span class="stack-trace" title="堆栈跟踪">$1</span>(<span class="stack-location">$2</span>)'
                           );
                       }
                       
                       // Caused by 高亮
                       if (processedLine.trim().startsWith('Caused by:')) {
                           processedLine = processedLine.replace(
                               /(Caused by:)\s*(.*)/g,
                               '<span class="exception-caused-by" title="异常原因">$1</span> $2'
                           );
                       }
                       
                       // Suppressed 高亮
                       if (processedLine.trim().startsWith('Suppressed:')) {
                           processedLine = processedLine.replace(
                               /(Suppressed:)\s*(.*)/g,
                               '<span class="exception-suppressed" title="被抑制的异常">$1</span> $2'
                           );
                       }
                       
                       // 添加缩进（除了第一行）
                       if (index > 0) {
                           processedLine = '&nbsp;&nbsp;&nbsp;&nbsp;' + processedLine;
                       }
                       
                       return processedLine;
                   });
                   
                   // 创建可折叠的异常显示
                   const firstLine = processedLines[0];
                   const stackTrace = processedLines.slice(1);
                   
                   if (stackTrace.length > 0) {
                       const stackTraceHtml = stackTrace.map(line => 
                           `<div class="stack-trace-line">${line}</div>`
                       ).join('');
                       
                       return `
                           <div class="exception-block">
                               <div class="exception-header" onclick="toggleException(this)">
                                   <span class="exception-toggle">▼</span>
                                   <span class="exception-summary">${firstLine}</span>
                               </div>
                               <div class="exception-details" style="display: none;">
                                   ${stackTraceHtml}
                               </div>
                           </div>
                       `;
                   }
                   
                   return processedLines.join('<br>');
               } else {
                   // 单行异常处理（原有逻辑）
                   const exceptionRegex = /(\w+Exception|\w+Error)(:.*)?/g;
                   content = content.replace(exceptionRegex, '<span class="exception-name" title="异常类型">$1</span>$2');
                   
                   // 检测堆栈跟踪
                   if (content.includes('at ') && content.includes('(')) {
                       content = content.replace(
                           /at\s+([\w.$]+)\(([^)]+)\)/g,
                           'at <span class="stack-trace" title="堆栈跟踪">$1</span>(<span class="stack-location">$2</span>)'
                       );
                   }
                   
                   return content;
               }
           }
           
           // 通用关键词高亮
           highlightKeywords(content) {
               const keywords = {
                   'ERROR': 'keyword-error',
                   'WARN': 'keyword-warn',
                   'SUCCESS': 'keyword-success',
                   'FAILED': 'keyword-error',
                   'TIMEOUT': 'keyword-warn',
                   'NULL': 'keyword-null'
               };
               
               Object.entries(keywords).forEach(([keyword, className]) => {
                   const regex = new RegExp(`\\b${keyword}\\b`, 'gi');
                   content = content.replace(regex, `<span class="${className}">${keyword}</span>`);
               });
               
               return content;
           }
           
           // 显示 JSON 模态框
           showJsonModal(jsonData) {
               // 创建模态框
               const modal = document.createElement('div');
               modal.className = 'fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50';
               modal.innerHTML = `
                   <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] flex flex-col">
                       <div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
                           <h3 class="text-xl font-semibold text-gray-900 dark:text-white">📄 JSON 数据查看器</h3>
                           <button class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 text-2xl json-modal-close">×</button>
                       </div>
                       <div class="flex-1 p-6 overflow-y-auto">
                           <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                               <pre class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap font-mono">${jsonData}</pre>
                           </div>
                       </div>
                       <div class="flex items-center justify-end space-x-3 p-6 border-t border-gray-200 dark:border-gray-700">
                           <button class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded-md transition-colors json-copy-btn">
                               📋 复制
                           </button>
                           <button class="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors json-modal-close">
                               关闭
                           </button>
                       </div>
                   </div>
               `;
               
               // 添加事件监听
               modal.addEventListener('click', (e) => {
                   if (e.target === modal || e.target.classList.contains('json-modal-close')) {
                       document.body.removeChild(modal);
                   }
                   
                   if (e.target.classList.contains('json-copy-btn')) {
                       navigator.clipboard.writeText(jsonData).then(() => {
                           e.target.textContent = '✅ 已复制';
                           setTimeout(() => {
                               e.target.textContent = '📋 复制';
                           }, 2000);
                       });
                   }
               });
               
               document.body.appendChild(modal);
           }

           // 应用语法高亮
           applySyntaxHighlighting(content, pluginType) {
               let highlighted = this.escapeHtml(content);
               
               // 根据插件类型应用不同的高亮
               switch (pluginType) {
                   case 'mybatis':
                       highlighted = this.highlightSql(highlighted);
                       break;
                   case 'json':
                       highlighted = this.highlightJson(highlighted);
                       break;
                   case 'error':
                       highlighted = this.highlightError(highlighted);
                       break;
                   default:
                       highlighted = this.highlightGeneral(highlighted);
               }

               return highlighted;
           }

           // SQL 语法高亮
           highlightSql(content) {
               const sqlKeywords = ['SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'DROP', 'ALTER', 'JOIN', 'INNER', 'LEFT', 'RIGHT', 'OUTER'];
               let highlighted = content;
               
               sqlKeywords.forEach(keyword => {
                   const regex = new RegExp(`\\b${keyword}\\b`, 'gi');
                   highlighted = highlighted.replace(regex, `<span class="log-content-keyword">${keyword}</span>`);
               });
               
               return highlighted;
           }

           // JSON 语法高亮
           highlightJson(content) {
               let highlighted = content;
               
               // 高亮 JSON 键
               highlighted = highlighted.replace(/"([^"]+)":/g, '<span class="log-content-json">"$1":</span>');
               
               // 高亮字符串值
               highlighted = highlighted.replace(/: "([^"]+)"/g, ': <span class="log-content-json">"$1"</span>');
               
               return highlighted;
           }

           // 错误语法高亮
           highlightError(content) {
               let highlighted = content;
               
               // 高亮异常类名
               const exceptionRegex = /(\w+Exception|\w+Error)/g;
               highlighted = highlighted.replace(exceptionRegex, '<span class="log-content-error">$1</span>');
               
               return highlighted;
           }

           // 通用语法高亮
           highlightGeneral(content) {
               return content;
           }

           // 选择日志行
           selectLogLine(lineElement, index) {
               // 移除之前的选择
               document.querySelectorAll('.log-line.selected').forEach(el => {
                   el.classList.remove('selected');
               });
               
               // 添加选择状态
               lineElement.classList.add('selected');
               this.currentLine = index + 1;
               
               // 更新状态栏
               this.updateStatusBar();
               
               // 滚动到选中行
               lineElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
           }

           // 更新侧边栏导航
           updateSidebarNavigation() {
               const sidebarContent = document.getElementById('pluginCategories');
               if (!sidebarContent) return;

               console.log('📊 更新侧边栏导航，日志条目数:', this.logLines.length);

               // 按日志级别分组
               this.pluginCategories = {};
               this.logLines.forEach((entry, index) => {
                   // 修复：使用统一的级别格式，与过滤逻辑保持一致
                   const level = entry.level || 'INFO'; // 默认为INFO，与过滤按钮一致
                   if (!this.pluginCategories[level]) {
                       this.pluginCategories[level] = [];
                   }
                   this.pluginCategories[level].push({ entry, index });
               });

               // 输出级别统计调试信息
               console.log('📊 日志级别统计:');
               Object.entries(this.pluginCategories).forEach(([level, items]) => {
                   console.log(`  ${level}: ${items.length} 条`);
               });

               // 渲染侧边栏
               sidebarContent.innerHTML = '';
               
               // 添加日志级别统计和过滤区域
               const levelFilterDiv = document.createElement('div');
               levelFilterDiv.className = 'mb-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800';
               levelFilterDiv.innerHTML = `
                   <h4 class="text-sm font-medium text-blue-700 dark:text-blue-300 mb-3 flex items-center">
                       <span class="mr-2">🏷️</span>
                       日志级别过滤
                   </h4>
                   <div class="space-y-2">
                       ${Object.entries(this.pluginCategories).map(([level, items]) => {
                           // 根据级别设置不同的配色（保持与标签一致）
                           const levelColors = {
                               'ERROR': {
                                   bg: 'hover:bg-red-50 dark:hover:bg-red-900/20',
                                   active: 'bg-red-50 dark:bg-red-900/30 ring-2 ring-red-300 dark:ring-red-600',
                                   badge: 'bg-red-500 text-white border-red-600',
                                   icon: '🔴'
                               },
                               'WARN': {
                                   bg: 'hover:bg-yellow-50 dark:hover:bg-yellow-900/20',
                                   active: 'bg-yellow-50 dark:bg-yellow-900/30 ring-2 ring-yellow-300 dark:ring-yellow-600',
                                   badge: 'bg-yellow-500 text-white border-yellow-600',
                                   icon: '🟡'
                               },
                               'INFO': {
                                   bg: 'hover:bg-blue-50 dark:hover:bg-blue-900/20',
                                   active: 'bg-blue-50 dark:bg-blue-900/30 ring-2 ring-blue-300 dark:ring-blue-600',
                                   badge: 'bg-blue-500 text-white border-blue-600',
                                   icon: '🔵'
                               },
                               'DEBUG': {
                                   bg: 'hover:bg-green-50 dark:hover:bg-green-900/20',
                                   active: 'bg-green-50 dark:bg-green-900/30 ring-2 ring-green-300 dark:ring-green-600',
                                   badge: 'bg-green-500 text-white border-green-600',
                                   icon: '🟢'
                               },
                               'TRACE': {
                                   bg: 'hover:bg-gray-50 dark:hover:bg-gray-800/30',
                                   active: 'bg-gray-50 dark:bg-gray-800/50 ring-2 ring-gray-300 dark:ring-gray-600',
                                   badge: 'bg-gray-500 text-white border-gray-600',
                                   icon: '⚪'
                               }
                           };
                           
                           const colors = levelColors[level] || levelColors['INFO'];
                           const isActive = this.currentFilter === level;
                           
                           return `
                               <div class="flex items-center justify-between p-3 rounded-lg ${colors.bg} cursor-pointer transition-all duration-200 ${isActive ? colors.active : ''}" onclick="app.filterByLevel('${level}')">
                                   <div class="flex items-center space-x-3">
                                       <span class="text-xl">${colors.icon}</span>
                                       <span class="text-sm font-medium ${this.getLevelTextColor(level)}">${level}</span>
                                   </div>
                                   <span class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold ${colors.badge} shadow-sm">
                                       ${items.length}
                                   </span>
                               </div>
                           `;
                       }).join('')}
                       <div class="mt-3 pt-2 border-t border-blue-200 dark:border-blue-700">
                           <div class="flex items-center justify-between p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-all duration-200 ${this.currentFilter === 'all' ? 'bg-gray-100 dark:bg-gray-700 ring-2 ring-gray-300 dark:ring-gray-600' : ''}" onclick="app.filterByLevel('all')">
                               <div class="flex items-center space-x-2">
                                   <span class="text-lg">📜</span>
                                   <span class="text-sm font-medium text-gray-700 dark:text-gray-300">显示全部</span>
                               </div>
                               <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200">
                                   ${this.logLines.length}
                               </span>
                           </div>
                       </div>
                   </div>
               `;
               sidebarContent.appendChild(levelFilterDiv);
               
               // 添加快速导航（仅滚动功能）
               const quickNavDiv = document.createElement('div');
               quickNavDiv.className = 'mb-4 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700';
               quickNavDiv.innerHTML = `
                   <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 flex items-center">
                       <span class="mr-2">🚀</span>
                       快速导航
                   </h4>
                   <div class="space-y-1">
                       <button class="w-full text-left px-2 py-1 text-xs rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" onclick="app.scrollToTop()">
                           🔝 跳转到顶部
                       </button>
                       <button class="w-full text-left px-2 py-1 text-xs rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" onclick="app.scrollToBottom()">
                           🔟 跳转到底部
                       </button>
                   </div>
               `;
               sidebarContent.appendChild(quickNavDiv);

           }

           // 获取日志级别图标
           getLevelIcon(level) {
               const icons = {
                   'ERROR': '🔴',
                   'WARN': '🟡', 
                   'INFO': '🔵',
                   'DEBUG': '🟢',
                   'TRACE': '⚪'
               };
               return icons[level] || '📄';
           }
           
           // 获取日志级别文本颜色（按照规范配色）
           getLevelTextColor(level) {
               if (!level) return 'text-gray-700 dark:text-gray-300';
               
               const normalizedLevel = level.toUpperCase();
               const colorMap = {
                   'ERROR': 'text-red-600 dark:text-red-400 font-semibold',
                   'WARN': 'text-yellow-600 dark:text-yellow-400 font-semibold',
                   'INFO': 'text-blue-600 dark:text-blue-400 font-semibold',
                   'DEBUG': 'text-green-600 dark:text-green-400 font-semibold',
                   'TRACE': 'text-gray-600 dark:text-gray-400 font-semibold'
               };
               
               return colorMap[normalizedLevel] || 'text-gray-700 dark:text-gray-300';
           }

           // 按级别过滤
           filterByLevel(level) {
               console.log('🔍 按级别过滤:', level);
               console.log('📊 当前日志总数:', this.logLines.length);
               
               // 输出前几条日志的level信息进行调试
               console.log('📋 前5条日志的级别信息:');
               this.logLines.slice(0, 5).forEach((entry, index) => {
                   console.log(`  ${index + 1}: level="${entry.level}", content="${entry.content?.substring(0, 50)}..."`);
               });
               
               this.currentFilter = level;
               
               if (level === 'all') {
                   this.filteredLines = [...this.logLines];
               } else {
                   this.filteredLines = this.logLines.filter(entry => {
                       const entryLevel = entry.level;
                       const filterLevel = level.toUpperCase();
                       
                       // 详细的匹配调试
                       const matches = entryLevel && entryLevel.toUpperCase() === filterLevel;
                       
                       if (filterLevel === 'ERROR' && entryLevel) {
                           console.log(`🔍 ERROR匹配检查: "${entryLevel}" vs "${filterLevel}" = ${matches}`);
                       }
                       
                       return matches;
                   });
               }
               
               console.log(`✅ 过滤完成: ${level} 级别共 ${this.filteredLines.length} 条日志`);
               
               this.renderLogLines();
               this.updateStatusBar();
           }

           // 滚动到顶部
           scrollToTop() {
               const logLinesContainer = document.getElementById('logLines');
               if (logLinesContainer) {
                   logLinesContainer.scrollTop = 0;
               }
           }

           // 滚动到底部
           scrollToBottom() {
               const logLinesContainer = document.getElementById('logLines');
               if (logLinesContainer) {
                   logLinesContainer.scrollTop = logLinesContainer.scrollHeight;
               }
           }

           // 获取插件显示名称
           getPluginDisplayName(pluginType) {
               const names = {
                   'mybatis': 'MyBatis',
                   'json': 'JSON修复',
                   'error': '异常',
                   'security': '敏感信息',
                   'default': '其他'
               };
               return names[pluginType] || pluginType;
           }

           // 按插件过滤
           filterByPlugin(pluginType) {
               this.currentFilter = pluginType;
               this.filteredLines = this.logLines.filter(entry => {
                   if (pluginType === 'all') return true;
                   return entry.plugin_type === pluginType;
               });
               
               this.renderLogLines();
           }

           // 简化状态更新：已移除过滤按钮，不需要更新按钮状态

           // 更新状态栏
           updateStatusBar() {
               const statusLine = document.getElementById('statusLine');
               const statusColumn = document.getElementById('statusColumn');
               const statusPlugins = document.getElementById('statusPlugins');
               const statusSearch = document.getElementById('statusSearch');
               const statusFile = document.getElementById('statusFile');

               if (statusLine) {
                   statusLine.textContent = `行 ${this.currentLine}/${this.totalLines}`;
               }
               
               if (statusColumn) {
                   statusColumn.textContent = `列 0`;
               }
               
               if (statusPlugins) {
                   const activePlugins = Object.keys(this.pluginCategories).join(', ');
                   statusPlugins.textContent = `插件：${activePlugins || '无'}`;
               }
               
               if (statusSearch) {
                   statusSearch.textContent = `搜索：${this.searchResults.length} 处匹配`;
               }
               
               if (statusFile) {
                   statusFile.textContent = `文件：${this.currentFile ? this.currentFile.name : '无'}`;
               }
           }

           // 搜索功能
           performSearch(searchTerm) {
               this.searchTerm = searchTerm;
               this.searchResults = [];
               
               if (!searchTerm.trim()) {
                   this.clearSearchHighlights();
                   return;
               }
               
               // 在日志行中搜索
               this.logLines.forEach((entry, index) => {
                   if (entry.content.toLowerCase().includes(searchTerm.toLowerCase())) {
                       this.searchResults.push({ entry, index });
                   }
               });
               
               // 高亮搜索结果
               this.highlightSearchResults();
               
               // 更新侧边栏搜索结果
               this.updateSearchResults();
               
               // 更新状态栏
               this.updateStatusBar();
           }

           // 高亮搜索结果
           highlightSearchResults() {
               document.querySelectorAll('.log-line').forEach(line => {
                   const content = line.querySelector('.log-line-content');
                   if (content && this.searchTerm) {
                       const text = content.textContent;
                       const highlighted = text.replace(
                           new RegExp(this.searchTerm, 'gi'),
                           `<span class="search-highlight">$&</span>`
                       );
                       content.innerHTML = highlighted;
                   }
               });
           }

           // 清除搜索高亮
           clearSearchHighlights() {
               document.querySelectorAll('.search-highlight').forEach(highlight => {
                   const parent = highlight.parentNode;
                   parent.replaceChild(document.createTextNode(highlight.textContent), highlight);
                   parent.normalize();
               });
           }

           // 更新搜索结果侧边栏
           updateSearchResults() {
               const searchResultsDiv = document.getElementById('searchResults');
               const searchResultsList = document.getElementById('searchResultsList');
               
               if (!searchResultsDiv || !searchResultsList) return;
               
               if (this.searchResults.length > 0) {
                   searchResultsDiv.style.display = 'block';
                   searchResultsList.innerHTML = '';
                   
                   this.searchResults.forEach((result, index) => {
                       const item = document.createElement('div');
                       item.className = 'sidebar-nav-item';
                       item.innerHTML = `
                           <span class="sidebar-nav-icon">🔍</span>
                           <span>${result.index + 1}: ${result.entry.content.substring(0, 50)}...</span>
                       `;
                       
                       item.addEventListener('click', () => {
                           this.scrollToLine(result.index);
                       });
                       
                       searchResultsList.appendChild(item);
                   });
               } else {
                   searchResultsDiv.style.display = 'none';
               }
           }

           // 滚动到指定行
           scrollToLine(lineIndex) {
               const lineElement = document.querySelector(`[data-original-index="${lineIndex}"]`);
               if (lineElement) {
                   lineElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
                   this.selectLogLine(lineElement, lineIndex);
               }
           }

           // 侧边栏折叠/展开
           toggleSidebar() {
               const sidebar = document.getElementById('sidebar');
               const toggleBtn = document.getElementById('sidebarToggle');
               
               if (!sidebar || !toggleBtn) {
                   console.warn('⚠️ 找不到侧边栏或折叠按钮');
                   return;
               }
               
               this.sidebarCollapsed = !this.sidebarCollapsed;
               
               console.log('🔄 切换侧边栏状态:', this.sidebarCollapsed ? '折叠' : '展开');
               
               if (this.sidebarCollapsed) {
                   sidebar.classList.add('sidebar-collapsed');
                   toggleBtn.innerHTML = '<span class="text-sm">▶</span>';
                   console.log('✅ 侧边栏已折叠');
               } else {
                   sidebar.classList.remove('sidebar-collapsed');
                   toggleBtn.innerHTML = '<span class="text-sm">◀</span>';
                   console.log('✅ 侧边栏已展开');
               }
           }
           
           // 切换主题
           async toggleTheme() {
               try {
                   const newMode = this.currentTheme === 'light' ? 'dark' : 'light';
                   console.log('🔄 开始切换主题:', this.currentTheme, '->', newMode);
                   
                   const response = await fetch(`${this.API_BASE_URL}/api/config/theme`, {
                       method: 'POST',
                       headers: { 'Content-Type': 'application/json' },
                       body: JSON.stringify({ mode: newMode })
                   });
                   
                   if (response.ok) {
                       const result = await response.json();
                       if (result.success) {
                           // 重新加载主题配置
                           await this.loadConfigs();
                           // 更新主题切换按钮图标
                           this.updateThemeToggleIcon();
                           console.log('✅ 主题切换成功:', newMode);
                       }
                   }
               } catch (error) {
                   console.error('❌ 主题切换失败:', error);
               }
           }
           
           // 更新主题配置
           async updateThemeConfig(updates) {
               try {
                   const response = await fetch(`${this.API_BASE_URL}/api/config/theme`, {
                       method: 'POST',
                       headers: { 'Content-Type': 'application/json' },
                       body: JSON.stringify(updates)
                   });
                   
                   if (response.ok) {
                       const result = await response.json();
                       if (result.success) {
                           // 重新加载主题配置
                           await this.loadConfigs();
                           console.log('✅ 主题配置更新成功');
                       }
                   }
               } catch (error) {
                   console.error('❌ 主题配置更新失败:', error);
               }
           }
    
    
    async initElectron() {
        console.log('🔎 检测 Electron 环境...');
        
        if (window.electronAPI) {
            this.isElectronEnv = true;
            console.log('✅ Electron 环境检测成功！');
            
            // 设置窗口控制
            this.setupWindowControls();
            
            // 获取 API 配置
            try {
                const config = await window.electronAPI.getApiConfig();
                if (config && config.port) {
                    this.API_BASE_URL = `http://127.0.0.1:${config.port}`;
                    console.log('🌐 API 配置:', this.API_BASE_URL);
                }
            } catch (error) {
                console.warn('⚠️ 获取 API 配置失败:', error);
            }
        } else {
            console.log('🌐 运行在浏览器环境中');
        }
    }
    
    setupWindowControls() {
        // Electron 使用原生窗口控制，不需要自定义按钮
        if (!this.isElectronEnv) return;
        console.log('✅ 使用 Electron 原生窗口控制');
    }
    
    setupEventListeners() {
        // 文件输入
        const fileInput = document.getElementById('fileInput');
        const fileDropZone = document.getElementById('fileDropZone');
        // parseBtn 已移除
        const clearBtn = document.getElementById('clearBtn');
        
        // 主题切换
        const themeToggle = document.getElementById('themeToggle');
        const themeSelect = document.getElementById('themeSelect');
        
        // 插件管理
        const pluginManager = document.getElementById('pluginManager');
        const pluginModal = document.getElementById('pluginModal');
        const pluginModalClose = document.getElementById('pluginModalClose');
        
        // 设置
        const settingsBtn = document.getElementById('settingsBtn');
        const settingsModal = document.getElementById('settingsModal');
        const settingsModalClose = document.getElementById('settingsModalClose');
        
        // 搜索
        const searchBtn = document.getElementById('searchBtn');
        const searchOverlay = document.getElementById('searchOverlay');
        const searchClose = document.getElementById('searchClose');
        
        // 导出
        const exportBtn = document.getElementById('exportBtn');
        
        // 文件选择
        if (fileInput) {
            fileInput.addEventListener('change', (e) => this.handleFileSelect(e));
        }

        // 上传按钮
        const uploadBtn = document.getElementById('uploadBtn');
        if (uploadBtn) {
            uploadBtn.addEventListener('click', () => {
                if (fileInput) {
                    fileInput.click();
                }
            });
        }

        // 搜索功能
        const searchInput = document.getElementById('searchInput');
        if (searchInput) {
            searchInput.addEventListener('input', (e) => {
                this.performSearch(e.target.value);
            });
        }

        // 注意：过滤器按钮已移除，现在使用左侧边栏进行过滤

        // 侧边栏折叠
        const sidebarToggle = document.getElementById('sidebarToggle');
        const collapsedHint = document.getElementById('collapsedHint');
        if (sidebarToggle) {
            sidebarToggle.addEventListener('click', () => {
                this.toggleSidebar();
            });
        }
        
        // 折叠提示区域点击展开
        if (collapsedHint) {
            collapsedHint.addEventListener('click', () => {
                if (this.sidebarCollapsed) {
                    this.toggleSidebar();
                }
            });
        }
        
        // JSON 内容点击事件监听
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('json-content')) {
                const jsonData = e.target.getAttribute('data-json');
                if (jsonData) {
                    this.showJsonModal(jsonData);
                }
            }
        });
        
        if (fileDropZone) {
            fileDropZone.addEventListener('click', () => fileInput?.click());
        }
        
        // 解析按钮已移除，选择文件后自动解析
        
        // 清空按钮
        if (clearBtn) {
            clearBtn.addEventListener('click', () => this.clearContent());
        }
        
        // 主题切换
        if (themeToggle) {
            themeToggle.addEventListener('click', async () => {
                await this.toggleTheme();
            });
        }
        
        if (themeSelect) {
            themeSelect.addEventListener('change', (e) => this.setTheme(e.target.value));
        }
        
        // 插件管理
        if (pluginManager) {
            pluginManager.addEventListener('click', () => this.showPluginManager());
        }
        
        if (pluginModalClose) {
            pluginModalClose.addEventListener('click', () => this.hideModal('pluginModal'));
        }
        
        // 设置
        if (settingsBtn) {
            settingsBtn.addEventListener('click', () => this.showSettings());
        }
        
        if (settingsModalClose) {
            settingsModalClose.addEventListener('click', () => this.hideModal('settingsModal'));
        }
        
        // 搜索
        if (searchBtn) {
            searchBtn.addEventListener('click', () => this.showSearch());
        }
        
        if (searchClose) {
            searchClose.addEventListener('click', () => this.hideModal('searchOverlay'));
        }
        
        if (searchInput) {
            searchInput.addEventListener('input', (e) => this.searchLogs(e.target.value));
        }
        
        // 导出
        if (exportBtn) {
            exportBtn.addEventListener('click', () => this.exportResults());
        }
        
        // 插件管理标签切换
        this.setupPluginTabs();
        
        // 模态框点击外部关闭
        this.setupModalClickOutside();
    }
    
    setupPluginTabs() {
        const tabButtons = document.querySelectorAll('[data-tab]');
        const tabContents = document.querySelectorAll('[id$="Tab"]');
        
        tabButtons.forEach(button => {
            button.addEventListener('click', () => {
                const tabName = button.dataset.tab;
                
                // 更新按钮状态
                tabButtons.forEach(btn => {
                    btn.classList.remove('text-primary-600', 'dark:text-primary-400', 'border-primary-500');
                    btn.classList.add('text-gray-500', 'dark:text-gray-400', 'border-transparent');
                });
                button.classList.add('text-primary-600', 'dark:text-primary-400', 'border-primary-500');
                button.classList.remove('text-gray-500', 'dark:text-gray-400', 'border-transparent');
                
                // 更新内容显示
                tabContents.forEach(content => {
                    content.classList.add('hidden');
                });
                
                const targetContent = document.getElementById(`${tabName}Tab`);
                if (targetContent) {
                    targetContent.classList.remove('hidden');
                }
                
                // 加载对应内容
                if (tabName === 'installed') {
                    this.loadInstalledPlugins();
                } else if (tabName === 'available') {
                    this.loadAvailablePlugins();
                }
            });
        });
    }
    
    setupModalClickOutside() {
        const modals = ['pluginModal', 'settingsModal', 'searchOverlay'];
        
        modals.forEach(modalId => {
            const modal = document.getElementById(modalId);
            if (modal) {
                modal.addEventListener('click', (e) => {
                    if (e.target === modal) {
                        this.hideModal(modalId);
                    }
                });
            }
        });
    }
    
    setupDragAndDrop() {
        const dropZone = document.getElementById('fileDropZone');
        if (!dropZone) return;
        
        dropZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            dropZone.classList.add('border-primary-500', 'bg-primary-50', 'dark:bg-primary-900/20');
        });
        
        dropZone.addEventListener('dragleave', (e) => {
            e.preventDefault();
            dropZone.classList.remove('border-primary-500', 'bg-primary-50', 'dark:bg-primary-900/20');
        });
        
        dropZone.addEventListener('drop', (e) => {
            e.preventDefault();
            dropZone.classList.remove('border-primary-500', 'bg-primary-50', 'dark:bg-primary-900/20');
            
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.handleFileSelect({ target: { files } });
            }
        });
    }
    
           // 初始化主题
           initTheme() {
               // 从本地存储加载主题设置
               const savedTheme = localStorage.getItem('logwhisper-theme') || 'light';
               this.setTheme(savedTheme);
               
               // 更新主题选择器
               const themeSelect = document.getElementById('themeSelect');
               if (themeSelect) {
                   themeSelect.value = savedTheme;
               }
               
               // 添加布局修复CSS
               this.addLayoutFixCSS();
           }
           
           // 添加布局修复CSS规则
           addLayoutFixCSS() {
               const style = document.createElement('style');
               style.id = 'layout-fix-css';
               style.textContent = `
                   /* 修复布局问题 - 确保日志内容区域正确使用flex布局 */
                   #mainApp {
                       height: 100vh !important;
                       overflow: hidden !important;
                   }
                   
                   #logEditor {
                       display: flex !important;
                       flex-direction: column !important;
                       height: 100% !important;
                       min-height: 0 !important;
                   }
                   
                   #logContent {
                       flex: 1 !important;
                       min-height: 0 !important;
                       overflow: hidden !important;
                       display: flex !important;
                       flex-direction: column !important;
                   }
                   
                   #logLines {
                       flex: 1 !important;
                       overflow-y: auto !important;
                       overflow-x: hidden !important;
                       min-height: 0 !important;
                   }
                   
                   /* 主编辑区域布局修复 */
                   .flex-1.flex.flex-col.min-h-0 {
                       display: flex !important;
                       flex-direction: column !important;
                       flex: 1 !important;
                       min-height: 0 !important;
                       overflow: hidden !important;
                   }
                   
                   main.flex.flex-1 {
                       display: flex !important;
                       flex: 1 !important;
                       min-height: 0 !important;
                       overflow: hidden !important;
                   }
                   
                   /* 移除按钮点击后的焦点边框 */
                   button:focus {
                       outline: none !important;
                       box-shadow: none !important;
                   }
                   
                   button:focus-visible {
                       outline: none !important;
                       box-shadow: none !important;
                   }
                   
                   /* 移除所有可聚焦元素的默认焦点样式 */
                   *:focus {
                       outline: none !important;
                   }
                   
                   /* 确保按钮在各种状态下都没有边框 */
                   button {
                       border: none !important;
                   }
                   
                   button:active {
                       outline: none !important;
                       box-shadow: none !important;
                   }
                   
                   /* 侧边栏折叠样式 */
                   #sidebar {
                       transition: all 0.3s ease-in-out !important;
                       overflow: hidden !important;
                   }
                   
                   .sidebar-collapsed {
                       width: 48px !important;
                       min-width: 48px !important;
                       max-width: 48px !important;
                   }
                   
                   .sidebar-collapsed .sidebar-title,
                   .sidebar-collapsed #sidebarContent {
                       opacity: 0 !important;
                       visibility: hidden !important;
                       transition: opacity 0.2s ease-in-out, visibility 0.2s ease-in-out !important;
                   }
                   
                   .sidebar-collapsed #sidebarToggle {
                       justify-self: center !important;
                       margin: 0 auto !important;
                   }
                   
                   /* 折叠状态下的提示区域 */
                   .sidebar-collapsed #collapsedHint {
                       display: flex !important;
                       opacity: 1 !important;
                       transition: opacity 0.3s ease-in-out 0.2s !important;
                       cursor: pointer !important;
                   }
                   
                   .sidebar-collapsed #collapsedHint:hover {
                       background-color: rgba(0, 0, 0, 0.05) !important;
                   }
                   
                   #sidebar:not(.sidebar-collapsed) #collapsedHint {
                       display: none !important;
                       opacity: 0 !important;
                   }
                   
                   /* 保证非折叠状态下的正常显示 */
                   #sidebar:not(.sidebar-collapsed) .sidebar-title,
                   #sidebar:not(.sidebar-collapsed) #sidebarContent {
                       opacity: 1 !important;
                       visibility: visible !important;
                       transition: opacity 0.3s ease-in-out 0.1s, visibility 0.3s ease-in-out 0.1s !important;
                   }
                   
                   /* ========== 优化的日志级别样式（只标签有颜色） ========== */
                   
                   /* 日志行基本样式 - 统一的纯净背景 */
                   .log-line {
                       display: flex !important;
                       align-items: flex-start !important;
                       padding: 8px 12px !important;
                       border-bottom: 1px solid #e5e7eb !important;
                       background: transparent !important;
                       font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace !important;
                       font-size: 13px !important;
                       line-height: 1.4 !important;
                       transition: all 0.2s ease !important;
                       margin-bottom: 1px !important;
                   }
                   
                   .dark .log-line {
                       border-bottom-color: #374151 !important;
                   }
                   
                   .log-line:hover {
                       background: rgba(0, 0, 0, 0.02) !important;
                       transform: translateX(2px) !important;
                       box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05) !important;
                   }
                   
                   .dark .log-line:hover {
                       background: rgba(255, 255, 255, 0.03) !important;
                   }
                   
                   .log-line.selected {
                       background: rgba(14, 165, 233, 0.08) !important;
                       border-left: 4px solid #0ea5e9 !important;
                       box-shadow: 0 0 0 1px rgba(14, 165, 233, 0.2) !important;
                   }
                   
                   .dark .log-line.selected {
                       background: rgba(14, 165, 233, 0.1) !important;
                   }
                   
                   /* 所有级别的日志行都使用统一背景 */
                   .log-line-error,
                   .log-line-warn,
                   .log-line-info,
                   .log-line-debug,
                   .log-line-trace,
                   .log-line-default {
                       background: inherit !important;
                       border-left: none !important;
                   }
                   
                   /* 日志行元素样式 */
                   .log-line-number {
                       width: 50px !important;
                       flex-shrink: 0 !important;
                       color: #9ca3af !important;
                       text-align: right !important;
                       margin-right: 12px !important;
                       user-select: none !important;
                   }
                   
                   .log-level-icon {
                       width: 20px !important;
                       flex-shrink: 0 !important;
                       text-align: center !important;
                       margin-right: 8px !important;
                   }
                   
                   .log-line-timestamp {
                       color: #6b7280 !important;
                       margin-right: 8px !important;
                       flex-shrink: 0 !important;
                       font-size: 12px !important;
                   }
                   
                   .log-line-timestamp.clickable {
                       cursor: pointer !important;
                       color: #3b82f6 !important;
                   }
                   
                   .log-line-timestamp.clickable:hover {
                       text-decoration: underline !important;
                   }
                   
                   .log-line-thread {
                       color: #8b5cf6 !important;
                       margin-right: 8px !important;
                       flex-shrink: 0 !important;
                       font-size: 11px !important;
                       background: rgba(139, 92, 246, 0.1) !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                   }
                   
                   .log-line-logger {
                       color: #059669 !important;
                       margin-right: 8px !important;
                       flex-shrink: 0 !important;
                       font-size: 11px !important;
                       background: rgba(5, 150, 105, 0.1) !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                   }
                   
                   /* 日志级别标签增强样式（主要色彩区分） */
                   .log-line-level {
                       margin-right: 12px !important;
                       flex-shrink: 0 !important;
                       font-weight: 700 !important;
                       font-size: 11px !important;
                       padding: 4px 10px !important;
                       border-radius: 8px !important;
                       text-align: center !important;
                       min-width: 60px !important;
                       letter-spacing: 0.5px !important;
                       border: 2px solid transparent !important;
                       text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2) !important;
                       box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15) !important;
                       transition: all 0.2s ease !important;
                   }
                   
                   /* ERROR级别 - 红色系标签 */
                   .level-error {
                       background: linear-gradient(135deg, #dc2626 0%, #b91c1c 100%) !important;
                       color: white !important;
                       border-color: #991b1b !important;
                   }
                   
                   .level-error:hover {
                       background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%) !important;
                       box-shadow: 0 4px 12px rgba(220, 38, 38, 0.4) !important;
                   }
                   
                   /* WARN级别 - 黄色系标签 */
                   .level-warn {
                       background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%) !important;
                       color: white !important;
                       border-color: #b45309 !important;
                   }
                   
                   .level-warn:hover {
                       background: linear-gradient(135deg, #fbbf24 0%, #f59e0b 100%) !important;
                       box-shadow: 0 4px 12px rgba(245, 158, 11, 0.4) !important;
                   }
                   
                   /* INFO级别 - 蓝色系标签 */
                   .level-info {
                       background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%) !important;
                       color: white !important;
                       border-color: #1d4ed8 !important;
                   }
                   
                   .level-info:hover {
                       background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%) !important;
                       box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4) !important;
                   }
                   
                   /* DEBUG级别 - 绿色系标签 */
                   .level-debug {
                       background: linear-gradient(135deg, #10b981 0%, #059669 100%) !important;
                       color: white !important;
                       border-color: #047857 !important;
                   }
                   
                   .level-debug:hover {
                       background: linear-gradient(135deg, #34d399 0%, #10b981 100%) !important;
                       box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4) !important;
                   }
                   
                   /* TRACE级别 - 灰色系标签 */
                   .level-trace {
                       background: linear-gradient(135deg, #6b7280 0%, #4b5563 100%) !important;
                       color: white !important;
                       border-color: #374151 !important;
                   }
                   
                   .level-trace:hover {
                       background: linear-gradient(135deg, #9ca3af 0%, #6b7280 100%) !important;
                       box-shadow: 0 4px 12px rgba(107, 114, 128, 0.4) !important;
                   }
                   
                   /* 默认级别标签 */
                   .level-default {
                       background: linear-gradient(135deg, #9ca3af 0%, #6b7280 100%) !important;
                       color: white !important;
                       border-color: #4b5563 !important;
                   }
                   
                   /* 内容区域样式 */
                   .log-line-content-wrapper {
                       flex: 1 !important;
                       min-width: 0 !important;
                       word-break: break-word !important;
                       overflow-wrap: break-word !important;
                   }
                   
                   /* JSON 内容样式 */
                   .json-content {
                       background: #dbeafe !important;
                       color: #1e40af !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                       cursor: pointer !important;
                       font-weight: 500 !important;
                   }
                   
                   .json-content:hover {
                       background: #bfdbfe !important;
                   }
                   
                   .invalid-json {
                       background: #fee2e2 !important;
                       color: #dc2626 !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                   }
                   
                   /* SQL 内容样式 */
                   .sql-statement {
                       background: #dcfce7 !important;
                       color: #166534 !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                       font-weight: 500 !important;
                   }
                   
                   .sql-parameters {
                       background: #fef3c7 !important;
                       color: #92400e !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                   }
                   
                   .sql-time {
                       background: #e0e7ff !important;
                       color: #3730a3 !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                       font-weight: 600 !important;
                   }
                   
                   /* 异常内容样式 */
                   .exception-name {
                       background: #fee2e2 !important;
                       color: #dc2626 !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                       font-weight: 600 !important;
                   }
                   
                   .stack-trace {
                       color: #7c3aed !important;
                       font-weight: 500 !important;
                   }
                   
                   .stack-location {
                       color: #059669 !important;
                       font-style: italic !important;
                   }
                   
                   /* 异常聚合显示样式 */
                   .exception-container {
                       border: 1px solid #f87171 !important;
                       border-radius: 8px !important;
                       padding: 8px !important;
                       margin: 4px 0 !important;
                       background: rgba(254, 226, 226, 0.2) !important;
                   }
                   
                   .dark .exception-container {
                       border-color: #dc2626 !important;
                       background: rgba(127, 29, 29, 0.2) !important;
                   }
                   
                   .exception-header {
                       color: #dc2626 !important;
                       font-weight: 600 !important;
                       margin-bottom: 4px !important;
                   }
                   
                   .exception-stack-trace {
                       background: rgba(249, 250, 251, 0.6) !important;
                       border: 1px solid #e5e7eb !important;
                       border-radius: 4px !important;
                       padding: 8px !important;
                       margin: 4px 0 !important;
                       font-family: 'Consolas', 'Monaco', monospace !important;
                       font-size: 12px !important;
                   }
                   
                   .dark .exception-stack-trace {
                       background: rgba(55, 65, 81, 0.4) !important;
                       border-color: #4b5563 !important;
                   }
                   
                   .stack-trace-line {
                       margin: 2px 0 !important;
                       line-height: 1.4 !important;
                   }
                   
                   .exception-toggle-btn {
                       background: #dc2626 !important;
                       color: white !important;
                       border: none !important;
                       padding: 4px 8px !important;
                       border-radius: 4px !important;
                       font-size: 12px !important;
                       cursor: pointer !important;
                       margin-top: 4px !important;
                   }
                   
                   .exception-toggle-btn:hover {
                       background: #b91c1c !important;
                   }
                   
                   .exception-caused-by {
                       background: #fbbf24 !important;
                       color: #92400e !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                       font-weight: 600 !important;
                   }
                   
                   .exception-suppressed {
                       background: #a78bfa !important;
                       color: #5b21b6 !important;
                       padding: 2px 6px !important;
                       border-radius: 4px !important;
                       font-weight: 600 !important;
                   }
                   
                   /* 关键词高亮 */
                   .keyword-error {
                       background: #dc2626 !important;
                       color: white !important;
                       padding: 1px 4px !important;
                       border-radius: 3px !important;
                       font-weight: 600 !important;
                   }
                   
                   .keyword-warn {
                       background: #f59e0b !important;
                       color: white !important;
                       padding: 1px 4px !important;
                       border-radius: 3px !important;
                       font-weight: 600 !important;
                   }
                   
                   .keyword-success {
                       background: #10b981 !important;
                       color: white !important;
                       padding: 1px 4px !important;
                       border-radius: 3px !important;
                       font-weight: 600 !important;
                   }
                   
                   .keyword-null {
                       background: #6b7280 !important;
                       color: white !important;
                       padding: 1px 4px !important;
                       border-radius: 3px !important;
                       font-weight: 600 !important;
                   }
                   
                   /* 侧边栏过滤按钮样式 */
                   .sidebar-filter-active {
                       background: rgba(59, 130, 246, 0.1) !important;
                       border: 2px solid rgba(59, 130, 246, 0.5) !important;
                       box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2) !important;
                   }
               `;
               
               // 检查是否已经添加过，避免重复
               const existingStyle = document.getElementById('layout-fix-css');
               if (existingStyle) {
                   existingStyle.remove();
               }
               
               document.head.appendChild(style);
               console.log('✅ 布局修复CSS已添加');
           }
    
    setTheme(theme) {
        this.currentTheme = theme;
        
        // 更新 HTML 类
        if (theme === 'dark') {
            document.documentElement.classList.add('dark');
        } else if (theme === 'light') {
            document.documentElement.classList.remove('dark');
        } else if (theme === 'auto') {
            // 跟随系统主题
            if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
                document.documentElement.classList.add('dark');
            } else {
                document.documentElement.classList.remove('dark');
            }
        }
        
        // 更新主题切换按钮图标
        this.updateThemeToggleIcon();
        
        // 保存设置
        localStorage.setItem('logwhisper-theme', theme);
    }
    
    // 更新主题切换按钮图标
    updateThemeToggleIcon() {
        const themeToggle = document.getElementById('themeToggle');
        if (themeToggle) {
            const icon = themeToggle.querySelector('span');
            if (icon) {
                icon.textContent = this.currentTheme === 'light' ? '🌙' : '☀️';
                console.log('🔄 主题按钮图标已更新:', this.currentTheme === 'light' ? '🌙' : '☀️');
            }
        }
    }
    
    async checkApiStatus() {
        console.log('🌐 开始检查 API 状态...');
        console.time('⏱️ API 检查耗时');
        
        const statusDot = document.getElementById('statusDot');
        
        try {
            console.log('📡 发送 API 请求到:', this.API_BASE_URL);
            const response = await fetch(`${this.API_BASE_URL}/health`);
            console.log('📡 API 响应状态:', response.status);
            console.log('📡 API 响应头:', response.headers);
            
            if (response.ok) {
                this.isApiAvailable = true;
                if (statusDot) {
                    statusDot.classList.remove('bg-gray-400');
                    statusDot.classList.add('bg-green-500', 'animate-pulse');
                }
                console.log('✅ API 服务器连接成功，isApiAvailable设置为:', this.isApiAvailable);
            } else {
                throw new Error(`API 响应异常: ${response.status}`);
            }
        } catch (error) {
            this.isApiAvailable = false;
            console.warn('⚠️ API 服务器连接失败:', error.message);
            console.warn('⚠️ 错误详情:', error);
        } finally {
            console.timeEnd('⏱️ API 检查耗时');
            console.log('🔍 最终API状态:', this.isApiAvailable);
        }
        
        // 返回 Promise 以支持链式调用
        return Promise.resolve();
    }
    
    initPluginManager() {
        // 初始化插件数据
        this.installedPlugins = [
            {
                id: 'mybatis-parser',
                name: 'MyBatis SQL 解析器',
                version: '1.0.0',
                description: '解析 MyBatis SQL 日志，提取 SQL 语句和执行时间',
                enabled: true,
                author: 'LogWhisper Team'
            },
            {
                id: 'docker-json-parser',
                name: 'Docker JSON 解析器',
                version: '1.0.0',
                description: '解析 Docker JSON 格式日志',
                enabled: true,
                author: 'LogWhisper Team'
            },
            {
                id: 'generic-text-parser',
                name: '通用文本解析器',
                version: '1.0.0',
                description: '解析通用文本格式日志',
                enabled: true,
                author: 'LogWhisper Team'
            }
        ];
        
        this.availablePlugins = [
            {
                id: 'nginx-parser',
                name: 'Nginx 访问日志解析器',
                version: '1.0.0',
                description: '解析 Nginx 访问日志，提取请求信息',
                author: 'LogWhisper Team',
                downloads: 1234,
                rating: 4.8
            },
            {
                id: 'apache-parser',
                name: 'Apache 日志解析器',
                version: '1.0.0',
                description: '解析 Apache 访问日志',
                author: 'LogWhisper Team',
                downloads: 856,
                rating: 4.6
            }
        ];
    }
    
    showPluginManager() {
        const modal = document.getElementById('pluginModal');
        if (modal) {
            modal.classList.remove('hidden');
            this.loadInstalledPlugins();
        }
    }
    
    showSettings() {
        const modal = document.getElementById('settingsModal');
        if (modal) {
            modal.classList.remove('hidden');
        }
    }
    
    showSearch() {
        const overlay = document.getElementById('searchOverlay');
        const input = document.getElementById('searchInput');
        if (overlay) {
            overlay.classList.remove('hidden');
            if (input) {
                input.focus();
            }
        }
    }
    
    hideModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.add('hidden');
        }
    }
    
    loadInstalledPlugins() {
        const container = document.getElementById('installedPlugins');
        if (!container) return;
        
        container.innerHTML = this.installedPlugins.map(plugin => `
            <div class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200">
                <div class="flex items-center justify-between mb-2">
                    <div>
                        <h4 class="font-medium text-gray-900 dark:text-white">${plugin.name}</h4>
                        <p class="text-sm text-gray-500 dark:text-gray-400">v${plugin.version}</p>
                    </div>
                    <div class="flex items-center space-x-2">
                        <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${plugin.enabled ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'}">
                            ${plugin.enabled ? '已启用' : '已禁用'}
                        </span>
                        <button class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" onclick="app.togglePlugin('${plugin.id}')">
                            ${plugin.enabled ? '禁用' : '启用'}
                        </button>
                    </div>
                </div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">${plugin.description}</p>
                <div class="flex items-center justify-between">
                    <span class="text-xs text-gray-500 dark:text-gray-400">作者: ${plugin.author}</span>
                    <div class="flex space-x-2">
                        <button class="text-xs text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300" onclick="app.configurePlugin('${plugin.id}')">
                            配置
                        </button>
                        <button class="text-xs text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300" onclick="app.uninstallPlugin('${plugin.id}')">
                            卸载
                        </button>
                    </div>
                </div>
            </div>
        `).join('');
    }
    
    loadAvailablePlugins() {
        const container = document.getElementById('availablePlugins');
        if (!container) return;
        
        container.innerHTML = this.availablePlugins.map(plugin => `
            <div class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200">
                <div class="flex items-center justify-between mb-2">
                    <div>
                        <h4 class="font-medium text-gray-900 dark:text-white">${plugin.name}</h4>
                        <p class="text-sm text-gray-500 dark:text-gray-400">v${plugin.version}</p>
                    </div>
                    <button class="px-3 py-1 bg-primary-600 hover:bg-primary-700 text-white text-sm rounded-lg transition-colors duration-200" onclick="app.installPlugin('${plugin.id}')">
                        安装
                    </button>
                </div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">${plugin.description}</p>
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4 text-xs text-gray-500 dark:text-gray-400">
                        <span>作者: ${plugin.author}</span>
                        <span>下载: ${plugin.downloads}</span>
                        <span>评分: ${plugin.rating} ⭐</span>
                    </div>
                </div>
            </div>
        `).join('');
    }
    
    togglePlugin(pluginId) {
        const plugin = this.installedPlugins.find(p => p.id === pluginId);
        if (plugin) {
            plugin.enabled = !plugin.enabled;
            this.loadInstalledPlugins();
            console.log(`插件 ${plugin.name} ${plugin.enabled ? '已启用' : '已禁用'}`);
        }
    }
    
    configurePlugin(pluginId) {
        console.log(`配置插件: ${pluginId}`);
        // 这里可以实现插件配置功能
    }
    
    uninstallPlugin(pluginId) {
        if (confirm('确定要卸载此插件吗？')) {
            this.installedPlugins = this.installedPlugins.filter(p => p.id !== pluginId);
            this.loadInstalledPlugins();
            console.log(`插件 ${pluginId} 已卸载`);
        }
    }
    
    installPlugin(pluginId) {
        const plugin = this.availablePlugins.find(p => p.id === pluginId);
        if (plugin) {
            // 模拟安装过程
            this.installedPlugins.push({
                ...plugin,
                enabled: true
            });
            this.loadInstalledPlugins();
            console.log(`插件 ${plugin.name} 已安装`);
        }
    }
    
    handleFileSelect(event) {
        console.log('📁 文件选择事件触发');
        const files = event.target.files;
        console.log('📁 选择的文件数量:', files.length);
        
        if (files.length > 0) {
            const file = files[0];
            
            // 在处理新文件之前，先清理所有历史数据和UI状态
            console.log('🧹 清理历史数据和UI状态...');
            this.clearAllState();
            
            this.currentFile = file;
            
            console.log('📁 文件已选择:', file.name);
            console.log('📁 文件大小:', file.size, 'bytes');
            console.log('📁 文件类型:', file.type);
            
            // 更新文件信息显示
            this.updateFileInfo(file);
            
            // 选择文件后直接触发解析
            console.log('🚀 开始自动解析...');
            console.log('🔍 当前API状态:', this.isApiAvailable);
            this.parseLog();
        } else {
            console.log('⚠️ 没有选择文件');
        }
    }
    
    // 更新文件信息显示
    updateFileInfo(file) {
        const fileInfoElement = document.getElementById('fileInfo');
        const statusFile = document.getElementById('statusFile');
        const statusFileSize = document.getElementById('statusFileSize');
        const statusParseTime = document.getElementById('statusParseTime');
        
        if (file) {
            const fileSize = this.formatFileSize(file.size);
            
            // 更新编辑器工具栏中的文件信息
            if (fileInfoElement) {
                fileInfoElement.innerHTML = `<span class="inline-block mr-1">📄</span>${file.name} (${fileSize})`;
                fileInfoElement.className = 'text-sm text-green-600 dark:text-green-400';
            }
            
            // 更新底部状态栏
            if (statusFile) {
                statusFile.textContent = `文件：${file.name}`;
            }
            if (statusFileSize) {
                statusFileSize.textContent = `大小：${fileSize}`;
                statusFileSize.classList.remove('hidden');
            }
        }
    }
    
    // 加载下一个分块
    async loadNextChunk(filePath, chunkIndex, chunkSize) {
        try {
            console.log('📦 加载第', chunkIndex + 1, '块...');
            
            const response = await fetch(`${this.API_BASE_URL}/api/parse`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    file_path: filePath,
                    plugin: 'auto',
                    chunk_size: chunkSize,
                    chunk_index: chunkIndex
                })
            });
            
            if (response.ok) {
                const result = await response.json();
                console.log('📦 分块', chunkIndex + 1, '加载完成:', result.entries.length, '条');
                
                if (result.success && result.entries) {
                    // 追加到现有条目
                    this.currentEntries = this.currentEntries.concat(result.entries);
                    this.chunkInfo = result.chunk_info;
                    
                    // 追加渲染
                    this.appendLogEntries(result.entries);
                    
                    // 如果有更多块，继续请求
                    if (result.chunk_info && result.chunk_info.has_more) {
                        this.loadNextChunk(filePath, result.chunk_info.current_chunk + 1, chunkSize);
                    } else {
                        console.log('✅ 所有分块加载完成');
                    }
                }
            } else {
                console.error('❌ 分块请求失败:', response.status);
            }
        } catch (error) {
            console.error('❌ 加载分块失败:', error);
        }
    }
    
    // 追加日志条目到DOM
    appendLogEntries(entries) {
        const logLinesContainer = document.getElementById('logLines');
        if (!logLinesContainer) return;
        
        entries.forEach((entry, index) => {
            const lineElement = this.createLogLineElement(entry, this.currentEntries.length - entries.length + index);
            logLinesContainer.appendChild(lineElement);
        });
        
        console.log('📝 追加渲染完成，当前总行数:', this.currentEntries.length);
    }
    
    // 更新解析时间显示
    updateParseTime(parseTime) {
        const statusParseTime = document.getElementById('statusParseTime');
        if (statusParseTime && parseTime) {
            statusParseTime.textContent = `解析：${parseTime}`;
            statusParseTime.classList.remove('hidden');
        }
    }
    
    // 格式化文件大小
    formatFileSize(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
    
    
    async parseLog() {
        if (!this.currentFile) {
            alert('请先选择日志文件');
            return;
        }
        
        console.log('🔍 检查API可用性:', this.isApiAvailable);
        console.log('🔍 API基础URL:', this.API_BASE_URL);
        
        if (!this.isApiAvailable) {
            console.warn('⚠️ API不可用，尝试重新检查...');
            await this.checkApiStatus();
            if (!this.isApiAvailable) {
                alert('API 服务器不可用，请检查连接');
                return;
            }
        }
        
        this.showLoading();
        
        try {
            // 获取文件路径
            console.log('📁 获取文件路径...');
            console.log('📁 文件对象:', this.currentFile);
            
            // 尝试不同的路径获取方式
            let filePath;
            let useFilePath = false;
            
            if (this.currentFile.path) {
                filePath = this.currentFile.path;
                useFilePath = true;
                console.log('📁 使用文件路径:', filePath);
            } else if (this.currentFile.webkitRelativePath) {
                filePath = this.currentFile.webkitRelativePath;
                useFilePath = true;
                console.log('📁 使用相对路径:', filePath);
            } else {
                console.log('📁 无法获取文件路径，回退到内容传输');
                useFilePath = false;
            }
            
            // 检查文件大小，决定是否使用分块处理
            const fileSize = this.currentFile.size;
            const useChunked = fileSize > 500000; // 500KB以上使用分块
            const chunkSize = useChunked ? 1000 : null; // 分块大小（行数）
            
            console.log('📊 文件大小:', fileSize, 'bytes');
            console.log('📊 使用分块处理:', useChunked);
            console.log('📊 分块大小:', chunkSize);
            
            console.log('🌐 发送解析请求到:', `${this.API_BASE_URL}/api/parse`);
            
            let requestBody;
            if (useFilePath) {
                // 使用文件路径模式
                requestBody = {
                    file_path: filePath,
                    plugin: 'auto',
                    chunk_size: chunkSize,
                    chunk_index: 0
                };
                console.log('🌐 使用文件路径模式');
            } else {
                // 回退到内容传输模式
                console.log('📖 读取文件内容...');
                const content = await this.readFileContent(this.currentFile);
                console.log('📖 文件内容长度:', content.length);
                
                requestBody = {
                    content: content,
                    plugin: 'auto',
                    chunk_size: chunkSize,
                    chunk_index: 0
                };
                console.log('🌐 使用内容传输模式');
            }
            
            console.log('🌐 请求体:', requestBody);
            console.log('🌐 请求体JSON:', JSON.stringify(requestBody));
            
            const requestStartTime = performance.now();
            const response = await fetch(`${this.API_BASE_URL}/api/parse`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(requestBody)
            });
            
            const requestEndTime = performance.now();
            console.log('📡 解析API响应状态:', response.status);
            console.log('📡 解析API响应头:', response.headers);
            console.log('⏱️ 请求耗时:', (requestEndTime - requestStartTime).toFixed(2), 'ms');
            
            if (response.ok) {
                const result = await response.json();
                console.log('📊 解析结果:', {
                    success: result.success,
                    entriesCount: result.entries?.length || 0,
                    stats: result.stats,
                    detectedFormat: result.detected_format // 新增：检测到的格式
                });
                
                // 设置格式标志，用于控制异常处理
                console.log('🔍 原始detected_format值:', JSON.stringify(result.detected_format));
                console.log('🔍 detected_format类型:', typeof result.detected_format);
                this.isDockerJsonFormat = result.detected_format === 'DockerJson';
                console.log('🔍 检测到的日志格式:', result.detected_format);
                console.log('🔍 是否Docker JSON格式:', this.isDockerJsonFormat);
                console.log('🔍 比较结果:', result.detected_format, '===', 'DockerJson', '=', result.detected_format === 'DockerJson');
                
                // 特殊处理SpringBoot格式的聚合异常
                if (result.detected_format === 'SpringBoot') {
                    console.log('🔍 SpringBoot格式检测到，启用异常聚合处理');
                    this.isDockerJsonFormat = false; // 确保异常处理被启用
                }
                
                // 保存解析时间
                if (result.stats && result.stats.parse_time_ms) {
                    this.parseTime = `${result.stats.parse_time_ms}ms`;
                    this.updateParseTime(this.parseTime);
                }
                
                // 检查是否是分块数据
                if (result.chunk_info) {
                    console.log('📊 分块信息:', result.chunk_info);
                    console.log('📊 当前块:', result.chunk_info.current_chunk + 1, '/', result.chunk_info.total_chunks);
                    console.log('📊 还有更多:', result.chunk_info.has_more);
                    
                    // 使用分块渲染
                    this.currentEntries = result.entries;
                    this.chunkInfo = result.chunk_info;
                    console.log('📊 开始分块渲染', result.entries.length, '条日志条目');
                    this.renderLogEditorChunked(result.entries);
                    
                    // 如果有更多块，继续请求
                    if (result.chunk_info.has_more) {
                        this.loadNextChunk(filePath, result.chunk_info.current_chunk + 1, chunkSize);
                    }
                } else {
                    // 传统全量处理
                    if (result.success && result.entries) {
                        this.currentEntries = result.entries;
                        console.log('📊 开始全量渲染', result.entries.length, '条日志条目');
                        this.renderLogEditorChunked(this.currentEntries);
                    } else {
                        console.log('📊 使用传统显示方式');
                        this.displayResults(result);
                    }
                }
            } else {
                console.error('❌ API请求失败:', response.status, response.statusText);
                
                // 尝试读取错误响应体
                try {
                    const errorText = await response.text();
                    console.error('❌ 错误响应体:', errorText);
                } catch (e) {
                    console.error('❌ 无法读取错误响应体:', e);
                }
                
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
        } catch (error) {
            console.error('解析失败:', error);
            alert('解析失败: ' + error.message);
        } finally {
            this.hideLoading();
        }
    }
    
    readFileContent(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = (e) => resolve(e.target.result);
            reader.onerror = (e) => reject(e);
            reader.readAsText(file);
        });
    }
    
    displayResults(result) {
        const welcomeScreen = document.getElementById('welcomeScreen');
        const resultsContainer = document.getElementById('resultsContainer');
        const resultsContent = document.getElementById('resultsContent');
        const resultsStats = document.getElementById('resultsStats');
        
        if (welcomeScreen) welcomeScreen.classList.add('hidden');
        if (resultsContainer) resultsContainer.classList.remove('hidden');
        
        if (resultsStats) {
            resultsStats.textContent = `共解析 ${result.entries?.length || 0} 条日志，耗时 ${result.stats?.parse_time || 0}ms`;
        }
        
        if (resultsContent && result.entries) {
            resultsContent.innerHTML = result.entries.map(entry => `
                <div class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg mb-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200 ${this.getLogEntryClass(entry.level)}" style="max-width: 100%; overflow: hidden;">
                    <div class="flex items-center justify-between mb-2">
                        <div class="flex items-center space-x-2">
                            <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${this.getLogLevelClass(entry.level)}">
                                ${entry.level || 'INFO'}
                            </span>
                            ${entry.timestamp ? `<span class="text-xs text-gray-500 dark:text-gray-400">${entry.timestamp}</span>` : ''}
                        </div>
                    </div>
                    <div class="text-sm text-gray-700 dark:text-gray-300 font-mono whitespace-pre-wrap break-words overflow-wrap-anywhere" style="word-break: break-all; max-width: 100%;">${this.escapeHtml(entry.content)}</div>
                </div>
            `).join('');
        }
        
        this.currentEntries = result.entries || [];
    }
    
    getLogEntryClass(level) {
        // 统一使用干净的背景，不区分级别
        return 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800';
    }
    
    getLogLevelClass(level) {
        if (!level) return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
        
        const normalizedLevel = level.toUpperCase();
        const levelMap = {
            'ERROR': 'bg-red-500 text-white border-red-600',
            'WARN': 'bg-yellow-500 text-white border-yellow-600',
            'INFO': 'bg-blue-500 text-white border-blue-600',
            'DEBUG': 'bg-green-500 text-white border-green-600',
            'TRACE': 'bg-gray-500 text-white border-gray-600'
        };
        return levelMap[normalizedLevel] || 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
    }
    
    searchLogs(term) {
        this.searchTerm = term.toLowerCase();
        
        if (!term.trim()) {
            // 显示所有结果
            this.displayAllResults();
            return;
        }
        
        const filteredEntries = this.currentEntries.filter(entry => 
            entry.content.toLowerCase().includes(this.searchTerm) ||
            (entry.level && entry.level.toLowerCase().includes(this.searchTerm)) ||
            (entry.timestamp && entry.timestamp.toLowerCase().includes(this.searchTerm))
        );
        
        this.displayFilteredResults(filteredEntries);
    }
    
    displayAllResults() {
        const resultsContent = document.getElementById('resultsContent');
        if (resultsContent && this.currentEntries) {
            this.displayResults({ entries: this.currentEntries });
        }
    }
    
    displayFilteredResults(entries) {
        const resultsContent = document.getElementById('resultsContent');
        if (resultsContent) {
            resultsContent.innerHTML = entries.map(entry => `
                <div class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg mb-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200 ${this.getLogEntryClass(entry.level)}">
                    <div class="flex items-center justify-between mb-2">
                        <div class="flex items-center space-x-2">
                            <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${this.getLogLevelClass(entry.level)}">
                                ${entry.level || 'INFO'}
                            </span>
                            ${entry.timestamp ? `<span class="text-xs text-gray-500 dark:text-gray-400">${entry.timestamp}</span>` : ''}
                        </div>
                    </div>
                    <div class="text-sm text-gray-700 dark:text-gray-300 font-mono whitespace-pre-wrap">${this.highlightSearchTerm(entry.content)}</div>
                </div>
            `).join('');
        }
    }
    
    highlightSearchTerm(text) {
        if (!this.searchTerm) return text;
        
        const regex = new RegExp(`(${this.searchTerm})`, 'gi');
        return text.replace(regex, '<mark class="bg-yellow-200 dark:bg-yellow-800">$1</mark>');
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    clearContent() {
        // 使用完整的状态清理函数
        this.clearAllState();
        
        // 清理文件输入框
        const fileInput = document.getElementById('fileInput');
        if (fileInput) fileInput.value = '';
        
        console.log('🗑️ 内容已清空');
    }
    
    // 清理所有状态和UI数据（用于文件切换时）
    clearAllState() {
        console.log('🧹 开始清理所有历史状态和UI数据...');
        
        // 1. 清理日志数据
        this.currentFile = null;
        this.currentEntries = [];
        this.logLines = [];
        this.filteredLines = [];
        this.searchResults = [];
        
        // 2. 重置搜索和过滤状态
        this.searchTerm = '';
        this.currentFilter = 'all';
        this.currentLine = 0;
        this.totalLines = 0;
        this.pluginCategories = {};
        
        // 3. 重置解析时间
        this.parseTime = null;
        
        // 4. 重置日志格式标志
        this.isDockerJsonFormat = false;
        
        // 4. 清理搜索输入框
        const searchInput = document.getElementById('searchInput');
        if (searchInput) {
            searchInput.value = '';
        }
        
        // 5. 清理日志编辑器UI
        const logEditor = document.getElementById('logEditor');
        const editorToolbar = document.getElementById('editorToolbar');
        if (logEditor) {
            logEditor.classList.add('hidden');
        }
        if (editorToolbar) {
            editorToolbar.classList.add('hidden');
        }
        
        // 6. 清理日志内容区域
        const logLines = document.getElementById('logLines');
        if (logLines) {
            logLines.innerHTML = '';
            logLines.className = 'log-editor'; // 重置类名
        }
        
        // 7. 清理侧边栏内容
        const pluginCategories = document.getElementById('pluginCategories');
        if (pluginCategories) {
            pluginCategories.innerHTML = '';
        }
        
        // 8. 重置状态栏
        this.resetStatusBar();
        
        // 9. 显示欢迎界面
        const welcomeScreen = document.getElementById('welcomeScreen');
        if (welcomeScreen) {
            welcomeScreen.classList.remove('hidden');
        }
        
        // 10. 隐藏结果容器
        const resultsContainer = document.getElementById('resultsContainer');
        if (resultsContainer) {
            resultsContainer.classList.add('hidden');
        }
        
        // 11. 清理所有选中状态
        document.querySelectorAll('.log-line.selected').forEach(el => {
            el.classList.remove('selected');
        });
        
        // 12. 清理搜索高亮
        this.clearSearchHighlights();
        
        // 13. 注意：过滤按钮已移除，不需要重置状态
        
        // 14. 清理文件信息显示
        const fileInfoElement = document.getElementById('fileInfo');
        if (fileInfoElement) {
            fileInfoElement.textContent = '未选择文件';
        }
        
        console.log('✅ 所有历史状态和UI数据已清理完成');
    }
    
    // 重置状态栏
    resetStatusBar() {
        const statusLine = document.getElementById('statusLine');
        const statusColumn = document.getElementById('statusColumn');
        const statusPlugins = document.getElementById('statusPlugins');
        const statusSearch = document.getElementById('statusSearch');
        const statusFile = document.getElementById('statusFile');
        const statusFileSize = document.getElementById('statusFileSize');
        const statusParseTime = document.getElementById('statusParseTime');
        
        if (statusLine) {
            statusLine.textContent = '行 0/0';
        }
        
        if (statusColumn) {
            statusColumn.textContent = '列 0';
        }
        
        if (statusPlugins) {
            statusPlugins.textContent = '插件：无';
        }
        
        if (statusSearch) {
            statusSearch.textContent = '搜索：0 处匹配';
        }
        
        if (statusFile) {
            statusFile.textContent = '文件：无';
        }
        
        if (statusFileSize) {
            statusFileSize.textContent = '大小：未知';
            statusFileSize.classList.add('hidden');
        }
        
        if (statusParseTime) {
            statusParseTime.textContent = '解析：未知';
            statusParseTime.classList.add('hidden');
        }
        
        console.log('✅ 状态栏已重置');
    }
    
    showLoading() {
        const overlay = document.getElementById('loadingOverlay');
        if (overlay) {
            overlay.classList.remove('hidden');
        }
    }
    
    hideLoading() {
        const overlay = document.getElementById('loadingOverlay');
        if (overlay) {
            overlay.classList.add('hidden');
        }
    }
}

// 初始化应用
let app;

// 简化初始化逻辑
function initApp() {
    if (!app) {
        app = new LogWhisperApp();
    }
}

// 确保 DOM 完全加载后再初始化
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initApp);
} else {
    // DOM 已经加载完成，立即初始化
    initApp();
}

// 监听系统主题变化
if (window.matchMedia) {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        if (app && app.currentTheme === 'auto') {
            app.setTheme('auto');
        }
    });
}