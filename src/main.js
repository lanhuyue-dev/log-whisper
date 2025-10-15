// LogWhisper 前端应用 - Tauri 版本 (基于 Tailwind CSS)
class LogWhisperApp {
    constructor() {
        this.currentFile = null;
        this.currentEntries = [];
        this.searchTerm = '';
        this.isLoading = false;
        this.currentTheme = 'light';
        this.debugMode = false;

        // 检测是否在 Tauri 环境中
        // Tauri 2.x API might be loaded asynchronously
        this.isTauriEnv = window.__TAURI__ !== undefined ||
                          window.__TAURI_INTERNALS__ !== undefined ||
                          document.documentElement.hasAttribute('data-tauri');

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
               console.log('🚀 LogWhisper Tauri 前端应用初始化...');
               console.time('⏱️ 初始化总耗时');

               // 更新加载状态
               this.updateLoadingStatus('初始化组件...');
               console.log('📋 1. 组件初始化开始');

               // 设置事件监听器
               this.setupEventListeners();
               console.log('📋 2. 事件监听器设置完成');

               // 拖拽功能已移除
               console.log('📋 3. 拖拽功能已移除');

               // 初始化主题
               this.initTheme();

               // 加载配置（异步等待）
               await this.loadConfigs();
               console.log('📋 4. 主题初始化完成');

               // 更新加载状态
               this.updateLoadingStatus('检测环境...');
               console.log('📋 5. 开始环境检测');

               // 初始化 Tauri 环境
               await this.initTauri();
               console.log('📋 6. Tauri 环境初始化完成');

               // 更新加载状态
               this.updateLoadingStatus('连接后端服务...');
               console.log('📋 7. 开始连接后端服务');

               // 检查后端状态（异步）
               await this.checkBackendStatus();
               console.log('📋 8. 后端连接完成，开始最终初始化');

               // 初始化插件管理
               this.initPluginManager();
               console.log('📋 9. 插件管理初始化完成');

               console.log('📋 10. 所有初始化步骤完成');

               // 更新加载状态
               this.updateLoadingStatus('准备就绪');

               // 隐藏加载界面
               setTimeout(() => {
                   const loadingOverlay = document.getElementById('loadingOverlay');
                   if (loadingOverlay) {
                       loadingOverlay.classList.add('opacity-0');
                       setTimeout(() => {
                           loadingOverlay.classList.add('hidden');
                       }, 300);
                   }
               }, 500);

               console.timeEnd('⏱️ 初始化总耗时');
               console.log('✅ LogWhisper Tauri 前端应用初始化完成');
           }

           // 初始化 Tauri 环境
           async initTauri() {
               // 等待 Tauri API 加载
               let retries = 0;
               const maxRetries = 50; // 增加等待时间

               while (!window.__TAURI__ && retries < maxRetries) {
                   await new Promise(resolve => setTimeout(resolve, 100));
                   retries++;
                   if (retries % 10 === 0) {
                       console.log(`🔄 等待 Tauri API 加载... (${retries}/${maxRetries})`);
                   }
               }

               // 更详细的环境检测
               if (window.__TAURI__) {
                   console.log('✅ Tauri 环境检测成功');
                   console.log('🔍 window.__TAURI__ 类型:', typeof window.__TAURI__);
                   console.log('🔍 window.__TAURI__.invoke:', typeof window.__TAURI__.invoke);

                   this.isTauriEnv = true;

                   // 初始化 Tauri API
                   this.tauri = window.__TAURI__;

                   // 测试 invoke API 是否可用
                   try {
                       console.log('🧪 测试 Tauri invoke API...');
                       if (typeof window.__TAURI__.invoke === 'function') {
                           console.log('✅ Tauri invoke API 可用');
                       } else {
                           console.warn('⚠️ Tauri invoke API 不是函数类型:', typeof window.__TAURI__.invoke);
                           // 尝试等待更多时间让 API 完全加载
                           await new Promise(resolve => setTimeout(resolve, 1000));
                           if (typeof window.__TAURI__.invoke === 'function') {
                               console.log('✅ 延迟后 Tauri invoke API 可用');
                           } else {
                               console.warn('⚠️ 延迟后 Tauri invoke API 仍不可用');
                           }
                       }
                   } catch (error) {
                       console.warn('⚠️ Tauri invoke API 测试失败:', error.message);
                   }

                   // 监听窗口事件 (使用全局 API)
                   try {
                       // 简化的窗口事件监听
                       if (window.__TAURI__.window) {
                           console.log('✅ 窗口 API 可用');
                       }
                   } catch (error) {
                       console.warn('⚠️ 窗口事件监听器设置失败:', error.message);
                   }
               } else {
                   console.warn('⚠️ 未检测到 Tauri 环境，某些功能可能不可用');
                   console.log('🔍 window.__TAURI__:', window.__TAURI__);
                   this.isTauriEnv = false;
               }
           }

           // 检查后端状态
           async checkBackendStatus() {
               if (!this.isTauriEnv) {
                   console.warn('⚠️ 非 Tauri 环境，跳过后端检查');
                   return;
               }

               try {
                   console.log('🔍 检查 Tauri 后端状态...');
                   const response = await this.invoke('health_check');

                   if (response && response.status === 'ok') {
                       this.isBackendAvailable = true;
                       console.log('✅ Tauri 后端连接成功');
                       console.log('📊 后端信息:', response);
                   } else {
                       this.isBackendAvailable = false;
                       console.warn('⚠️ Tauri 后端响应异常');
                   }
               } catch (error) {
                   this.isBackendAvailable = false;
                   console.warn('⚠️ Tauri 后端连接失败:', error.message);
               }
           }

