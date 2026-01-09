import React, { useEffect, useRef, forwardRef } from 'react';
import { FixedSizeList as List, ListChildComponentProps } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';

interface LogLine {
  id: string;
  content: string;
  level: string;
  timestamp: string;
  formatted: string;
  lineNumber: number;
}

interface LogListProps {
  logs: LogLine[];
  theme: 'light' | 'dark';
  autoScroll: boolean;
}

const Row = ({ index, style, data }: ListChildComponentProps<LogLine[]>) => {
  const log = data[index];
  
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

  return (
    <div style={style} className={`flex items-center px-2 hover:bg-gray-100 dark:hover:bg-gray-800 border-b border-gray-100 dark:border-gray-800 text-sm font-mono whitespace-nowrap ${levelStyle}`}>
      {/* 行号 */}
      <span className="w-12 text-gray-400 dark:text-gray-600 text-right mr-4 select-none text-xs">
        {log.lineNumber}
      </span>
      
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
      
      {/* 内容 */}
      <span className="text-gray-900 dark:text-gray-100 truncate">
        {log.formatted || log.content}
      </span>
    </div>
  );
};

export const LogList: React.FC<LogListProps> = ({ logs, theme, autoScroll }) => {
  const listRef = useRef<List>(null);

  useEffect(() => {
    if (autoScroll && listRef.current) {
      listRef.current.scrollToItem(logs.length - 1, 'end');
    }
  }, [logs.length, autoScroll]);

  return (
    <div className="flex-1 h-full w-full">
      <AutoSizer>
        {({ height, width }) => (
          <List
            ref={listRef}
            height={height}
            itemCount={logs.length}
            itemSize={28} // 固定行高
            itemData={logs}
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
