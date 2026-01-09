import React, { useEffect } from 'react';
import { useFileStore, FileEntry } from '../../store/fileStore';
import { useTabStore } from '../../store/tabStore';
import { Folder, FileText, ArrowUp, Loader } from 'lucide-react';

export const FilesView = () => {
  const { entries, currentPath, isLoading, init, loadDir, goUp } = useFileStore();
  const { openTab } = useTabStore();

  useEffect(() => {
    init();
  }, []);

  const handleFileClick = (entry: FileEntry) => {
    if (entry.is_dir) {
      loadDir(entry.path);
    } else {
      openTab({
        id: `file-${entry.path}`,
        type: 'file',
        title: entry.name,
        target: entry.path,
        metadata: { path: entry.path }
      });
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="h-9 flex items-center px-4 border-b border-gray-200 dark:border-gray-800 gap-2 bg-gray-50 dark:bg-gray-900">
        <button 
          onClick={goUp} 
          className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded text-gray-500" 
          title="Up"
        >
          <ArrowUp size={14} />
        </button>
        <span className="text-xs font-medium text-gray-600 dark:text-gray-400 truncate flex-1 font-mono" title={currentPath}>
          {currentPath ? (currentPath.length > 20 ? '...' + currentPath.slice(-20) : currentPath) : 'Explorer'}
        </span>
        {isLoading && <Loader size={14} className="animate-spin text-blue-500" />}
      </div>
      
      <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
        {entries.map(entry => (
          <div 
            key={entry.path} 
            className="px-2 py-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer flex items-center gap-2 group transition-colors text-xs"
            onClick={() => handleFileClick(entry)}
          >
            {entry.is_dir ? (
              <Folder size={14} className="text-blue-500 flex-shrink-0 fill-blue-500/20" />
            ) : (
              <FileText size={14} className="text-gray-400 flex-shrink-0" />
            )}
            <span className="truncate text-gray-700 dark:text-gray-300">{entry.name}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