           // Tauri invoke 封装
           async invoke(command, args = {}) {
               if (!this.isTauriEnv) {
                   throw new Error('Tauri 环境不可用');
               }

               try {
                   console.log(`🔧 调用 Tauri 命令: ${command}`, args);

                   // Use global window.__TAURI__ object
                   if (!window.__TAURI__ || !window.__TAURI__.invoke) {
                       throw new Error('Tauri invoke API 不可用');
                   }

                   const result = await window.__TAURI__.invoke(command, args);

                   console.log(`✅ Tauri 命令 ${command} 执行成功:`, result);
                   return result;
               } catch (error) {
                   console.error(`❌ Tauri 命令 ${command} 执行失败:`, error);
                   throw error;
               }
           }

           // 加载配置
           async loadConfigs() {
               if (!this.isTauriEnv) {
                   console.warn('⚠️ 非 Tauri 环境，跳过配置加载');
                   return;
               }

               try {
                   console.log('📋 开始加载配置...');

                   // 加载主题配置
                   try {
                       const themeData = await this.invoke('get_theme_config');
                       this.configs.theme = themeData;
                       this.applyTheme(themeData);
                       console.log('✅ 主题配置加载成功');
                   } catch (error) {
                       console.warn('⚠️ 主题配置加载失败:', error.message);
                   }

                   // 加载插件配置
                   try {
                       const pluginData = await this.invoke('get_plugin_config');
                       this.configs.plugin = pluginData;
                       console.log('✅ 插件配置加载成功');
                   } catch (error) {
                       console.warn('⚠️ 插件配置加载失败:', error.message);
                   }

                   // 加载窗口配置
                   try {
                       const windowData = await this.invoke('get_window_config');
                       this.configs.window = windowData;
                       console.log('✅ 窗口配置加载成功');
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

           // 更新加载状态
           updateLoadingStatus(status) {
               const loadingStatus = document.getElementById('loadingStatus');
               if (loadingStatus) {
                   loadingStatus.textContent = status;
               }
               const loadingProgress = document.getElementById('loadingProgress');
               if (loadingProgress) {
                   loadingProgress.textContent = status;
               }
           }

           // 设置事件监听器
           setupEventListeners() {
               console.log('🔧 开始设置事件监听器');

               // 文件选择
               const fileInput = document.getElementById('fileInput');
               console.log('📁 查找文件输入元素:', !!fileInput);
               if (fileInput) {
                   console.log('✅ 设置文件选择事件监听器');
                   fileInput.addEventListener('change', (e) => {
                       console.log('🔍 文件选择事件触发');
                       this.handleFileSelect(e);
                   });
               } else {
                   console.error('❌ 未找到文件输入元素');
               }

               // 拖拽功能已移除 - 不再设置拖拽事件监听器
               console.log('📋 拖拽功能已移除');

               // 搜索框
               const searchInput = document.getElementById('searchInput');
               if (searchInput) {
                   searchInput.addEventListener('input', (e) => this.handleSearch(e.target.value));
               }

               // 筛选按钮
               const filterButtons = document.querySelectorAll('.filter-btn');
               filterButtons.forEach(btn => {
                   btn.addEventListener('click', () => {
                       const filter = btn.dataset.filter;
                       this.setFilter(filter);
                   });
               });

               // 主题切换
               const themeToggle = document.getElementById('themeToggle');
               if (themeToggle) {
                   themeToggle.addEventListener('click', () => this.toggleTheme());
               }

               // 设置按钮
               const settingsBtn = document.getElementById('settingsBtn');
               if (settingsBtn) {
                   settingsBtn.addEventListener('click', () => this.openSettings());
               }

               // 文件重新选择按钮
               const resetBtn = document.getElementById('resetBtn');
               if (resetBtn) {
                   resetBtn.addEventListener('click', () => this.resetFile());
               }

               // 导出按钮
               const exportBtn = document.getElementById('exportBtn');
               if (exportBtn) {
                   exportBtn.addEventListener('click', () => this.exportResults());
               }

               console.log('✅ 事件监听器设置完成');
           }

  
           // 初始化主题
           initTheme() {
               // 检查系统主题偏好
               const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

               // 初始应用系统主题
               if (prefersDark) {
                   document.documentElement.classList.add('dark');
                   document.body.classList.add('dark');
                   this.currentTheme = 'dark';
               }

               // 监听系统主题变化
               window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
                   if (this.configs.theme?.mode === 'auto') {
                       if (e.matches) {
                           document.documentElement.classList.add('dark');
                           document.body.classList.add('dark');
                           this.currentTheme = 'dark';
                       } else {
                           document.documentElement.classList.remove('dark');
                           document.body.classList.remove('dark');
                           this.currentTheme = 'light';
                       }
                       this.updateThemeToggleIcon();
                   }
               });

               console.log('✅ 主题初始化完成');
           }

           // 初始化插件管理器
           initPluginManager() {
               if (!this.isTauriEnv) {
                   console.warn('⚠️ 非 Tauri 环境，跳过插件管理器初始化');
                   return;
               }

               this.loadAvailablePlugins();
               console.log('✅ 插件管理器初始化完成');
           }

           // 加载可用插件
           async loadAvailablePlugins() {
               try {
                   const response = await this.invoke('get_plugins');
                   this.availablePlugins = response.plugins || [];
                   console.log('✅ 可用插件加载完成:', this.availablePlugins);
               } catch (error) {
                   console.warn('⚠️ 加载可用插件失败:', error.message);
               }
           }

           // 处理文件选择
           handleFileSelect(event) {
               console.log('🔍 handleFileSelect 被调用');
               console.log('📁 事件对象:', event);
               console.log('📁 选择的文件:', event.target.files);

               const file = event.target.files[0];
               if (file) {
                   console.log('✅ 检测到文件:', file.name, '大小:', file.size, 'bytes');
                   this.loadFile(file);
               } else {
                   console.log('❌ 没有选择文件');
               }
           }

    
           // 加载文件
           async loadFile(file) {
               console.log('📁 开始加载文件:', file.name);
               console.log('🔧 Tauri环境状态:', this.isTauriEnv);
               console.log('📂 文件路径:', file.path);

               this.currentFile = file;
               this.isLoading = true;

               // 更新UI状态
               console.log('🔄 更新加载UI状态');
               this.updateLoadingUI(true);

               try {
                   let fileContent;

                   if (this.isTauriEnv && file.path) {
                       // Tauri 环境，使用文件路径读取
                       console.log('📁 使用 Tauri API 读取文件:', file.path);
                       fileContent = await this.readTextFile(file.path);
                   } else {
                       // 浏览器环境或没有文件路径，使用 FileReader API
                       console.log('📁 使用 FileReader API 读取文件');
                       fileContent = await this.readFileWithFileReader(file);
                   }

                   console.log('📄 文件内容读取完成，大小:', fileContent.length, '字节');
                   console.log('📝 文件内容预览:', fileContent.substring(0, 200) + '...');

                   // 检测文件类型和选择合适的插件
                   console.log('🔍 开始检测插件类型');
                   const detectedPlugin = this.detectPlugin(fileContent);
                   console.log('🔍 检测到插件类型:', detectedPlugin);

                   // 解析日志
                   console.log('⚙️ 开始解析日志内容');
                   await this.parseLogContent(fileContent, detectedPlugin);
                   console.log('✅ 日志解析完成');

               } catch (error) {
                   console.error('❌ 文件加载失败:', error);
                   this.showError(`文件加载失败: ${error.message}`);
               } finally {
                   this.isLoading = false;
                   console.log('🔄 结束加载UI状态');
                   this.updateLoadingUI(false);
               }
           }

           // 使用 Tauri API 读取文本文件
           async readTextFile(filePath) {
               try {
                   // Use Tauri 1.x API for filesystem operations
                   if (!window.__TAURI__ || !window.__TAURI__.invoke) {
                       throw new Error('Tauri 文件系统 API 不可用');
                   }

                   const content = await window.__TAURI__.invoke('read_text_file', {
                       path: filePath
                   });
                   return content;
               } catch (error) {
                   throw new Error(`读取文件失败: ${error.message}`);
               }
           }

           // 使用 FileReader API 读取文件
           async readFileWithFileReader(file) {
               return new Promise((resolve, reject) => {
                   const reader = new FileReader();

                   reader.onload = (e) => {
                       resolve(e.target.result);
                   };

                   reader.onerror = (e) => {
                       reject(new Error(`文件读取失败: ${e.target.error.message}`));
                   };

                   reader.readAsText(file);
               });
           }

           // 检测插件类型
           detectPlugin(content) {
               const lines = content.split('\n').slice(0, 100); // 只检查前100行

               // Docker JSON 检测 - 扩展检测逻辑
               const dockerJsonCount = lines.filter(line => {
                   const trimmed = line.trim();
                   if (!trimmed.startsWith('{') || !trimmed.endsWith('}')) {
                       return false;
                   }

                   // 检查是否为有效的JSON
                   try {
                       const json = JSON.parse(trimmed);
                       // 检查多种Docker/微服务JSON日志格式
                       return (
                           // 标准Docker容器日志格式
                           (json.log && json.stream && json.time) ||
                           // 微服务JSON日志格式
                           (json.timestamp && json.level && json.message) ||
                           (json.timestamp && json.level && json.service) ||
                           (json.time && json.level && json.msg) ||
                           // 通用JSON日志格式
                           (json.level && (json.message || json.msg || json.text))
                       );
                   } catch (e) {
                       return false;
                   }
               }).length;

               if (dockerJsonCount > lines.length / 2) {
                   return 'docker_json';
               }

               // MyBatis 检测
               const mybatisCount = lines.filter(line =>
                   line.includes('Preparing:') ||
                   line.includes('Parameters:') ||
                   line.includes('==>')
               ).length;

               if (mybatisCount > 0) {
                   return 'mybatis';
               }

               // Spring Boot 检测
               const springBootCount = lines.filter(line =>
                   line.includes('INFO') ||
                   line.includes('ERROR') ||
                   line.includes('WARN') ||
                   line.includes('DEBUG')
               ).length;

               if (springBootCount > lines.length / 2) {
                   return 'springboot';
               }

               return 'auto'; // 默认使用自动检测
           }

           // 解析日志内容
           async parseLogContent(content, plugin = 'auto') {
               console.log('🔍 parseLogContent 开始');
               console.log('🔧 Tauri环境检查:', this.isTauriEnv);

               if (!this.isTauriEnv) {
                   throw new Error('请在 Tauri 环境中使用此功能');
               }

               console.log(`🔍 开始解析日志内容，使用插件: ${plugin}`);
               console.log('📊 内容长度:', content.length);

               try {
                   console.log('📡 调用 Tauri parse_log 命令');
                   const response = await this.invoke('parse_log', {
                       content: content,
                       plugin: plugin
                   });

                   console.log('📡 收到后端响应:', response);

                   if (response.success) {
                       console.log('✅ 解析成功，处理数据');
                       this.currentEntries = response.entries || [];
                       this.parseTime = response.stats?.parse_time_ms || 0;

                       console.log(`📊 获得 ${this.currentEntries.length} 条解析结果`);
                       console.log('📝 第一条记录预览:', this.currentEntries[0]);

                       // 渲染日志编辑器
                       console.log('🎨 开始渲染日志编辑器');
                       this.renderLogEditor(this.currentEntries);

                       // 更新状态栏
                       console.log('📊 更新状态栏');
                       this.updateStatusBar();

                       console.log(`✅ 日志解析完成，处理了 ${this.currentEntries.length} 条记录`);
                   } else {
                       console.error('❌ 解析响应失败:', response);
                       throw new Error(response.error || '解析失败');
                   }
               } catch (error) {
                   console.error('❌ 日志解析失败:', error);
                   this.showError(`日志解析失败: ${error.message}`);
               }
           }

           // 渲染日志编辑器
           renderLogEditor(entries) {
               console.log('📝 renderLogEditor 开始');
               console.log('📊 接收到的条目数量:', entries.length);
               console.log('📝 第一条条目:', entries[0]);

               this.logLines = entries;
               this.totalLines = entries.length;
               this.filteredLines = [...entries];

               console.log('🔍 查找DOM元素');
               // 隐藏欢迎界面，显示编辑器
               const welcomeScreen = document.getElementById('welcomeScreen');
               const logEditor = document.getElementById('logEditor');
               const editorToolbar = document.getElementById('editorToolbar');

               console.log('📋 DOM元素状态:', {
                   welcomeScreen: !!welcomeScreen,
                   logEditor: !!logEditor,
                   editorToolbar: !!editorToolbar
               });

               if (welcomeScreen) {
                   console.log('🔄 隐藏欢迎界面');
                   welcomeScreen.classList.add('hidden');
               }
               if (logEditor) {
                   console.log('🔄 显示日志编辑器');
                   logEditor.classList.remove('hidden');
                   logEditor.style.removeProperty('height');
                   logEditor.style.removeProperty('max-height');
               }
               if (editorToolbar) {
                   console.log('🔄 显示编辑器工具栏');
                   editorToolbar.classList.remove('hidden');
               }

               // 渲染日志行
               console.log('📄 开始渲染日志行');
               this.renderLogLines();

               // 更新侧边栏导航
               console.log('📊 更新侧边栏导航');
               this.updateSidebarNavigation();

               // 更新状态栏
               console.log('📊 更新状态栏');
               this.updateStatusBar();

               console.log('✅ 日志编辑器渲染完成');
           }

           // 渲染日志行
           renderLogLines() {
               const logLinesContainer = document.getElementById('logLines');
               if (!logLinesContainer) return;

               logLinesContainer.innerHTML = '';

               this.filteredLines.forEach((entry, index) => {
                   const lineElement = this.createLogLineElement(entry, index);
                   logLinesContainer.appendChild(lineElement);
               });
           }

           // 创建日志行元素 - 增强版本
           createLogLineElement(entry, index) {
               const div = document.createElement('div');
               div.className = 'log-line';
               if (entry.level) {
                   div.classList.add(entry.level.toLowerCase());
               }
               div.dataset.lineNumber = entry.line_number;

               // 创建日志行布局
               const layout = document.createElement('div');
               layout.className = 'log-line-layout';

               // 行号
               const lineNumber = document.createElement('div');
               lineNumber.className = 'log-line-number';
               lineNumber.textContent = entry.line_number;

               // 日志级别徽章
               const levelBadge = this.createLevelBadge(entry.level);

               // 时间戳
               const timestamp = this.createTimestamp(entry.timestamp);

               // 日志前缀（如果有）
               const prefix = this.createPrefix(entry);

               // 日志内容
               const content = this.createLogContent(entry);

               // 组装布局
               layout.appendChild(lineNumber);
               layout.appendChild(levelBadge);
               if (timestamp) layout.appendChild(timestamp);
               if (prefix) layout.appendChild(prefix);
               layout.appendChild(content);

               div.appendChild(layout);

               // 添加点击事件
               div.addEventListener('click', () => {
                   this.selectLine(index);
               });

               return div;
           }

           // 创建日志级别徽章
           createLevelBadge(level) {
               const badge = document.createElement('div');
               badge.className = 'log-level-badge';

               if (!level) {
                   badge.classList.add('debug');
                   badge.textContent = 'UNKNOWN';
                   return badge;
               }

               const levelUpper = level.toUpperCase();
               badge.classList.add(levelUpper.toLowerCase());
               badge.textContent = levelUpper;

               // 添加图标
               let icon = '';
               switch (levelUpper) {
                   case 'ERROR':
                       icon = '❌ ';
                       break;
                   case 'WARN':
                       icon = '⚠️ ';
                       break;
                   case 'INFO':
                       icon = 'ℹ️ ';
                       break;
                   case 'DEBUG':
                       icon = '🐛 ';
                       break;
                   default:
                       icon = '📝 ';
               }
               badge.textContent = icon + levelUpper;

               return badge;
           }

           // 创建时间戳
           createTimestamp(timestamp) {
               if (!timestamp) return null;

               const timestampDiv = document.createElement('div');
               timestampDiv.className = 'log-timestamp';

               // 格式化时间戳为 HH:MM:SS
               const date = new Date(timestamp);
               const timeStr = date.toLocaleTimeString('zh-CN', {
                   hour12: false,
                   hour: '2-digit',
                   minute: '2-digit',
                   second: '2-digit'
               });

               timestampDiv.textContent = timeStr;
               timestampDiv.title = date.toLocaleString('zh-CN');

               return timestampDiv;
           }

           // 创建日志前缀
           createPrefix(entry) {
               // 从元数据中提取前缀信息
               const prefixParts = [];

               if (entry.metadata) {
                   if (entry.metadata.thread) {
                       prefixParts.push(entry.metadata.thread);
                   }
                   if (entry.metadata.class_name) {
                       const className = entry.metadata.class_name.split('.').pop();
                       prefixParts.push(className);
                   }
                   if (entry.metadata.method_name) {
                       prefixParts.push(entry.metadata.method_name);
                   }
               }

               // 如果没有元数据前缀，尝试从内容中提取
               if (prefixParts.length === 0 && entry.content) {
                   const match = entry.content.match(/^\[([^\]]+)\]/);
                   if (match) {
                       prefixParts.push(match[1]);
                   }
               }

               if (prefixParts.length === 0) return null;

               const prefixDiv = document.createElement('div');
               prefixDiv.className = 'log-prefix';
               prefixDiv.textContent = prefixParts.join(' ');
               prefixDiv.title = prefixParts.join(' ');

               // 添加点击展开功能
               prefixDiv.addEventListener('click', (e) => {
                   e.stopPropagation();
                   prefixDiv.classList.toggle('collapsed');
               });

               return prefixDiv;
           }

