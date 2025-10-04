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
               if (logEditor) logEditor.classList.remove('hidden');
               if (editorToolbar) editorToolbar.classList.remove('hidden');
               
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
               
               // 重新设计布局系统
               const logContentForHeight = document.getElementById('logContent');
               const logEditorForHeight = document.getElementById('logEditor');
               
               // 计算可用高度
               const windowHeight = window.innerHeight;
               const headerHeight = 48; // 头部高度
               const footerHeight = 32; // 底部高度
               const availableHeight = windowHeight - headerHeight - footerHeight;
               
               if (logContentForHeight) {
                   logContentForHeight.style.setProperty('height', `${availableHeight}px`, 'important');
                   logContentForHeight.style.setProperty('max-height', `${availableHeight}px`, 'important');
                   logContentForHeight.style.setProperty('overflow-y', 'auto', 'important');
                   logContentForHeight.style.setProperty('overflow-x', 'hidden', 'important');
                   console.log('🔧 设置logContent高度为:', availableHeight + 'px');
               }
               if (logEditorForHeight) {
                   logEditorForHeight.style.setProperty('height', `${availableHeight}px`, 'important');
                   logEditorForHeight.style.setProperty('max-height', `${availableHeight}px`, 'important');
                   logEditorForHeight.style.setProperty('overflow', 'hidden', 'important');
                   console.log('🔧 设置logEditor高度为:', availableHeight + 'px');
               }
               
               // 强制设置logLines容器高度
               logLinesContainer.style.setProperty('height', 'auto', 'important');
               logLinesContainer.style.setProperty('max-height', 'none', 'important');
               logLinesContainer.style.setProperty('overflow', 'visible', 'important');
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
               
               // 渲染完成后再次强制设置高度
               const logContentAfter = document.getElementById('logContent');
               const logEditorAfter = document.getElementById('logEditor');
               if (logContentAfter) {
                   logContentAfter.style.setProperty('height', '500px', 'important');
                   logContentAfter.style.setProperty('max-height', '500px', 'important');
                   console.log('🔧 渲染后重新设置logContent高度');
               }
               if (logEditorAfter) {
                   logEditorAfter.style.setProperty('height', '500px', 'important');
                   logEditorAfter.style.setProperty('max-height', '500px', 'important');
                   console.log('🔧 渲染后重新设置logEditor高度');
               }
               
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
               
               // 设置容器高度和滚动
               logLinesContainer.style.height = '600px';
               logLinesContainer.style.overflowY = 'auto';
               
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

           // 创建日志行元素
           createLogLineElement(entry, index) {
               const lineDiv = document.createElement('div');
               lineDiv.className = 'log-line';
               lineDiv.dataset.lineNumber = index + 1;
               lineDiv.dataset.originalIndex = this.logLines.indexOf(entry);

               // 行号
               const lineNumber = document.createElement('div');
               lineNumber.className = 'log-line-number';
               lineNumber.textContent = (index + 1).toString().padStart(4, ' ');
               lineDiv.appendChild(lineNumber);

               // 左侧边距区域（插件图标）
               const marginDiv = document.createElement('div');
               marginDiv.className = 'log-line-margin';
               
               // 根据插件类型添加图标
               if (entry.plugin_type) {
                   const icon = this.getPluginIcon(entry.plugin_type);
                   marginDiv.innerHTML = `<span class="log-line-icon">${icon}</span>`;
               }
               
               lineDiv.appendChild(marginDiv);

               // 时间戳
               if (entry.timestamp) {
                   const timestamp = document.createElement('span');
                   timestamp.className = 'log-line-timestamp';
                   timestamp.textContent = entry.timestamp;
                   lineDiv.appendChild(timestamp);
               }

               // 日志级别
               if (entry.level) {
                   const level = document.createElement('span');
                   level.className = `log-line-level ${entry.level.toLowerCase()}`;
                   level.textContent = entry.level;
                   lineDiv.appendChild(level);
               }

               // 日志内容
               const contentDiv = document.createElement('div');
               contentDiv.className = 'log-line-content';
               
               // 应用语法高亮
               const highlightedContent = this.applySyntaxHighlighting(entry.content, entry.plugin_type);
               contentDiv.innerHTML = highlightedContent;
               
               lineDiv.appendChild(contentDiv);

               // 插件装饰器（行尾标签）
               if (entry.decorator) {
                   const decorator = document.createElement('div');
                   decorator.className = 'log-decorator';
                   decorator.textContent = entry.decorator;
                   lineDiv.appendChild(decorator);
               }

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
                   const level = entry.level || 'Info';
                   if (!this.pluginCategories[level]) {
                       this.pluginCategories[level] = [];
                   }
                   this.pluginCategories[level].push({ entry, index });
               });

               // 渲染侧边栏
               sidebarContent.innerHTML = '';
               
               // 添加文件信息
               const fileInfoDiv = document.createElement('div');
               fileInfoDiv.className = 'mb-4 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg';
               fileInfoDiv.innerHTML = `
                   <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">📄 文件信息</h4>
                   <div class="text-xs text-gray-600 dark:text-gray-400 space-y-1">
                       <div>总行数: ${this.totalLines}</div>
                       <div>解析时间: ${this.parseTime || '未知'}</div>
                   </div>
               `;
               sidebarContent.appendChild(fileInfoDiv);
               
               // 添加日志级别统计
               const levelStatsDiv = document.createElement('div');
               levelStatsDiv.className = 'mb-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg';
               levelStatsDiv.innerHTML = `
                   <h4 class="text-sm font-medium text-blue-700 dark:text-blue-300 mb-2">📊 日志统计</h4>
                   <div class="space-y-1">
                       ${Object.entries(this.pluginCategories).map(([level, items]) => `
                           <div class="flex justify-between text-xs">
                               <span class="text-gray-600 dark:text-gray-400">${level}</span>
                               <span class="font-medium text-blue-600 dark:text-blue-400">${items.length}</span>
                           </div>
                       `).join('')}
                   </div>
               `;
               sidebarContent.appendChild(levelStatsDiv);
               
               // 添加快速导航
               const quickNavDiv = document.createElement('div');
               quickNavDiv.className = 'mb-4';
               quickNavDiv.innerHTML = `
                   <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">🚀 快速导航</h4>
                   <div class="space-y-1">
                       <button class="w-full text-left px-2 py-1 text-xs rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" onclick="app.scrollToTop()">
                           📄 跳转到顶部
                       </button>
                       <button class="w-full text-left px-2 py-1 text-xs rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" onclick="app.scrollToBottom()">
                           📄 跳转到底部
                       </button>
                       <button class="w-full text-left px-2 py-1 text-xs rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" onclick="app.filterByLevel('ERROR')">
                           🔴 仅显示错误
                       </button>
                       <button class="w-full text-left px-2 py-1 text-xs rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" onclick="app.filterByLevel('WARN')">
                           🟡 仅显示警告
                       </button>
                   </div>
               `;
               sidebarContent.appendChild(quickNavDiv);
               
               Object.entries(this.pluginCategories).forEach(([level, items]) => {
                   const categoryDiv = document.createElement('div');
                   categoryDiv.className = 'mb-2';
                   
                   const header = document.createElement('div');
                   header.className = 'flex items-center justify-between p-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-colors';
                   header.innerHTML = `
                       <span class="text-sm">${this.getLevelIcon(level)} ${level}</span>
                       <span class="text-xs text-gray-500 dark:text-gray-400">${items.length}</span>
                   `;
                   
                   header.addEventListener('click', () => {
                       this.filterByLevel(level);
                   });
                   
                   categoryDiv.appendChild(header);
                   sidebarContent.appendChild(categoryDiv);
               });
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

           // 按级别过滤
           filterByLevel(level) {
               console.log('🔍 按级别过滤:', level);
               this.currentFilter = level;
               this.filteredLines = this.logLines.filter(entry => entry.level === level);
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
               this.updateFilterButtons();
           }

           // 更新过滤器按钮状态
           updateFilterButtons() {
               document.querySelectorAll('.filter-btn').forEach(btn => {
                   btn.classList.remove('active');
               });
               
               const activeBtn = document.querySelector(`[data-filter="${this.currentFilter}"]`);
               if (activeBtn) {
                   activeBtn.classList.add('active');
               }
           }

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
               
               if (!sidebar || !toggleBtn) return;
               
               this.sidebarCollapsed = !this.sidebarCollapsed;
               
               if (this.sidebarCollapsed) {
                   sidebar.classList.add('sidebar-collapsed');
                   toggleBtn.innerHTML = '<span class="text-sm">▶</span>';
               } else {
                   sidebar.classList.remove('sidebar-collapsed');
                   toggleBtn.innerHTML = '<span class="text-sm">◀</span>';
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

        // 过滤器按钮
        document.querySelectorAll('.filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const filter = e.target.dataset.filter;
                this.currentFilter = filter;
                this.filterByPlugin(filter);
            });
        });

        // 侧边栏折叠
        const sidebarToggle = document.getElementById('sidebarToggle');
        if (sidebarToggle) {
            sidebarToggle.addEventListener('click', () => {
                this.toggleSidebar();
            });
        }
        
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
    
    initTheme() {
        // 从本地存储加载主题设置
        const savedTheme = localStorage.getItem('logwhisper-theme') || 'light';
        this.setTheme(savedTheme);
        
        // 更新主题选择器
        const themeSelect = document.getElementById('themeSelect');
        if (themeSelect) {
            themeSelect.value = savedTheme;
        }
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
    
    // updateParseButton 方法已移除，选择文件后自动解析
    
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
                    stats: result.stats
                });
                
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
        const levelMap = {
            'ERROR': 'border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20',
            'WARN': 'border-yellow-200 dark:border-yellow-800 bg-yellow-50 dark:bg-yellow-900/20',
            'INFO': 'border-blue-200 dark:border-blue-800 bg-blue-50 dark:bg-blue-900/20',
            'DEBUG': 'border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/20'
        };
        return levelMap[level] || levelMap['INFO'];
    }
    
    getLogLevelClass(level) {
        const levelMap = {
            'ERROR': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
            'WARN': 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
            'INFO': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
            'DEBUG': 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'
        };
        return levelMap[level] || levelMap['INFO'];
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
        const fileInput = document.getElementById('fileInput');
        const welcomeScreen = document.getElementById('welcomeScreen');
        const resultsContainer = document.getElementById('resultsContainer');
        
        if (fileInput) fileInput.value = '';
        
        if (welcomeScreen) welcomeScreen.classList.remove('hidden');
        if (resultsContainer) resultsContainer.classList.add('hidden');
        
        this.currentFile = null;
        this.currentEntries = [];
        
        console.log('🗑️ 内容已清空');
    }
    
    exportResults() {
        if (this.currentEntries.length === 0) {
            alert('没有可导出的结果');
            return;
        }
        
        const data = this.currentEntries.map(entry => ({
            timestamp: entry.timestamp,
            level: entry.level,
            content: entry.content
        }));
        
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = `logwhisper-export-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        console.log('📤 结果已导出');
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