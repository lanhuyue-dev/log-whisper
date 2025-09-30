// LogWhisper 前端应用
class LogWhisperApp {
    constructor() {
        this.currentFile = null;
        this.currentEntries = [];
        this.currentPlugin = 'auto';
        this.searchTerm = '';
        this.isLoading = false;
        this.currentTheme = 'light'; // 默认亮色主题
        this.debugMode = false; // 调试模式
        this.debugStats = {
            parseCount: 0,
            totalParseTime: 0,
            cacheHits: 0,
            cacheMisses: 0
        };
        
        // 虚拟滚动配置
        this.virtualScroll = {
            enabled: true,
            itemHeight: 120, // 每个日志项的高度（增加以适应复杂内容）
            visibleCount: 15, // 可见区域显示的项目数
            bufferSize: 10, // 缓冲区大小（增加以减少频繁渲染）
            scrollTop: 0,
            startIndex: 0,
            endIndex: 0,
            containerHeight: 600,
            totalItems: 0,
            renderedItems: [], // 当前渲染的项目
            viewportStartIndex: 0,
            viewportEndIndex: 0
        };
        
        // 分块加载配置（移除，改用Rust后端API）
        // this.chunkLoading = {
        //     enabled: true,
        //     chunkSize: 100,
        //     loadedChunks: new Set(),
        //     totalChunks: 0,
        //     maxChunkSize: 1000,
        //     minChunkSize: 50,
        //     adaptiveChunkSize: true,
        //     loadingQueue: [],
        //     isProcessing: false,
        //     checkInterval: null
        // };
        
        // Rust后端分块加载配置
        this.backendChunkLoader = {
            initialized: false,
            currentFileMetadata: null,
            loadedChunks: new Set(),
            totalChunks: 0,
            chunkSize: 100
        };
        
        // 内存管理配置（移除，改用Rust后端API）
        // this.memoryManager = {
        //     maxMemoryUsage: 5000 * 1024 * 1024,
        //     currentMemoryUsage: 0,
        //     gcThreshold: 2048 * 1024 * 1024,
        //     enableMonitoring: true,
        //     lastGcTime: 0,
        //     chunkSize: 1000,
        //     maxCachedChunks: 50
        // };
        
        // 日志系统配置
        this.logger = {
            level: 'DEBUG', // DEBUG, INFO, WARN, ERROR
            console: false, // 控制台输出（关闭）
            file: true, // 文件输出
            maxFileSize: 10 * 1024 * 1024, // 10MB
            maxFiles: 5,
            logs: [], // 内存中的日志（仅用于调试面板显示）
            maxMemoryLogs: 100 // 内存中最多保存100条日志
        };
        
        this.init();
    }
    
    async init() {
        this.info('APP', 'LogWhisper 前端应用初始化...');
        this.info('LOGGER', '日志文件位置: ' + window.location.pathname.replace('/src/index.html', '') + '/logs/logwhisper_' + new Date().toISOString().split('T')[0] + '.log');
        
        // 设置事件监听器
        this.setupEventListeners();
        
        // 设置拖拽功能
        this.setupDragAndDrop();
        
        // 初始化主题
        this.initTheme();
        
        // 初始化Tauri API
        await this.initTauriAPI();
        
        // 测试日志系统
        this.testLogging();
        
        console.log('LogWhisper 前端应用初始化完成');
    }
    
    async initTauriAPI() {
        try {
            // 等待更长时间，让 Tauri 完成初始化
            await new Promise(resolve => setTimeout(resolve, 500));
            
            console.log('🔍 开始 Tauri API 检测...');
            console.log('window.__TAURI__ 存在:', typeof window.__TAURI__ !== 'undefined');
            
            if (typeof window.__TAURI__ !== 'undefined') {
                console.log('window.__TAURI__ 内容:', window.__TAURI__);
                console.log('tauri 对象存在:', typeof window.__TAURI__.tauri !== 'undefined');
                console.log('invoke 方法存在:', typeof window.__TAURI__.tauri?.invoke !== 'undefined');
            }
            
            // 检查Tauri API是否可用
            if (typeof window.__TAURI__ !== 'undefined' && window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) {
                console.log('✅ Tauri API 已加载');
                this.info('TAURI', 'Tauri 环境检测成功');
                
                // 测试一个简单的命令
                try {
                    console.log('🧪 测试 Tauri 命令...');
                    const result = await window.__TAURI__.tauri.invoke('get_available_plugins');
                    console.log('✅ Tauri 命令测试成功:', result);
                    return true;
                } catch (testError) {
                    console.warn('⚠️ Tauri 命令测试失败:', testError);
                    // 即使命令失败，API 仍然可用
                    return true;
                }
            } else {
                console.warn('⚠️ Tauri API 不可用');
                console.log('详细调试信息:', {
                    hasTAURI: typeof window.__TAURI__ !== 'undefined',
                    tauri: window.__TAURI__?.tauri,
                    invoke: window.__TAURI__?.tauri?.invoke,
                    dialog: window.__TAURI__?.dialog
                });
                this.warn('TAURI', 'Tauri 环境检测失败，请使用桌面应用启动');
                return false;
            }
        } catch (error) {
            console.error('❌ 初始化Tauri API失败:', error);
            this.error('TAURI', `Tauri API 初始化失败: ${error.message}`);
            return false;
        }
    }
    
    setupEventListeners() {
        // 文件选择按钮
        document.getElementById('openFileBtn').addEventListener('click', async (e) => {
            console.log('📁 文件选择按钮被点击');
            this.debug('UI_OPERATION', '文件选择按钮被点击');
            try {
                if (window.__TAURI__ && window.__TAURI__.dialog && window.__TAURI__.dialog.open) {
                    const selected = await window.__TAURI__.dialog.open({
                        multiple: false,
                        filters: [{ name: 'Logs', extensions: ['log', 'txt'] }]
                    });
                    if (selected) {
                        this.handleFile(selected);
                        return;
                    }
                }
            } catch (err) {
                console.warn('使用 Tauri 对话框选择文件失败，回退到浏览器文件输入:', err);
            }
            document.getElementById('fileInput').click();
        });
        
        // 文件输入变化
        document.getElementById('fileInput').addEventListener('change', (e) => {
            console.log('📁 文件输入变化事件触发');
            this.debug('FILE_OPERATION', '文件输入变化事件触发');
            if (e.target.files.length > 0) {
                this.handleFile(e.target.files[0]);
            }
        });
        
        // 监听 fileInput 的 click 事件，追踪被谁触发
        document.getElementById('fileInput').addEventListener('click', (e) => {
            console.log('📁 fileInput click 事件被触发');
            console.log('📁 调用堆栈:', new Error().stack);
            this.debug('FILE_OPERATION', 'fileInput click 事件被触发');
        });
        
        // 插件切换
        document.getElementById('pluginSelect').addEventListener('change', (e) => {
            this.switchPlugin(e.target.value);
        });
        
        // 搜索输入
        document.getElementById('searchInput').addEventListener('input', (e) => {
            this.searchTerm = e.target.value;
            this.info('SEARCH', `搜索输入: "${this.searchTerm}"`, { searchTerm: this.searchTerm });
            this.filterLogs();
        });
        
        // 清除搜索
        document.getElementById('clearSearchBtn').addEventListener('click', () => {
            this.clearSearch();
        });
        
        // 主题切换
        document.getElementById('themeToggleBtn').addEventListener('click', () => {
            this.toggleTheme();
        });
        
        // 调试面板
        document.getElementById('debugToggleBtn').addEventListener('click', () => {
            this.toggleDebugPanel();
        });
        
        document.getElementById('debugCloseBtn').addEventListener('click', () => {
            this.toggleDebugPanel();
        });
        
        // 调试工具
        document.getElementById('clearLogsBtn').addEventListener('click', () => {
            this.clearDebugLogs();
        });
        
        document.getElementById('exportLogsBtn').addEventListener('click', () => {
            this.exportLogs();
        });
        
        document.getElementById('flushLogsBtn').addEventListener('click', () => {
            this.manualFlushLogs();
        });
        
        document.getElementById('performanceTestBtn').addEventListener('click', () => {
            this.runPerformanceTest();
        });
        
        // 性能面板
        document.getElementById('performanceToggleBtn').addEventListener('click', () => {
            this.togglePerformancePanel();
        });
        
        document.getElementById('performanceCloseBtn').addEventListener('click', () => {
            this.togglePerformancePanel();
        });
        
        // 性能控制
        document.getElementById('virtualScrollEnabled').addEventListener('change', (e) => {
            this.virtualScroll.enabled = e.target.checked;
            this.logDebug(`🎯 虚拟滚动: ${e.target.checked ? '启用' : '禁用'}`);
        });
        
        // 注释掉原有的分块加载控制，因为现在由Rust后端处理
        // document.getElementById('chunkLoadingEnabled').addEventListener('change', (e) => {
        //     this.chunkLoading.enabled = e.target.checked;
        //     this.logDebug(`📦 分块加载: ${e.target.checked ? '启用' : '禁用'}`);
        // });
        
        // document.getElementById('chunkSize').addEventListener('change', (e) => {
        //     this.chunkLoading.chunkSize = parseInt(e.target.value) || 100;
        //     this.logDebug(`📦 块大小设置为: ${this.chunkLoading.chunkSize}`);
        // });
        
        document.getElementById('cleanupMemoryBtn').addEventListener('click', () => {
            this.cleanupMemoryViaBackend(); // 更改为调用后端内存清理API
        });
        
        // 键盘快捷键
        document.addEventListener('keydown', (e) => {
            // 检查是否在输入框中，如果是则不处理快捷键
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.isContentEditable) {
                return;
            }
            
            // Ctrl/Cmd + 快捷键
            if (e.ctrlKey || e.metaKey) {
                switch (e.key) {
                    case 'o':
                        console.log('🔍 Ctrl+O 快捷键被触发');
                        this.debug('UI_OPERATION', 'Ctrl+O 快捷键触发文件选择');
                        e.preventDefault();
                        document.getElementById('fileInput').click();
                        break;
                    case 'f':
                        e.preventDefault();
                        document.getElementById('searchInput').focus();
                        break;
                    case 'r':
                        e.preventDefault();
                        this.clearSearch();
                        break;
                }
                return; // Ctrl/Cmd组合键处理完毕，不再处理方向键
            }
            
            // 方向键导航 - 只有在没有按Ctrl/Cmd时才处理
            // 重新启用键盘导航，但增加更多安全检查
            if (this.virtualScroll.enabled && this.currentEntries.length > 0) {
                // 只处理特定的导航键
                const navigationKeys = ['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End'];
                if (navigationKeys.includes(e.key)) {
                    // 确保不在滚动过程中触发键盘导航
                    const containers = [document.getElementById('originalLog'), document.getElementById('parsedLog')];
                    const allContainersReady = containers.every(c => c && c._virtualContainer);
                    
                    if (allContainersReady) {
                        this.handleKeyboardNavigation(e);
                    }
                }
            }
        });
        