           // 创建日志内容
           createLogContent(entry) {
               const contentDiv = document.createElement('div');
               contentDiv.className = 'log-content';

               // 使用格式化内容（如果有的话）
               const text = entry.formatted_content || entry.content || '';

               if (!text) {
                   contentDiv.textContent = '';
                   return contentDiv;
               }

               // 检查是否包含特殊内容（JSON、SQL、异常）
               if (entry.metadata) {
                   // SQL 内容
                   if (entry.metadata.sql_statement || entry.metadata.sql_parameters) {
                       const sqlBlock = this.createSQLBlock(entry);
                       contentDiv.appendChild(sqlBlock);
                       return contentDiv;
                   }

                   // JSON 内容
                   if (entry.metadata.json_content) {
                       const jsonBlock = this.createJSONBlock(entry);
                       contentDiv.appendChild(jsonBlock);
                       return contentDiv;
                   }

                   // 异常内容
                   if (entry.metadata.exception_type || entry.metadata.exception_message) {
                       const exceptionBlock = this.createExceptionBlock(entry);
                       contentDiv.appendChild(exceptionBlock);
                       return contentDiv;
                   }
               }

               // 检查文本内容中的特殊模式
               if (this.isSQLContent(text)) {
                   const sqlBlock = this.createSQLBlockFromText(text);
                   contentDiv.appendChild(sqlBlock);
               } else if (this.isJSONContent(text)) {
                   const jsonBlock = this.createJSONBlockFromText(text);
                   contentDiv.appendChild(jsonBlock);
               } else if (this.isExceptionContent(text)) {
                   const exceptionBlock = this.createExceptionBlockFromText(text);
                   contentDiv.appendChild(exceptionBlock);
               } else {
                   // 普通文本内容
                   const textSpan = document.createElement('span');
                   textSpan.textContent = text;
                   contentDiv.appendChild(textSpan);
               }

               return contentDiv;
           }

