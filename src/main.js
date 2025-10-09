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

               // 设置拖拽功能
               this.setupDragAndDrop();
               console.log('📋 3. 拖拽功能设置完成');

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
               // 文件选择
               const fileInput = document.getElementById('fileInput');
               if (fileInput) {
                   fileInput.addEventListener('change', (e) => this.handleFileSelect(e));
               }

               // 拖拽事件
               const dropZone = document.getElementById('dropZone');
               if (dropZone) {
                   ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
                       dropZone.addEventListener(eventName, (e) => {
                           e.preventDefault();
                           e.stopPropagation();
                       });
                   });

                   ['dragenter', 'dragover'].forEach(eventName => {
                       dropZone.addEventListener(eventName, () => {
                           dropZone.classList.add('border-blue-500', 'bg-blue-50');
                       });
                   });

                   ['dragleave', 'drop'].forEach(eventName => {
                       dropZone.addEventListener(eventName, () => {
                           dropZone.classList.remove('border-blue-500', 'bg-blue-50');
                       });
                   });

                   dropZone.addEventListener('drop', (e) => this.handleFileDrop(e));
               }

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

           // 设置拖拽功能
           setupDragAndDrop() {
               // 拖拽事件已在 setupEventListeners 中处理
               console.log('✅ 拖拽功能设置完成');
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
               const file = event.target.files[0];
               if (file) {
                   this.loadFile(file);
               }
           }

           // 处理文件拖拽
           handleFileDrop(event) {
               const files = event.dataTransfer.files;
               if (files.length > 0) {
                   this.loadFile(files[0]);
               }
           }

           // 加载文件
           async loadFile(file) {
               if (!this.isTauriEnv) {
                   this.showError('请在 Tauri 环境中使用此功能');
                   return;
               }

               console.log('📁 开始加载文件:', file.name);

               this.currentFile = file;
               this.isLoading = true;

               // 更新UI状态
               this.updateLoadingUI(true);

               try {
                   // 使用 Tauri API 读取文件
                   const fileContent = await this.readTextFile(file.path);

                   // 检测文件类型和选择合适的插件
                   const detectedPlugin = this.detectPlugin(fileContent);

                   // 解析日志
                   await this.parseLogContent(fileContent, detectedPlugin);

               } catch (error) {
                   console.error('❌ 文件加载失败:', error);
                   this.showError(`文件加载失败: ${error.message}`);
               } finally {
                   this.isLoading = false;
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

           // 检测插件类型
           detectPlugin(content) {
               const lines = content.split('\n').slice(0, 100); // 只检查前100行

               // Docker JSON 检测
               const dockerJsonCount = lines.filter(line =>
                   line.trim().startsWith('{') &&
                   line.includes('"log":') &&
                   line.includes('"stream":')
               ).length;

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
               if (!this.isTauriEnv) {
                   throw new Error('请在 Tauri 环境中使用此功能');
               }

               console.log(`🔍 开始解析日志内容，使用插件: ${plugin}`);

               try {
                   const response = await this.invoke('parse_log', {
                       content: content,
                       plugin: plugin
                   });

                   if (response.success) {
                       this.currentEntries = response.entries || [];
                       this.parseTime = response.stats?.parse_time_ms || 0;

                       // 渲染日志编辑器
                       this.renderLogEditor(this.currentEntries);

                       // 更新状态栏
                       this.updateStatusBar();

                       console.log(`✅ 日志解析完成，处理了 ${this.currentEntries.length} 条记录`);
                   } else {
                       throw new Error(response.error || '解析失败');
                   }
               } catch (error) {
                   console.error('❌ 日志解析失败:', error);
                   this.showError(`日志解析失败: ${error.message}`);
               }
           }

           // 渲染日志编辑器
           renderLogEditor(entries) {
               console.log('📝 开始渲染日志编辑器...');
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
                   logEditor.style.removeProperty('height');
                   logEditor.style.removeProperty('max-height');
               }
               if (editorToolbar) editorToolbar.classList.remove('hidden');

               // 渲染日志行
               this.renderLogLines();

               // 更新侧边栏导航
               this.updateSidebarNavigation();

               // 更新状态栏
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

           // 创建日志行元素
           createLogLineElement(entry, index) {
               const div = document.createElement('div');
               div.className = 'log-line flex border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800';
               div.dataset.lineNumber = entry.line_number;

               // 行号
               const lineNumber = document.createElement('div');
               lineNumber.className = 'line-number w-20 text-right pr-4 text-gray-500 dark:text-gray-400 text-sm font-mono';
               lineNumber.textContent = entry.line_number;

               // 内容
               const content = document.createElement('div');
               content.className = 'content flex-1 py-1 px-2 font-mono text-sm';

               // 根据日志级别设置颜色
               if (entry.level) {
                   switch (entry.level.toUpperCase()) {
                       case 'ERROR':
                           content.classList.add('text-red-600', 'dark:text-red-400');
                           break;
                       case 'WARN':
                           content.classList.add('text-yellow-600', 'dark:text-yellow-400');
                           break;
                       case 'INFO':
                           content.classList.add('text-blue-600', 'dark:text-blue-400');
                           break;
                       case 'DEBUG':
                           content.classList.add('text-gray-600', 'dark:text-gray-400');
                           break;
                       default:
                           content.classList.add('text-gray-800', 'dark:text-gray-200');
                   }
               } else {
                   content.classList.add('text-gray-800', 'dark:text-gray-200');
               }

               content.textContent = entry.content || entry.formatted_content || '';

               div.appendChild(lineNumber);
               div.appendChild(content);

               // 添加点击事件
               div.addEventListener('click', () => {
                   this.selectLine(index);
               });

               return div;
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