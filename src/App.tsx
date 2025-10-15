import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
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

      // 读取文件内容
      const content = await readFileContent(file)
      setFileContent(content)

      // 解析日志
      await parseLogContent(content)

    } catch (error) {
      console.error('文件处理失败:', error)
    } finally {
      setLoading(false)
    }
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

  // 重置应用状态
  const resetApp = () => {
    setFileContent('')
    setParsedLogs([])
    setStats(null)
    setFileName('')
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
            <label className="btn-primary cursor-pointer inline-flex items-center space-x-1">
              <Upload className="w-4 h-4" />
              <span>选择文件</span>
              <input
                type="file"
                accept=".log,.txt,.json,.csv"
                className="hidden"
                onChange={(e) => e.target.files?.[0] && handleFileSelect(e.target.files[0])}
              />
            </label>
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
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">解析成功:</span>
                      <span className="text-green-600">{stats.success_lines}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">解析失败:</span>
                      <span className="text-red-600">{stats.error_lines}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">解析时间:</span>
                      <span className="text-gray-900 dark:text-white">{stats.parse_time_ms}ms</span>
                    </div>
                  </div>
                </div>
              )}
            </aside>

            {/* 日志内容区 */}
            <div className="flex-1 flex flex-col">
              {/* 文件信息 */}
              <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    文件: {fileName}
                  </span>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    显示 {filteredLogs.length} / {parsedLogs.length} 条日志
                  </span>
                </div>
              </div>

              {/* 日志内容 */}
              <div className="flex-1 overflow-y-auto bg-white dark:bg-gray-900">
                <div className="font-mono text-sm">
                  {filteredLogs.map((log) => (
                    <div
                      key={log.line_number}
                      className={`log-line ${
                        log.level ? log.level.toLowerCase() : ''
                      }`}
                    >
                      <div className="flex items-start space-x-3">
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
              </div>
            </div>
          </div>
        )}
      </main>

      {/* 状态栏 */}
      <footer className="status-bar">
        <div className="flex items-center justify-between w-full">
          <div className="flex items-center space-x-4">
            <span>行 {filteredLogs.length}/{stats?.total_lines || 0}</span>
            {searchTerm && <span>搜索: {filteredLogs.length} 处匹配</span>}
          </div>
          <div className="flex items-center space-x-4">
            {stats && <span>解析: {stats.parse_time_ms}ms</span>}
          </div>
        </div>
      </footer>
    </div>
  )
}

export default App