           // 创建 SQL 块
           createSQLBlock(entry) {
               const sqlDiv = document.createElement('div');
               sqlDiv.className = 'log-sql collapsed';

               const header = document.createElement('div');
               header.className = 'log-sql-header';

               const title = document.createElement('span');
               title.textContent = '📝 SQL 查询';

               const toggle = document.createElement('span');
               toggle.className = 'log-sql-toggle';
               toggle.textContent = '▶';

               header.appendChild(title);
               header.appendChild(toggle);

               const content = document.createElement('div');
               content.className = 'log-sql-content';

               if (entry.metadata.sql_statement) {
                   const statement = document.createElement('div');
                   statement.className = 'log-sql-statement';
                   statement.textContent = entry.metadata.sql_statement;
                   content.appendChild(statement);
               }

               if (entry.metadata.sql_parameters) {
                   const params = document.createElement('div');
                   params.className = 'log-sql-parameters';
                   params.textContent = '🔧 参数: ' + entry.metadata.sql_parameters;
                   content.appendChild(params);
               }

               if (entry.metadata.sql_result) {
                   const result = document.createElement('div');
                   result.className = 'log-sql-result';
                   result.textContent = '✅ 结果: ' + entry.metadata.sql_result;
                   content.appendChild(result);
               }

               sqlDiv.appendChild(header);
               sqlDiv.appendChild(content);

               // 添加展开/收起功能
               header.addEventListener('click', (e) => {
                   e.stopPropagation();
                   sqlDiv.classList.toggle('collapsed');
                   toggle.textContent = sqlDiv.classList.contains('collapsed') ? '▶' : '▼';
               });

               return sqlDiv;
           }