        // 添加全局点击事件监听器用于调试（可选）
        // document.addEventListener('click', (e) => {
        //     console.log('🐆 全局点击事件:', {
        //         target: e.target.tagName + (e.target.id ? '#' + e.target.id : ''),
        //         className: e.target.className,
        //         textContent: e.target.textContent?.substring(0, 30)
        //     });
        // }, true);
        
    }
    
    setupDragAndDrop() {
        const app = document.getElementById('app');
        
        app.addEventListener('dragover', (e) => {
            e.preventDefault();
            app.classList.add('drag-over');
        });
        
        app.addEventListener('dragleave', (e) => {
            e.preventDefault();
            app.classList.remove('drag-over');
        });
        
        app.addEventListener('drop', (e) => {
            e.preventDefault();
            app.classList.remove('drag-over');
            
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.handleFile(files[0]);
            }
        });
    }
    
    async handleFile(fileOrPath) {
        // 统一处理：支持传入 File 对象或 绝对路径字符串
        const filePath = typeof fileOrPath === 'string' ? fileOrPath : (fileOrPath.path || fileOrPath.name);
        // 使用后端API验证文件
        const validationResult = await this.validateFileWithBackend(filePath);
        if (!validationResult.valid) {
            this.showError(validationResult.error || '不支持的文件格式，请选择 .log 或 .txt 文件');
            return;
        }
        
        // 大文件检测和警告
        const fileSize = validationResult.fileSize || (typeof fileOrPath !== 'string' ? fileOrPath.size : 0);
        const fileSizeMB = (fileSize / (1024 * 1024)).toFixed(2);
        
        this.info('FILE_OPERATION', `文件大小: ${fileSizeMB}MB`, { fileSize, fileSizeMB });
        
        // 大文件警告
        if (fileSize > 100 * 1024 * 1024) { // 100MB
            const confirmed = confirm(
                `正在加载大文件（${fileSizeMB}MB）。\n` +
                `将使用Rust后端的高性能解析和分块加载。\n\n` +
                `继续加载吗？`
            );
            
            if (!confirmed) {
                return;
            }
        }
        
        this.showLoading('正在解析文件...');
        const displayName = typeof fileOrPath === 'string' ? this.basename(filePath) : fileOrPath.name;
        this.info('FILE_OPERATION', `开始处理文件: ${displayName}`, { 
            fileName: displayName, 
            fileSize: fileSize, 
            formattedSize: this.formatFileSize(fileSize) 
        });
        
        const startTime = performance.now();
        
        try {
            // 先初始化文件分块元数据
            this.debug('BACKEND_API', '初始化文件分块...');
            const chunkMetadata = await this.initializeFileChunks(filePath);
            
            if (chunkMetadata) {
                // 使用Rust后端的分块加载
                await this.loadFileWithBackend(chunkMetadata);
            }
            
            const parseTime = performance.now() - startTime;
            this.debugStats.parseCount++;
            this.debugStats.totalParseTime += parseTime;
            
            this.currentFile = { path: filePath, name: displayName, size: fileSize };
            
            this.info('PARSER', `解析成功: ${this.currentEntries.length} 行日志，耗时 ${Math.round(parseTime)}ms`, { 
                entryCount: this.currentEntries.length, 
                parseTime: Math.round(parseTime),
                successCount: this.currentEntries.filter(r => !r.is_error).length,
                errorCount: this.currentEntries.filter(r => r.is_error).length
            });
            
            this.renderResults();
            this.updateStatus(`已加载 ${this.currentEntries.length} 行日志`);
            this.updateFileInfo({ name: displayName, size: fileSize });
            
            this.updateDebugStats();
            
        } catch (error) {
            const parseTime = performance.now() - startTime;
            this.logDebug(`💥 解析异常: ${error.message}，耗时 ${Math.round(parseTime)}ms`, 'error');
            console.error('解析文件失败:', error);
            this.showError(`解析错误: ${error.message}`);
        } finally {
            this.hideLoading();
        }
    }
    
    // ===== Rust后端分块加载API =====
    
    /**
     * 初始化文件分块元数据
     */
    async initializeFileChunks(filePath) {
        try {
            this.debug('BACKEND_API', `初始化文件分块: ${filePath}`);
            const metadata = await this.invokeTauriCommand('initialize_file_chunks', {
                file_path: filePath
            });
            
            this.backendChunkLoader.initialized = true;
            this.backendChunkLoader.currentFileMetadata = metadata;
            this.backendChunkLoader.totalChunks = metadata.total_chunks;
            this.backendChunkLoader.chunkSize = metadata.chunk_size;
            
            this.info('BACKEND_API', `文件分块初始化成功: ${metadata.total_chunks} 块`, metadata);
            return metadata;
        } catch (error) {
            this.error('BACKEND_API', `文件分块初始化失败: ${error.message}`);
            return null;
        }
    }
    
    /**
     * 加载指定的数据块
     */
    async loadChunks(chunkIndices, priority = 'Normal') {
        try {
            const request = {
                file_path: this.backendChunkLoader.currentFileMetadata?.file_path || '',
                chunk_indices: chunkIndices,
                plugin_name: this.currentPlugin,
                priority: priority
            };
            
            this.debug('BACKEND_API', `加载数据块: [${chunkIndices.join(', ')}]`);
            const response = await this.invokeTauriCommand('load_chunks', request);
            
            if (response.success) {
                // 更新已加载块集合
                chunkIndices.forEach(index => {
                    this.backendChunkLoader.loadedChunks.add(index);
                });
                
                this.info('BACKEND_API', `数据块加载成功: ${Object.keys(response.chunks).length} 块`);
                return response;
            } else {
                throw new Error(response.error || '数据块加载失败');
            }
        } catch (error) {
            this.error('BACKEND_API', `数据块加载失败: ${error.message}`);
            throw error;
        }
    }
    
    /**
     * 使用Rust后端加载文件
     */
    async loadFileWithBackend(chunkMetadata) {
        this.info('BACKEND_API', '使用Rust后端分块加载文件');
        
        // 先加载第一个块以快速显示内容
        const firstChunkResponse = await this.loadChunks([0], 'Immediate');
        
        if (firstChunkResponse.chunks[0]) {
            this.currentEntries = firstChunkResponse.chunks[0];
            this.renderResults();
            this.updateStatus(`已加载第1块，总共${chunkMetadata.total_chunks}块`);
        }
        
        // 预加载接下来的几个块
        if (chunkMetadata.total_chunks > 1) {
            const preloadChunks = [];
            for (let i = 1; i < Math.min(6, chunkMetadata.total_chunks); i++) {
                preloadChunks.push(i);
            }
            
            if (preloadChunks.length > 0) {
                this.loadChunks(preloadChunks, 'High').catch(error => {
                    this.warn('BACKEND_API', `预加载块失败: ${error.message}`);
                });
            }
        }
        
        // 初始化虚拟滚动，并设置需要时加载其他块
        if (this.virtualScroll.enabled) {
            this.setupBackendVirtualScroll(chunkMetadata);
        }
    }
    
    /**
     * 设置后端支持的虚拟滚动
     */
    setupBackendVirtualScroll(chunkMetadata) {
        this.virtualScroll.totalItems = chunkMetadata.total_lines;
        
        // 设置滚动监听，在需要时加载对应的块
        this.setupScrollListener(() => {
            this.checkAndLoadRequiredChunksFromBackend();
        });
    }
    
    /**
     * 检查并加载需要的数据块（后端版本）
     */
    async checkAndLoadRequiredChunksFromBackend() {
        if (!this.backendChunkLoader.initialized) return;
        
        const startChunk = Math.floor(this.virtualScroll.startIndex / this.backendChunkLoader.chunkSize);
        const endChunk = Math.floor(this.virtualScroll.endIndex / this.backendChunkLoader.chunkSize);
        
        const chunksToLoad = [];
        for (let chunkIndex = startChunk; chunkIndex <= endChunk; chunkIndex++) {
            if (!this.backendChunkLoader.loadedChunks.has(chunkIndex)) {
                chunksToLoad.push(chunkIndex);
            }
        }
        
        if (chunksToLoad.length > 0) {
            this.debug('BACKEND_API', `需要加载的数据块: [${chunksToLoad.join(', ')}]`);
            try {
                const response = await this.loadChunks(chunksToLoad, 'Normal');
                
                // 将新加载的数据合并到当前条目中
                this.mergeChunksToCurrentEntries(response.chunks);
                this.renderVisibleItems();
            } catch (error) {
                this.error('BACKEND_API', `动态加载数据块失败: ${error.message}`);
            }
        }
    }
    
    /**
     * 将新加载的数据块合并到当前条目
     */
    mergeChunksToCurrentEntries(chunks) {
        Object.entries(chunks).forEach(([chunkIndex, chunkData]) => {
            const startIndex = parseInt(chunkIndex) * this.backendChunkLoader.chunkSize;
            chunkData.forEach((entry, index) => {
                this.currentEntries[startIndex + index] = entry;
            });
        });
    }
    
    /**
     * 获取内存信息（后端API）
     */
    async getMemoryInfoFromBackend() {
        try {
            const memoryInfo = await this.invokeTauriCommand('get_memory_info', {});
            this.debug('BACKEND_API', '获取内存信息成功', memoryInfo);
            return memoryInfo;
        } catch (error) {
            this.error('BACKEND_API', `获取内存信息失败: ${error.message}`);
            return null;
        }
    }
    
    /**
     * 清理内存（后端API）
     */
    async cleanupMemoryViaBackend() {
        try {
            this.info('BACKEND_API', '开始后端内存清理...');
            const cleanedCount = await this.invokeTauriCommand('cleanup_memory', {});
            
            this.info('BACKEND_API', `后端内存清理完成: 清理了 ${cleanedCount} 个块`);
            this.showToast(`内存清理完成，清理了 ${cleanedCount} 个数据块`);
            
            // 更新性能统计
            this.updatePerformanceStatsFromBackend();
        } catch (error) {
            this.error('BACKEND_API', `后端内存清理失败: ${error.message}`);
            this.showToast('内存清理失败，请查看控制台');
        }
    }
    
    /**
     * 从后端更新性能统计
     */
    async updatePerformanceStatsFromBackend() {
        const memoryInfo = await this.getMemoryInfoFromBackend();
        if (memoryInfo) {
            // 更新性能面板显示
            document.getElementById('memoryUsage').textContent = 
                this.formatFileSize(memoryInfo.current_usage);
            document.getElementById('cachedChunks').textContent = 
                `${memoryInfo.cached_chunks} / ${memoryInfo.max_cached_chunks}`;
            document.getElementById('gcCount').textContent = memoryInfo.gc_count;
        }
    }
    
    // ===== 结束 Rust后端分块加载API =====
    
    /**
     * 使用后端API验证文件
     */
    async validateFileWithBackend(filePath) {
        try {
            this.debug('FILE_VALIDATION', `使用后端API验证文件: ${filePath}`);
            
            const response = await this.invokeTauriCommand('validate_file', {
                file_path: filePath
            });
            
            if (response.valid) {
                this.debug('FILE_VALIDATION', `后端验证通过: ${filePath} (${this.formatFileSize(response.file_size || 0)})`);
            return {
                    valid: true,
                    fileSize: response.file_size || 0,
                    fileType: response.file_type
                };
            } else {
                this.warn('FILE_VALIDATION', `后端验证失败: ${response.error}`);
            return {
                    valid: false,
                    error: response.error
                };
            }
        } catch (error) {
            this.error('FILE_VALIDATION', `后端验证API调用失败: ${error.message}`);
            return { valid: false, error: error.message };
        }
    }
    
    /**
     * 前端文件验证（备用方案）
     */
    isValidFile(file) {
        if (!file) {
            return false;
        }
        
        // 检查文件类型
        const validExtensions = ['.log', '.txt'];
        const fileName = file.name.toLowerCase();
        const hasValidExtension = validExtensions.some(ext => fileName.endsWith(ext));
        
        if (!hasValidExtension) {
            this.debug('FILE_VALIDATION', `文件扩展名无效: ${file.name}`);
            return false;
        }
        
        // 检查文件大小（可选）
        const maxSize = 1024 * 1024 * 1024; // 1GB
        if (file.size > maxSize) {
            this.warn('FILE_VALIDATION', `文件过大: ${this.formatFileSize(file.size)}`);
            // 不直接拒绝，让用户决定
        }
        
        this.debug('FILE_VALIDATION', `前端验证通过: ${file.name} (${this.formatFileSize(file.size)})`);
        return true;
    }
    
    async invokeTauriCommand(command, args) {
        // 检查 Tauri 环境
        if (typeof window.__TAURI__ !== 'undefined' && window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) {
            try {
                return await window.__TAURI__.tauri.invoke(command, args);
            } catch (error) {
                console.error(`Tauri 命令执行失败: ${command}`, error);
                throw error;
            }
        }
        
        // 如果不在 Tauri 环境中，提供更友好的错误信息
        const message = '未检测到 Tauri 运行环境。请使用以下方式启动应用：\n\n1. 运行 `cargo tauri dev` 启动开发模式\n2. 运行 `cargo tauri build` 构建应用\n3. 不要直接用浏览器打开 index.html';
        console.error(message);
        this.showError(message);
        throw new Error(message);
    }
    
    
    renderResults() {
        if (this.virtualScroll.enabled) {
            this.renderVirtualScroll();
        } else {
        this.renderOriginalLog();
        this.renderParsedLog();
        }
    }
    
    renderOriginalLog() {
        const container = document.getElementById('originalLog');
        container.innerHTML = '';
        
        if (this.currentEntries.length === 0) {
            container.innerHTML = `
                <div class="text-gray-500 text-center py-8">
                    <div class="text-4xl mb-4">📄</div>
                    <p>没有日志数据</p>
                </div>
            `;
            return;
        }
        
        this.currentEntries.forEach(entry => {
            const div = document.createElement('div');
            div.className = `log-line ${this.getLogLevelClass(entry.original.level)} fade-in`;
            div.innerHTML = `
                <div class="text-xs text-gray-500 mb-1">第 ${entry.original.line_number} 行</div>
                <div class="font-mono text-sm">${this.escapeHtml(entry.original.content)}</div>
            `;
            container.appendChild(div);
        });
    }
    
    renderParsedLog() {
        const container = document.getElementById('parsedLog');
        container.innerHTML = '';
        
        if (this.currentEntries.length === 0) {
            container.innerHTML = `
                <div class="text-gray-500 text-center py-8">
                    <div class="text-4xl mb-4">🔍</div>
                    <p>没有解析结果</p>
                </div>
            `;
            return;
        }
        
        this.currentEntries.forEach(entry => {
            if (entry.rendered_blocks.length === 0) return;
            
            const div = document.createElement('div');
            div.className = `mb-4 ${entry.is_error ? 'bg-red-50' : entry.is_warning ? 'bg-yellow-50' : ''} fade-in`;
            
            let html = `<div class="text-xs text-gray-500 mb-2">第 ${entry.original.line_number} 行</div>`;
            
            entry.rendered_blocks.forEach(block => {
                html += this.renderBlock(block);
            });
            
            div.innerHTML = html;
            container.appendChild(div);
        });
    }
    
    renderBlock(block) {
        const blockClass = this.getBlockClass(block.block_type);
        const icon = this.getBlockIcon(block.block_type);
        
        return `
            <div class="rendered-block ${blockClass} slide-in">
                <div class="block-header">
                    <div class="block-title">
                        <span>${icon}</span>
                        <span>${block.title}</span>
                    </div>
                    <div class="block-actions">
                        <button onclick="app.copyToClipboard('${block.id}')" 
                                class="copy-btn">
                            复制
                        </button>
                    </div>
                </div>
                <div class="block-content">${this.escapeHtml(block.formatted_content)}</div>
            </div>
        `;
    }
    
    getBlockClass(blockType) {
        const classes = {
            'Sql': 'sql',
            'Json': 'json',
            'Error': 'error',
            'Warning': 'warning',
            'Info': 'info',
            'Raw': 'raw'
        };
        return classes[blockType] || 'raw';
    }
    
    getBlockIcon(blockType) {
        const icons = {
            'Sql': '🔍',
            'Json': '📄',
            'Error': '⚠️',
            'Warning': '⚠️',
            'Info': 'ℹ️',
            'Raw': '📝'
        };
        return icons[blockType] || '📝';
    }
    
    getLogLevelClass(level) {
        const classes = {
            'Error': 'error',
            'Warn': 'warning',
            'Info': 'info',
            'Debug': 'debug'
        };
        return classes[level] || 'debug';
    }
    
    async copyToClipboard(blockId) {
        try {
            const block = this.findBlockById(blockId);
            if (block) {
                if (navigator.clipboard) {
                    await navigator.clipboard.writeText(block.formatted_content);
                } else {
                    // 降级方案
                    const textArea = document.createElement('textarea');
                    textArea.value = block.formatted_content;
                    document.body.appendChild(textArea);
                    textArea.select();
                    document.execCommand('copy');
                    document.body.removeChild(textArea);
                }
                this.showToast('已复制到剪贴板');
            }
        } catch (error) {
            console.error('复制失败:', error);
            this.showToast('复制失败', 'error');
        }
    }
    
    findBlockById(blockId) {
        for (const entry of this.currentEntries) {
            for (const block of entry.rendered_blocks) {
                if (block.id === blockId) {
                    return block;
                }
            }
        }
        return null;
    }
    
    switchPlugin(pluginName) {
        const oldPlugin = this.currentPlugin;
        this.currentPlugin = pluginName;
        
        this.info('PLUGIN', `切换到插件: ${pluginName}`, { 
            from: oldPlugin, 
            to: pluginName 
        });
        
        // 如果有当前文件，重新解析
        if (this.currentFile) {
            this.handleFile(this.currentFile);
        }
    }
    
    filterLogs() {
        const originalContainer = document.getElementById('originalLog');
        const parsedContainer = document.getElementById('parsedLog');
        
        if (!this.searchTerm.trim()) {
            // 显示所有日志
            this.showAllLogs();
            return;
        }
        
        const searchTerm = this.searchTerm.toLowerCase();
        const originalLogs = originalContainer.querySelectorAll('.log-line');
        const parsedLogs = parsedContainer.querySelectorAll('.mb-4');
        
        originalLogs.forEach(log => {
            const content = log.textContent.toLowerCase();
            if (content.includes(searchTerm)) {
                log.style.display = 'block';
                this.highlightSearchTerm(log, searchTerm);
            } else {
                log.style.display = 'none';
            }
        });
        
        parsedLogs.forEach(log => {
            const content = log.textContent.toLowerCase();
            if (content.includes(searchTerm)) {
                log.style.display = 'block';
                this.highlightSearchTerm(log, searchTerm);
            } else {
                log.style.display = 'none';
            }
        });
    }
    
    showAllLogs() {
        const originalLogs = document.querySelectorAll('#originalLog .log-line');
        const parsedLogs = document.querySelectorAll('#parsedLog .mb-4');
        
        originalLogs.forEach(log => {
            log.style.display = 'block';
            this.removeSearchHighlight(log);
        });
        
        parsedLogs.forEach(log => {
            log.style.display = 'block';
            this.removeSearchHighlight(log);
        });
    }
    
    highlightSearchTerm(element, searchTerm) {
        this.removeSearchHighlight(element);
        
        const walker = document.createTreeWalker(
            element,
            NodeFilter.SHOW_TEXT,
            null,
            false
        );
        
        const textNodes = [];
        let node;
        while (node = walker.nextNode()) {
            textNodes.push(node);
        }
        
        textNodes.forEach(textNode => {
            const text = textNode.textContent;
            const regex = new RegExp(`(${searchTerm})`, 'gi');
            const highlightedText = text.replace(regex, '<span class="search-highlight">$1</span>');
            
            if (highlightedText !== text) {
                const wrapper = document.createElement('div');
                wrapper.innerHTML = highlightedText;
                textNode.parentNode.replaceChild(wrapper, textNode);
            }
        });
    }
    
    removeSearchHighlight(element) {
        const highlights = element.querySelectorAll('.search-highlight');
        highlights.forEach(highlight => {
            const parent = highlight.parentNode;
            parent.replaceChild(document.createTextNode(highlight.textContent), highlight);
            parent.normalize();
        });
    }
    
    clearSearch() {
        document.getElementById('searchInput').value = '';
        this.searchTerm = '';
        this.showAllLogs();
    }
    
    showLoading(message) {
        this.isLoading = true;
        document.getElementById('loadingText').textContent = message;
        document.getElementById('loadingOverlay').classList.remove('hidden');
    }
    
    hideLoading() {
        this.isLoading = false;
        document.getElementById('loadingOverlay').classList.add('hidden');
    }
    
    showError(message) {
        this.updateStatus(`错误: ${message}`);
        this.showToast(message, 'error');
    }
    
    showToast(message, type = 'success') {
        const toast = document.getElementById('toast');
        const toastMessage = document.getElementById('toastMessage');
        
        toastMessage.textContent = message;
        toast.className = `fixed top-4 right-4 px-4 py-2 rounded shadow-lg z-50 ${
            type === 'error' ? 'bg-red-500' : 'bg-green-500'
        } text-white`;
        
        toast.classList.remove('hidden');
        
        setTimeout(() => {
            toast.classList.add('hidden');
        }, 3000);
    }
    
    updateStatus(message) {
        document.getElementById('statusText').textContent = message;
    }
    
    updateFileInfo(file) {
        const fileInfo = document.getElementById('fileInfo');
        const size = this.formatFileSize(file.size);
        fileInfo.textContent = `${file.name} (${size})`;
    }
    
    formatFileSize(bytes) {
        const units = ['B', 'KB', 'MB', 'GB'];
        let size = bytes;
        let unitIndex = 0;
        
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        
        return `${size.toFixed(1)} ${units[unitIndex]}`;
    }

    basename(path) {
        if (!path) return '';
        const sep = path.includes('\\') ? '\\' : '/';
        const parts = path.split(sep);
        return parts[parts.length - 1];
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    // 主题管理方法
    initTheme() {
        // 从本地存储读取主题设置
        const savedTheme = localStorage.getItem('logwhisper-theme');
        if (savedTheme) {
            this.currentTheme = savedTheme;
        } else {
            // 检测系统主题偏好
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            this.currentTheme = prefersDark ? 'dark' : 'light';
        }
        
        this.applyTheme();
    }
    
    toggleTheme() {
        const oldTheme = this.currentTheme;
        this.currentTheme = this.currentTheme === 'light' ? 'dark' : 'light';
        this.applyTheme();
        
        // 保存到本地存储
        localStorage.setItem('logwhisper-theme', this.currentTheme);
        
        // 显示切换提示
        this.showToast(`已切换到${this.currentTheme === 'light' ? '亮色' : '暗色'}主题`);
        
        this.info('UI_OPERATION', '主题切换', { 
            from: oldTheme, 
            to: this.currentTheme 
        });
    }
    
    applyTheme() {
        const body = document.body;
        const themeIcon = document.getElementById('themeIcon');
        
        if (this.currentTheme === 'dark') {
            body.setAttribute('data-theme', 'dark');
            themeIcon.textContent = '☀️';
        } else {
            body.setAttribute('data-theme', 'light');
            themeIcon.textContent = '🌙';
        }
    }
    
    // 调试面板管理
    toggleDebugPanel() {
        const debugPanel = document.getElementById('debugPanel');
        this.debugMode = !this.debugMode;
        
        if (this.debugMode) {
            debugPanel.classList.remove('hidden');
            this.info('UI_OPERATION', '调试面板已打开');
            this.updateDebugStats();
        } else {
            debugPanel.classList.add('hidden');
            this.info('UI_OPERATION', '调试面板已关闭');
        }
    }
    
    logDebug(message, type = 'info') {
        if (!this.debugMode) return;
        
        const debugLogs = document.getElementById('debugLogs');
        const timestamp = new Date().toLocaleTimeString();
        const logEntry = document.createElement('div');
        
        const colors = {
            info: 'text-blue-400',
            warn: 'text-yellow-400', 
            error: 'text-red-400',
            success: 'text-green-400'
        };
        
        logEntry.className = `${colors[type]} mb-1`;
        logEntry.innerHTML = `[${timestamp}] ${message}`;
        
        debugLogs.appendChild(logEntry);
        debugLogs.scrollTop = debugLogs.scrollHeight;
        
        // 限制日志条数
        const logs = debugLogs.children;
        if (logs.length > 100) {
            debugLogs.removeChild(logs[0]);
        }
    }
    
    updateDebugStats() {
        if (!this.debugMode) return;
        
        document.getElementById('parseCount').textContent = this.debugStats.parseCount;
        document.getElementById('avgParseTime').textContent = 
            this.debugStats.parseCount > 0 ? 
            Math.round(this.debugStats.totalParseTime / this.debugStats.parseCount) + 'ms' : 
            '0ms';
        
        // 模拟内存使用
        if (performance.memory) {
            const memoryMB = Math.round(performance.memory.usedJSHeapSize / 1024 / 1024);
            document.getElementById('memoryUsage').textContent = memoryMB + 'MB';
        }
        
        // 计算缓存命中率
        const totalCache = this.debugStats.cacheHits + this.debugStats.cacheMisses;
        const hitRate = totalCache > 0 ? 
            Math.round((this.debugStats.cacheHits / totalCache) * 100) : 0;
        document.getElementById('cacheHitRate').textContent = hitRate + '%';
    }
    
    clearDebugLogs() {
        const debugLogs = document.getElementById('debugLogs');
        debugLogs.innerHTML = '<div class="text-gray-500">日志已清空</div>';
        this.logDebug('🧹 调试日志已清空');
    }
    
    exportDebugLogs() {
        const debugLogs = document.getElementById('debugLogs');
        const logs = Array.from(debugLogs.children).map(log => log.textContent).join('\n');
        
        const blob = new Blob([logs], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `logwhisper-debug-${new Date().toISOString().slice(0, 19)}.log`;
        a.click();
        URL.revokeObjectURL(url);
        
        this.logDebug('📁 调试日志已导出');
    }
    
    async runPerformanceTest() {
        this.logDebug('🚀 开始性能测试...', 'info');
        
        const testData = this.generateRealisticLogData();
        const iterations = 10;
        const startTime = performance.now();
        
        for (let i = 0; i < iterations; i++) {
            // 模拟解析过程
            await new Promise(resolve => setTimeout(resolve, 10));
            this.debugStats.parseCount++;
        }
        
        const endTime = performance.now();
        const totalTime = endTime - startTime;
        const avgTime = totalTime / iterations;
        
        this.logDebug(`✅ 性能测试完成: ${iterations}次迭代, 总耗时${Math.round(totalTime)}ms, 平均${Math.round(avgTime)}ms/次`, 'success');
        
        this.updateDebugStats();
    }
    
    // 性能面板管理
    togglePerformancePanel() {
        const performancePanel = document.getElementById('performancePanel');
        this.performanceMode = !this.performanceMode;
        
        if (this.performanceMode) {
            performancePanel.classList.remove('hidden');
            this.logDebug('⚡ 性能面板已打开');
            this.updatePerformanceStats();
        } else {
            performancePanel.classList.add('hidden');
            this.logDebug('⚡ 性能面板已关闭');
        }
    }
    
    updatePerformanceStats() {
        if (!this.performanceMode) return;
        
        // 更新虚拟滚动统计
        document.getElementById('visibleItems').textContent = this.virtualScroll.endIndex - this.virtualScroll.startIndex;
        document.getElementById('totalItems').textContent = this.currentEntries.length;
        
        // 更新分块加载统计
        document.getElementById('loadedChunks').textContent = this.chunkLoading.loadedChunks.size;
        document.getElementById('totalChunks').textContent = this.chunkLoading.totalChunks;
        
        // 更新内存使用
        if (performance.memory) {
            const memoryMB = Math.round(performance.memory.usedJSHeapSize / 1024 / 1024);
            document.getElementById('memoryUsage').textContent = memoryMB + 'MB';
        }
        
        // 更新性能指标
        document.getElementById('parseCount').textContent = this.debugStats.parseCount;
        document.getElementById('avgParseTime').textContent = 
            this.debugStats.parseCount > 0 ? 
            Math.round(this.debugStats.totalParseTime / this.debugStats.parseCount) + 'ms' : 
            '0ms';
        
        const totalCache = this.debugStats.cacheHits + this.debugStats.cacheMisses;
        const hitRate = totalCache > 0 ? 
            Math.round((this.debugStats.cacheHits / totalCache) * 100) : 0;
        document.getElementById('cacheHitRate').textContent = hitRate + '%';
    }
    
    cleanupMemory() {
        this.logDebug('🧹 开始内存清理...');
        
        // 清理不可见数据
        this.cleanupInvisibleData();
        
        // 强制垃圾回收（如果可用）
        if (window.gc) {
            window.gc();
        }
        
        // 清理事件监听器
        this.cleanupEventListeners();
        
        this.logDebug('✅ 内存清理完成');
        this.updatePerformanceStats();
        this.showToast('内存清理完成');
    }
    
    cleanupEventListeners() {
        // 清理可能的内存泄漏
        const containers = document.querySelectorAll('.virtual-scroll-container');
        containers.forEach(container => {
            // 移除旧的事件监听器
            const newContainer = container.cloneNode(true);
            container.parentNode.replaceChild(newContainer, container);
        });
    }
    
    // 虚拟滚动实现
    renderVirtualScroll() {
        this.debug('VIRTUAL_SCROLL', '启用虚拟滚动渲染');
        
        // 计算总高度
        const totalHeight = this.currentEntries.length * this.virtualScroll.itemHeight;
        
        // 设置容器高度
        const originalContainer = document.getElementById('originalLog');
        const parsedContainer = document.getElementById('parsedLog');
        
        this.setupVirtualScrollContainer(originalContainer, totalHeight, 'original');
        this.setupVirtualScrollContainer(parsedContainer, totalHeight, 'parsed');
        
        // 初始渲染
        this.updateVirtualScroll();
        
        // 绑定滚动事件
        this.bindScrollEvents();
    }
    
    setupVirtualScrollContainer(container, totalHeight, type) {
        this.debug('VIRTUAL_SCROLL', `初始化虚拟滚动容器: ${type}, 总高度: ${totalHeight}px`);
        
        // 检查容器尺寸
        const containerRect = container.getBoundingClientRect();
        const actualHeight = Math.max(containerRect.height, 400); // 使用实际高度或最小高度
        this.debug('VIRTUAL_SCROLL', `容器 ${type} 实际尺寸: ${containerRect.width}x${containerRect.height}, 使用高度: ${actualHeight}`);
        
        // 创建虚拟滚动容器
        const virtualContainer = document.createElement('div');
        virtualContainer.className = 'virtual-scroll-container';
        virtualContainer.style.cssText = `
            height: ${actualHeight}px;
            overflow-y: auto;
            position: relative;
            background-color: var(--bg-content);
            border: 1px solid var(--border-primary);
            display: block;
        `;
        
        // 创建占位容器
        const placeholder = document.createElement('div');
        placeholder.className = 'virtual-scroll-placeholder';
        placeholder.style.cssText = `
            height: ${totalHeight}px;
            position: relative;
            background-color: transparent;
            width: 100%;
            display: block;
        `;
        
        // 创建可见内容容器
        const visibleContainer = document.createElement('div');
        visibleContainer.className = 'virtual-scroll-visible';
        visibleContainer.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            width: 100%;
            background-color: transparent;
            display: block;
            z-index: 1;
        `;
        
        placeholder.appendChild(visibleContainer);
        virtualContainer.appendChild(placeholder);
        
        // 清空原容器并添加虚拟滚动容器
        container.innerHTML = '';
        container.appendChild(virtualContainer);
        
        // 存储引用
        container._virtualContainer = virtualContainer;
        container._placeholder = placeholder;
        container._visibleContainer = visibleContainer;
        container._type = type;
        container._actualHeight = actualHeight; // 存储实际高度
        
        // 更新虚拟滚动参数
        if (type === 'original') {
            // 只在第一个容器初始化时更新
            this.virtualScroll.containerHeight = actualHeight;
        }
        
        // 检查创建结果
        this.debug('VIRTUAL_SCROLL', `虚拟容器创建完成: ${type}`);
        this.debug('VIRTUAL_SCROLL', `虚拟容器尺寸: ${virtualContainer.offsetWidth}x${virtualContainer.offsetHeight}`);
        this.debug('VIRTUAL_SCROLL', `占位器高度: ${placeholder.offsetHeight}px`);
    }
    
    bindScrollEvents() {
        const originalContainer = document.getElementById('originalLog');
        const parsedContainer = document.getElementById('parsedLog');
        
        // 防抖处理滚动事件
        let scrollTimeout;
        let isScrolling = false;
        
        const handleScroll = (e) => {
            this.virtualScroll.scrollTop = e.target.scrollTop;
            isScrolling = true;
            
            // 清除之前的定时器
            if (scrollTimeout) {
                clearTimeout(scrollTimeout);
            }
            
            // 立即更新虚拟滚动
            this.updateVirtualScroll();
            
            // 设置防抖，避免频繁更新
            scrollTimeout = setTimeout(() => {
                isScrolling = false;
                this.updateVirtualScroll();
            }, 16); // 约60fps
        };
        
        // 添加滚动开始和结束事件
        const handleScrollStart = () => {
            this.logDebug('🔄 滚动开始');
            isScrolling = true;
        };
        
        const handleScrollEnd = () => {
            this.logDebug('🔄 滚动结束');
            isScrolling = false;
            this.updateVirtualScroll();
        };
        
        originalContainer._virtualContainer.addEventListener('scroll', handleScroll);
        parsedContainer._virtualContainer.addEventListener('scroll', handleScroll);
        
        // 添加滚动开始和结束事件
        originalContainer._virtualContainer.addEventListener('scrollstart', handleScrollStart);
        originalContainer._virtualContainer.addEventListener('scrollend', handleScrollEnd);
        parsedContainer._virtualContainer.addEventListener('scrollstart', handleScrollStart);
        parsedContainer._virtualContainer.addEventListener('scrollend', handleScrollEnd);
        
        // 同步滚动
        originalContainer._virtualContainer.addEventListener('scroll', (e) => {
            if (parsedContainer._virtualContainer.scrollTop !== e.target.scrollTop) {
                parsedContainer._virtualContainer.scrollTop = e.target.scrollTop;
            }
        });
        
        parsedContainer._virtualContainer.addEventListener('scroll', (e) => {
            if (originalContainer._virtualContainer.scrollTop !== e.target.scrollTop) {
                originalContainer._virtualContainer.scrollTop = e.target.scrollTop;
            }
        });
    }
    
    updateVirtualScroll() {
        const scrollTop = this.virtualScroll.scrollTop;
        const containerHeight = this.virtualScroll.containerHeight || 400; // 使用实际容器高度
        
        // 计算可见范围，使用更精确的计算
        const itemHeight = this.virtualScroll.itemHeight;
        const startIndex = Math.floor(scrollTop / itemHeight);
        const visibleItemCount = Math.ceil(containerHeight / itemHeight);
        const endIndex = Math.min(
            startIndex + visibleItemCount + this.virtualScroll.bufferSize * 2, // 双向缓冲
            this.currentEntries.length
        );
        
        const newStartIndex = Math.max(0, startIndex - this.virtualScroll.bufferSize);
        const newEndIndex = endIndex;
        
        // 检查是否需要更新渲染范围
        if (newStartIndex !== this.virtualScroll.startIndex || newEndIndex !== this.virtualScroll.endIndex) {
            this.virtualScroll.startIndex = newStartIndex;
            this.virtualScroll.endIndex = newEndIndex;
            
            this.debug('VIRTUAL_SCROLL', `虚拟滚动更新: 显示 ${this.virtualScroll.startIndex}-${this.virtualScroll.endIndex} / ${this.currentEntries.length}`);
            this.debug('VIRTUAL_SCROLL', `滚动位置: ${scrollTop}px, 容器高度: ${containerHeight}px, 项目高度: ${itemHeight}px, 可见项目数: ${visibleItemCount}`);
            
            // 检查并加载需要的数据块
            this.loadRequiredChunks();
            
            // 管理滚动数据
            this.manageScrollData();
            
            // 渲染可见项目
            this.renderVisibleItems();
            
            // 确保滚动同步
            this.ensureScrollSync();
        }
    }
    
    // ==================== 日志系统 ====================
    
    // 应用日志记录
    appLog(level, module, message, data) {
        if (data === undefined) data = null;
        var timestamp = new Date().toISOString();
        var logEntry = {
            timestamp: timestamp,
            level: level,
            module: module,
            message: message,
            data: data,
            id: Math.random().toString(36).substr(2, 9)
        };
        
        // 检查日志级别
        var logLevels = { DEBUG: 0, INFO: 1, WARN: 2, ERROR: 3 };
        if (logLevels[level] < logLevels[this.logger.level]) {
            return;
        }
        
        // 添加到内存日志
        this.logger.logs.push(logEntry);
        
        // 限制内存中的日志数量
        if (this.logger.logs.length > this.logger.maxMemoryLogs) {
            this.logger.logs.shift();
        }
        
        // 格式化日志消息
        var time = timestamp.split('T')[1].split('.')[0];
        var formattedMessage = '[' + time + '] [' + level + '] [' + module + '] ' + message;
        
        // 控制台输出
        if (this.logger.console) {
            var styles = {
                DEBUG: 'color: #6B7280; background: #F3F4F6;',
                INFO: 'color: #3B82F6; background: #EFF6FF;',
                WARN: 'color: #F59E0B; background: #FFFBEB;',
                ERROR: 'color: #EF4444; background: #FEF2F2;'
            };
            console.log('%c' + formattedMessage, styles[level] || styles.DEBUG);
            if (data) {
                console.log('数据:', data);
            }
        }
        
        // 文件输出
        if (this.logger.file) {
            this.fileLog(logEntry);
        }
        
        // 更新调试面板
        this.updateDebugLogs(logEntry);
    }
    
    // 文件日志 - 直接写入，无缓冲区
    fileLog(logEntry) {
        var self = this;
        try {
            // 格式化单条日志
            var logText = logEntry.timestamp + ' [' + logEntry.level + '] [' + logEntry.module + '] ' + logEntry.message + 
                         (logEntry.data ? '\n数据: ' + JSON.stringify(logEntry.data, null, 2) : '') + '\n';
            
            // 检查是否在 Tauri 环境中
            if (window.__TAURI__ && window.__TAURI__.invoke) {
                // 调用Tauri命令直接写入文件
                window.__TAURI__.invoke('write_log', { 
                    content: logText,
                    append: true 
                }).then(function(response) {
                    // 成功写入，静默处理
                    // console.log('日志已写入文件'); // 移除这行避免过多输出
                }).catch(function(error) {
                    console.error('文件日志写入失败:', error);
                    // 备用方案：输出到控制台
                    console.log('备用日志输出:', logText);
                });
            } else {
                // 浏览器环境或开发环境 - 模拟文件写入
                // console.log('开发环境日志:', logText.trim()); // 移除频繁的控制台输出
                
                // 使用更智能的缓冲区管理
                if (this.logger.fileBuffer === undefined) {
                    this.logger.fileBuffer = [];
                    this.logger.lastFlushTime = Date.now();
                }
                
                this.logger.fileBuffer.push(logText);
                
                // 改进缓冲区刷新策略：基于时间和数量双重条件
                const now = Date.now();
                const timeSinceLastFlush = now - (this.logger.lastFlushTime || 0);
                const bufferFull = this.logger.fileBuffer.length >= 500; // 增加缓冲区大小
                const timeExpired = timeSinceLastFlush > 30000; // 30秒自动刷新一次
                
                // 只有在缓冲区满了或者长时间未刷新时才触发
                if (bufferFull || timeExpired) {
                    this.flushLogBuffer();
                    this.logger.lastFlushTime = now;
                }
            }
        } catch (error) {
            console.error('文件日志写入失败:', error);
            // 备用方案：输出到控制台
            console.log('备用日志输出:', logText);
        }
    }
    
    // 刷新日志缓冲区（浏览器环境）
    flushLogBuffer() {
        if (!this.logger.fileBuffer || this.logger.fileBuffer.length === 0) {
            return;
        }
        
        // 防止重复刷新（防抖）
        if (this.logger.isFlushingBuffer) {
            return;
        }
        
        this.logger.isFlushingBuffer = true;
        
        try {
            const logContent = this.logger.fileBuffer.join('');
            const blob = new Blob([logContent], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `logwhisper-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.log`;
            a.style.display = 'none';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            
            URL.revokeObjectURL(url);
            
            // 清空缓冲区
            this.logger.fileBuffer = [];
            
            console.log('日志缓冲区已刷新到文件');
        } catch (error) {
            console.error('刷新日志缓冲区失败:', error);
        } finally {
            // 延迟重置状态，防止频繁触发
            setTimeout(() => {
                this.logger.isFlushingBuffer = false;
            }, 2000); // 2秒防抖
        }
    }
    
    // 更新调试面板日志
    updateDebugLogs(logEntry) {
        var debugLogs = document.getElementById('debugLogs');
        if (!debugLogs) return;
        
        var logElement = document.createElement('div');
        logElement.className = 'debug-log-item ' + logEntry.level.toLowerCase();
        logElement.innerHTML = 
            '<span class="log-time">' + logEntry.timestamp.split('T')[1].split('.')[0] + '</span>' +
            '<span class="log-level">[' + logEntry.level + ']</span>' +
            '<span class="log-module">[' + logEntry.module + ']</span>' +
            '<span class="log-message">' + logEntry.message + '</span>';
        
        debugLogs.appendChild(logElement);
        
        var maxDebugLogs = 100;
        while (debugLogs.children.length > maxDebugLogs) {
            debugLogs.removeChild(debugLogs.firstChild);
        }
        
        debugLogs.scrollTop = debugLogs.scrollHeight;
    }
    
    // 便捷的日志方法
    debug(module, message, data) {
        if (data === undefined) data = null;
        this.appLog('DEBUG', module, message, data);
    }
    
    info(module, message, data) {
        if (data === undefined) data = null;
        this.appLog('INFO', module, message, data);
    }
    
    warn(module, message, data) {
        if (data === undefined) data = null;
        this.appLog('WARN', module, message, data);
    }
    
    error(module, message, data) {
        if (data === undefined) data = null;
        this.appLog('ERROR', module, message, data);
    }
    
    // 导出日志
    exportLogs() {
        var logs = this.logger.logs;
        var logText = logs.map(function(log) {
            return log.timestamp + ' [' + log.level + '] [' + log.module + '] ' + log.message + 
                   (log.data ? '\n数据: ' + JSON.stringify(log.data, null, 2) : '');
        }).join('\n');
        
        var blob = new Blob([logText], { type: 'text/plain' });
        var url = URL.createObjectURL(blob);
        var a = document.createElement('a');
        a.href = url;
        a.download = 'logwhisper_logs_' + new Date().toISOString().split('T')[0] + '.txt';
        a.click();
        URL.revokeObjectURL(url);
        
        this.info('LOGGER', '日志已导出');
    }
    
    // 手动刷新日志缓冲区
    manualFlushLogs() {
        if (window.__TAURI__ && window.__TAURI__.invoke) {
            // Tauri环境下，日志已经实时写入文件
            this.showToast('日志已实时写入到 logs/ 目录');
            this.info('LOGGER', '日志实时写入已启用，检查 logs/ 目录');
        } else {
            // 浏览器环境下，检查是否有日志需要刷新
            if (!this.logger.fileBuffer || this.logger.fileBuffer.length === 0) {
                this.showToast('暂无日志内容需要刷新');
                return;
            }
            
            // 浏览器环境下，手动触发缓冲区刷新
            this.flushLogBuffer();
            this.showToast(`日志缓冲区已刷新，导出 ${this.logger.fileBuffer.length} 条日志`);
            this.info('LOGGER', '日志缓冲区已手动刷新');
        }
    }
    
    // 清空日志
    clearLogs() {
        // 清空内存中的日志
        this.logger.logs = [];
        
        // 清空调试面板
        var debugLogs = document.getElementById('debugLogs');
        if (debugLogs) {
            debugLogs.innerHTML = '';
        }
        
        this.info('LOGGER', '日志已清空');
    }
    
    // 设置日志级别
    setLogLevel(level) {
        this.logger.level = level;
        this.info('LOGGER', '日志级别设置为: ' + level);
    }
    
    // 测试日志系统
    testLogging() {
        console.log('开始测试日志系统...');
        this.info('TEST', '开始测试日志系统...');
        this.debug('TEST', '调试日志测试');
        this.warn('TEST', '警告日志测试');
        this.error('TEST', '错误日志测试');
        this.info('TEST', '日志系统测试完成');
        console.log('日志系统测试完成');
    }
    
    loadRequiredChunks() {
        if (!this.chunkLoading.enabled || this.chunkLoading.totalChunks === 0) return;
        
        // 计算当前可见范围需要的数据块
        const startChunk = Math.floor(this.virtualScroll.startIndex / this.chunkLoading.chunkSize);
        const endChunk = Math.floor(this.virtualScroll.endIndex / this.chunkLoading.chunkSize);
        
        // 扩展范围以预加载相邻块
        const preloadRange = 2; // 预加载前后2个块
        const extendedStartChunk = Math.max(0, startChunk - preloadRange);
        const extendedEndChunk = Math.min(this.chunkLoading.totalChunks - 1, endChunk + preloadRange);
        
        // 检查哪些块需要加载
        const chunksToLoad = [];
        for (let chunkIndex = extendedStartChunk; chunkIndex <= extendedEndChunk; chunkIndex++) {
            if (!this.chunkLoading.loadedChunks.has(chunkIndex)) {
                chunksToLoad.push(chunkIndex);
            }
        }
        
        if (chunksToLoad.length > 0) {
            this.debug('CHUNK_LOADING', `需要加载数据块: ${chunksToLoad.join(', ')} (扩展范围: ${extendedStartChunk}-${extendedEndChunk})`);
            this.loadChunksAsync(chunksToLoad);
        }
    }
    
    // 优先级加载：优先加载可见区域的数据
    loadChunksWithPriority(chunksToLoad) {
        // 按优先级排序：可见区域 > 预加载区域
        const visibleStartChunk = Math.floor(this.virtualScroll.startIndex / this.chunkLoading.chunkSize);
        const visibleEndChunk = Math.floor(this.virtualScroll.endIndex / this.chunkLoading.chunkSize);
        
        const priorityChunks = [];
        const normalChunks = [];
        
        chunksToLoad.forEach(chunkIndex => {
            if (chunkIndex >= visibleStartChunk && chunkIndex <= visibleEndChunk) {
                priorityChunks.push(chunkIndex);
            } else {
                normalChunks.push(chunkIndex);
            }
        });
        
        // 先加载优先级高的块
        if (priorityChunks.length > 0) {
            this.logDebug(`🚀 优先加载可见区域数据块: ${priorityChunks.join(', ')}`);
            this.loadChunksAsync(priorityChunks);
        }
        
        // 然后加载普通块
        if (normalChunks.length > 0) {
            this.logDebug(`📦 预加载数据块: ${normalChunks.join(', ')}`);
            setTimeout(() => {
                this.loadChunksAsync(normalChunks);
            }, 100);
        }
    }
    
    async loadChunksAsync(chunkIndexes) {
        for (const chunkIndex of chunkIndexes) {
            if (!this.chunkLoading.loadedChunks.has(chunkIndex)) {
                await this.loadChunk(chunkIndex);
                
                // 如果当前可见区域包含这个块，立即重新渲染
                const chunkStart = chunkIndex * this.chunkLoading.chunkSize;
                const chunkEnd = Math.min(chunkStart + this.chunkLoading.chunkSize, this.currentEntries.length);
                
                if (chunkStart < this.virtualScroll.endIndex && chunkEnd > this.virtualScroll.startIndex) {
                    this.logDebug(`🔄 数据块 ${chunkIndex} 加载完成，重新渲染可见区域`);
                    this.renderVisibleItems();
                }
            }
        }
    }
    
    renderVisibleItems() {
        const originalContainer = document.getElementById('originalLog');
        const parsedContainer = document.getElementById('parsedLog');
        
        // 检查容器是否存在
        if (!originalContainer || !parsedContainer) {
            this.error('VIRTUAL_SCROLL', '无法找到日志容器');
            return;
        }
        
        // 检查虚拟滚动容器是否正确初始化
        if (!originalContainer._visibleContainer || !parsedContainer._visibleContainer) {
            this.error('VIRTUAL_SCROLL', '虚拟滚动容器未正确初始化，重新初始化...');
            this.renderVirtualScroll();
            return;
        }
        
        // 添加详细的DOM调试信息
        this.debug('VIRTUAL_SCROLL', `容器状态检查:`);        
        this.debug('VIRTUAL_SCROLL', `原始容器尺寸: ${originalContainer.offsetWidth}x${originalContainer.offsetHeight}`);
        this.debug('VIRTUAL_SCROLL', `虚拟容器尺寸: ${originalContainer._virtualContainer.offsetWidth}x${originalContainer._virtualContainer.offsetHeight}`);
        this.debug('VIRTUAL_SCROLL', `可见容器元素数: ${originalContainer._visibleContainer.children.length}`);
        this.debug('VIRTUAL_SCROLL', `渲染范围: ${this.virtualScroll.startIndex}-${this.virtualScroll.endIndex} / ${this.currentEntries.length}`);
        
        // 清空可见容器
        originalContainer._visibleContainer.innerHTML = '';
        parsedContainer._visibleContainer.innerHTML = '';
        
        // 渲染可见范围内的项目
        let renderedCount = 0;
        for (let i = this.virtualScroll.startIndex; i < this.virtualScroll.endIndex; i++) {
            if (i >= this.currentEntries.length) break;
            
            const entry = this.currentEntries[i];
            
            // 检查数据是否已加载
            if (!entry || entry === undefined) {
                // 显示加载占位符
                this.renderLoadingPlaceholder(originalContainer._visibleContainer, i, 'original');
                this.renderLoadingPlaceholder(parsedContainer._visibleContainer, i, 'parsed');
                renderedCount++;
                continue;
            }
            
            // 渲染原始日志
            this.renderVirtualLogItem(originalContainer._visibleContainer, entry, i, 'original');
            
            // 渲染解析结果
            this.renderVirtualLogItem(parsedContainer._visibleContainer, entry, i, 'parsed');
            renderedCount++;
        }
        
        // 更新可见容器的位置
        const offsetY = this.virtualScroll.startIndex * this.virtualScroll.itemHeight;
        originalContainer._visibleContainer.style.transform = `translateY(${offsetY}px)`;
        parsedContainer._visibleContainer.style.transform = `translateY(${offsetY}px)`;
        
        // 检查渲染后的DOM状态
        // this.debug('VIRTUAL_SCROLL', `渲染后状态:`); // 减少调试日志
        // this.debug('VIRTUAL_SCROLL', `原始容器子元素数: ${originalContainer._visibleContainer.children.length}`);
        // this.debug('VIRTUAL_SCROLL', `解析容器子元素数: ${parsedContainer._visibleContainer.children.length}`);
        // this.debug('VIRTUAL_SCROLL', `实际渲染项目数: ${renderedCount}`);
        // this.debug('VIRTUAL_SCROLL', `容器偏移量: ${offsetY}px`);
        
        // 只在重要情况下记录日志
        if (renderedCount === 0 && this.currentEntries.length > 0) {
            this.error('VIRTUAL_SCROLL', '渲染项目数为0，可能存在问题');
        }
        
        // 强制重绘以确保内容可见
        originalContainer._visibleContainer.style.display = 'none';
        parsedContainer._visibleContainer.style.display = 'none';
        
        // 使用 requestAnimationFrame 确保重绘完成
        requestAnimationFrame(() => {
            originalContainer._visibleContainer.style.display = 'block';
            parsedContainer._visibleContainer.style.display = 'block';
            this.debug('VIRTUAL_SCROLL', `强制重绘完成`);
        });
        
        this.debug('VIRTUAL_SCROLL', `渲染完成: 显示 ${renderedCount} 个项目`);
    }
    
    renderLoadingPlaceholder(container, index, type) {
        const item = document.createElement('div');
        item.className = 'virtual-log-item loading-placeholder';
        item.style.cssText = `
            height: ${this.virtualScroll.itemHeight}px;
            padding: 8px;
            border-bottom: 1px solid var(--border-primary);
            display: flex;
            align-items: center;
            background: linear-gradient(90deg, var(--bg-tertiary) 25%, var(--bg-secondary) 50%, var(--bg-tertiary) 75%);
            background-size: 200% 100%;
            animation: loading 1.5s infinite;
            width: 100%;
            box-sizing: border-box;
            min-height: ${this.virtualScroll.itemHeight}px;
            color: var(--text-primary);
        `;
        
        item.innerHTML = `
            <div class="text-xs mr-2" style="color: var(--text-tertiary); min-width: 60px; flex-shrink: 0;">第 ${index + 1} 行</div>
            <div class="flex-1">
                <div class="h-4 rounded animate-pulse" style="background-color: var(--text-tertiary); margin-bottom: 4px;"></div>
                <div class="h-3 rounded animate-pulse" style="background-color: var(--text-muted); width: 80%;"></div>
            </div>
        `;
        
        container.appendChild(item);
        
        // this.debug('VIRTUAL_SCROLL', `添加加载占位符: ${type} ${index + 1}`); // 减少调试日志
    }
    
    renderVirtualLogItem(container, entry, index, type) {
        const item = document.createElement('div');
        item.className = 'virtual-log-item';
        item.style.cssText = `
            height: ${this.virtualScroll.itemHeight}px;
            padding: 8px;
            border-bottom: 1px solid var(--border-primary);
            display: flex;
            align-items: center;
            background-color: var(--bg-content);
            color: var(--text-primary);
            width: 100%;
            box-sizing: border-box;
            min-height: ${this.virtualScroll.itemHeight}px;
            position: relative;
        `;
        
        if (type === 'original') {
            // 检查原始数据是否存在
            const content = entry.original && entry.original.content ? entry.original.content : '无数据';
            item.innerHTML = `
                <div class="text-xs mr-2" style="color: var(--text-tertiary); min-width: 60px; flex-shrink: 0;">第 ${index + 1} 行</div>
                <div class="font-mono text-sm flex-1" style="color: var(--text-primary); word-break: break-all; overflow-wrap: anywhere;">${this.escapeHtml(content)}</div>
            `;
        } else {
            // 检查解析结果是否存在
            if (entry.rendered_blocks && entry.rendered_blocks.length > 0) {
                const block = entry.rendered_blocks[0];
                const title = block.title || '解析结果';
                const content = block.content || '无内容';
                item.innerHTML = `
                    <div class="text-xs mr-2" style="color: var(--text-tertiary); min-width: 60px; flex-shrink: 0;">第 ${index + 1} 行</div>
                    <div class="flex-1">
                        <div class="font-semibold text-sm mb-1" style="color: var(--text-primary);">${this.escapeHtml(title)}</div>
                        <div class="text-xs" style="color: var(--text-secondary); word-break: break-all; overflow-wrap: anywhere;">${this.escapeHtml(content.substring(0, 100))}${content.length > 100 ? '...' : ''}</div>
                    </div>
                    <button onclick="app.copyToClipboard('${block.id || ''}')" class="copy-btn text-xs px-2 py-1 flex-shrink-0" style="background-color: var(--color-primary); color: var(--text-inverse); border: none; border-radius: 4px; cursor: pointer;">
                        复制
                    </button>
                `;
            } else {
                item.innerHTML = `
                    <div class="text-xs mr-2" style="color: var(--text-tertiary); min-width: 60px; flex-shrink: 0;">第 ${index + 1} 行</div>
                    <div class="text-sm" style="color: var(--text-secondary);">无解析结果</div>
                `;
            }
        }
        
        // 添加鼠标悬停效果
        item.addEventListener('mouseenter', () => {
            item.style.backgroundColor = 'var(--bg-hover)';
        });
        
        item.addEventListener('mouseleave', () => {
            item.style.backgroundColor = 'var(--bg-content)';
        });
        
        container.appendChild(item);
        
        // 只在需要时记录调试信息
        // this.debug('VIRTUAL_SCROLL', `添加项目: ${type} ${index + 1}, 容器子元素数: ${container.children.length}`);
    }
    
    // 分块加载实现
    async loadChunk(chunkIndex) {
        if (this.chunkLoading.loadedChunks.has(chunkIndex)) {
            return; // 已经加载过了
        }
        
        this.logDebug(`📦 加载数据块: ${chunkIndex}`);
        
        const startIndex = chunkIndex * this.chunkLoading.chunkSize;
        const endIndex = Math.min(startIndex + this.chunkLoading.chunkSize, this.currentEntries.length);
        
        // 模拟分块加载（实际应用中这里会从服务器或文件系统加载）
        const chunk = this.currentEntries.slice(startIndex, endIndex);
        
        // 标记为已加载
        this.chunkLoading.loadedChunks.add(chunkIndex);
        
        this.logDebug(`✅ 数据块 ${chunkIndex} 加载完成，包含 ${chunk.length} 条日志`);
        
        return chunk;
    }
    
    // 内存优化：清理不可见的数据
    cleanupInvisibleData() {
        if (!this.virtualScroll.enabled) return;
        
        const visibleStart = this.virtualScroll.startIndex;
        const visibleEnd = this.virtualScroll.endIndex;
        const bufferSize = this.virtualScroll.bufferSize * 3; // 扩大缓冲区
        
        let cleanedCount = 0;
        
        // 清理远离可见区域的数据
        this.currentEntries.forEach((entry, index) => {
            if (index < visibleStart - bufferSize || index > visibleEnd + bufferSize) {
                // 清理渲染块数据以节省内存
                if (entry && entry.rendered_blocks) {
                    entry.rendered_blocks.forEach(block => {
                        if (block.formatted_content && block.formatted_content.length > 1000) {
                            block.formatted_content = block.formatted_content.substring(0, 1000) + '...';
                            cleanedCount++;
                        }
                    });
                }
            }
        });
        
        if (cleanedCount > 0) {
            this.logDebug(`🧹 内存清理完成，清理了 ${cleanedCount} 个长内容块`);
        }
    }
    
    // 滚动时的数据管理
    manageScrollData() {
        // 检查并加载需要的数据
        this.loadRequiredChunks();
        
        // 定期清理不可见数据
        if (Math.random() < 0.1) { // 10% 概率清理
            this.cleanupInvisibleData();
        }
    }
    
    // 性能监控
    getPerformanceMetrics() {
        const metrics = {
            totalEntries: this.currentEntries.length,
            visibleEntries: this.virtualScroll.endIndex - this.virtualScroll.startIndex,
            loadedChunks: this.chunkLoading.loadedChunks.size,
            totalChunks: this.chunkLoading.totalChunks,
            memoryUsage: performance.memory ? Math.round(performance.memory.usedJSHeapSize / 1024 / 1024) : 0
        };
        
        this.logDebug(`📊 性能指标: ${JSON.stringify(metrics)}`);
        return metrics;
    }
    
    // 键盘导航处理
    handleKeyboardNavigation(e) {
        console.log(`⌨️ 键盘导航: ${e.key}`);
        this.debug('KEYBOARD_NAV', `键盘导航: ${e.key}`);
        
        const container = document.getElementById('originalLog');
        if (!container._virtualContainer) {
            console.log('⚠️ 虚拟容器不存在，取消导航');
            return;
        }
        
        const currentScrollTop = container._virtualContainer.scrollTop;
        const itemHeight = this.virtualScroll.itemHeight;
        const containerHeight = this.virtualScroll.containerHeight || 400;
        const maxScrollTop = this.currentEntries.length * itemHeight - containerHeight;
        
        let newScrollTop = currentScrollTop;
        let shouldPreventDefault = false;
        
        switch (e.key) {
            case 'ArrowUp':
                newScrollTop = Math.max(0, currentScrollTop - itemHeight);
                shouldPreventDefault = true;
                break;
            case 'ArrowDown':
                newScrollTop = Math.min(maxScrollTop, currentScrollTop + itemHeight);
                shouldPreventDefault = true;
                break;
            case 'PageUp':
                newScrollTop = Math.max(0, currentScrollTop - containerHeight);
                shouldPreventDefault = true;
                break;
            case 'PageDown':
                newScrollTop = Math.min(maxScrollTop, currentScrollTop + containerHeight);
                shouldPreventDefault = true;
                break;
            case 'Home':
                newScrollTop = 0;
                shouldPreventDefault = true;
                break;
            case 'End':
                newScrollTop = maxScrollTop;
                shouldPreventDefault = true;
                break;
        }
        
        if (shouldPreventDefault) {
            e.preventDefault();
            
            // 平滑滚动到新位置
            this.smoothScrollTo(newScrollTop);
            
            console.log(`⌨️ 键盘导航: ${e.key} -> 滚动到 ${Math.round(newScrollTop)}px`);
            this.debug('KEYBOARD_NAV', `键盘导航: ${e.key} -> 滚动到 ${Math.round(newScrollTop)}px`);
        }
    }
    
    // 平滑滚动
    smoothScrollTo(targetScrollTop) {
        const originalContainer = document.getElementById('originalLog');
        const parsedContainer = document.getElementById('parsedLog');
        
        if (!originalContainer._virtualContainer) return;
        
        const startScrollTop = originalContainer._virtualContainer.scrollTop;
        const distance = targetScrollTop - startScrollTop;
        const duration = Math.min(300, Math.abs(distance) * 0.5); // 动态调整滚动时间
        const startTime = performance.now();
        
        const animateScroll = (currentTime) => {
            const elapsed = currentTime - startTime;
            const progress = Math.min(elapsed / duration, 1);
            
            // 使用缓动函数
            const easeOutCubic = 1 - Math.pow(1 - progress, 3);
            const currentScrollTop = startScrollTop + distance * easeOutCubic;
            
            originalContainer._virtualContainer.scrollTop = currentScrollTop;
            parsedContainer._virtualContainer.scrollTop = currentScrollTop;
            
            if (progress < 1) {
                requestAnimationFrame(animateScroll);
            } else {
                // 滚动完成，确保数据加载
                this.updateVirtualScroll();
            }
        };
        
        requestAnimationFrame(animateScroll);
    }
    
    // 确保滚动条和内容对应
    ensureScrollSync() {
        const originalContainer = document.getElementById('originalLog');
        const parsedContainer = document.getElementById('parsedLog');
        
        if (!originalContainer._virtualContainer || !parsedContainer._virtualContainer) return;
        
        // 同步滚动位置
        if (originalContainer._virtualContainer.scrollTop !== parsedContainer._virtualContainer.scrollTop) {
            parsedContainer._virtualContainer.scrollTop = originalContainer._virtualContainer.scrollTop;
        }
        
        // 确保虚拟滚动状态同步
        this.virtualScroll.scrollTop = originalContainer._virtualContainer.scrollTop;
        this.updateVirtualScroll();
    }
    
    // 启用大文件模式
    enableLargeFileMode() {
        this.info('MEMORY_MANAGER', '启用大文件优化模式');
        
        // 开启虚拟滚动
        this.virtualScroll.enabled = true;
        this.virtualScroll.bufferSize = 20; // 增加缓冲区
        
        // 开启分块加载
        this.chunkLoading.enabled = true;
        this.chunkLoading.chunkSize = 500; // 较小的块大小
        this.chunkLoading.adaptiveChunkSize = true;
        
        // 开启内存监控
        this.memoryManager.enableMonitoring = true;
        
        // 更新UI状态
        if (document.getElementById('virtualScrollEnabled')) {
            document.getElementById('virtualScrollEnabled').checked = true;
        }
        if (document.getElementById('chunkLoadingEnabled')) {
            document.getElementById('chunkLoadingEnabled').checked = true;
        }
        
        this.logDebug('🚀 大文件优化模式已启用');
    }
    
    // 检查内存使用情况
    checkMemoryUsage() {
        if (!this.memoryManager.enableMonitoring) return;
        
        try {
            // 估算当前内存使用量
            const estimatedUsage = this.estimateMemoryUsage();
            this.memoryManager.currentMemoryUsage = estimatedUsage;
            
            // 检查是否需要GC
            if (estimatedUsage > this.memoryManager.gcThreshold) {
                this.performGarbageCollection();
            }
            
        } catch (error) {
            this.warn('MEMORY_MANAGER', '内存检查失败', { error: error.message });
        }
    }
    
    // 估算内存使用量
    estimateMemoryUsage() {
        let totalSize = 0;
        
        // 估算日志数据大小
        if (this.currentEntries && this.currentEntries.length > 0) {
            const sampleEntry = this.currentEntries[0];
            const entrySize = JSON.stringify(sampleEntry).length * 2; // UTF-16字符
            totalSize += entrySize * this.currentEntries.length;
        }
        
        // 估算DOM元素大小
        const domElements = document.querySelectorAll('.log-line, .rendered-block');
        totalSize += domElements.length * 500; // 每个DOM元素估计500字节
        
        // 估算日志缓冲区大小
        if (this.logger.logs) {
            totalSize += this.logger.logs.length * 200;
        }
        
        return totalSize;
    }
    
    // 执行垃圾回收
    performGarbageCollection() {
        const now = Date.now();
        if (now - this.memoryManager.lastGcTime < 30000) { // 30秒内不重复GC
            return;
        }
        
        this.info('MEMORY_MANAGER', '开始内存清理');
        
        try {
            // 清理旧的日志数据
            if (this.logger.logs.length > this.logger.maxMemoryLogs) {
                const removeCount = this.logger.logs.length - this.logger.maxMemoryLogs;
                this.logger.logs.splice(0, removeCount);
                this.debug('MEMORY_MANAGER', `清理了 ${removeCount} 条历史日志`);
            }
            
            // 清理不可见的数据
            this.cleanupInvisibleData();
            
            // 清理DOM元素
            this.cleanupUnusedDomElements();
            
            // 强制垃圾回收（如果浏览器支持）
            if (window.gc) {
                window.gc();
            }
            
            this.memoryManager.lastGcTime = now;
            this.debug('MEMORY_MANAGER', '内存清理完成');
            
        } catch (error) {
            this.error('MEMORY_MANAGER', '内存清理失败', { error: error.message });
        }
    }
    
    // 清理未使用的DOM元素
    cleanupUnusedDomElements() {
        // 清理已经不在可见区域的DOM元素
        const containers = [document.getElementById('originalLog'), document.getElementById('parsedLog')];
        
        containers.forEach(container => {
            if (!container || !container._visibleContainer) return;
            
            const visibleContainer = container._visibleContainer;
            const children = Array.from(visibleContainer.children);
            
            // 如果子元素过多，清理一些
            if (children.length > this.virtualScroll.visibleCount * 3) {
                const removeCount = children.length - this.virtualScroll.visibleCount * 2;
                for (let i = 0; i < removeCount; i++) {
                    if (children[i]) {
                        children[i].remove();
                    }
                }
                this.debug('MEMORY_MANAGER', `清理了 ${removeCount} 个未使用的DOM元素`);
            }
        });
    }
    
    // 更新内存使用情况
    updateMemoryUsage() {
        if (!this.memoryManager.enableMonitoring) return;
        
        const currentUsage = this.estimateMemoryUsage();
        this.memoryManager.currentMemoryUsage = currentUsage;
        
        // 更新UI显示
        const memoryUsageElement = document.getElementById('memoryUsage');
        if (memoryUsageElement) {
            const usageMB = (currentUsage / (1024 * 1024)).toFixed(1);
            const maxMB = (this.memoryManager.maxMemoryUsage / (1024 * 1024)).toFixed(0);
            memoryUsageElement.textContent = `${usageMB}MB / ${maxMB}MB`;
        }
        
        // 检查是否接近限制
        const usagePercentage = (currentUsage / this.memoryManager.maxMemoryUsage) * 100;
        if (usagePercentage > 80) {
            this.warn('MEMORY_MANAGER', `内存使用率过高: ${usagePercentage.toFixed(1)}%`);
            
            // 自动清理
            if (usagePercentage > 90) {
                this.performGarbageCollection();
            }
        }
    }
}

// 初始化应用
var app = new LogWhisperApp();

// 页面加载完成后立即测试日志
window.addEventListener('load', function() {
    console.log('页面加载完成，开始测试日志系统...');
    app.info('PAGE', '页面加载完成');
    app.debug('PAGE', '开始测试日志输出');
    app.warn('PAGE', '这是一个警告日志');
    app.error('PAGE', '这是一个错误日志');
    app.info('PAGE', '日志测试完成');
    console.log('日志测试完成，请检查日志文件');
});
