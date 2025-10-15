import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { FileText, Upload, Moon, Sun, Filter, Search, X } from 'lucide-react'

interface LogEntry {
  line_number: number
  content: string
  timestamp?: string
  level?: string
  formatted_content?: string
  metadata: Record<string, string>
  processed_by: string[]
}

interface ParseResponse {
  success: boolean
  entries: LogEntry[]
  stats: {
    total_lines: number
    success_lines: number
    error_lines: number
    parse_time_ms: number
  }
  chunk_info?: {
    total_chunks: number
    current_chunk: number
    has_more: boolean
  }
  error?: string
  detected_format?: string
}

interface Plugin {
  name: string
  description: string
  version: string
}

interface PluginsResponse {
  plugins: Plugin[]
}

interface FileInfoResponse {
  file_path: string
  file_size: number
  total_lines: number
  recommended_chunk_size: number
  is_large_file: boolean
}

function App() {
  const [darkMode, setDarkMode] = useState(false)
  const [loading, setLoading] = useState(true)
  const [fileContent, setFileContent] = useState<string>('')
  const [parsedLogs, setParsedLogs] = useState<LogEntry[]>([])
  const [plugins, setPlugins] = useState<Plugin[]>([])
  const [selectedFilter, setSelectedFilter] = useState<string>('all')
  const [searchTerm, setSearchTerm] = useState<string>('')
  const [isDragging, setIsDragging] = useState(false)
  const [stats, setStats] = useState<ParseResponse['stats'] | null>(null)
  const [fileName, setFileName] = useState<string>('')
  const [filePath, setFilePath] = useState<string>('')
  const [fileInfo, setFileInfo] = useState<FileInfoResponse | null>(null)
  const [currentChunk, setCurrentChunk] = useState(0)
  const [totalChunks, setTotalChunks] = useState(0)
  const [isLoadingChunk, setIsLoadingChunk] = useState(false)
  const [chunkProgress, setChunkProgress] = useState(0)

  // 初始化应用
  useEffect(() => {
    initializeApp()
  }, [])

  // 初始化应用
  const initializeApp = async () => {
    try {
      // 加载插件列表
      await loadPlugins()

      // 检测系统主题
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      setDarkMode(prefersDark)

      setLoading(false)
    } catch (error) {
      console.error('应用初始化失败:', error)
      setLoading(false)
    }
  }

  // 加载插件列表
  const loadPlugins = async () => {
    try {
      const response = await invoke<PluginsResponse>('get_plugins')
      setPlugins(response.plugins)
    } catch (error) {
      console.error('加载插件失败:', error)
      // 设置默认插件列表作为回退
      setPlugins([
        { name: 'auto', description: '自动检测', version: '1.0.0' },
        { name: 'mybatis', description: 'MyBatis SQL 解析器', version: '1.0.0' },
        { name: 'docker_json', description: 'Docker JSON 日志', version: '1.0.0' },
        { name: 'raw', description: '原始文本', version: '1.0.0' }
      ])
    }
  }

  // 切换主题
  const toggleTheme = () => {
    setDarkMode(!darkMode)
  }

  // 应用主题到文档
  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [darkMode])

  // 处理文件选择
  const handleFileSelect = async (file: File) => {
    if (!file) return

    try {
      setFileName(file.name)
      setLoading(true)
      resetApp()

      // 对于小文件，使用内容模式
      if (file.size <= 50 * 1024 * 1024) { // 50MB以下使用内容模式
        await processWholeFile(file)
      } else {
        // 大文件提示使用文件选择对话框
        alert('大文件请使用"选择文件"按钮而不是拖拽，以获得更好的性能')
      }

    } catch (error) {
      console.error('文件处理失败:', error)
    } finally {
      setLoading(false)
    }
  }

  // 处理文件选择对话框
  const handleFileSelectDialog = async () => {
    try {
      console.log('🚀 [DEBUG] handleFileSelectDialog 开始执行')
      setLoading(true)
      resetApp()

      // 打开文件选择对话框
      console.log('📂 [DEBUG] 准备打开文件选择对话框')
      const selected = await open({
        multiple: false,
        filters: [{
          name: '日志文件',
          extensions: ['log', 'txt', 'json', 'csv']
        }]
      })
      console.log('📂 [DEBUG] 文件选择结果:', selected)

      if (selected && typeof selected === 'string') {
        // 使用文件路径模式
        console.log('📁 [DEBUG] 使用文件路径模式:', selected)
        setFilePath(selected)
        setFileName(selected.split('\\').pop() || selected.split('/').pop() || selected)

        // 获取文件信息
        console.log('📊 [DEBUG] 开始获取文件信息')
        const fileInfo = await getFileInfo(selected)
        console.log('📊 [DEBUG] 文件信息获取结果:', fileInfo)
        setFileInfo(fileInfo)

        console.log('🔄 [DEBUG] 开始分块处理')
        await processFileInChunks(selected, fileInfo)
      } else {
        console.log('❌ [DEBUG] 用户取消选择文件或选择无效')
      }

    } catch (error) {
      console.error('❌ [DEBUG] 文件选择失败:', error)
    } finally {
      setLoading(false)
    }
  }


  // 获取文件信息
  const getFileInfo = async (filePath: string): Promise<FileInfoResponse> => {
    try {
      console.log('🔍 [DEBUG] 调用 get_file_info 命令，文件路径:', filePath)
      const response = await invoke<FileInfoResponse>('get_file_info', { filePath })
      console.log('✅ [DEBUG] get_file_info 响应:', response)
      return response
    } catch (error) {
      console.error('❌ [DEBUG] 获取文件信息失败:', error)
      // 返回默认值
      return {
        file_path: filePath,
        file_size: 0,
        total_lines: 0,
        recommended_chunk_size: 1000,
        is_large_file: false
      }
    }
  }

  // 处理整个文件（小文件）
  const processWholeFile = async (file: File) => {
    const content = await readFileContent(file)
    setFileContent(content)
    await parseLogContent(content)
  }

  // 分块处理文件
  const processFileInChunks = async (filePath: string, fileInfo: FileInfoResponse) => {
    console.log('🔄 [DEBUG] 开始分块处理文件')
    console.log('📊 [DEBUG] 文件信息:', fileInfo)

    const totalChunks = Math.ceil(fileInfo.total_lines / fileInfo.recommended_chunk_size)
    console.log('📊 [DEBUG] 计算分块信息:', { totalChunks, totalLines: fileInfo.total_lines, chunkSize: fileInfo.recommended_chunk_size })

    setTotalChunks(totalChunks)
    setCurrentChunk(0)
    setParsedLogs([])

    // 处理第一块
    console.log('🔄 [DEBUG] 开始加载第一块')
    await loadChunk(filePath, 0, fileInfo.recommended_chunk_size)
    console.log('✅ [DEBUG] 第一块加载完成')
  }

  // 加载指定块
  const loadChunk = async (filePath: string, chunkIndex: number, chunkSize: number) => {
    console.log(`📦 [DEBUG] 开始加载第 ${chunkIndex} 块，大小: ${chunkSize}`)
    setIsLoadingChunk(true)
    setChunkProgress(Math.round((chunkIndex / totalChunks) * 100))

    try {
      const request = {
        file_path: filePath,
        chunk_size: chunkSize,
        chunk_index: chunkIndex,
        plugin: 'auto'
      }
      console.log('📤 [DEBUG] 发送解析请求:', request)

      const response = await invoke<ParseResponse>('parse_log', { request })
      console.log('📥 [DEBUG] 解析响应:', response)

      if (response.success) {
        console.log(`✅ [DEBUG] 第 ${chunkIndex} 块解析成功，条目数: ${response.entries.length}`)

        // 将新块的数据追加到现有数据
        setParsedLogs(prev => {
          const newLogs = [...prev, ...response.entries]
          console.log(`📊 [DEBUG] 日志条目更新: ${prev.length} -> ${newLogs.length}`)
          return newLogs
        })
        setStats(response.stats)

        // 更新分块信息
        if (response.chunk_info) {
          console.log('📊 [DEBUG] 分块信息:', response.chunk_info)
          setTotalChunks(response.chunk_info.total_chunks)
          setCurrentChunk(response.chunk_info.current_chunk)

          // 如果还有更多块，自动加载下一块
          if (response.chunk_info.has_more && chunkIndex < 10) { // 限制前10块
            console.log(`🔄 [DEBUG] 自动加载下一块: ${chunkIndex + 1}`)
            setTimeout(() => {
              loadChunk(filePath, chunkIndex + 1, chunkSize)
            }, 100)
          }
        }
      } else {
        console.error('❌ [DEBUG] 分块解析失败:', response.error)
      }
    } catch (error) {
      console.error('❌ [DEBUG] 分块解析异常:', error)
    } finally {
      setIsLoadingChunk(false)
      setChunkProgress(100)
    }
  }

  // 加载更多块
  const loadMoreChunks = async () => {
    if (!filePath || !fileInfo || currentChunk >= totalChunks - 1) return

    await loadChunk(filePath, currentChunk + 1, fileInfo.recommended_chunk_size)
  }

  // 读取文件内容
  const readFileContent = (file: File): Promise<string> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = (e) => {
        if (e.target?.result) {
          resolve(e.target.result as string)
        } else {
          reject(new Error('文件读取失败'))
        }
      }
      reader.onerror = () => reject(new Error('文件读取失败'))
      reader.readAsText(file, 'utf-8')
    })
  }

  // 解析日志内容
  const parseLogContent = async (content: string) => {
    try {
      const response = await invoke<ParseResponse>('parse_log', {
        request: {
          content: content,
          plugin: 'auto'
        }
      })

      if (response.success) {
        setParsedLogs(response.entries)
        setStats(response.stats)
      } else {
        console.error('日志解析失败:', response.error)
      }
    } catch (error) {
      console.error('日志解析异常:', error)
    }
  }

  // 处理文件拖拽
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
  }

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)

    const files = e.dataTransfer.files
    if (files.length > 0) {
      await handleFileSelect(files[0])
    }
  }

  // 过滤日志
  const filteredLogs = parsedLogs.filter(log => {
    // 级别过滤
    if (selectedFilter !== 'all' && log.level?.toLowerCase() !== selectedFilter) {
      return false
    }

    // 搜索过滤
    return !(searchTerm && !log.content.toLowerCase().includes(searchTerm.toLowerCase()));

  })

  // 添加日志渲染调试信息
  console.log(`🎨 [DEBUG] 渲染状态: 总日志=${parsedLogs.length}, 过滤后=${filteredLogs.length}, 过滤器=${selectedFilter}, 搜索词=${searchTerm}`)

  // 重置应用状态
  const resetApp = () => {
    setFileContent('')
    setParsedLogs([])
    setStats(null)
    setFileName('')
    setFilePath('')
    setFileInfo(null)
    setCurrentChunk(0)
    setTotalChunks(0)
    setIsLoadingChunk(false)
    setChunkProgress(0)
    setSearchTerm('')
    setSelectedFilter('all')
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="text-center space-y-4">
          <div className="w-16 h-16 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin mx-auto"></div>
          <div className="space-y-2">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">LogWhisper</h2>
            <p className="text-gray-600 dark:text-gray-400">正在加载...</p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={`h-screen flex flex-col ${darkMode ? 'dark' : ''}`}>
      {/* 顶部工具栏 */}
      <header className="flex items-center justify-between h-12 px-4 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 shadow-sm flex-shrink-0">
        {/* 左侧：应用标题和文件操作 */}
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <span className="text-lg font-bold text-gray-900 dark:text-white">LogWhisper</span>
          </div>

          {/* 文件操作 */}
          <div className="flex items-center space-x-2">
            <button
              onClick={handleFileSelectDialog}
              className="btn-primary inline-flex items-center space-x-1"
            >
              <Upload className="w-4 h-4" />
              <span>选择文件</span>
            </button>
          </div>
        </div>

        {/* 中间：搜索 */}
        <div className="flex items-center space-x-4 flex-1 max-w-2xl mx-8">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              placeholder="搜索日志内容..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
        </div>

        {/* 右侧：工具按钮 */}
        <div className="flex items-center space-x-2">
          {fileName && (
            <button
              onClick={resetApp}
              className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              title="重置"
            >
              <X className="w-4 h-4" />
            </button>
          )}
          <button
            onClick={toggleTheme}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
            title="切换主题"
          >
            {darkMode ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
          </button>
        </div>
      </header>

      {/* 主内容区 */}
      <main className="flex flex-1 bg-gray-50 dark:bg-gray-900 min-h-0">
        {parsedLogs.length === 0 ? (
          /* 欢迎界面 */
          <div className="flex-1 flex items-center justify-center p-4">
            <div className="text-center space-y-4 max-w-lg">

              {/* 支持格式 */}
              <div className="text-left bg-gray-50 dark:bg-gray-800 rounded-lg p-4 mt-6">
                <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  支持的日志格式：
                </h4>
                <ul className="space-y-1 text-sm text-gray-600 dark:text-gray-400">
                  {plugins.map((plugin) => (
                    <li key={plugin.name} className="flex items-center space-x-2">
                      <span className="text-blue-500">•</span>
                      <span>{plugin.description}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        ) : (
          /* 日志分析界面 */
          <div className="flex-1 flex">
            {/* 左侧过滤面板 */}
            <aside className="w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
              <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center space-x-2">
                  <Filter className="w-4 h-4" />
                  <span>日志级别过滤</span>
                </h3>
                <div className="space-y-2">
                  {['all', 'error', 'warn', 'info', 'debug'].map((filter) => (
                    <button
                      key={filter}
                      onClick={() => setSelectedFilter(filter)}
                      className={`w-full text-left px-3 py-2 text-sm rounded-lg transition-colors ${
                        selectedFilter === filter
                          ? 'bg-blue-500 text-white'
                          : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                      }`}
                    >
                      {filter === 'all' ? '全部' : filter.toUpperCase()}
                    </button>
                  ))}
                </div>
              </div>

              {/* 统计信息 */}
              {stats && (
                <div className="p-4">
                  <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">统计信息</h3>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">总行数:</span>
                      <span className="text-gray-900 dark:text-white">{stats.total_lines}</span>
                    </div>
                  </div>
                </div>
              )}
            </aside>

            {/* 日志内容区 */}
            <div className="flex-1 flex flex-col">
              {/* 文件信息和分块进度 */}
              <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    文件: {fileName}
                  </span>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    显示 {filteredLogs.length} / {parsedLogs.length} 条日志
                  </span>
                </div>

                {/* 分块处理进度 */}
                {(fileInfo?.is_large_file || totalChunks > 1) && (
                  <div className="mt-2 space-y-1">
                    <div className="flex items-center justify-between text-xs text-gray-600 dark:text-gray-400">
                      <span>分块加载进度: {currentChunk + 1} / {totalChunks}</span>
                      <span>{chunkProgress}%</span>
                    </div>
                    <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
                      <div
                        className="bg-blue-600 h-1.5 rounded-full transition-all duration-300"
                        style={{ width: `${chunkProgress}%` }}
                      ></div>
                    </div>
                    {fileInfo && (
                      <div className="text-xs text-gray-500 dark:text-gray-500">
                        文件大小: {(fileInfo.file_size / 1024 / 1024).toFixed(2)} MB |
                        总行数: {fileInfo.total_lines.toLocaleString()} |
                        块大小: {fileInfo.recommended_chunk_size} 行
                      </div>
                    )}
                  </div>
                )}
              </div>

              {/* 日志内容 - 移除 overflow-x-hidden 以显示水平滚动条 */}
              <div className="flex-1 overflow-auto bg-white dark:bg-gray-900">
                <div className="font-mono text-sm" style={{ minWidth: 'fit-content' }}>
                  {filteredLogs.map((log) => (
                    <div
                      key={log.line_number}
                      className={`log-line ${
                        log.level ? log.level.toLowerCase() : ''
                      }`}
                      style={{ whiteSpace: 'nowrap', minWidth: '100%' }}
                    >
                      <div className="flex items-start space-x-3" style={{ minWidth: '100%' }}>
                        <span className="text-gray-500 dark:text-gray-500 text-xs w-12 flex-shrink-0">
                          {log.line_number}
                        </span>
                        <div className="flex-1 min-w-0">
                          <div className="text-gray-900 dark:text-white">
                            {log.formatted_content || log.content}
                          </div>
                        </div>
                        {log.level && (
                          <span
                            className={`text-xs px-2 py-1 rounded flex-shrink-0 ${
                              log.level === 'ERROR'
                                ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
                                : log.level === 'WARN'
                                ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
                                : log.level === 'INFO'
                                ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
                                : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-400'
                            }`}
                          >
                            {log.level}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>

                {/* 加载更多按钮 */}
                {fileInfo?.is_large_file && currentChunk < totalChunks - 1 && !isLoadingChunk && (
                  <div className="sticky bottom-0 bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-700 p-4">
                    <button
                      onClick={loadMoreChunks}
                      className="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center justify-center space-x-2"
                    >
                      <span>加载更多日志</span>
                      <span className="text-sm opacity-75">({totalChunks - currentChunk - 1} 块剩余)</span>
                    </button>
                  </div>
                )}

                {/* 加载中的指示器 */}
                {isLoadingChunk && (
                  <div className="sticky bottom-0 bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-700 p-4">
                    <div className="flex items-center justify-center space-x-2">
                      <div className="w-4 h-4 border-2 border-blue-200 border-t-blue-600 rounded-full animate-spin"></div>
                      <span className="text-sm text-gray-600 dark:text-gray-400">正在加载分块 {currentChunk + 1} / {totalChunks}...</span>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  )
}

export default App