           // 创建 JSON 块
           createJSONBlock(entry) {
               const jsonDiv = document.createElement('div');
               jsonDiv.className = 'log-json collapsed';

               const header = document.createElement('div');
               header.className = 'log-json-header';

               const title = document.createElement('span');
               title.textContent = '📄 JSON 数据';

               const toggle = document.createElement('span');
               toggle.className = 'log-json-toggle';
               toggle.textContent = '▶';

               header.appendChild(title);
               header.appendChild(toggle);

               const content = document.createElement('div');
               content.className = 'log-json-content';
               content.textContent = entry.metadata.json_content;

               jsonDiv.appendChild(header);
               jsonDiv.appendChild(content);

               // 添加展开/收起功能
               header.addEventListener('click', (e) => {
                   e.stopPropagation();
                   jsonDiv.classList.toggle('collapsed');
                   toggle.textContent = jsonDiv.classList.contains('collapsed') ? '▶' : '▼';
               });

               return jsonDiv;
           }

           // 创建异常块
           createExceptionBlock(entry) {
               const exceptionDiv = document.createElement('div');
               exceptionDiv.className = 'log-exception collapsed';

               const header = document.createElement('div');
               header.className = 'log-exception-header';

               const title = document.createElement('span');
               title.textContent = '💥 异常信息';

               const toggle = document.createElement('span');
               toggle.className = 'log-exception-toggle';
               toggle.textContent = '▶';

               header.appendChild(title);
               header.appendChild(toggle);

               const content = document.createElement('div');
               content.className = 'log-exception-content';

               if (entry.metadata.exception_type) {
                   const type = document.createElement('div');
                   type.className = 'font-medium text-red-800 dark:text-red-200';
                   type.textContent = entry.metadata.exception_type;
                   content.appendChild(type);
               }

               if (entry.metadata.exception_message) {
                   const message = document.createElement('div');
                   message.className = 'text-red-700 dark:text-red-300 mt-1';
                   message.textContent = entry.metadata.exception_message;
                   content.appendChild(message);
               }

               if (entry.metadata.stack_trace) {
                   const stack = document.createElement('pre');
                   stack.className = 'text-red-600 dark:text-red-400 mt-2';
                   stack.textContent = entry.metadata.stack_trace;
                   content.appendChild(stack);
               }

               exceptionDiv.appendChild(header);
               exceptionDiv.appendChild(content);

               // 添加展开/收起功能
               header.addEventListener('click', (e) => {
                   e.stopPropagation();
                   exceptionDiv.classList.toggle('collapsed');
                   toggle.textContent = exceptionDiv.classList.contains('collapsed') ? '▶' : '▼';
               });

               return exceptionDiv;
           }

