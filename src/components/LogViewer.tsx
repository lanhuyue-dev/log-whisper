import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { LogList } from './LogList';
import { LogTab } from '../../store/tabStore';

// 重新定义接口 (或者从 shared types 导入)
interface LogLine {
  id: string;
  content: string;
  level: string;
  timestamp: string;
  formatted: string;
  lineNumber: number;
  metadata?: Record<string, string>;
}

interface LogEntry {
  line_number: number;
  content: string;
  timestamp?: string;
  level?: string;
  formatted_content?: string;
  thread?: string;
  logger?: string;
  message?: string;
  metadata?: Record<string, string>;
}

interface LogViewerProps {
  tab: LogTab;
  isActive: boolean;
}

export const LogViewer: React.FC<LogViewerProps> = ({ tab, isActive }) => {
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  // const [error, setError] = useState<string | null>(null);

  // 转换逻辑
  const convertToLogLine = (entry: LogEntry): LogLine => ({
    id: `log-${entry.line_number}-${Date.now()}-${Math.random()}`, // 确保唯一
    content: entry.formatted_content || entry.content,
    level: entry.level || 'info',
    timestamp: entry.timestamp || '',
    formatted: entry.formatted_content || entry.content,
    lineNumber: entry.line_number, // 注意：tail 传回的 lineNumber 可能是 0
    metadata: entry.metadata
  });

  // 启动/停止 Tail
  useEffect(() => {
    // Session ID 即 Tab ID
    const sessionId = tab.id;

    const startTail = async () => {
      try {
        console.log(`Starting tail for ${tab.target} (${tab.type}) Session: ${sessionId}`);
        
        if (tab.type === 'docker') {
          await invoke('start_docker_tail', { sessionId, containerId: tab.target });
        } else if (tab.type === 'k8s') {
           const { namespace, podName } = tab.metadata || {}; 
           if (namespace && podName) {
             await invoke('start_k8s_tail', { sessionId, namespace, podName, containerName: null });
           }
        } else if (tab.type === 'file') {
           await invoke('start_tail', { sessionId, filePath: tab.target });
        }
      } catch (e) {
        console.error('Failed to start tail:', e);
      }
    };

    startTail();

    return () => {
      // 当组件卸载（Tab关闭）时，发送停止信号
      // 注意：如果是 Tab 隐藏（KeepAlive），这个 Effect 可能不会触发 cleanup (取决于父组件如何渲染)
      // 在我们的 EditorArea 实现中，hidden tab 并没有卸载，只是 display: none。
      // 所以只有 Tab 被关闭时才会触发 cleanup。
      console.log(`Stopping tail session: ${sessionId}`);
      invoke('stop_tail', { sessionId }).catch(console.error);
      invoke('stop_docker_tail', { sessionId }).catch(console.error);
      invoke('stop_k8s_tail', { sessionId }).catch(console.error);
      invoke('stop_journal_tail', { sessionId }).catch(console.error);
    };
  }, [tab]); // 依赖项只有 tab，这意味着只要 Tab 存在，Tail 就一直运行

  // 监听事件
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      unlisten = await listen<LogEntry[]>('tail-update', (event) => {
        const newEntries = event.payload;
        // 过滤：只处理属于当前 Session 的日志
        // 后端 LogEntry.metadata._session_id
        const myEntries = newEntries.filter(e => e.metadata?.['_session_id'] === tab.id);
        
        if (myEntries.length > 0) {
          const newLogLines = myEntries.map(convertToLogLine);
          
          setLogs(prev => {
            const startLine = prev.length + 1;
            const fixedLines = newLogLines.map((l, i) => ({
                ...l,
                lineNumber: startLine + i
            }));
            return [...prev, ...fixedLines].slice(-10000); 
          });
        }
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [tab]);

  return (
    <div className="h-full w-full flex flex-col">
      {/* Toolbar (Search, Filter, Autoscroll) - 可以提取为组件 */}
      <div className="h-10 border-b border-gray-200 dark:border-gray-800 flex items-center px-4 gap-4 bg-white dark:bg-gray-900">
         {/* ... filters ... */}
         <div className="text-xs text-gray-400">
            {logs.length} lines
         </div>
      </div>
      
      <div className="flex-1 overflow-hidden">
        <LogList 
          logs={logs} 
          theme="dark" 
          autoScroll={autoScroll}
          // onShowContext={} 
        />
      </div>
    </div>
  );
};
