import React, { useEffect, useRef, memo } from 'react';
import { FixedSizeList as List, ListChildComponentProps } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';

interface LogLine {
  id: string;
  content: string;
  level: string;
  timestamp: string;
  formatted: string;
  lineNumber: number;
  metadata?: Record<string, string>;
}

interface LogListProps {
  logs: LogLine[];
  theme: 'light' | 'dark';
  autoScroll: boolean;
  onShowContext?: (lineNumber: number) => void;
}

// 传递给 List 的 itemData 结构
interface ItemData {
  logs: LogLine[];
  onShowContext?: (lineNumber: number) => void;
}

const Row = memo(({ index, style, data }: ListChildComponentProps<ItemData>) => {
  const { logs, onShowContext } = data;
  const log = logs[index];
  
  // 根据日志级别获取样式
  const getLevelStyle = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error': return 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/10';
      case 'warn': return 'text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900/10';
      case 'info': return 'text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/10';
      case 'debug': return 'text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-800/10';
      default: return 'text-gray-600 dark:text-gray-400';
    }
  };

  const levelStyle = getLevelStyle(log.level);
  
  // 提取分析数据
  const traceId = log.metadata?.['_trace_id'];
  const durationMs = log.metadata?.['_duration_ms'];
  const isSlow = durationMs ? parseFloat(durationMs) > 500 : false;

  return (
    <div style={style} className={`flex items-center px-2 hover:bg-gray-100 dark:hover:bg-gray-800 border-b border-gray-100 dark:border-gray-800 text-sm font-mono whitespace-nowrap ${levelStyle} group`}>
      {/* 行号 */}
      <span className="w-12 text-gray-400 dark:text-gray-600 text-right mr-4 select-none text-xs flex-shrink-0">
        {log.lineNumber}
      </span>
      
      {/* 快捷操作区 (鼠标悬停显示) */}
      <div className="opacity-0 group-hover:opacity-100 absolute left-14 z-10 bg-white dark:bg-gray-800 shadow-sm border border-gray-200 dark:border-gray-700 rounded px-1 flex space-x-1">
        {onShowContext && (
          <button 
            onClick={(e) => { e.stopPropagation(); onShowContext(log.lineNumber); }}
            className="text-xs text-blue-600 hover:text-blue-800 dark:text-blue-400 p-0.5"
            title="查看上下文"
          >
            👁️
          </button>
        )}
      </div>
      
      {/* 时间戳 */}
      {log.timestamp && (
        <span className="w-40 text-gray-500 dark:text-gray-500 mr-4 text-xs flex-shrink-0">
          {log.timestamp}
        </span>
      )}
      
      {/* 级别 */}
      <span className={`w-16 font-bold mr-4 text-xs flex-shrink-0 ${
        log.level === 'error' ? 'text-red-600' :
        log.level === 'warn' ? 'text-yellow-600' :
        log.level === 'info' ? 'text-blue-600' :
        'text-gray-500'
      }`}>
        {log.level.toUpperCase()}
      </span>
      
      {/* Trace ID Badge */}
      {traceId && (
        <span className="mr-2 px-1.5 py-0.5 rounded-full text-[10px] bg-indigo-100 text-indigo-800 dark:bg-indigo-900/50 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 cursor-pointer hover:bg-indigo-200" title={`Trace ID: ${traceId}`}>
          🆔 {traceId.substring(0, 8)}...
        </span>
      )}

      {/* Duration Badge */}
      {durationMs && (
        <span className={`mr-2 px-1.5 py-0.5 rounded-full text-[10px] border ${
          isSlow 
            ? 'bg-orange-100 text-orange-800 dark:bg-orange-900/50 dark:text-orange-300 border-orange-200 dark:border-orange-800 font-bold' 
            : 'bg-green-50 text-green-700 dark:bg-green-900/30 dark:text-green-400 border-green-100 dark:border-green-800'
        }`} title="耗时">
          ⏱️ {durationMs}ms
        </span>
      )}
      
      {/* 内容 */}
      <span className="text-gray-900 dark:text-gray-100 truncate">
        {log.formatted || log.content}
      </span>
    </div>
  );
});

export const LogList: React.FC<LogListProps> = ({ logs, theme, autoScroll, onShowContext }) => {
  const listRef = useRef<List>(null);

  useEffect(() => {
    if (autoScroll && listRef.current) {
      listRef.current.scrollToItem(logs.length - 1, 'end');
    }
  }, [logs.length, autoScroll]);

  // 构造 itemData
  const itemData = React.useMemo(() => ({
    logs,
    onShowContext
  }), [logs, onShowContext]);

  return (
    <div className="flex-1 h-full w-full">
      <AutoSizer>
        {({ height, width }) => (
          <List
            ref={listRef}
            height={height}
            itemCount={logs.length}
            itemSize={28} // 固定行高
            itemData={itemData}
            width={width}
            className="scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600"
          >
            {Row}
          </List>
        )}
      </AutoSizer>
    </div>
  );
};