           // 从文本创建 SQL 块
           createSQLBlockFromText(text) {
               const sqlDiv = document.createElement('div');
               sqlDiv.className = 'log-sql collapsed';

               const header = document.createElement('div');
               header.className = 'log-sql-header';

               const title = document.createElement('span');
               title.textContent = '📝 SQL 查询';

               const toggle = document.createElement('span');
               toggle.className = 'log-sql-toggle';
               toggle.textContent = '▶';

               header.appendChild(title);
               header.appendChild(toggle);

               const content = document.createElement('div');
               content.className = 'log-sql-content';
               content.textContent = text;

               sqlDiv.appendChild(header);
               sqlDiv.appendChild(content);

               header.addEventListener('click', (e) => {
                   e.stopPropagation();
                   sqlDiv.classList.toggle('collapsed');
                   toggle.textContent = sqlDiv.classList.contains('collapsed') ? '▶' : '▼';
               });

               return sqlDiv;
           }

           // 从文本创建 JSON 块
           createJSONBlockFromText(text) {
               const jsonDiv = document.createElement('div');
               jsonDiv.className = 'log-json collapsed';

               const header = document.createElement('div');
               header.className = 'log-json-header';

               const title = document.createElement('span');
               title.textContent = '📄 JSON 数据';

               const toggle = document.createElement('span');
               toggle.className = 'log-json-toggle';
               toggle.textContent = '▶';

               header.appendChild(title);
               header.appendChild(toggle);

               const content = document.createElement('div');
               content.className = 'log-json-content';
               content.textContent = text;

               jsonDiv.appendChild(header);
               jsonDiv.appendChild(content);

               header.addEventListener('click', (e) => {
                   e.stopPropagation();
                   jsonDiv.classList.toggle('collapsed');
                   toggle.textContent = jsonDiv.classList.contains('collapsed') ? '▶' : '▼';
               });

               return jsonDiv;
           }

           // 从文本创建异常块
           createExceptionBlockFromText(text) {
               const exceptionDiv = document.createElement('div');
               exceptionDiv.className = 'log-exception collapsed';

               const header = document.createElement('div');
               header.className = 'log-exception-header';

               const title = document.createElement('span');
               title.textContent = '💥 异常信息';

               const toggle = document.createElement('span');
               toggle.className = 'log-exception-toggle';
               toggle.textContent = '▶';

               header.appendChild(title);
               header.appendChild(toggle);

               const content = document.createElement('div');
               content.className = 'log-exception-content';
               content.textContent = text;

               exceptionDiv.appendChild(header);
               exceptionDiv.appendChild(content);

               header.addEventListener('click', (e) => {
                   e.stopPropagation();
                   exceptionDiv.classList.toggle('collapsed');
                   toggle.textContent = exceptionDiv.classList.contains('collapsed') ? '▶' : '▼';
               });

               return exceptionDiv;
           }

           // 检查是否为 SQL 内容
           isSQLContent(text) {
               const sqlPatterns = [
                   /select\s+.*\s+from/i,
                   /insert\s+into/i,
                   /update\s+.*\s+set/i,
                   /delete\s+from/i,
                   /preparing:/i,
                   /parameters:/i,
                   /==>\s*preparing/i,
                   /==>\s*parameters/i
               ];

               return sqlPatterns.some(pattern => pattern.test(text));
           }

