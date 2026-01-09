import React, { useEffect } from 'react';
import { useDockerStore } from '../../store/dockerStore';
import { useTabStore } from '../../store/tabStore';
import { RefreshCw, Play, Square } from 'lucide-react';

export const DockerView = () => {
  const { containers, isLoading, loadContainers } = useDockerStore();
  const { openTab } = useTabStore();

  useEffect(() => {
    loadContainers();
  }, []);

  const handleContainerClick = (container: any) => {
    openTab({
      id: `docker-${container.id}`,
      type: 'docker',
      title: container.name,
      target: container.id,
      metadata: { image: container.image }
    });
  };

  return (
    <div className="flex flex-col h-full">
      <div className="h-9 flex items-center justify-between px-4 border-b border-gray-200 dark:border-gray-800">
        <span className="text-xs font-semibold uppercase tracking-wider text-gray-500">Containers</span>
        <button 
          onClick={() => loadContainers()} 
          className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded text-gray-500"
          title="Refresh"
        >
          <RefreshCw size={14} className={isLoading ? 'animate-spin' : ''} />
        </button>
      </div>
      
      <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
        {containers.map(container => (
          <div 
            key={container.id} 
            className="px-2 py-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer flex items-center gap-2 group transition-colors"
            onClick={() => handleContainerClick(container)}
          >
            <div className="flex-shrink-0 mt-0.5">
               {container.state === 'running' ? (
                 <div className="w-2 h-2 rounded-full bg-green-500" />
               ) : (
                 <div className="w-2 h-2 rounded-full bg-red-500" />
               )}
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium truncate text-gray-700 dark:text-gray-300">{container.name}</div>
              <div className="text-[10px] text-gray-400 truncate" title={container.image}>{container.image}</div>
            </div>
          </div>
        ))}
        
        {containers.length === 0 && !isLoading && (
          <div className="text-center p-4 text-gray-400 text-xs">
            No containers found
          </div>
        )}
      </div>
    </div>
  );
};
