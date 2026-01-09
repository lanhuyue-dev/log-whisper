import React from 'react';
import { useTabStore } from '../../store/tabStore';
import { LogViewer } from '../LogViewer';
import { X } from 'lucide-react';

export const EditorArea = () => {
  const { tabs, activeTabId, setActiveTab, closeTab } = useTabStore();

  return (
    <div className="flex-1 flex flex-col min-h-0 bg-white dark:bg-gray-900">
      {/* Tabs Header */}
      <div className="flex bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-800 overflow-x-auto no-scrollbar h-9">
        {tabs.map(tab => (
          <div
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`
              flex items-center gap-2 px-3 min-w-[120px] max-w-[200px] border-r border-gray-200 dark:border-gray-700 cursor-pointer text-xs select-none
              ${activeTabId === tab.id 
                ? 'bg-white dark:bg-gray-900 text-blue-600 dark:text-blue-400 border-t-2 border-t-blue-500' 
                : 'text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 border-t-2 border-t-transparent'}
            `}
          >
            <span className="truncate flex-1">{tab.title}</span>
            <button 
              onClick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
              className="p-0.5 hover:bg-gray-300 dark:hover:bg-gray-600 rounded opacity-0 group-hover:opacity-100"
            >
              <X size={12} />
            </button>
          </div>
        ))}
      </div>
      
      {/* Content Area - KeepAlive */}
      <div className="flex-1 relative">
        {tabs.length === 0 ? (
          <div className="absolute inset-0 flex items-center justify-center text-gray-400 text-sm flex-col gap-2">
            <div className="text-4xl">👋</div>
            <div>Select a container or file to start</div>
          </div>
        ) : (
          tabs.map(tab => (
            <div 
              key={tab.id} 
              className={`absolute inset-0 bg-white dark:bg-gray-900 ${activeTabId === tab.id ? 'z-10' : 'z-0 hidden'}`}
            >
              <LogViewer tab={tab} isActive={activeTabId === tab.id} />
            </div>
          ))
        )}
      </div>
    </div>
  );
};
