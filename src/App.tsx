import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'

interface LogLine {
  id: string
  content: string
  level: string
  timestamp: string
  formatted: string
  lineNumber: number
}

interface ParseResult {
  success: boolean
  entries: LogEntry[]
  stats: ParseStats
  chunk_info?: any
  error?: string
}

interface LogEntry {
  line_number: number
  content: string
  timestamp?: string
  level?: string
  formatted_content?: string
  thread?: string
  logger?: string
  message?: string
}

interface ParseStats {
  total_lines: number
  processed_lines: number
  format_detected: string
}

interface ParseRequest {
  content?: string
  file_path?: string
  format?: string
  plugin?: string
  chunk_size?: number
}

function App() {
  const [logs, setLogs] = useState<LogLine[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [pasteContent, setPasteContent] = useState('')
  const [showPasteDialog, setShowPasteDialog] = useState(false)
  const [selectedFilter, setSelectedFilter] = useState('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [theme, setTheme] = useState<'light' | 'dark'>('light')
  const [currentFile, setCurrentFile] = useState<string | null>(null)

  // 检查后端状态
  useEffect(() => {
    const checkBackend = async () => {
      try {
        console.log('🔧 检查 Tauri 后端状态...')
        const response = await invoke('health_check')
        console.log('✅ Tauri 后端连接成功:', response)
      } catch (error) {
        console.error('❌ Tauri 后端连接失败:', error)
        setError('后端连接失败，请重启应用')
      }
    }

    checkBackend()
  }, [])

  // 获取可用插件
  useEffect(() => {
    const loadPlugins = async () => {
      try {
        const plugins = await invoke('get_plugins')
        console.log('✅ 可用插件:', plugins)
      } catch (error) {
        console.error('❌ 获取插件失败:', error)
      }
    }

    loadPlugins()
  }, [])

  // 加载主题配置
  useEffect(() => {
    const loadTheme = async () => {
      try {
        console.log('🔧 加载主题配置...')
        const themeConfig = await invoke('get_theme_config')
        console.log('✅ 主题配置加载成功:', themeConfig)

        // 将后端主题模式转换为前端格式
        const frontendTheme = themeConfig.mode === 'dark' ? 'dark' : 'light'
        setTheme(frontendTheme)

        // 应用主题到HTML元素
        document.documentElement.classList.toggle('dark', frontendTheme === 'dark')
        document.documentElement.setAttribute('data-theme', themeConfig.mode)

        console.log('✅ 主题应用成功:', frontendTheme)
      } catch (error) {
        console.error('❌ 加载主题配置失败:', error)
        // 使用默认主题
        document.documentElement.classList.toggle('dark', theme === 'dark')
        document.documentElement.setAttribute('data-theme', theme)
      }
    }

    loadTheme()
  }, [])

  // 主题切换处理函数
  const handleThemeToggle = async () => {
    const newTheme = theme === 'light' ? 'dark' : 'light'
    const backendMode = newTheme === 'dark' ? 'dark' : 'light'

    try {
      console.log('🔧 切换主题到:', backendMode)

      // 保存主题配置到后端
      await invoke('update_theme_config', {
        request: {
          mode: backendMode,
          primary_color: '#3b82f6',
          accent_color: '#10b981',
          font_size: 14,
          font_family: 'system-ui'
        }
      })

      console.log('✅ 主题配置保存成功')

      // 更新前端状态
      setTheme(newTheme)

      // 应用主题到HTML元素
      document.documentElement.classList.toggle('dark', newTheme === 'dark')
      document.documentElement.setAttribute('data-theme', backendMode)

      console.log('✅ 主题切换完成:', newTheme)
    } catch (error) {
      console.error('❌ 主题切换失败:', error)
      // 即使保存失败，也更新本地状态
      setTheme(newTheme)
      document.documentElement.classList.toggle('dark', newTheme === 'dark')
      document.documentElement.setAttribute('data-theme', backendMode)
    }
  }

  // 处理文件选择
  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: '日志文件',
          extensions: ['log', 'txt', 'out']
        }]
      })

      if (selected && typeof selected === 'string') {
        setCurrentFile(selected)
        await parseFile(selected)
      }
    } catch (error) {
      console.error('❌ 文件选择失败:', error)
      setError(`文件选择失败: ${error}`)
    }
  }

  // 转换后端数据格式为前端格式
  const convertLogEntriesToLogLines = (entries: LogEntry[]): LogLine[] => {
    return entries.map(entry => ({
      id: `log-${entry.line_number}`,
      content: entry.formatted_content || entry.content,
      level: entry.level || 'info',
      timestamp: entry.timestamp || '',
      formatted: entry.formatted_content || entry.content,
      lineNumber: entry.line_number
    }))
  }

  // 解析文件
  const parseFile = async (filePath: string) => {
    setIsLoading(true)
    setError(null)

    try {
      console.log('🔧 开始解析文件:', filePath)

      const request: ParseRequest = {
        file_path: filePath,
        plugin: 'auto',
        chunk_size: 1000
      }

      const result = await invoke<ParseResult>('parse_log', {
        request: request
      })

      console.log('✅ 文件解析成功:', result)

      if (result.success && result.entries) {
        const logLines = convertLogEntriesToLogLines(result.entries)
        setLogs(logLines)
        console.log(`✅ 转换了 ${logLines.length} 条日志记录`)
      } else {
        setError(result.error || '解析失败')
      }
    } catch (error) {
      console.error('❌ 文件解析失败:', error)
      const errorMessage = error?.message || error?.toString() || '未知错误'
      console.error('❌ 详细错误信息:', {
        name: error?.name,
        message: error?.message,
        stack: error?.stack,
        error: error
      })
      setError(`文件解析失败: ${errorMessage}`)
    } finally {
      setIsLoading(false)
    }
  }

  // 处理粘贴确认
  const handlePasteConfirm = async () => {
    if (!pasteContent.trim()) {
      setError('请输入日志内容')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      console.log('🔧 开始解析日志内容...')

      const request: ParseRequest = {
        content: pasteContent,
        plugin: 'auto',
        chunk_size: 1000
      }

      const result = await invoke<ParseResult>('parse_log', {
        request: request
      })

      console.log('✅ 日志解析成功:', result)

      if (result.success && result.entries) {
        const logLines = convertLogEntriesToLogLines(result.entries)
        setLogs(logLines)
        console.log(`✅ 转换了 ${logLines.length} 条日志记录`)
        setShowPasteDialog(false)
        setPasteContent('')
      } else {
        setError(result.error || '解析失败')
      }
    } catch (error) {
      console.error('❌ 日志解析失败:', error)
      const errorMessage = error?.message || error?.toString() || '未知错误'
      console.error('❌ 详细错误信息:', {
        name: error?.name,
        message: error?.message,
        stack: error?.stack,
        error: error
      })
      setError(`日志解析失败: ${errorMessage}`)
    } finally {
      setIsLoading(false)
    }
  }

  // 过滤日志
  const filteredLogs = logs.filter(log => {
    const matchesFilter = selectedFilter === 'all' || log.level === selectedFilter
    const matchesSearch = !searchQuery || log.content.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesFilter && matchesSearch
  })

  // 统计信息
  const statistics = {
    total: logs.length,
    error: logs.filter(l => l.level === 'error').length,
    warn: logs.filter(l => l.level === 'warn').length,
    info: logs.filter(l => l.level === 'info').length,
    debug: logs.filter(l => l.level === 'debug').length,
  }

  return (
    <div className={`min-h-screen ${theme === 'dark' ? 'dark bg-gray-900' : 'bg-gray-50'}`}>
      {/* 主应用 */}
      <div className="h-screen flex flex-col">
        {/* 顶部工具栏 */}
        <header className="flex items-center justify-between h-12 px-4 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 shadow-sm flex-shrink-0">
          {/* 左侧：应用标题和文件操作 */}
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <div className="text-xl">📊</div>
              <span className="text-lg font-bold text-gray-900 dark:text-white">LogWhisper</span>
            </div>

            {/* 文件操作 */}
            <div className="flex items-center space-x-2">
              <button
                onClick={handleFileSelect}
                disabled={isLoading}
                className="inline-flex items-center space-x-1 px-3 py-1.5 text-sm rounded-md font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 bg-primary-600 hover:bg-primary-700 text-white focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <span className="text-sm">📁</span>
                <span>选择文件</span>
              </button>
              <button
                onClick={() => setShowPasteDialog(true)}
                disabled={isLoading}
                className="inline-flex items-center space-x-1 px-3 py-1.5 text-sm rounded-md font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 bg-primary-600 hover:bg-primary-700 text-white focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <span className="text-sm">📋</span>
                <span>粘贴</span>
              </button>
            </div>
          </div>

          {/* 中间：搜索 */}
          <div className="flex items-center space-x-4 flex-1 max-w-2xl mx-8">
            <div className="relative flex-1">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="搜索日志内容..."
                className="w-full px-3 py-1.5 pl-8 pr-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-primary-500 focus:border-transparent text-sm"
              />
              <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
                </svg>
              </div>
            </div>
          </div>

          {/* 右侧：工具按钮 */}
          <div className="flex items-center space-x-2">
            <button
              onClick={handleThemeToggle}
              className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-200"
              title={theme === 'light' ? '切换到深色模式' : '切换到浅色模式'}
            >
              <span className="text-lg">{theme === 'light' ? '🌙' : '☀️'}</span>
            </button>
          </div>
        </header>

        {/* 主内容区域 */}
        <main className="flex flex-1 bg-gray-50 dark:bg-gray-900 min-h-0">
          {/* 左侧导航面板 */}
          <aside className="w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
            {/* 当前文件信息 */}
            {currentFile && (
              <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">当前文件</h4>
                <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                  {currentFile}
                </div>
              </div>
            )}

            {/* 过滤器 */}
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">日志级别过滤</h4>
              <div className="space-y-2">
                {['all', 'error', 'warn', 'info', 'debug'].map(filter => (
                  <button
                    key={filter}
                    onClick={() => setSelectedFilter(filter)}
                    className={`w-full text-left px-3 py-2 text-sm rounded-md transition-colors ${
                      selectedFilter === filter
                        ? 'bg-blue-500 text-white'
                        : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'
                    }`}
                  >
                    {filter === 'all' ? '全部' : filter.toUpperCase()}
                  </button>
                ))}
              </div>
            </div>

            {/* 统计信息 */}
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">统计信息</h4>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">总计:</span>
                  <span className="font-medium text-gray-900 dark:text-white">{statistics.total}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">错误:</span>
                  <span className="font-medium text-red-600">{statistics.error}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">警告:</span>
                  <span className="font-medium text-yellow-600">{statistics.warn}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">信息:</span>
                  <span className="font-medium text-blue-600">{statistics.info}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">调试:</span>
                  <span className="font-medium text-gray-600">{statistics.debug}</span>
                </div>
              </div>
            </div>
          </aside>

          {/* 日志显示区域 */}
          <div className="flex-1 flex flex-col min-h-0">
            {logs.length === 0 ? (
              <div className="flex-1 flex items-center justify-center p-4">
                <div className="text-center space-y-4 max-w-lg">
                  <div className="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 text-center bg-white dark:bg-gray-800">
                    <div className="text-4xl mb-4">📄</div>
                    <p className="text-gray-700 dark:text-gray-300 mb-2 font-medium">请选择文件或粘贴日志内容</p>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">支持各种日志格式的智能解析</p>
                    <div className="flex justify-center space-x-4">
                      <button
                        onClick={handleFileSelect}
                        disabled={isLoading}
                        className="inline-flex items-center space-x-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        <span>📁</span>
                        <span>选择文件</span>
                      </button>
                      <button
                        onClick={() => setShowPasteDialog(true)}
                        disabled={isLoading}
                        className="inline-flex items-center space-x-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        <span>📋</span>
                        <span>粘贴日志</span>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="flex-1 overflow-auto bg-gray-50 dark:bg-gray-900 p-4">
                <div className="space-y-2 font-mono text-sm">
                  {filteredLogs.map((log) => (
                    <div
                      key={log.id}
                      className={`p-3 rounded-l-lg border-l-4 transition-colors duration-200 ${
                        log.level === 'error' ? 'border-red-500 bg-red-50 dark:bg-red-900/30 dark:border-red-400' :
                        log.level === 'warn' ? 'border-yellow-500 bg-yellow-50 dark:bg-yellow-900/30 dark:border-yellow-400' :
                        log.level === 'info' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 dark:border-blue-400' :
                        log.level === 'debug' ? 'border-gray-400 bg-gray-50 dark:bg-gray-800/50 dark:border-gray-500' :
                        'border-gray-300 bg-gray-50 dark:bg-gray-800/50'
                      }`}
                    >
                      <div className="flex items-start space-x-3">
                        {/* Log level badge */}
                        <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium flex-shrink-0 mt-0.5 ${
                          log.level === 'error' ? 'bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-300' :
                          log.level === 'warn' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/50 dark:text-yellow-300' :
                          log.level === 'info' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-300' :
                          log.level === 'debug' ? 'bg-gray-100 text-gray-800 dark:bg-gray-700/50 dark:text-gray-300' :
                          'bg-gray-100 text-gray-800 dark:bg-gray-700/50 dark:text-gray-300'
                        }`}>
                          {log.level.toUpperCase()}
                        </span>

                        {/* Timestamp */}
                        <span className="text-xs text-gray-500 dark:text-gray-400 flex-shrink-0 mt-0.5 min-w-fit">
                          {log.timestamp}
                        </span>

                        {/* Log content */}
                        <span className="flex-1 text-gray-900 dark:text-white leading-relaxed" style={{
                          wordBreak: 'break-word',
                          overflowWrap: 'break-word',
                          whiteSpace: 'pre-wrap',
                          minWidth: 0
                        }}>
                          {log.formatted_content || log.content}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </main>
      </div>

      {/* 粘贴对话框 */}
      {showPasteDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl mx-4 max-h-[80vh] flex flex-col">
            {/* 对话框标题 */}
            <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">粘贴日志内容</h3>
              <button
                onClick={() => setShowPasteDialog(false)}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
              </button>
            </div>

            {/* 对话框内容 */}
            <div className="flex-1 p-4 overflow-hidden">
              <div className="mb-4">
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                  请将需要分析的日志内容粘贴到下方文本框中，支持各种日志格式。
                </p>
              </div>

              {/* 文本输入区域 */}
              <div className="flex flex-col h-full">
                <textarea
                  value={pasteContent}
                  onChange={(e) => setPasteContent(e.target.value)}
                  className="flex-1 w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 font-mono text-sm resize-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                  placeholder="在此粘贴日志内容..."
                  rows={15}
                />
              </div>

              {/* 错误信息 */}
              {error && (
                <div className="mt-2 p-3 bg-red-100 dark:bg-red-900/20 border border-red-300 dark:border-red-700 rounded-md">
                  <p className="text-sm text-red-700 dark:text-red-400">{error}</p>
                </div>
              )}
            </div>

            {/* 对话框按钮 */}
            <div className="flex items-center justify-end space-x-3 p-4 border-t border-gray-200 dark:border-gray-700">
              <button
                onClick={() => {
                  setShowPasteDialog(false)
                  setPasteContent('')
                  setError(null)
                }}
                className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 rounded-md transition-colors duration-200"
              >
                取消
              </button>
              <button
                onClick={handlePasteConfirm}
                disabled={isLoading}
                className="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? '解析中...' : '分析日志'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default App