           // 检查是否为 JSON 内容
           isJSONContent(text) {
               return text.trim().startsWith('{') && text.trim().endsWith('}');
           }

           // 检查是否为异常内容
           isExceptionContent(text) {
               const exceptionPatterns = [
                   /exception/i,
                   /error/i,
                   /at\s+[\w.$]+\([^)]*\)/,
                   /Caused by:/i,
                   /Stack trace:/i
               ];

               return exceptionPatterns.some(pattern => pattern.test(text));
           }

           // 选择行
           selectLine(index) {
               // 移除之前的选择
               const previousSelected = document.querySelector('.log-line.selected');
               if (previousSelected) {
                   previousSelected.classList.remove('selected', 'bg-blue-100', 'dark:bg-blue-900');
               }

               // 添加新的选择
               const currentLine = document.querySelector(`[data-line-number="${this.filteredLines[index].line_number}"]`);
               if (currentLine) {
                   currentLine.classList.add('selected', 'bg-blue-100', 'dark:bg-blue-900');
                   currentLine.scrollIntoView({ behavior: 'smooth', block: 'center' });
               }

               this.currentLine = index;
           }

           // 处理搜索
           handleSearch(searchTerm) {
               this.searchTerm = searchTerm.toLowerCase();

               if (!this.searchTerm) {
                   this.filteredLines = [...this.logLines];
               } else {
                   this.filteredLines = this.logLines.filter(entry =>
                       (entry.content && entry.content.toLowerCase().includes(this.searchTerm)) ||
                       (entry.formatted_content && entry.formatted_content.toLowerCase().includes(this.searchTerm))
                   );
               }

               this.renderLogLines();
               this.updateStatusBar();
           }

           // 设置过滤器
           setFilter(filter) {
               this.currentFilter = filter;

               // 更新按钮状态
               document.querySelectorAll('.filter-btn').forEach(btn => {
                   btn.classList.remove('bg-blue-500', 'text-white');
                   btn.classList.add('bg-gray-200', 'text-gray-700', 'dark:bg-gray-700', 'dark:text-gray-300');
               });

               const activeBtn = document.querySelector(`[data-filter="${filter}"]`);
               if (activeBtn) {
                   activeBtn.classList.remove('bg-gray-200', 'text-gray-700', 'dark:bg-gray-700', 'dark:text-gray-300');
                   activeBtn.classList.add('bg-blue-500', 'text-white');
               }

               // 应用过滤
               if (filter === 'all') {
                   this.filteredLines = [...this.logLines];
               } else {
                   this.filteredLines = this.logLines.filter(entry =>
                       entry.level && entry.level.toLowerCase() === filter
                   );
               }

               this.renderLogLines();
               this.updateStatusBar();
           }

           // 切换主题
           async toggleTheme() {
               if (!this.isTauriEnv) {
                   // 非 Tauri 环境下的主题切换
                   if (this.currentTheme === 'light') {
                       document.documentElement.classList.add('dark');
                       document.body.classList.add('dark');
                       this.currentTheme = 'dark';
                   } else {
                       document.documentElement.classList.remove('dark');
                       document.body.classList.remove('dark');
                       this.currentTheme = 'light';
                   }
                   this.updateThemeToggleIcon();
                   return;
               }

               try {
                   const newMode = this.currentTheme === 'light' ? 'dark' : 'light';

                   await this.invoke('update_theme_config', {
                       mode: newMode
                   });

                   // 更新本地配置
                   if (this.configs.theme) {
                       this.configs.theme.mode = newMode;
                   }

                   // 应用主题
                   this.applyTheme({ mode: newMode });

               } catch (error) {
                   console.error('❌ 主题切换失败:', error);
                   this.showError(`主题切换失败: ${error.message}`);
               }
           }

           // 更新主题切换按钮图标
           updateThemeToggleIcon() {
               const themeToggle = document.getElementById('themeToggle');
               if (!themeToggle) return;

               const icon = themeToggle.querySelector('i');
               if (!icon) return;

               if (this.currentTheme === 'dark') {
                   icon.classList.remove('fa-moon');
                   icon.classList.add('fa-sun');
               } else {
                   icon.classList.remove('fa-sun');
                   icon.classList.add('fa-moon');
               }
           }

           // 更新状态栏
           updateStatusBar() {
               const statusBar = document.getElementById('statusBar');
               if (!statusBar) return;

               const totalLines = this.totalLines;
               const filteredLines = this.filteredLines.length;
               const parseTime = this.parseTime;

               statusBar.innerHTML = `
                   <div class="text-sm text-gray-600 dark:text-gray-400">
                       总行数: ${totalLines} | 显示: ${filteredLines} | 解析耗时: ${parseTime}ms
                   </div>
               `;
           }

           // 更新侧边栏导航
           updateSidebarNavigation() {
               const sidebarNav = document.getElementById('sidebarNav');
               if (!sidebarNav) return;

               // 计算日志级别统计
               const stats = {
                   total: this.logLines.length,
                   error: 0,
                   warn: 0,
                   info: 0,
                   debug: 0
               };

               this.logLines.forEach(entry => {
                   if (entry.level) {
                       const level = entry.level.toLowerCase();
                       if (stats.hasOwnProperty(level)) {
                           stats[level]++;
                       }
                   }
               });

               // 更新导航
               sidebarNav.innerHTML = `
                   <div class="space-y-2">
                       <div class="flex justify-between items-center">
                           <span class="text-sm font-medium">全部</span>
                           <span class="text-sm text-gray-500">${stats.total}</span>
                       </div>
                       <div class="flex justify-between items-center">
                           <span class="text-sm font-medium text-red-600">错误</span>
                           <span class="text-sm text-gray-500">${stats.error}</span>
                       </div>
                       <div class="flex justify-between items-center">
                           <span class="text-sm font-medium text-yellow-600">警告</span>
                           <span class="text-sm text-gray-500">${stats.warn}</span>
                       </div>
                       <div class="flex justify-between items-center">
                           <span class="text-sm font-medium text-blue-600">信息</span>
                           <span class="text-sm text-gray-500">${stats.info}</span>
                       </div>
                       <div class="flex justify-between items-center">
                           <span class="text-sm font-medium text-gray-600">调试</span>
                           <span class="text-sm text-gray-500">${stats.debug}</span>
                       </div>
                   </div>
               `;
           }

           // 更新加载UI
           updateLoadingUI(isLoading) {
               const loadingOverlay = document.getElementById('loadingOverlay');
               if (loadingOverlay) {
                   if (isLoading) {
                       loadingOverlay.classList.remove('hidden');
                       loadingOverlay.classList.remove('opacity-0');
                   } else {
                       loadingOverlay.classList.add('opacity-0');
                       setTimeout(() => {
                           loadingOverlay.classList.add('hidden');
                       }, 300);
                   }
               }

               const progressBar = document.getElementById('progressBar');
               if (progressBar) {
                   if (isLoading) {
                       progressBar.classList.remove('hidden');
                   } else {
                       progressBar.classList.add('hidden');
                   }
               }
           }

           // 重置文件
           resetFile() {
               this.currentFile = null;
               this.currentEntries = [];
               this.logLines = [];
               this.filteredLines = [];
               this.searchTerm = '';
               this.parseTime = null;

               // 重置UI
               const welcomeScreen = document.getElementById('welcomeScreen');
               const logEditor = document.getElementById('logEditor');
               const editorToolbar = document.getElementById('editorToolbar');

               if (welcomeScreen) welcomeScreen.classList.remove('hidden');
               if (logEditor) logEditor.classList.add('hidden');
               if (editorToolbar) editorToolbar.classList.add('hidden');

               // 重置文件输入
               const fileInput = document.getElementById('fileInput');
               if (fileInput) {
                   fileInput.value = '';
               }

               // 重置搜索
               const searchInput = document.getElementById('searchInput');
               if (searchInput) {
                   searchInput.value = '';
               }

               this.updateStatusBar();
           }

           // 导出结果
           async exportResults() {
               if (!this.filteredLines || this.filteredLines.length === 0) {
                   this.showError('没有可导出的数据');
                   return;
               }

               try {
                   // 准备导出数据
                   const exportData = this.filteredLines.map(entry => ({
                       line_number: entry.line_number,
                       timestamp: entry.timestamp,
                       level: entry.level,
                       content: entry.content || entry.formatted_content
                   }));

                   const jsonData = JSON.stringify(exportData, null, 2);

                   // 使用 Tauri API 保存文件
                   if (this.isTauriEnv) {
                       try {
                           // Use global Tauri API for dialog operations
                           if (!window.__TAURI__ || !window.__TAURI__.invoke) {
                               throw new Error('Tauri dialog API 不可用');
                           }

                           const filePath = await window.__TAURI__.invoke('save_dialog', {
                               defaultPath: `log-export-${new Date().toISOString().slice(0, 10)}.json`,
                               filters: [{
                                   name: 'JSON Files',
                                   extensions: ['json']
                               }]
                           });

                           if (filePath) {
                               await window.__TAURI__.invoke('write_file', {
                                   path: filePath,
                                   contents: jsonData
                               });
                               this.showSuccess('导出成功');
                           }
                       } catch (error) {
                           console.warn('⚠️ Tauri 文件保存失败，使用浏览器下载:', error.message);
                           // 回退到浏览器下载
                           const blob = new Blob([jsonData], { type: 'application/json' });
                           const url = URL.createObjectURL(blob);
                           const a = document.createElement('a');
                           a.href = url;
                           a.download = `log-export-${new Date().toISOString().slice(0, 10)}.json`;
                           a.click();
                           URL.revokeObjectURL(url);
                           this.showSuccess('导出成功');
                       }
                   } else {
                       // 回退到浏览器下载
                       const blob = new Blob([jsonData], { type: 'application/json' });
                       const url = URL.createObjectURL(blob);
                       const a = document.createElement('a');
                       a.href = url;
                       a.download = `log-export-${new Date().toISOString().slice(0, 10)}.json`;
                       a.click();
                       URL.revokeObjectURL(url);
                       this.showSuccess('导出成功');
                   }
               } catch (error) {
                   console.error('❌ 导出失败:', error);
                   this.showError(`导出失败: ${error.message}`);
               }
           }

           // 打开设置
           openSettings() {
               // TODO: 实现设置对话框
               console.log('打开设置对话框');
           }

           // 显示错误消息
           showError(message) {
               console.error('❌ 错误:', message);
               // TODO: 实现更好的错误提示UI
               alert(message);
           }

           // 显示成功消息
           showSuccess(message) {
               console.log('✅ 成功:', message);
               // TODO: 实现更好的成功提示UI
               alert(message);
           }
       }

// 初始化应用
document.addEventListener('DOMContentLoaded', () => {
    new LogWhisperApp();
});