import React, { useEffect } from 'react';
import { useK8sStore, K8sPodInfo } from '../../store/k8sStore';
import { useTabStore } from '../../store/tabStore';
import { RefreshCw, Circle } from 'lucide-react';

export const K8sView = () => {
  const { namespaces, currentNamespace, pods, isLoading, loadNamespaces, setNamespace, loadPods } = useK8sStore();
  const { openTab } = useTabStore();

  useEffect(() => {
    loadNamespaces();
    loadPods(currentNamespace);
  }, []);

  const handlePodClick = (pod: K8sPodInfo) => {
    openTab({
      id: `k8s-${currentNamespace}-${pod.name}`,
      type: 'k8s',
      title: pod.name,
      target: pod.name, // LogViewer uses this
      metadata: { namespace: currentNamespace, podName: pod.name }
    });
  };

  return (
    <div className="flex flex-col h-full">
      <div className="p-3 border-b border-gray-200 dark:border-gray-800 flex flex-col gap-2 bg-gray-50 dark:bg-gray-900">
        <div className="flex justify-between items-center">
           <span className="text-xs font-semibold uppercase text-gray-500 tracking-wider">Kubernetes</span>
           <button 
             onClick={() => loadPods(currentNamespace)} 
             className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded text-gray-500"
             title="Refresh"
           >
             <RefreshCw size={14} className={isLoading ? 'animate-spin' : ''} />
           </button>
        </div>
        <select 
          value={currentNamespace} 
          onChange={(e) => setNamespace(e.target.value)}
          className="w-full text-xs bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded px-2 py-1.5 focus:outline-none focus:ring-1 focus:ring-blue-500"
        >
          {namespaces.map(ns => <option key={ns} value={ns}>{ns}</option>)}
        </select>
      </div>
      
      <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
        {pods.map(pod => (
          <div 
            key={pod.name} 
            className="px-2 py-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer flex items-center gap-2 text-xs transition-colors group"
            onClick={() => handlePodClick(pod)}
          >
            <Circle size={6} className={pod.status === 'Running' ? 'text-green-500 fill-current' : 'text-red-500 fill-current'} />
            <div className="flex-1 min-w-0">
              <div className="font-medium truncate text-gray-700 dark:text-gray-300">{pod.name}</div>
              <div className="text-[10px] text-gray-400 flex justify-between mt-0.5">
                 <span>{pod.status}</span>
                 {pod.restart_count > 0 && <span className="text-orange-400">R:{pod.restart_count}</span>}
              </div>
            </div>
          </div>
        ))}
        
        {pods.length === 0 && !isLoading && (
          <div className="text-center p-4 text-gray-400 text-xs">
            No pods found in {currentNamespace}
          </div>
        )}
      </div>
    </div>
  );
};
