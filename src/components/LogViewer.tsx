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
    // 只有当组件挂载时才启动 tail
    // TODO: 如何处理多个 tab 同时 tail? 后端目前的设计是全局单例的 stopper
    // 这意味着我们目前只能支持 ONE active tail at a time!
    // 这是一个架构限制。我们需要修改后端支持多个 tail sessions。
    
    // 暂时 workaround: 只有 active tab 才能 tail。
    // 当切换 tab 时，停止上一个，启动这一个。
    
    if (!isActive) return;

    const startTail = async () => {
      try {
        console.log(`Starting tail for ${tab.target} (${tab.type})`);
        // 先停止所有 (后端目前是全局互斥的)
        // await invoke('stop_tail');
        // await invoke('stop_docker_tail');
        // await invoke('stop_k8s_tail');

        if (tab.type === 'docker') {
          await invoke('start_docker_tail', { containerId: tab.target });
        } else if (tab.type === 'k8s') {
           const { namespace, podName } = tab.metadata;
           await invoke('start_k8s_tail', { namespace, podName, containerName: null });
        } else if (tab.type === 'file') {
           // TODO: Implement file tail
        }
      } catch (e) {
        console.error('Failed to start tail:', e);
      }
    };

    startTail();

    return () => {
      // Cleanup? 
      // 如果我们切换 tab，新的 tab 会在它的 effect 里调用 stop -> start
      // 但如果 tab 关闭，我们需要 stop
    };
  }, [isActive, tab]);

  // 监听事件
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      // 注意：所有 tabs 都在监听同一个 'tail-update' 事件！
      // 后端广播给所有窗口。
      // 我们需要过滤吗？后端目前没有在 event payload 里带 source ID。
      // LogEntry.metadata 里可能有 container name / pod name。
      
      unlisten = await listen<LogEntry[]>('tail-update', (event) => {
        if (!isActive) return; // 只处理 active tab 的更新 (配合上面的 workaround)

        const newEntries = event.payload;
        // 简单过滤：如果 metadata 匹配当前 tab
        // 这需要后端配合。目前先假设因为只有 active tab 启动了 tail，所以收到的就是它的。
        
        if (newEntries.length > 0) {
          const newLogLines = newEntries.map(convertToLogLine);
          
          // 修正行号 (如果是 stream，累加)
          setLogs(prev => {
            const startLine = prev.length + 1;
            const fixedLines = newLogLines.map((l, i) => ({
                ...l,
                lineNumber: startLine + i
            }));
            return [...prev, ...fixedLines].slice(-10000); // 限制 10k 行
          });
        }
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [isActive, tab]);

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
